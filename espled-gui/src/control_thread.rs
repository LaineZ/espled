use std::{
    collections::HashMap,
    fmt::write,
    io::{BufRead, BufReader, Read},
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc,
    },
    thread,
    time::Duration,
};

use protocol::{ParameterTypes, Request};
use serde::de::DeserializeOwned;
use serialport::{self, SerialPort, SerialPortInfo};
use std::sync::Mutex;

#[derive(Debug, PartialEq)]
pub enum Command {
    ProbeControllersOnSerials,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ChannelStatus {
    ProbingControllers(String),
    NoControllers,
    Done,
}

#[derive(Clone, Debug)]
pub struct Controller {
    pub name: String,
    pub serial_port: SerialPortInfo,
    pub options: HashMap<String, ParameterTypes>,
    pub effect_list: Vec<String>,
    selected_effect: String,
}

impl Controller {
    pub fn set_effect<S: Into<String>>(&mut self, effect_name: S) {
        self.selected_effect = effect_name.into();
        let index = self
            .effect_list
            .iter()
            .position(|x| *x == self.selected_effect)
            .unwrap_or_default();
        let _: Option<bool> = serial_request(self.serial_port.clone(), &Request::SetEffect(index));
        self.options =
            serial_request(self.serial_port.clone(), &protocol::Request::GetParameters).unwrap();
    }


    pub fn get_effect(&self) -> String {
        self.selected_effect.clone()
    }

    pub fn set_options(&self) {
        for (option, value) in self.options.iter() {
            let _: Option<bool> = serial_request(
                self.serial_port.clone(),
                &Request::SetOption(option.clone(), *value),
            );
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
            ChannelStatus::NoControllers => {
                write!(f, "No COM/Serial Ports controllers are present in system")
            }
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
                                    .find(|x: &&Controller| x.serial_port == p)
                                    .is_none()
                                {
                                    controller_lock.push(controller);
                                }
                                drop(controller_lock);
                            }
                        }
                        let controller_lock = controller_clone.lock().unwrap();
                        if controller_lock.is_empty() {
                            status_tx_clone.send(ChannelStatus::NoControllers).unwrap();
                        } else {
                            status_tx_clone.send(ChannelStatus::Done).unwrap();
                        }
                        drop(controller_lock);
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

    pub fn acknown_status(&mut self) {
        self.last_status = ChannelStatus::Done;
    }

    pub fn status(&mut self) -> ChannelStatus {
        if let Ok(message) = self.status_rx.try_recv() {
            self.last_status = message;
        }

        self.last_status.clone()
    }
}

fn serial_request<T>(p: SerialPortInfo, request: &impl serde::Serialize) -> Option<T>
where
    T: DeserializeOwned,
{
    let request_json = serde_json::to_string(request).unwrap();
    log::debug!("← {}", request_json);

    if let Ok(mut port) = serialport::new(p.port_name.clone(), 115200)
        .timeout(Duration::from_millis(5000))
        .open()
    {
        let mut fails = 0;
        port.write_all(format!("{request_json}\n").as_bytes())
            .unwrap();

        loop {
            let mut reader = BufReader::new(port.try_clone().expect("Failed to clone port"));
            let mut response_string = String::new();

            match reader.read_line(&mut response_string) {
                Ok(_) => {
                    log::debug!("→ {}: {}", port.name().unwrap_or_default(), response_string);

                    if let Ok(response) = serde_json::from_str::<T>(&response_string) {
                        return Some(response);
                    } else {
                        log::warn!(
                            "invalid JSON sequence from: {}, excepted json - got: {}",
                            p.port_name,
                            response_string
                        );
                        return None;
                    }
                }
                Err(err) => {
                    fails += 1;
                    if fails >= 2 {
                        log::warn!("failed to read from: {} error: {}", p.port_name, err);
                        break;
                    }
                }
            }
        }
    } else {
        log::error!("Cannot open port: {}", p.port_name);
    }

    None
}

pub fn probe_controller_on_serial_port(p: SerialPortInfo) -> Option<Controller> {
    let name: String = serial_request(p.clone(), &protocol::Request::GetName)?;
    let options: HashMap<String, ParameterTypes> =
        serial_request(p.clone(), &protocol::Request::GetParameters)?;
    let selected_effect: String = serial_request(p.clone(), &protocol::Request::GetEffect)?;
    let effect_list: Vec<String> = serial_request(p.clone(), &protocol::Request::GetEffects)?;

    Some(Controller {
        name,
        options,
        selected_effect,
        effect_list,
        serial_port: p.clone(),
    })
}
