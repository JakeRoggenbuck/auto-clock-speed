use super::cpu::{Speed, CPU};
use super::system::list_cpus;
use super::Error;

pub trait Checker {
    fn run(&mut self);
    fn print(&mut self);
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub verbose: bool,
}

impl Checker for Daemon {
    fn run(&mut self) {
        if self.verbose {
            self.print();
        }
    }

    fn print(&mut self) {
        for cpu in &self.cpus {
            println!("{:?}", cpu);
        }
    }
}

pub fn daemon_init(verbose: bool) -> Result<Daemon, Error> {
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
        verbose,
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
