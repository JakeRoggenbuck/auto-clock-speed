use std::any::Any;
use std::cmp::PartialEq;
use std::fmt;
use std::fs;
use std::path::Path;

use super::create_issue;
use super::Error;

pub mod battery;

const LID_STATUS_PATH: [&str; 4] = [
    "/proc/acpi/button/lid/LID/state",
    "/proc/acpi/button/lid/LID0/state",
    "/proc/acpi/button/lid/LID1/state",
    "/proc/acpi/button/lid/LID2/state",
];

const POWER_SOURCE_PATH: [&str; 4] = [
    "/sys/class/power_supply/AC/online",
    "/sys/class/power_supply/AC0/online",
    "/sys/class/power_supply/AC1/online",
    "/sys/class/power_supply/ACAD/online",
];

#[derive(PartialEq, Eq)]
pub enum LidState {
    Open,
    Closed,
    Unapplicable,
    Unknown,
}

impl fmt::Display for LidState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LidState::Open => write!(f, "open"),
            LidState::Closed => write!(f, "closed"),
            LidState::Unapplicable => write!(f, "unapplicable"),
            LidState::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn get_best_path(paths: [&'static str; 4]) -> Result<&str, Error> {
    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(path);
        }
    }

    Err(Error::Unknown)
}

pub fn read_lid_state() -> Result<LidState, Error> {
    let path: &str = match get_best_path(LID_STATUS_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("Could not detect your lid state.");
            create_issue!("If you are on a laptop");
            return Ok(LidState::Unapplicable);
        }
    };

    let lid_str = fs::read_to_string(path)?;

    let state = if lid_str.contains("open") {
        LidState::Open
    } else if lid_str.contains("closed") {
        LidState::Closed
    } else {
        LidState::Unknown
    };

    Ok(state)
}

pub fn read_power_source() -> Result<bool, Error> {
    let path: &str = match get_best_path(POWER_SOURCE_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("We could not detect your AC power source.");
            create_issue!("If you have a power source");
            return Ok(true);
        }
    };

    let mut pwr_str = fs::read_to_string(path)?;

    // Remove the \n char
    pwr_str.pop();

    Ok(pwr_str == "1")
}
