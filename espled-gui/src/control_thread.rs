use std::{
    fmt::write, io::{BufRead, BufReader}, sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    }, thread, time::Duration
};

use serialport::{self, SerialPortInfo};
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
            ChannelStatus::ProbingControllers(serial_name) => write!(f, "Probing controller on address: {serial_name}"),
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
                                if controller_lock.iter().find(|x: &&Controller| x.serial_path == p.port_name).is_none() {
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
    if let Ok(mut port) = serialport::new(p.port_name.clone(), 115200)
        .timeout(Duration::from_millis(1000))
        .open()
    {
        let _ = port.write_all("name\n".as_bytes());
        let mut fails = 0;

        loop {
            let mut reader = BufReader::new(port.try_clone().expect("Failed to clone port"));
            let mut response = String::new();
            let _ = port.write("name\n".as_bytes());
            match reader.read_line(&mut response) {
                Ok(_) => {
                    log::trace!("{response}");
                    if response.contains("ame:") {
                        let device_name = response.split(':').last().unwrap_or("");
                        let device_name_clean = device_name.replace("\0", "").trim().to_string();
                        if !device_name_clean.is_empty() && device_name_clean != "" {
                            log::debug!("found controller: {:?}", p.port_name);
                            return Some(Controller {
                                name: device_name_clean,
                                serial_path: port.name().unwrap_or_default(),
                            });
                        }
                    }
                }
                Err(_) => {
                    fails += 1;
                    if fails > 2 {
                        log::warn!("fail to read name from: {}", p.port_name);
                        return None;
                    }
                }
            }
        }
    }

    None
}
