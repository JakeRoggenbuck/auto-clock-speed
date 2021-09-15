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

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"a")
    }
}
