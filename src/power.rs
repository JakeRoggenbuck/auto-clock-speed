use std::fmt;
use super::Error;
use std::fs::File;
use std::path::Path;
use std::io::{Read};

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

pub fn read_lid_state() -> Result<LidState, Error> {
    if !Path::new("/proc/acpi/button/lid/LID0/state").exists() {
        return Ok(LidState::Unapplicable);
    }

    let mut lid_str: String = String::new();
    File::open("/proc/acpi/button/lid/LID0/state")?.read_to_string(&mut lid_str)?;
    
    if lid_str.contains("open") {
        return Ok(LidState::Open)
    } else if lid_str.contains("closed") {
        return Ok(LidState::Closed)
    }

    Ok(LidState::Unknown)
}
