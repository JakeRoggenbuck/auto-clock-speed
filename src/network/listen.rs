use crate::logger;
use crate::network::handle_stream;
use crate::network::log_to_daemon;
use crate::network::Daemon;
use crate::network::UnixListener;
use std::os::unix::fs::PermissionsExt;
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
