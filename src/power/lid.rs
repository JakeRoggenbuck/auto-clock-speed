use crate::Error;
use std::cmp::PartialEq;
use std::fmt;
use std::fs;
use std::path::Path;

#[derive(PartialEq, Eq)]
pub enum LidState {
    Open,
    Closed,
    Unapplicable,
    Unknown,
}

fn set_best_path() -> Option<&'static str> {
    static LID_STATUS_PATH: [&str; 4] = [
        "/proc/acpi/button/lid/LID/state",
        "/proc/acpi/button/lid/LID0/state",
        "/proc/acpi/button/lid/LID1/state",
        "/proc/acpi/button/lid/LID2/state",
    ];

    // Find if any AC power path exists

    LID_STATUS_PATH
        .iter()
        .find(|&path| Path::new(path).exists());

    None
}

pub struct Lid {
    pub best_path: &'static str,
    found_path: bool,
}

pub trait LidRetriever {
    fn new() -> Self;
    fn read_lid_state(&self) -> Result<LidState, Error>;
}

impl LidRetriever for Lid {
    fn new() -> Self {
        if let Some(lid) = set_best_path() {
            Lid {
                best_path: lid,
                found_path: true,
            }
        } else {
            Lid {
                best_path: "",
                found_path: false,
            }
        }
    }

    fn read_lid_state(&self) -> Result<LidState, Error> {
        if !self.found_path {
            return Ok(LidState::Unapplicable);
        }

        let lid_str = fs::read_to_string(self.best_path)?;

        let state = if lid_str.contains("open") {
            LidState::Open
        } else if lid_str.contains("closed") {
            LidState::Closed
        } else {
            LidState::Unknown
        };
        Ok(state)
    }
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
