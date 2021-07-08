use super::cpu::CPU;
use super::system::{list_cpus, check_min_speed_by_cpu, check_max_speed_by_cpu, check_speed_by_cpu};

struct Daemon {
    cpus: Vec<CPU>,
}

pub fn daemon_init() ->  Result<i32, Error>{
    let daemon: Daemon = Daemon { cpus: Vec::<CPU>::new()};

}
