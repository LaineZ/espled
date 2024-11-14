pub mod serial_configuration;
use std::io::BufRead;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::{delay, gpio, prelude::*};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::ClientConfiguration;
use serial_configuration::parse_argument;
use server::Server;

pub mod rgb;
pub mod server;
pub mod rgbcontrol;

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
    
    let mut nvs_handle = EspNvs::new(nvs.clone(), "wifi", true)?;
    let mut nvs_handle_rgb = EspNvs::new(nvs.clone(), "wifi", true)?;
    loop {
        FreeRtos::delay_ms(10);
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();

        match handle.read_line(&mut buffer) {
            Ok(_) => {
                let mut split = buffer.split_whitespace();
                let command = split.next().unwrap_or_default();
                let trailing = split.collect::<Vec<&str>>().join(" ");
                let command_result: anyhow::Result<()> = match command {
                    "wifi" | "wifisetup" | "ws" => {
                        let ssid = parse_argument(trailing.as_str(), 0)?;
                        let password = parse_argument(trailing.as_str(), 1)?;
                        nvs_handle.set_str("ssid", &ssid)?;
                        nvs_handle.set_str("password", &password)?;
                        println!("Settings saved! Please restart device to apply changes!");
                        Ok(())
                    }
                    "rgbcolor" | "color" => {
                        let color = u32::from_str_radix(&parse_argument(trailing.as_str(), 0)?, 16)?;

                        Ok(())
                    }
                    _ => {
                        anyhow::bail!("Unknown command {command}")
                    }
                };

                if let Err(error) = command_result {
                    println!("Error executing command {command}: {error}");
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
