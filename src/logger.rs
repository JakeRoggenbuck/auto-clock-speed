extern crate chrono;

use std::fmt;
use std::time::SystemTime;

use chrono::prelude::DateTime;
use chrono::Utc;
use termion::{color, style};

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
            Severity::Error => format!(
                "{}{}error:{}",
                color::Fg(color::Red),
                style::Bold,
                style::Reset
            ),
            Severity::Warning => format!(
                "{}{}warn:{}",
                color::Fg(color::Yellow),
                style::Bold,
                style::Reset
            ),
            Severity::Log => format!(
                "{}{}notice:{}",
                color::Fg(color::Blue),
                style::Bold,
                style::Reset
            ),
        };

        let time = DateTime::<Utc>::from(self.timestamp).format("%Y-%m-%d %H:%M:%S");
        write!(f, "{} {} -> {}", severity, time, self.message)
    }
}

pub struct Logger {
    pub logs: Vec<Log>,
}

impl Interface for Logger {
    /// Create a Log from message and severity, add the timestamp
    fn log(&mut self, msg: &str, sev: Severity) {
        let time = SystemTime::now();

        let loggable = Log {
            message: msg.to_string(),
            severity: sev,
            timestamp: time,
        };

        println!("{}", &loggable);
        self.logs.push(loggable);
    }
}
