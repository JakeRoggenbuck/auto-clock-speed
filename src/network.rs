use super::daemon::Daemon;
use super::error::Error;
use super::logger;
use super::logger::Interface;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::io::{BufRead, BufReader, BufWriter};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

#[derive(PartialEq, Eq, Debug)]
pub enum Packet {
    Hello(String),
    HelloResponse(String, u32),
    Unknown,
}

pub fn parse_packet(packet: &String) -> Result<Packet, Error> {
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

pub fn handle_stream(stream: UnixStream, c_daemon_mutex: &Arc<Mutex<Daemon>>) {
    log_to_daemon(c_daemon_mutex, "Received connection", logger::Severity::Log);

    let inner_daemon_mutex = c_daemon_mutex.clone();

    thread::spawn(move || {
        let reader = BufReader::new(&stream);
        for line in reader.lines() {
            let actual_line = match line {
                Ok(line) => line,
                Err(e) => match e.kind() {
                    std::io::ErrorKind::BrokenPipe => {
                        return;
                    }
                    _ => {
                        log_to_daemon(
                            &inner_daemon_mutex.clone(),
                            &format!("Failed to read line: {}", e),
                            logger::Severity::Error,
                        );
                        return;
                    }
                },
            };
            match parse_packet(&actual_line).unwrap_or(Packet::Unknown) {
                Packet::Hello(hi) => {
                    let hello_packet = Packet::HelloResponse(hi.clone(), 0);
                    log_to_daemon(
                        &inner_daemon_mutex.clone(),
                        &format!("Received hello packet: {}", hi),
                        logger::Severity::Log,
                    );
                    let mut writer = BufWriter::new(&stream);
                    writer
                        .write_all(format!("{}", hello_packet).as_bytes())
                        .unwrap();
                    writer.flush().unwrap();
                }
                Packet::HelloResponse(_, _) => {}
                Packet::Unknown => {}
            };
        }
    });
}

pub fn listen(path: &'static str, c_daemon_mutex: Arc<Mutex<Daemon>>) {
    thread::spawn(move || {
        // Get rid of the old sock
        std::fs::remove_file(path).ok();

        // Try to handle sock connections then
        let listener = match UnixListener::bind(path) {
            Ok(listener) => listener,
            Err(e) => {
                log_to_daemon(
                    &c_daemon_mutex,
                    &format!("Failed to bind to {}: {}", path, e),
                    logger::Severity::Error,
                );
                return;
            }
        };

        // Set the permissions on the sock
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o777)).ok();

        // Spawn a new thread to listen for commands
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(stream) => {
                        handle_stream(stream, &c_daemon_mutex);
                    }
                    Err(err) => {
                        log_to_daemon(
                            &c_daemon_mutex,
                            &format!("Failed to accept connection: {}", err),
                            logger::Severity::Error,
                        );
                        break;
                    }
                }
            }
        });
    });
}

pub fn hook(path: &'static str, _c_daemon_mutex: Arc<Mutex<Daemon>>) {
    thread::spawn(move || {
        let mut stream = match UnixStream::connect(path) {
            Ok(stream) => stream,
            Err(e) => {
                println!(
                    "Failed to hook into daemon at {} (is the daemon running?): {}",
                    path, e
                );
                return;
            }
        };
        let packet = Packet::Hello("sup!".to_string());
        println!("(debug not for production) Sending out: {}", packet);
        stream
            .write_all((format!("{}", packet)).as_bytes())
            .unwrap();
        stream.flush().unwrap();
        // Read the response
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        println!("(debug not for production) Response: {}", line);
        stream.shutdown(std::net::Shutdown::Both).unwrap();
    });
}

#[cfg(test)]
mod tests {
    use crate::network::{parse_packet, Packet};

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
