use std::fs;
use std::path::Path;

use super::Error;

pub mod battery;
pub mod lid;

/// Called once at the start of read_power_source
fn set_best_path() -> Option<&'static str> {
    // Only loaded once
    static POWER_SOURCE_PATH: [&str; 4] = [
        "/sys/class/power_supply/AC/online",
        "/sys/class/power_supply/AC0/online",
        "/sys/class/power_supply/AC1/online",
        "/sys/class/power_supply/ACAD/online",
    ];

    // Find if any AC power path exists
    for path in POWER_SOURCE_PATH {
        if Path::new(path).exists() {
            // Mutate Power struct and leave
            return Some(path);
        }
    }

    None
}

pub struct Power {
    pub best_path: &'static str,
    found_path: bool,
}

pub trait PowerRetriever {
    fn new() -> Self;
    fn read_power_source(&self) -> Result<bool, Error>;
}

impl PowerRetriever for Power {
    fn new() -> Self {
        if let Some(path) = set_best_path() {
            Power {
                best_path: path,
                found_path: true,
            }
        } else {
            Power {
                best_path: "",
                found_path: false,
            }
        }
    }

    fn read_power_source(&self) -> Result<bool, Error> {
        if !self.found_path {
            return Err(Error::HdwNotFound);
        }

        let mut pwr_str = fs::read_to_string(self.best_path)?;

        // Remove the \n char
        pwr_str.pop();

        Ok(pwr_str == "1")
    }
}
