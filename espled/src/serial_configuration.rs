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

pub fn parse_argument<S: AsRef<str>>(
    input: S,
    at: usize,
) -> std::result::Result<String, ArguementMissingError> {
    let split = input.as_ref().split_whitespace().nth(at);

    if let Some(s) = split {
        Ok(s.to_string())
    } else {
        Err(ArguementMissingError(at))
    }
}
