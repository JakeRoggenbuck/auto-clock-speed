use super::cpu::{Speed, CPU};
use super::power::has_battery;
use super::system::list_cpus;
use super::Error;
use std::{thread, time};
use termion::{color, style};

pub trait Checker {
    fn run(&mut self);
    fn update_all(&mut self);
    fn print(&self);
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub verbose: bool,
    pub delay: u64,
    pub edit: bool,
    pub message: String,
}

impl Checker for Daemon {
    fn run(&mut self) {
        let timeout = time::Duration::from_millis(self.delay);

        loop {
            // Update all the values for each cpu before they get used
            self.update_all();

            if self.edit {
                // TODO: Logic to check battery, charge, etc. and set max, min, and gov accordingly
            }

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
        println!(
            "{}\n\n{}{}",
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            self.message,
        );
        for cpu in &self.cpus {
            cpu.print();
        }
        println!("\n\nctrl+c to quit")
    }
}

pub fn daemon_init(verbose: bool, delay: u64, mut edit: bool) -> Result<Daemon, Error> {
    let started_as_edit = edit;
    let mut forced_reason: String = String::new();
    // Check if the device has a battery, otherwise force it to monitor mode
    match has_battery() {
        Ok(a) => {
            if !a {
                edit = false;
                forced_reason = "the device has no battery".to_string();
            }
        }
        Err(_) => eprintln!("Could not check battery"),
    }

    // TODO: Check if the executable has permission to edit speeds, otherwise for to monitor mode

    let message = format!(
        "Auto Clock Speed daemon has been initialized in {} mode with a delay of {} seconds{}\n",
        if edit { "edit" } else { "monitor" },
        delay,
        if started_as_edit != edit {
            format!(
                "\n{}Forced to monitor mode because {}!{}",
                color::Fg(color::Red),
                forced_reason,
                style::Reset
            )
        } else {
            "".to_string()
        }
    );

    // Create a new Daemon
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
        verbose,
        delay,
        edit,
        message,
    };

    // Make a cpu struct for each cpu listed
    for mut cpu in list_cpus()? {
        // Fill that value that were zero with real values
        cpu.init_cpu();
        daemon.cpus.push(cpu);
    }

    Ok(daemon)
}
