#![feature(try_blocks)]

pub mod serial_configuration;
use std::io::{BufRead, Read};
use std::sync::{Arc, Mutex};

use effects::ParameterTypes;
use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::ledc::config::TimerConfig;
use esp_idf_hal::ledc::{LedcDriver, LedcTimerDriver};
use esp_idf_hal::prelude::*;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::ClientConfiguration;
use serde::{Deserialize, Serialize};
use server::Server;

use crate::rgbcontrol::RgbControl;

pub mod effects;
pub mod rgb;
pub mod rgbcontrol;
pub mod server;

const NAME: &str = "LentO'Chka";


#[derive(Serialize, Deserialize, Debug)]
pub enum Request {
    GetEffects,
    GetEffect,
    GetParameters,
    GetName,
    SetEffect(usize),
    SetOption(String, ParameterTypes),
}

fn nvs_get_string(key: &str, nvs: EspNvsPartition<NvsDefault>) -> String {
    let mut buffer: [u8; 128] = [0; 128];
    let nvs_wifi = EspNvs::new(nvs, "wifi", true).unwrap();
    let stro4ka = nvs_wifi.get_str(key, &mut buffer).unwrap_or_default();
    return stro4ka.unwrap_or_default().to_string();
}

fn main() -> anyhow::Result<()> {
    esp_idf_hal::sys::link_patches();
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;
    let peripherals = Peripherals::take()?;

    // led initialization
    let timer_config = TimerConfig::new().frequency(25.kHz().into());

    let ledc_timer_driver_b = LedcTimerDriver::new(peripherals.ledc.timer0, &timer_config)?;
    let ledc_timer_driver_r = LedcTimerDriver::new(peripherals.ledc.timer1, &timer_config)?;
    let ledc_timer_driver_g = LedcTimerDriver::new(peripherals.ledc.timer2, &timer_config)?;

    let channel_b = LedcDriver::new(
        peripherals.ledc.channel0,
        ledc_timer_driver_b,
        peripherals.pins.gpio2,
    )?;

    let channel_r = LedcDriver::new(
        peripherals.ledc.channel1,
        ledc_timer_driver_r,
        peripherals.pins.gpio3,
    )?;

    let channel_g = LedcDriver::new(
        peripherals.ledc.channel2,
        ledc_timer_driver_g,
        peripherals.pins.gpio10,
    )?;

    let controller = Arc::new(Mutex::new(RgbControl::new(
        channel_r,
        channel_g,
        channel_b,
        nvs.clone(),
    )));

    let mut controller_lock = controller.lock().unwrap();
    controller_lock.init()?;
    drop(controller_lock);

    let mut server = Server::new(sys_loop.clone(), peripherals.modem)?;
    server
        .connect(
            sys_loop,
            ClientConfiguration {
                ssid: nvs_get_string("ssid", nvs.clone())
                    .as_str()
                    .try_into()
                    .unwrap_or_default(),
                password: nvs_get_string("password", nvs.clone())
                    .as_str()
                    .try_into()
                    .unwrap_or_default(),
                ..Default::default()
            },
        )
        .unwrap_or_else(|op| println!("Wi-Fi connection error: {op}. I sorry about that..."));

    server.handle_response(controller.clone())?;

    let mut nvs_handle = EspNvs::new(nvs.clone(), "wifi", true)?;
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();
    let mut buffer = Vec::new();
    let mut byte = [0u8; 1];

    loop {
        match handle.read_exact(&mut byte) {
            Ok(_) => {
                println!("Read byte: {:?}", byte[0]);
                if byte[0] == 0 {
                    if let Ok(json) = String::from_utf8(buffer.clone()) {
                        match serde_json::from_str::<Request>(&json) {
                            Ok(data) => {
                                let controller = controller.clone();
                                let mut controller_lock = controller.lock().unwrap();
                                match data {
                                    Request::GetEffects => {
                                        let json = serde_json::to_string(&controller_lock.get_effects_name()).unwrap();
                                        println!("{json}\0");
                                    },
                                    Request::GetEffect => {
                                        let json = serde_json::to_string(&controller_lock.get_effect_name()).unwrap();
                                        println!("{json}\0");
                                    },
                                    Request::GetParameters => {
                                        let json = serde_json::to_string(&controller_lock.get_effect_options()).unwrap();
                                        println!("{json}\0");
                                    },
                                    Request::GetName => {
                                        let json = serde_json::to_string(&NAME).unwrap();
                                        println!("{json}\0");
                                    },
                                    Request::SetEffect(index) => {
                                        let json = serde_json::to_string(&controller_lock.set_effect(index)).unwrap();
                                        println!("{json}\0");
                                    },
                                    Request::SetOption(name, parameter_type) => {
                                        // TODO: error handling for this request
                                        let json = serde_json::to_string(&true).unwrap();
                                        controller_lock.set_effect_parameter(&name, parameter_type);
                                        println!("{json}\0"); 
                                    },
                                }
                            },
                            Err(e) => eprintln!("Failed to parse JSON: {}", e),
                        }
                    } else {
                        eprintln!("Invalid UTF-8 sequence");
                    }
                    buffer.clear();
                } else {
                    buffer.push(byte[0]);
                }
            }
            Err(e) => match e.kind() {
                std::io::ErrorKind::WouldBlock
                | std::io::ErrorKind::TimedOut
                | std::io::ErrorKind::Interrupted => {
                    log::info!("Error: {e}\r\n");
                    FreeRtos::delay_ms(10);
                    continue;
                }
                _ => {
                    log::info!("Error: {e}\r\n");
                    continue;
                }
            },
        }
    }
}
