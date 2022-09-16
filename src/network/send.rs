use std::{
    io::{BufRead, BufReader, Write},
    os::unix::net::UnixStream,
};

use crate::{error::Error, network::parse_packet};

use super::Packet;

#[macro_export]
macro_rules! write_packet {
    ($a:expr, $p:expr) => {{
        $a.write_all((format!("{}", $p)).as_bytes()).unwrap();
        $a.flush().unwrap();
    }};
}

/// Sends a singular packet to the running daemon and returns the response
/// Will hold up thread until response is received
pub fn query_one(path: &'static str, packet: Packet) -> Result<Packet, Error> {
    let mut stream = UnixStream::connect(path)?;
    write_packet!(stream, packet);
    let mut reader = BufReader::new(&stream);
    let mut line = String::new();
    reader.read_line(&mut line)?;
    line.truncate(line.len() - 1);
    parse_packet(&line).map_err(|_| Error::Unknown)
}
