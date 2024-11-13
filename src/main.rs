pub mod serial_configuration;
use std::io::BufRead;

use esp_idf_hal::delay::FreeRtos;
use esp_idf_hal::{delay, gpio, prelude::*};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::nvs::{EspDefaultNvsPartition, EspNvs, EspNvsPartition, NvsDefault};
use esp_idf_svc::wifi::ClientConfiguration;
use serial_configuration::execute_command;
use server::Server;

pub mod server;
pub mod rgb;

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

    server.connect(sys_loop, ClientConfiguration {
        ssid: nvs_get_string("ssid", nvs.clone()).as_str().try_into().unwrap_or_default(),
        password: nvs_get_string("password", nvs.clone()).as_str().try_into().unwrap_or_default(),
        ..Default::default()
    }).unwrap_or_else(|op| println!("Wi-Fi connection error: {op}. I sorry about that..."));

    loop {
        FreeRtos::delay_ms(10);
        let mut buffer = String::new();
        let stdin = std::io::stdin();
        let mut handle = stdin.lock();

        match handle.read_line(&mut buffer) {
            Ok(_) => {
                if let Err(error) = execute_command(buffer, nvs.clone()) {
                    println!("Error occured while executing the command: {error}");
                }
            }
            Err(e) => {
                match e.kind() {
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
                }
            }
        }
    }
}
