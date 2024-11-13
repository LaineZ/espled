use std::{error::Error, fmt};

use esp_idf_svc::nvs::{EspNvs, EspNvsPartition, NvsDefault};

#[derive(Debug, Clone)]
pub struct ArguementMissingError(pub usize);


impl Error for ArguementMissingError {}
impl fmt::Display for ArguementMissingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "argument at position {} is missing", self.0)
    }
}

fn parse_argument<S: AsRef<str>>(input: S, at: usize) -> std::result::Result<String, ArguementMissingError> {
    let split = input.as_ref().split_whitespace().nth(at);

    if let Some(s) = split {
        Ok(s.to_string())
    } else {
        Err(ArguementMissingError(at))
    }
}

pub fn execute_command<S: AsRef<str>>(input: S, nvs: EspNvsPartition<NvsDefault>) -> anyhow::Result<()> {
    let mut split = input.as_ref().split_whitespace();
    let command = split.next().unwrap_or_default();
    let trailing = split.collect::<Vec<&str>>().join(" ");
    match command {
        "wifi" | "wifisetup" | "ws" => {
            let mut nvs_handle = EspNvs::new(nvs, "wifi", true)?;
            let ssid = parse_argument(trailing.as_str(), 0)?;
            let password = parse_argument(trailing.as_str(), 1)?;
            nvs_handle.set_str("ssid", &ssid)?;
            nvs_handle.set_str("password", &password)?;
            println!("Settings saved! Please restart device to apply changes!"); 
        },
        _ => {
            println!("Unknown command {command}")
        }
    }

    Ok(())
}
