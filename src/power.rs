use super::create_issue;
use super::Error;
use std::any::Any;
use std::cmp::PartialEq;
use std::fmt;
use std::fs::{read_dir, File};
use std::io::Read;
use std::path::Path;

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

pub fn has_battery() -> Result<bool, Error> {
    let power_dir = Path::new("/sys/class/power_supply/");
    let dir = read_dir(power_dir)?;
    Ok(dir
        .into_iter()
        .map(|x| x.unwrap().path().to_str().unwrap().to_string())
        .collect::<String>()
        .len()
        > 0)
}

pub fn read_lid_state() -> Result<LidState, Error> {
    let lid_status_path: Vec<&str> = vec![
        "/proc/acpi/button/lid/LID/state",
        "/proc/acpi/button/lid/LID0/state",
        "/proc/acpi/button/lid/LID1/state",
        "/proc/acpi/button/lid/LID2/state",
    ];

    let path: &str = match get_best_path(lid_status_path) {
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

    let mut lid_str: String = String::new();
    File::open(path)?.read_to_string(&mut lid_str)?;

    let state = if lid_str.contains("open") {
        LidState::Open
    } else if lid_str.contains("closed") {
        LidState::Closed
    } else {
        LidState::Unknown
    };

    Ok(state)
}

pub fn read_battery_charge() -> Result<i8, Error> {
    let battery_charge_path: Vec<&str> = vec![
        "/sys/class/power_supply/BAT/capacity",
        "/sys/class/power_supply/BAT0/capacity",
        "/sys/class/power_supply/BAT1/capacity",
        "/sys/class/power_supply/BAT2/capacity",
    ];

    let path: &str = match get_best_path(battery_charge_path) {
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

    let mut cap_str: String = String::new();
    File::open(path)?.read_to_string(&mut cap_str)?;

    // Remove the \n char
    cap_str.pop();

    Ok(cap_str.parse::<i8>().unwrap())
}

pub fn read_power_source() -> Result<bool, Error> {
    let power_source_path: Vec<&str> = vec![
        "/sys/class/power_supply/AC/online",
        "/sys/class/power_supply/AC0/online",
        "/sys/class/power_supply/AC1/online",
        "/sys/class/power_supply/ACAD/online",
    ];

    let path: &str = match get_best_path(power_source_path) {
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

    let mut pwr_str: String = String::new();
    File::open(path)?.read_to_string(&mut pwr_str)?;

    // Remove the \n char
    pwr_str.pop();

    return Ok(pwr_str == "1");
}

pub fn get_best_path(paths: Vec<&str>) -> Result<&str, Error> {
    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(path);
        }
    }

    return Err(Error::Unknown);
}
