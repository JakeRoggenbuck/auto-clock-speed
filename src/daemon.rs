use super::cpu::{Speed, CPU};
use super::power::{has_battery, read_battery_charge, read_lid_state, read_power_source, LidState};
use super::system::list_cpus;
use super::Error;
use nix::unistd::Uid;
use std::{thread, time};
use termion::{color, style};

pub trait Checker {
    fn log(&mut self, message: &str);
    fn apply_to_cpus(&mut self, operation: &dyn Fn(&mut CPU));
    fn run(&mut self) -> Result<(), Error>;
    fn update_all(&mut self);
    fn print(&self);
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub verbose: bool,
    pub delay: u64,
    pub edit: bool,
    pub message: String,
    pub lid_state: LidState,
    pub charging: bool,
    pub logs: Vec<String>,
}

fn make_gov_powersave(cpu: &mut CPU) {
    cpu.set_gov("powersave".to_string())
}

fn make_gov_performance(cpu: &mut CPU) {
    cpu.set_gov("performance".to_string())
}

impl Checker for Daemon {
    fn log(&mut self, message: &str) {
        if self.verbose {
            self.logs.push(message.to_string());
        }
    }

    /// Apply a function to every cpu
    fn apply_to_cpus(&mut self, operation: &dyn Fn(&mut CPU)) {
        for cpu in self.cpus.iter_mut() {
            operation(cpu);
        }
    }

    fn run(&mut self) -> Result<(), Error> {
        let timeout = time::Duration::from_millis(self.delay);

        // The state for rules
        let mut already_under_20_percent: bool = false;
        let mut already_charging: bool = self.charging;

        loop {
            // Update all the values for each cpu before they get used
            self.update_all();

            if self.edit {

                // Lid close rule -> gov powersave
                // If the lid just closed, turn on powersave
                if read_lid_state()? == LidState::Closed && self.lid_state != LidState::Closed {
                    self.log("Governor set to powersave because lid closed");
                    self.apply_to_cpus(&make_gov_powersave);
                    self.lid_state = LidState::Closed;
                }
                if read_lid_state()? == LidState::Open {
                    self.lid_state = LidState::Open;
                }

                // Under 20% rule -> gov powersave
                // If the battery life is below 20%, set gov to powersave
                if read_battery_charge()? < 20 && !already_under_20_percent {
                    self.log("Governor set to powersave because battery was less than 20");
                    self.apply_to_cpus(&make_gov_powersave);
                    already_under_20_percent = true;
                    // Make sure to reset state
                }
                if read_battery_charge()? >= 20 {
                    already_under_20_percent = false;
                }

                // Charging rule -> gov performance
                // Update charging status
                self.charging = read_power_source()?;

                // If the battery is charging, set to performance
                if self.charging && !already_charging {
                    self.log("Governor set to performance because battery is charging");
                    self.apply_to_cpus(&make_gov_performance);
                    already_charging = true;
                }
                if !self.charging && already_charging {
                    self.log("Governor set to powersave because battery is no longer charging");
                    self.apply_to_cpus(&make_gov_powersave);
                    already_charging = false;
                }
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
            // TODO: Don't clear each print
            // clear at start and replace the first lines
            termion::clear::All,
            termion::cursor::Goto(1, 1),
            self.message,
        );
        println!("{}Name  Max\tMin\tFreq\tTemp\tGovernor", style::Bold);
        for cpu in &self.cpus {
            cpu.print();
        }
        println!("\nctrl+c to stop running\n\n");
        if self.verbose {
            for log in &self.logs {
                println!("{}", log)
            }
        }
    }
}

fn format_message(edit: bool, started_as_edit: bool, forced_reason: String, delay: u64) -> String {
    // Create the message for why it force switched to monitor mode
    let force = if started_as_edit != edit {
        format!(
            "\n{}Forced to monitor mode because {}!{}",
            color::Fg(color::Red),
            forced_reason,
            style::Reset
        )
    } else {
        "".to_string()
    };

    // Format the original message with mode and delay, along with the forced message if it
    // was forced to switched modes
    format!(
        "Auto Clock Speed daemon has been initialized in {} mode with a delay of {} milliseconds{}\n",
        if edit {
            format!("{}edit{}", color::Fg(color::Red), style::Reset)
        } else {
            "monitor".to_string()
        },
        delay,
        force
    )
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

    // Check if effective permissions are enough for edit
    if edit {
        if !Uid::effective().is_root() {
            println!(
                "{}{}{}{}",
                color::Fg(color::Red),
                "In order to properly run the daemon in edit mode you must give the executable root privileges.\n", 
                "Continuing anyway in 5 seconds...",
                style::Reset
            );
            let timeout = time::Duration::from_millis(5000);
            thread::sleep(timeout);
            edit = false;
            forced_reason = "acs was not run as root".to_string();
        }
    }

    let message = format_message(edit, started_as_edit, forced_reason, delay);
    // Create a new Daemon
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
        verbose,
        delay,
        // If the program is supposed to change any values (needs root)
        edit,
        message,
        // TODO: Get the lid state if possible or set to Unknown if not
        lid_state: LidState::Unknown,
        // If edit is still true, then there is definitely a bool result to read_power_source
        // otherwise, there is a real problem, because there should be a power source possible
        charging: if edit { read_power_source()? } else { false },
        logs: Vec::<String>::new(),
    };

    // Make a cpu struct for each cpu listed
    for mut cpu in list_cpus()? {
        // Fill that value that were zero with real values
        cpu.init_cpu();
        daemon.cpus.push(cpu);
    }

    Ok(daemon)
}
