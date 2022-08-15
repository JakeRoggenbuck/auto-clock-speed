use super::daemon::Daemon;
use super::error::Error;
use super::logger;
use super::logger::Interface;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::BufWriter;
use std::os::unix::net::UnixListener;
use std::sync::Arc;
use std::sync::Mutex;

pub mod hook;
pub mod listen;

#[derive(PartialEq, Eq, Debug)]
pub enum Packet {
    Hello(String),
    HelloResponse(String, u32),
    Unknown,
}

pub fn parse_packet(packet: &str) -> Result<Packet, Error> {
    let mut packet_split = packet.split('|');
    let packet_type = packet_split.next().unwrap_or("?");
    let packet_data = packet_split.next().unwrap_or("?");
    match packet_type {
        "0" => Ok(Packet::Hello(packet_data.to_string())),
        "1" => Ok(Packet::HelloResponse(
            packet_data.to_string(),
            packet_split.next().unwrap().parse::<u32>().unwrap_or(0),
        )),
        _ => Ok(Packet::Unknown),
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Packet::Hello(data) => writeln!(f, "0|{}", data),
            Packet::HelloResponse(data, version) => writeln!(f, "1|{}|{}", data, version),
            Packet::Unknown => writeln!(f),
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
