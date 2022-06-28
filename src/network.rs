use super::error::Error;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(PartialEq, Debug)]
pub enum Packet {
    Hello(String),
    HelloResponse(String, u32),
    Unknown,
}

pub fn parse_packet(packet: &String) -> Result<Packet, Error> {
    let mut packet_split = packet.split("|");
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
            Packet::Hello(data) => write!(f, "0|{}", data),
            Packet::HelloResponse(data, version) => write!(f, "1|{}|{}", data, version),
            Packet::Unknown => write!(f, ""),
        }
    }
}

mod tests {
    use super::*;

    #[test]
    fn parse_packet_test() {
        assert!(parse_packet(&"0|test".to_string()).unwrap() == Packet::Hello("test".to_string()));
        assert!(
            parse_packet(&"1|test|5".to_string()).unwrap()
                == Packet::HelloResponse("test".to_string(), 5)
        );
        assert!(
            parse_packet(&"0|test".to_string()).unwrap()
                != Packet::HelloResponse("test".to_string(), 5)
        );
    }
}
