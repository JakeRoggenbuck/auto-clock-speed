extern crate chrono;

use chrono::prelude::DateTime;
use chrono::Utc;

use std::fmt;
use std::time::SystemTime;
use termion::{color, style};

pub enum Severity {
    Error, // There is a problem that causing the program to be unstable to be unable to run properly
    Warning, // There is a problem that might cause the program to be unstable or to be unable to run properly
    Log,     // There is information to note
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
