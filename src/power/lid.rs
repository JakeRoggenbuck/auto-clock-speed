use crate::create_issue;
use crate::power::get_best_path;
use crate::Error;
use std::any::Any;
use std::cmp::PartialEq;
use std::fmt;
use std::fs;

const LID_STATUS_PATH: [&str; 4] = [
    "/proc/acpi/button/lid/LID/state",
    "/proc/acpi/button/lid/LID0/state",
    "/proc/acpi/button/lid/LID1/state",
    "/proc/acpi/button/lid/LID2/state",
];

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

#[derive(PartialEq, Eq)]
pub enum LidState {
    Open,
    Closed,
    Unapplicable,
    Unknown,
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
