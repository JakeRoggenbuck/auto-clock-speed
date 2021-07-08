use super::cpu::CPU;
use super::system::list_cpus;
use super::Error;

struct Daemon {
    cpus: Vec<CPU>,
}

pub fn daemon_init() ->  Result<(), Error>{
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new()
    };

    for cpu in list_cpus()? {
        let new = CPU {
            name: cpu,
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
        };
        daemon.cpus.push(new);
    }

    Ok(())
}
