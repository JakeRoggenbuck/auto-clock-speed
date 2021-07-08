use super::cpu::{CPU, Speed};
use super::system::list_cpus;
use super::Error;

pub struct Daemon {
    pub cpus: Vec<CPU>,
}

pub fn daemon_init() -> Result<Daemon, Error> {
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
    };

    for cpu in list_cpus()? {
        let mut new = CPU {
            name: cpu,
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
        };
        new.init_cpu();
        daemon.cpus.push(new);
    }

    Ok(daemon)
}
