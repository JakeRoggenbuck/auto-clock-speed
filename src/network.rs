use super::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter, Result as FmtResult};

pub enum Packet {
    Hello(String),
    HelloResponse(String, u32),
}

pub fn parse_packet(packet: String) -> Result<Packet, Error> {
    let mut packet_split = packet.split("|");
    let packet_type = packet_split.next().unwrap();
    let packet_data = packet_split.next().unwrap();
    match packet_type {
        "0" => Ok(Packet::Hello(packet_data.to_string())),
        "1" => Ok(Packet::HelloResponse(
            packet_data.to_string(),
            packet_split.next().unwrap().parse::<u32>().unwrap_or(0),
        )),
        _ => Err(Error::Unknown),
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Packet::Hello(data) => write!(f, "0|{}", data),
            Packet::HelloResponse(data, id) => write!(f, "1|{}|{}", data, id),
        }
    }
}
