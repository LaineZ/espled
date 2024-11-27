use std::{
    sync::{mpsc::{self, Receiver, Sender}, Arc}, thread, time::Duration
};

use std::sync::Mutex;
use serialport;

#[derive(Debug, PartialEq)]
pub enum Command {
    ProbeControllersOnSerials,
}


#[derive(Debug, PartialEq)]
pub enum ChannelStatus {
    ProbingControllers,
    Done
}

#[derive(Clone)]
pub struct Controller {
    pub name: String,
    pub serial_path: String
}

impl std::fmt::Display for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}


impl std::fmt::Display for ChannelStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChannelStatus::ProbingControllers => write!(f, "Probing controllers on serial ports in progress..."),
            ChannelStatus::Done => write!(f, "Done"),
            _ => write!(f, "Please wait..")
        }
    }
}

pub struct ControlChannel {
    sender: Sender<Command>,
    controllers: Arc<Mutex<Vec<Controller>>>,
    status_rx: Receiver<ChannelStatus>
}

impl ControlChannel {
    pub fn new() -> Self {
        let (tx, rx): (Sender<Command>, Receiver<Command>) = mpsc::channel();
        let (status_tx, status_rx): (Sender<ChannelStatus>, Receiver<ChannelStatus>) = mpsc::channel();
        let controllers = Arc::new(Mutex::new(Vec::new()));

        let controller_clone = controllers.clone();
        let status_tx_clone = status_tx.clone();

        thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(msg) => {
                        match msg {
                            Command::ProbeControllersOnSerials => {
                                let _ = status_tx_clone.send(ChannelStatus::ProbingControllers).unwrap();
                                let controller_list = probe_controllers_on_serial_ports();
                                let mut controller_lock = controller_clone.lock().unwrap();
                                *controller_lock = controller_list;
                                drop(controller_lock);
                                let _ = status_tx_clone.send(ChannelStatus::Done);
                            }
                        }
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
        });

        Self {
            sender: tx,
            status_rx,
            controllers,
        }
    }


    pub fn discover_controllers(&self) {
        self.sender.send(Command::ProbeControllersOnSerials).unwrap();
    }

    pub fn get_controllers(&self) -> Vec<Controller> {
        let lock = self.controllers.try_lock();
        match lock {
            Ok(t) => return t.clone(),
            Err(_) => return Vec::new()
        }
    } 

    pub fn status(&self) -> ChannelStatus {
        if let Ok(message) = self.status_rx.try_recv() {
            return message
        } else {
            return ChannelStatus::Done
        }
    }
}

pub fn probe_controllers_on_serial_ports() -> Vec<Controller> {
    let ports = serialport::available_ports().unwrap();
    let mut espshki = Vec::new();
    for p in ports {
        if let Ok(mut port) = serialport::new(p.port_name.clone(), 115200)
            .timeout(Duration::from_millis(1000))
            .open()
        {
            let mut clone = port.try_clone().expect("Failed to clone!");
            std::thread::spawn(move || clone.write("name\n".as_bytes()).unwrap());
            let mut fails = 0;
            loop {
                let mut buffer: [u8; 255] = [0; 255];
                match port.read(&mut buffer) {
                    Ok(_) => {
                        let string = String::from_utf8_lossy(&buffer).into_owned();
                        let string = string.trim();
                        //println!("{string}");
                        if string.contains("ame:") {
                            let split = string.split(":");
                            let device_name = split.last().unwrap_or("name");
                            if device_name != "name" {
                                let device_name_normal = device_name.replace("\0", "").trim().to_string();
                                espshki.push(Controller {
                                    name: device_name_normal,
                                    serial_path: port.name().unwrap_or_default()
                                });
                                break;
                            }
                        }
                    }

                    Err(_) => {}
                }

                fails += 1;

                if fails > 5 {
                    println!("WARN: Failure to read name from: {}", p.port_name);
                    break;
                }
            }
        }
    }

    espshki
}
