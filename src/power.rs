use std::any::Any;
use std::fs;
use std::path::Path;

use super::Error;

pub mod battery;
pub mod lid;

const POWER_SOURCE_PATH: [&str; 4] = [
    "/sys/class/power_supply/AC/online",
    "/sys/class/power_supply/AC0/online",
    "/sys/class/power_supply/AC1/online",
    "/sys/class/power_supply/ACAD/online",
];

pub fn get_best_path(paths: [&'static str; 4]) -> Result<&str, Error> {
    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(path);
        }
    }

    Err(Error::Unknown)
}

pub fn read_power_source() -> Result<bool, Error> {
    let path: &str = match get_best_path(POWER_SOURCE_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            return Err(Error::HdwNotFound);
        }
    };

    let mut pwr_str = fs::read_to_string(path)?;

    // Remove the \n char
    pwr_str.pop();

    Ok(pwr_str == "1")
}
