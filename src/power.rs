use std::any::Any;
use std::cmp::PartialEq;
use std::fmt;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::Path;

use super::create_issue;
use super::Error;

const LID_STATUS_PATH: [&'static str; 4] = [
    "/proc/acpi/button/lid/LID/state",
    "/proc/acpi/button/lid/LID0/state",
    "/proc/acpi/button/lid/LID1/state",
    "/proc/acpi/button/lid/LID2/state",
];

const BATTERY_CHARGE_PATH: [&'static str; 4] = [
    "/sys/class/power_supply/BAT/capacity",
    "/sys/class/power_supply/BAT0/capacity",
    "/sys/class/power_supply/BAT1/capacity",
    "/sys/class/power_supply/BAT2/capacity",
];

const POWER_SOURCE_PATH: [&'static str; 4] = [
    "/sys/class/power_supply/AC/online",
    "/sys/class/power_supply/AC0/online",
    "/sys/class/power_supply/AC1/online",
    "/sys/class/power_supply/ACAD/online",
];

#[derive(PartialEq)]
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

    return Err(Error::Unknown);
}

pub trait Power {
    fn has_battery(&mut self) -> bool;
    fn read_lid_state(&mut self) -> Result<LidState, Error>;
    fn read_battery_charge(&mut self) -> Result<i8, Error>;
    fn read_power_source(&mut self) -> Result<bool, Error>;
}

pub struct DevicePower {
    pub _has_battery: bool,
    pub _best_lid_path: String,
    pub _best_battery_charge_path: String,
    pub _best_power_source_path: String,

    pub did_init: bool,
}

impl Power for DevicePower {
    fn has_battery(&mut self) -> bool {
        // Cache has_battery
        if !self.did_init {
            let power_dir = Path::new("/sys/class/power_supply/");
            let dir_count = read_dir(power_dir).into_iter().len();
            self._has_battery = dir_count > 0;
            self.did_init = true;
        }

        self._has_battery
    }

    fn read_lid_state(&mut self) -> Result<LidState, Error> {
        // Get and cache best_path for lid
        if !self.did_init {
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
            self._best_lid_path = path.to_string();
        }

        let mut lid_str: String = String::new();
        File::open(&self._best_lid_path)?.read_to_string(&mut lid_str)?;

        let state = if lid_str.contains("open") {
            LidState::Open
        } else if lid_str.contains("closed") {
            LidState::Closed
        } else {
            LidState::Unknown
        };

        Ok(state)
    }

    fn read_battery_charge(&mut self) -> Result<i8, Error> {
        if !self.did_init {
            let path: &str = match get_best_path(BATTERY_CHARGE_PATH) {
                Ok(path) => path,
                Err(error) => {
                    if error.type_id() == Error::IO.type_id() {
                        // Make sure to return IO error if one occurs
                        return Err(error);
                    }
                    // If it doesn't exist then it is plugged in so make it 100% percent capacity
                    eprintln!("We could not detect your battery.");
                    create_issue!("If you are on a laptop");
                    return Ok(100);
                }
            };
            self._best_battery_charge_path = path.to_string();
        }

        let mut cap_str: String = String::new();
        File::open(&self._best_battery_charge_path)?.read_to_string(&mut cap_str)?;

        // Remove the \n char
        cap_str.pop();

        Ok(cap_str.parse::<i8>().unwrap())
    }

    fn read_power_source(&mut self) -> Result<bool, Error> {
        if !self.did_init {
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
            self._best_power_source_path = path.to_string();
        }

        let mut pwr_str: String = String::new();
        File::open(&self._best_battery_charge_path)?.read_to_string(&mut pwr_str)?;

        // Remove the \n char
        pwr_str.pop();

        return Ok(pwr_str == "1");
    }
}
