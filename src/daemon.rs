struct Daemon {
    cpus: Vec<CPU>,
}

pub fn daemon_init() {
    let daemon: Daemon = Daemon { cpus: Vec::<CPU>::new()};
}
