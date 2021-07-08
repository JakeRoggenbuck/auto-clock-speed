use super::cpu::{Speed, CPU};
use super::system::list_cpus;
use super::Error;
use std::{thread, time};

pub trait Checker {
    fn run(&mut self);
    fn update_all(&mut self);
    fn print(&self);
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub verbose: bool,
    pub delay: u64,
}

impl Checker for Daemon {
    fn run(&mut self) {
        let timeout = time::Duration::from_secs(self.delay);

        loop {
            // Update all the values for each cpu before they get used
            self.update_all();

            // Print the each cpu, each iteration
            if self.verbose {
                self.print();
            }

            thread::sleep(timeout);
        }
    }

    /// Calls update on each cpu to update the state of each one
    fn update_all(&mut self) {
        for cpu in self.cpus.iter_mut() {
            cpu.update();
        }
    }

    /// Output the values from each cpu
    fn print(&self) {
        for cpu in &self.cpus {
            cpu.print();
        }
    }
}

pub fn daemon_init(verbose: bool, delay: u64) -> Result<Daemon, Error> {
    // Create a new Daemon
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
        verbose,
        delay,
    };

    if verbose {
        println!(
            "Daemon has been initialized with a delay of {} seconds\n\n",
            delay
        );
    }

    // Make a cpu struct for each cpu listed
    for mut cpu in list_cpus()? {
        // Fill that value that were zero with real values
        cpu.init_cpu();
        daemon.cpus.push(cpu);
    }

    Ok(daemon)
}
