extern crate chrono;

use std::fmt;
use std::time::SystemTime;

use chrono::prelude::DateTime;
use chrono::Utc;
use colored::Colorize;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
    Log,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum Origin {
    Unknown,
    Daemon,
    Client,
}

pub trait Interface {
    fn log(&mut self, msg: &str, sev: Severity);
    fn logo(&mut self, msg: &str, sev: Severity, origin: Origin);
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Log {
    pub message: String,
    pub severity: Severity,
    pub timestamp: SystemTime,
    pub origin: Origin,
}

impl fmt::Display for Log {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let severity = match &self.severity {
            Severity::Error => "error:".bold().red(),
            Severity::Warning => "warn:".bold().yellow(),
            Severity::Log => "notice:".bold().blue(),
        };

        let origin = match &self.origin {
            Origin::Unknown => "".blue(),
            Origin::Daemon => "d: ".bold().purple(),
            Origin::Client => "c: ".bold().green(),
        };

        let time = DateTime::<Utc>::from(self.timestamp).format("%Y-%m-%d %H:%M:%S");
        write!(f, "{}{} {} -> {}", origin, severity, time, self.message)
    }
}

pub struct Logger {
    pub logs: Vec<Log>,
}

impl Interface for Logger {
    /// Create a Log with the timestamp from message and severity
    fn log(&mut self, msg: &str, sev: Severity) {
        self.logo(msg, sev, Origin::Unknown);
    }

    fn logo(&mut self, msg: &str, sev: Severity, origin: Origin) {
        let time = SystemTime::now();

        let loggable = Log {
            message: msg.to_string(),
            severity: sev,
            timestamp: time,
            origin,
        };

        self.logs.push(loggable);
    }
}
