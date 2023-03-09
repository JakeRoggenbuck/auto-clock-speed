extern crate chrono;

use efcl::{color, Color};
use std::fmt;
use std::time::SystemTime;

use chrono::prelude::DateTime;
use chrono::Utc;

pub enum Severity {
    Error,
    Warning,
    Log,
}

pub trait Interface {
    fn log(&mut self, msg: &str, sev: Severity);
}

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

        let time = DateTime::<Utc>::from(self.timestamp).format("%Y-%m-%d %H:%M:%S");
        write!(f, "{} {} -> {}", severity, time, self.message)
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
