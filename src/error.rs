use std::fmt;

const BUG_REPORT: &str = "\nGive us a bug report by opening an issue at
https://github.com/JakeRoggenbuck/auto-clock-speed/issues/new/choose";

pub enum Error {
    IO(std::io::Error),
    TimeError(std::time::SystemTimeError),
    Parse,
    HdwNotFound,
    Unknown,
    DivisionByZero,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

impl From<std::time::SystemTimeError> for Error {
    fn from(e: std::time::SystemTimeError) -> Error {
        Error::TimeError(e)
    }
}

impl From<std::num::ParseIntError> for Error {
    fn from(_: std::num::ParseIntError) -> Self {
        Error::Parse
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "An error occured: {}",
            match self {
                Error::IO(e) => format!("IO Error: {}", e),
                Error::TimeError(e) => format!("Time Error: {}", e),
                Error::HdwNotFound =>
                    "Hardware component missing, operating system may not be supported".to_string(),
                Error::Unknown => "Unknown error occured".to_string(),
                Error::DivisionByZero => "Division by zero occured".to_string(),
                Error::Parse => "Failed to parse data".to_string(),
            }
        )
    }
}

pub struct GovSetError;

pub struct SpeedSetError;

pub struct GovGetError;

pub struct SpeedGetError;

pub struct TempGetError;

impl fmt::Display for GovSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Didn't have permission to set the governor, try running with sudo"
        )
    }
}

impl fmt::Display for SpeedSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(
            f,
            "Didn't have permission to set the speed, try running with sudo"
        )
    }
}

impl fmt::Display for GovGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not get the governor, possibly incompatible cpu.\n{}",
            BUG_REPORT
        )
    }
}

impl fmt::Display for SpeedGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not get the speed, possibly incompatible cpu.{}",
            BUG_REPORT
        )
    }
}

impl fmt::Display for TempGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not get the temperature, possibly incompatible cpu or system.{}",
            BUG_REPORT
        )
    }
}
