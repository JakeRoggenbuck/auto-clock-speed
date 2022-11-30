use crate::error::Error;
use crate::logger::{Log, Origin};
use crate::network::send::query_one;
use crate::network::{log_to_daemon, logger, parse_packet, Daemon, Packet};
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
                log_to_daemon(
                    &c_daemon_mutex,
                    "Hooked into daemon, restoring logs",
                    logger::Severity::Log,
                );
                let daemon = &mut c_daemon_mutex.lock().unwrap();
                match restore_logs(&mut stream, &mut daemon.logger.logs) {
                    Ok(_) => {}
                    Err(e) => log_to_daemon(
                        &c_daemon_mutex,
                        &format!("Failed to restore logs: {:?}", e),
                        logger::Severity::Error,
                    ),
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

                return;
            }
        }

        stream.shutdown(std::net::Shutdown::Both).unwrap();
    });
}

fn restore_logs(stream: &mut UnixStream, logs: &mut Vec<Log>) -> Result<(), Error> {
    let packet = Packet::DaemonLogRequest();
    write_packet!(stream, packet);

    let mut reader = BufReader::new(stream);
    let mut line = String::new();

    reader.read_line(&mut line)?;

    match parse_packet(&line) {
        Ok(p) => match p {
            Packet::DaemonLogResponse(new_logs) => {
                for mut log in new_logs {
                    log.origin = Origin::Daemon;
                    logs.push(log)
                }
                Ok(())
            }
            _ => Err(Error::Parse),
        },
        Err(_) => Err(Error::Parse),
    }
}
