use crate::error::Error;
use crate::logger::{Log, Origin};
use crate::network::{log_to_daemon, log_to_daemon_origin, logger, parse_packet, Daemon, Packet};
use crate::write_packet;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::UnixStream;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn hook(path: &'static str, c_daemon_mutex: Arc<Mutex<Daemon>>) {
    thread::spawn(move || {
        let mut stream = match UnixStream::connect(path) {
            Ok(stream) => stream,
            Err(e) => {
                log_to_daemon(
                    &c_daemon_mutex,
                    &format!(
                        "Failed to connect to {} (is the daemon running?): {}",
                        path, e
                    ),
                    logger::Severity::Error,
                );
                return;
            }
        };
        let packet = Packet::Hello("sup!".to_string());
        write_packet!(stream, packet);
        // Read the response
        let mut reader = BufReader::new(&stream);
        let mut line = String::new();

        match reader.read_line(&mut line) {
            Ok(_) => {
                log_to_daemon_origin(
                    &c_daemon_mutex,
                    "Hooked into daemon, restoring logs",
                    logger::Severity::Log,
                    logger::Origin::Client,
                );
                let packet = Packet::DaemonLogRequest();
                write_packet!(stream, packet);

                let mut reader = BufReader::new(&stream);
                let mut line = String::new();

                loop {
                    match reader.read_line(&mut line) {
                        Ok(_) => {}
                        Err(_) => {
                            log_to_daemon_origin(
                                &c_daemon_mutex,
                                "Faied to read response from daemon",
                                logger::Severity::Error,
                                logger::Origin::Client,
                            );
                        }
                    };

                    match parse_packet(&line) {
                        Ok(p) => match p {
                            Packet::DaemonLogResponse(new_logs) => {
                                for mut log in new_logs {
                                    log.origin = Origin::Daemon;
                                    let daemon = &mut c_daemon_mutex.lock().unwrap();
                                    daemon.logger.logs.push(log)
                                }
                            }
                            Packet::DaemonLogEvent(log) => {
                                let daemon = &mut c_daemon_mutex.lock().unwrap();
                                daemon.logger.logs.push(log)
                            }
                            _ => {
                                log_to_daemon_origin(
                                &c_daemon_mutex,
                                "Unexpected response packet from daemon? Incorrect protocol version?",
                                logger::Severity::Error,
                                logger::Origin::Client,
                            );
                            }
                        },
                        Err(_) => {
                            log_to_daemon_origin(
                                &c_daemon_mutex,
                                "Failed to parse packet from daemon? Incorrect protocol version?",
                                logger::Severity::Error,
                                logger::Origin::Client,
                            );
                        }
                    }
                }
            }
            Err(e) => {
                log_to_daemon(
                    &c_daemon_mutex,
                    &format!(
                        "Failed to connect to {} (is the daemon running?): {:?}",
                        path, e
                    ),
                    logger::Severity::Error,
                );
            }
        }
        //stream.shutdown(std::net::Shutdown::Both).unwrap();
    });
}
