use super::daemon::Daemon;
use super::error::Error;
use super::logger;
use super::logger::Interface;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::io::{BufRead, BufReader};
use std::os::unix::net::UnixListener;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

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

pub fn listen(path: &'static str, c_daemon_mutex: Arc<Mutex<Daemon>>) {
    thread::spawn(move || {
        // Get rid of the old sock
        std::fs::remove_file(path).ok();

        // Try to handle sock connections then
        let listener = UnixListener::bind(path).unwrap();

        // Spawn a new thread to listen for commands
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        let mut daemon = c_daemon_mutex.lock().unwrap();
                        daemon.logger.log(
                            &format!(
                                "Received connection from socket on {:?}",
                                stream.peer_addr().expect("Couldn't get local addr")
                            ),
                            logger::Severity::Log,
                        );
                        drop(daemon);

                        let stream_clone = stream.try_clone().unwrap();
                        let reader = BufReader::new(stream_clone);

                        thread::spawn(move || {
                            for line in reader.lines() {
                                match parse_packet(&line.unwrap()).unwrap_or(Packet::Unknown) {
                                    Packet::Hello(hi) => {
                                        let hello_packet = Packet::HelloResponse(hi, 0);
                                        stream
                                            .write_all(format!("{}", hello_packet).as_bytes())
                                            .unwrap();
                                    }
                                    Packet::HelloResponse(_, _) => {}
                                    Packet::Unknown => {}
                                };
                            }
                        });
                    }
                    Err(err) => {
                        let mut daemon = c_daemon_mutex.lock().unwrap();
                        daemon.logger.log(
                            &format!("Failed to connect from socket with error: {}", err),
                            logger::Severity::Error,
                        );
                        break;
                    }
                }
            }
        });
    });
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
