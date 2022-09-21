use crate::logger;
use crate::logger::Interface;
use crate::network::log_to_daemon;
use crate::network::parse_packet;
use crate::network::BufWriter;
use crate::network::Daemon;
use crate::network::Packet;
use crate::network::UnixListener;
use crate::write_packet;
use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

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

pub fn handle_stream(stream: UnixStream, c_daemon_mutex: &Arc<Mutex<Daemon>>) {
    // We don't need to log ALL the time
    // log_to_daemon(c_daemon_mutex, "Received connection", logger::Severity::Log);

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
            let packet = match parse_packet(&actual_line) {
                Ok(p) => p,
                Err(e) => {
                    log_to_daemon(
                        &inner_daemon_mutex.clone(),
                        &format!("Received malfomed packet: {}", e),
                        logger::Severity::Error,
                    );
                    Packet::Unknown
                }
            };
            match packet {
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
                Packet::DaemonDisableRequest() => {
                    let mut inner_daemon = inner_daemon_mutex.lock().unwrap();
                    let response;
                    if inner_daemon.paused {
                        response = Packet::DaemonDisableResponse(false);
                    } else {
                        response = Packet::DaemonDisableResponse(true);
                        inner_daemon.logger.log(
                            "Daemon has been disabled by a client",
                            logger::Severity::Log,
                        );
                        inner_daemon.paused = true;
                    }
                    let mut writer = BufWriter::new(&stream);
                    write_packet!(writer, response);
                }
                Packet::DaemonDisableResponse(_) => {}
                Packet::DaemonEnableRequest() => {
                    let mut inner_daemon = inner_daemon_mutex.lock().unwrap();
                    let response;
                    if !inner_daemon.paused {
                        response = Packet::DaemonEnableResponse(false);
                    } else {
                        response = Packet::DaemonEnableResponse(true);
                        inner_daemon
                            .logger
                            .log("Daemon has been enabled by a client", logger::Severity::Log);
                        inner_daemon.paused = false;
                    }
                    let mut writer = BufWriter::new(&stream);
                    write_packet!(writer, response);
                }
                Packet::DaemonEnableResponse(_) => {}
                Packet::DaemonStatusRequest() => {
                    let response =
                        Packet::DaemonStatusResponse(!inner_daemon_mutex.lock().unwrap().paused);
                    let mut writer = BufWriter::new(&stream);
                    write_packet!(writer, response);
                }
                Packet::DaemonStatusResponse(_) => todo!(),
            };
        }
    });
}
