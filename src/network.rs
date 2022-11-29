use crate::logger::Log;

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
use std::time::SystemTime;

pub mod hook;
pub mod listen;
pub mod send;

#[derive(PartialEq, Eq, Debug)]
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
    let mut packet_split = packet.split('|');
    let packet_type = packet_split.next().ok_or(PacketParseError)?;
    match packet_type {
        "0" => Ok(Packet::Hello(
            packet_split.next().ok_or(PacketParseError)?.to_string(),
        )),
        "1" => Ok(Packet::HelloResponse(
            packet_split.next().ok_or(PacketParseError)?.to_string(),
            packet_split
                .next()
                .ok_or(PacketParseError)?
                .parse::<u32>()?,
        )),
        "2" => Ok(Packet::DaemonDisableRequest()),
        "3" => Ok(Packet::DaemonDisableResponse(
            packet_split
                .next()
                .ok_or(PacketParseError)?
                .parse::<bool>()?,
        )),
        "4" => Ok(Packet::DaemonEnableRequest()),
        "5" => Ok(Packet::DaemonEnableResponse(
            packet_split
                .next()
                .ok_or(PacketParseError)?
                .parse::<bool>()?,
        )),
        "6" => Ok(Packet::DaemonStatusRequest()),
        "7" => Ok(Packet::DaemonStatusResponse(
            packet_split
                .next()
                .ok_or(PacketParseError)?
                .parse::<bool>()?,
        )),
        "8" => Ok(Packet::DaemonLogRequest()),
        "9" => Ok(Packet::DaemonLogResponse(
            packet_split
                .map(|x| {
                    serde_json::from_str(x).unwrap_or(Log {
                        message: "?".to_string(),
                        severity: logger::Severity::Error,
                        timestamp: SystemTime::now(),
                    })
                })
                .collect(),
        )),
        "10" => Ok(Packet::DaemonLogEvent(
            serde_json::from_str(packet_split.next().ok_or(PacketParseError)?).unwrap(),
        )),
        _ => Err(PacketParseError),
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Packet::Hello(data) => writeln!(f, "0|{}", data),
            Packet::HelloResponse(data, version) => writeln!(f, "1|{}|{}", data, version),
            Packet::DaemonDisableRequest() => writeln!(f, "2"),
            Packet::DaemonDisableResponse(data) => writeln!(f, "3|{}", data),
            Packet::DaemonEnableRequest() => writeln!(f, "4"),
            Packet::DaemonEnableResponse(data) => writeln!(f, "5|{}", data),
            Packet::DaemonStatusRequest() => writeln!(f, "6"),
            Packet::DaemonStatusResponse(data) => writeln!(f, "7|{}", data),
            Packet::DaemonLogRequest() => writeln!(f, "8"),
            Packet::DaemonLogResponse(data) => {
                writeln!(
                    f,
                    "9|{}",
                    data.iter()
                        .map(|x| { serde_json::to_string(x).unwrap_or("?".to_string()) })
                        .collect::<String>()
                )
            }

            Packet::Unknown => writeln!(f),
            Packet::DaemonLogEvent(_) => todo!(),
        }
    }
}

fn log_to_daemon(daemon: &Arc<Mutex<Daemon>>, message: &str, severity: logger::Severity) {
    let mut daemon = daemon.lock().unwrap();
    daemon.logger.log(message, severity);
}

#[cfg(test)]
mod tests {
    use crate::network::{parse_packet, Packet};

    #[test]
    fn parse_packet_test() {
        assert!(parse_packet("0|test").unwrap() == Packet::Hello("test".to_string()));
        assert!(parse_packet("1|test|5").unwrap() == Packet::HelloResponse("test".to_string(), 5));
        assert!(parse_packet("0|test").unwrap() != Packet::HelloResponse("test".to_string(), 5));
    }
}
