use std::fmt;

pub enum Error {
    IO(std::io::Error),
    Unknown,
    DivisionByZero,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

pub struct GovSetError;
pub struct SpeedSetError;

pub struct GovGetError;
pub struct SpeedGetError;

pub struct TempGetError;

impl fmt::Display for GovSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Didn't have permission to set the governor, try running with sudo"
        )
    }
}

impl fmt::Display for SpeedSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Didn't have permission to set the speed, try running with sudo"
        )
    }
}

impl fmt::Display for GovGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not get the governor, possibly incompatible cpu.
Give us a bug report by opening an issue at
https://github.com/JakeRoggenbuck/auto-clock-speed/issues/new/choose"
        )
    }
}

impl fmt::Display for SpeedGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not get the speed, possibly incompatible cpu.
Give us a bug report by opening an issue at
https://github.com/JakeRoggenbuck/auto-clock-speed/issues/new/choose"
        )
    }
}

impl fmt::Display for TempGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Could not get the temperature, possibly incompatible cpu or system.
Give us a bug report by opening an issue at
https://github.com/JakeRoggenbuck/auto-clock-speed/issues/new/choose"
        )
    }
}

impl fmt::Debug for GovSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl fmt::Debug for SpeedSetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl fmt::Debug for GovGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl fmt::Debug for SpeedGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}

impl fmt::Debug for TempGetError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ file: {}, line: {} }}", file!(), line!())
    }
}
