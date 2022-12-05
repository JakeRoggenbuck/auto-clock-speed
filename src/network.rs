use serde::{Deserialize, Serialize};

use crate::logger::{Log, Origin};

use super::daemon::Daemon;
use super::logger;
use super::logger::Interface;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::BufWriter;
use std::num::ParseIntError;
use std::os::unix::net::UnixListener;
use std::str::ParseBoolError;
use std::sync::Arc;
use std::sync::Mutex;

pub mod hook;
pub mod listen;
pub mod send;

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Packet {
    Hello(String),
    HelloResponse(String, u32),
    DaemonDisableRequest(),
    DaemonDisableResponse(bool),
    DaemonEnableRequest(),
    DaemonEnableResponse(bool),
    DaemonStatusRequest(),
    DaemonStatusResponse(bool),
    DaemonLogRequest(),
    DaemonLogResponse(Vec<Log>),
    DaemonLogEvent(Log),
    DaemonLogEventRequest(),
    Unknown,
}

#[derive(Debug)]
pub struct PacketParseError;

impl Display for PacketParseError {
    fn fmt(&self, f: &mut Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "Packet parse error occured")
    }
}

impl From<ParseIntError> for PacketParseError {
    fn from(_err: ParseIntError) -> Self {
        PacketParseError
    }
}

impl From<ParseBoolError> for PacketParseError {
    fn from(_err: ParseBoolError) -> Self {
        PacketParseError
    }
}

pub fn parse_packet(packet: &str) -> Result<Packet, PacketParseError> {
    serde_json::from_str(packet).map_err(|_| PacketParseError)
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        writeln!(
            f,
            "{}",
            serde_json::to_string(self).unwrap_or_else(|_| "?".to_string())
        )
    }
}

fn log_to_daemon(daemon: &Arc<Mutex<Daemon>>, message: &str, severity: logger::Severity) {
    let mut daemon = daemon.lock().unwrap();
    daemon.logger.log(message, severity);
}

fn log_to_daemon_origin(
    daemon: &Arc<Mutex<Daemon>>,
    message: &str,
    severity: logger::Severity,
    origin: Origin,
) {
    let mut daemon = daemon.lock().unwrap();
    daemon.logger.logo(message, severity, origin);
}
