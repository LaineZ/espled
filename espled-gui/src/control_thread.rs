use std::{
    fmt::write,
    io::{BufRead, BufReader, Read},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use serialport::{self, SerialPort, SerialPortInfo};
use std::sync::Mutex;

#[derive(Debug, PartialEq)]
pub enum Command {
    ProbeControllersOnSerials,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChannelStatus {
    ProbingControllers(String),
    Done,
}

#[derive(Clone)]
pub struct Controller {
    pub name: String,
    pub serial_path: String,
}

impl Controller {
    pub fn apply_color(&self, color: u32) {
        if let Ok(mut port) = serialport::new(self.serial_path.clone(), 115200)
            .timeout(Duration::from_millis(100))
            .open()
        {
            let color = format!("color {:x}\n", color);
            port.write(color.as_bytes()).unwrap();
        }
    }
}

impl std::fmt::Display for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl std::fmt::Display for ChannelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelStatus::Done => write!(f, "Done"),
            ChannelStatus::ProbingControllers(serial_name) => {
                write!(f, "Probing controller on address: {serial_name}")
            }
            _ => write!(f, "Please wait.."),
        }
    }
}

pub struct ControlChannel {
    sender: Sender<Command>,
    controllers: Arc<Mutex<Vec<Controller>>>,
    last_status: ChannelStatus,
    status_rx: Receiver<ChannelStatus>,
}

impl ControlChannel {
    pub fn new() -> Self {
        let (tx, rx): (Sender<Command>, Receiver<Command>) = mpsc::channel();
        let (status_tx, status_rx): (Sender<ChannelStatus>, Receiver<ChannelStatus>) =
            mpsc::channel();
        let controllers = Arc::new(Mutex::new(Vec::new()));

        let controller_clone = controllers.clone();
        let status_tx_clone = status_tx.clone();

        thread::spawn(move || loop {
            match rx.recv() {
                Ok(msg) => match msg {
                    Command::ProbeControllersOnSerials => {
                        let ports = serialport::available_ports().unwrap();
                        for p in ports {
                            let _ = status_tx_clone
                                .send(ChannelStatus::ProbingControllers(p.clone().port_name))
                                .unwrap();

                            if let Some(controller) = probe_controller_on_serial_port(p.clone()) {
                                let mut controller_lock = controller_clone.lock().unwrap();
                                if controller_lock
                                    .iter()
                                    .find(|x: &&Controller| x.serial_path == p.port_name)
                                    .is_none()
                                {
                                    controller_lock.push(controller);
                                }
                                drop(controller_lock);
                            }
                        }
                        let _ = status_tx_clone.send(ChannelStatus::Done);
                    }
                },
                Err(_) => {
                    break;
                }
            }
        });

        Self {
            sender: tx,
            status_rx,
            controllers,
            last_status: ChannelStatus::Done,
        }
    }

    pub fn discover_controllers(&self) {
        self.sender
            .send(Command::ProbeControllersOnSerials)
            .unwrap();
    }

    pub fn get_controllers(&self) -> Vec<Controller> {
        let lock = self.controllers.try_lock();
        match lock {
            Ok(t) => return t.clone(),
            Err(err) => {
                log::trace!("cant lock: {err}");
                return Vec::new();
            }
        }
    }

    pub fn status(&mut self) -> ChannelStatus {
        if let Ok(message) = self.status_rx.try_recv() {
            self.last_status = message;
        }

        self.last_status.clone()
    }
}

pub fn probe_controller_on_serial_port(p: SerialPortInfo) -> Option<Controller> {
    let request_json = serde_json::to_string(&protocol::Request::GetName).unwrap();
    log::trace!("request: {}", request_json);
    if let Ok(mut port) = serialport::new(p.port_name.clone(), 115200)
        .timeout(Duration::from_millis(5000))
        .open()
    {
        let mut fails = 0;
        port.write_all(format!("{request_json}\n").as_bytes()).unwrap();
        loop {
            let mut reader = BufReader::new(port.try_clone().expect("Failed to clone port"));
            let mut response_string = String::new();
            log::trace!("probing: {:?} len: {:?}", port.name(), reader.buffer().len());
            match reader.read_line(&mut response_string) {
                Ok(_) => {
                    log::trace!("{:?}: {}", port.name(), response_string);
                    if let Ok(name) = serde_json::from_str(&response_string) {
                        return Some(Controller {
                            name,
                            serial_path: port.name().unwrap(),
                        });
                    } else {
                        log::warn!("invalid json sequence from: {}", p.port_name);
                        return None;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    log::warn!("timeout... continue reading!!!");
                }
                Err(err) => {
                    fails += 1;
                    if fails >= 2 {
                        log::warn!("fail to read name from: {} error: {}", p.port_name, err);
                        break;
                    }
                }
            }
        }
    } else {
        log::error!("CANT OPEN PORT!!!");
    }

    None
}
