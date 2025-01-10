//! This file contains a logging system for the Auto Clock Speed (ACS) project.
//!
//! It allows the program to log messages with different severity levels (error, warning, log) and display them in a human-readable format.
//!
//! The log messages contain a timestamp, a severity level and the message. The logs are stored in a vector and are serializable and deserializable. The logs can also be displayed in a human-readable format.

use chrono::{DateTime, Local};
use efcl::{color, Color};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::SystemTime;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
/// The Severity enum is used to represent the different levels of severity of a log message. It has three possible values:
///
/// - `Error` represents an error message that indicates that something went wrong.
/// - `Warning` represents a warning message that indicates that something unexpected happened but the program can still continue to run.
/// - `Log` represents a log message that contains information about the program execution.
///
///  This enum is used in the log function of the Logger struct to specify the severity level of the log message when creating a new log. It also used in the fmt function of the Log struct, where it is matched to colorize the output based on the severity level of the log message.
pub enum Severity {
    Error,
    Warning,
    Log,
}

pub trait Interface {
    fn log(&mut self, msg: &str, sev: Severity);
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
    pub message: String,
    pub severity: Severity,
    pub timestamp: SystemTime,
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let severity = match &self.severity {
            Severity::Error => color!(Color::RED, "error:"),
            Severity::Warning => color!(Color::YELLOW, "warn:"),
            Severity::Log => color!(Color::BLUE, "notice:"),
        };

        let local_time: DateTime<Local> = DateTime::<Local>::from(self.timestamp);

        write!(
            f,
            "{} {} -> {}",
            severity,
            local_time.format("%Y-%m-%d %H:%M:%S"),
            self.message
        )
    }
}

pub struct Logger {
    pub logs: Vec<Log>,
}

impl Interface for Logger {
    /// Create a Log with the timestamp from message and severity
    fn log(&mut self, msg: &str, sev: Severity) {
        let time = SystemTime::now();

        let loggable = Log {
            message: msg.to_string(),
            severity: sev,
            timestamp: time,
        };

        self.logs.push(loggable);
    }
}
