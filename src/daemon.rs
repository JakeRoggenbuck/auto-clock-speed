use super::cpu::{Speed, CPU};
use super::system::list_cpus;
use super::Error;
use std::{thread, time};

pub trait Checker {
    fn run(&mut self);
    fn print(&mut self);
    fn update_all(&mut self);
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub verbose: bool,
}

impl Checker for Daemon {
    fn run(&mut self) {
        let five_seconds = time::Duration::from_secs(5);

        loop {
            // Update all the speed from the cpus before they may get displayed or used
            self.update_all();

            if self.verbose {
                self.print();
            }

            thread::sleep(five_seconds);
        }
    }

    /// Output the values from each cpu
    fn print(&mut self) {
        for cpu in &self.cpus {
            println!("{:?}", cpu);
        }
    }

    fn update_all(&mut self) {
        // TODO: find a way to go through self.cpus and run update() on each one
    }
}

pub fn daemon_init(verbose: bool) -> Result<Daemon, Error> {
    // Create a new Daemon
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
        verbose,
    };

    // Make a cpu struct for each cpu listed
    for cpu in list_cpus()? {
        let mut new = CPU {
            name: cpu,
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
        };
        // Fill that value that were zero with real values
        new.init_cpu();
        daemon.cpus.push(new);
    }

    Ok(daemon)
}
