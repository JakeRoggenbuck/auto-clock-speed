use crate::create_issue;
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

pub struct Lid {
    best_path: &'static str,
}

trait Retriever {
    fn set_best_path(&mut self) -> Result<(), Error>;
    fn read_lid_state(&self) -> Result<LidState, Error>;
}

impl Retriever for Lid {
    fn set_best_path(&mut self) -> Result<(), Error> {
        static lid_status_path: [&str; 4] = [
            "/proc/acpi/button/lid/LID/state",
            "/proc/acpi/button/lid/LID0/state",
            "/proc/acpi/button/lid/LID1/state",
            "/proc/acpi/button/lid/LID2/state",
        ];

        // Find if any AC power path exists
        for path in lid_status_path.iter() {
            if Path::new(path).exists() {
                // Mutate Power struct and leave
                self.best_path = path;
                return Ok(());
            }
        }

        Err(Error::HdwNotFound)
    }

    fn read_lid_state(&self) -> Result<LidState, Error> {
        match self.set_best_path() {
            Ok(path) => path,
            Err(error) => {
                eprintln!("Could not detect your lid state.");
                create_issue!("If you are on a laptop");
                return Ok(LidState::Unapplicable);
            }
        };

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
