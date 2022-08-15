use crate::network::{log_to_daemon, logger, Daemon, Packet};
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
