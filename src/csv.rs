//! # CSV logging
//! This file is reponsible for logging data about the running daemon.
//! CSV Logging is by default disabled, however it can be enabled by passing a parameter to the
//! daemon. Upon the daemon being initialized it will generate a CSVWriter object based on the
//! settings passed in to the daemon.
//!
//! It is the responsibility of the implementation of the CSV logger to call the write and init
//! methods in order to actually log data.

use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{
    fs::OpenOptions,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use crate::settings::Settings;
use crate::{
    cpu::CPU,
    logger::{self, Interface, Logger},
};

pub const CSV_HEADER: &str = "epoch,name,number,max_freq,min_freq,cur_freq,cur_temp,cur_usage,gov";

pub struct CSVWriter {
    log_size_cutoff: i32,
    path: String,
    enabled: bool,
}

/// Generates a new instance of CSVWriten based on a settings object.
pub fn gen_writer(settings: &Settings) -> CSVWriter {
    CSVWriter {
        log_size_cutoff: settings.log_size_cutoff,
        path: settings.csv_file.to_string(),
        enabled: settings.log_csv,
    }
}

pub trait Writer {
    fn write<'a>(&mut self, writables: impl Iterator<Item = &'a dyn Writable>, logger: &mut Logger);
    fn init(&mut self, logger: &mut Logger);
}

/// A generic trait representing any object that can be converted into a CSV column
pub trait Writable {
    /// Convert this object into a column for CSV logging
    fn to_csv(&self) -> String;
}

impl Writable for CPU {
    fn to_csv(&self) -> String {
        format!(
            "{},{},{},{},{},{},{},{},{}\n",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::new(0_u64, 1_u32))
                .as_secs(),
            self.name,
            self.number,
            self.max_freq,
            self.min_freq,
            self.cur_freq,
            self.cur_temp,
            self.cur_usage,
            self.gov
        )
    }
}

impl Writer for CSVWriter {
    /// Writes out all the cpu data from the daemon to the csv file
    ///
    /// This method gets called every `daemon.settings.delay` millis or every `daemon.settings.delay_battery` millis when on battery
    ///
    /// Each time this method gets called it creates a new row in the csv file. If the csv file
    /// gets larger than `self.log_size_cutoff` MB it will cease logging.
    ///
    /// If an error occurs it will log the error to the daemon logger.
    fn write<'a>(
        &mut self,
        writables: impl Iterator<Item = &'a dyn Writable>,
        logger: &mut Logger,
    ) {
        if !self.enabled {
            return;
        }

        let lines = writables.map(|c| c.to_csv()).collect::<String>();

        // Open file in append mode
        let mut file = match OpenOptions::new().write(true).append(true).open(&self.path) {
            Ok(file) => file,
            Err(..) => {
                logger.log("Could not open file for writing.", logger::Severity::Error);
                return;
            }
        };

        // If file is smaller than log_size_cutoff
        if file.metadata().unwrap().len() < (self.log_size_cutoff * 1_000_000) as u64 {
            // Try to write the cpus
            match write!(file, "{}", lines) {
                Ok(_) => {}
                Err(..) => {
                    logger.log("Could not write to CSV file.", logger::Severity::Warning);
                }
            };
        } else {
            logger.log(
                &format!("Max log file size reached of {}MB", self.log_size_cutoff),
                logger::Severity::Warning,
            );
            // Deactivate csv logging after file size max
            self.enabled = false;
        }
    }

    /// Initializes a new csv file. If ones currently exists it will keep it. If not it will
    /// generate a new file.
    ///
    /// # Generating a new file
    ///
    /// The file will be created and the column titles will be filled in
    /// If an error occurs while generating a file it will be logged to the daemon
    fn init(&mut self, logger: &mut Logger) {
        if !self.enabled {
            return;
        }
        // If file does not exist
        if Path::new(&self.path).exists() {
            // File did exist, use it
            logger.log(
                &format!(
                    "File \"{}\" already exists, continuing in append mode.",
                    &self.path
                ),
                logger::Severity::Warning,
            );
            return;
        }

        // Try to create file
        match File::create(&self.path) {
            Ok(a) => {
                // Write header and show error if broken
                match writeln!(&a, "{}", CSV_HEADER) {
                    Ok(_) => {}
                    Err(..) => {
                        logger.log("Could not write to CSV file.", logger::Severity::Warning);
                    }
                };
            }
            // File did not get created
            Err(..) => {
                logger.log(
                    "Could not create file. Turning csv log mode off and continuing.",
                    logger::Severity::Warning,
                );
                self.enabled = false;
            }
        }
    }
}
