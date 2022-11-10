use std::fs;
use std::path::Path;

use super::Error;

pub mod battery;
pub mod lid;

pub struct Power {
    best_path: &'static str,
}

trait Retriever {
    fn set_best_path(&mut self) -> Result<(), Error>;
    fn read_power_source(&self) -> Result<bool, Error>;
}

impl Retriever for Power {
    /// Called once at the start of read_power_source
    fn set_best_path(&mut self) -> Result<(), Error> {
        // Only loaded once
        static power_source_path: [&str; 4] = [
            "/sys/class/power_supply/AC/online",
            "/sys/class/power_supply/AC0/online",
            "/sys/class/power_supply/AC1/online",
            "/sys/class/power_supply/ACAD/online",
        ];

        // Find if any AC power path exists
        for path in power_source_path.iter() {
            if Path::new(path).exists() {
                // Mutate Power struct and leave
                self.best_path = path;
                return Ok(());
            }
        }

        Err(Error::HdwNotFound)
    }

    fn read_power_source(&self) -> Result<bool, Error> {
        // Set self.best_path or HdwNotFound
        match self.set_best_path() {
            Ok(a) => a,
            Err(e) => return Err(e),
        };

        let mut pwr_str = fs::read_to_string(self.best_path)?;

        // Remove the \n char
        pwr_str.pop();

        Ok(pwr_str == "1")
    }
}
