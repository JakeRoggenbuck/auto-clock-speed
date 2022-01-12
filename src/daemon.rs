use super::cpu::{Speed, CPU};
use super::logger;
use super::logger::Interface;
use super::power::{has_battery, read_battery_charge, read_lid_state, read_power_source, LidState};
use super::system::{check_turbo_enabled, list_cpus};
use super::Error;
use nix::unistd::Uid;
use std::{thread, time};
use termion::{color, style};

pub trait Checker {
    fn apply_to_cpus(
        &mut self,
        operation: &dyn Fn(&mut CPU) -> Result<(), Error>,
    ) -> Result<(), Error>;

    fn run(&mut self) -> Result<(), Error>;
    fn update_all(&mut self) -> Result<(), Error>;
    fn print(&self);
    fn set_govs(&mut self, gov: String) -> Result<(), Error>;
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub verbose: bool,
    pub delay: u64,
    pub edit: bool,
    pub message: String,
    pub lid_state: LidState,
    pub charging: bool,
    pub charge: i8,
    pub logger: logger::Logger,
}

fn make_gov_powersave(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("powersave".to_string())?;
    Ok(())
}

fn make_gov_performance(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("performance".to_string())?;
    Ok(())
}

impl Checker for Daemon {
    /// Apply a function to every cpu
    fn apply_to_cpus(
        &mut self,
        operation: &dyn Fn(&mut CPU) -> Result<(), Error>,
    ) -> Result<(), Error> {
        for cpu in self.cpus.iter_mut() {
            operation(cpu)?;
        }
        Ok(())
    }

    fn set_govs(&mut self, gov: String) -> Result<(), Error> {
        if gov == "performance".to_string() {
            return self.apply_to_cpus(&make_gov_performance);
        } else if gov == "powersave".to_string() {
            return self.apply_to_cpus(&make_gov_powersave);
        } else {
            eprintln!("Gov \"{}\" not available", gov);
        }
        Ok(())
    }

    fn run(&mut self) -> Result<(), Error> {
        let timeout = time::Duration::from_millis(self.delay);

        // The state for rules
        let mut already_under_20_percent: bool = false;
        let mut already_charging: bool = false;
        let mut already_closed: bool = false;
        let mut first_run: bool = true;

        loop {
            // Update all the values for each cpu before they get used
            self.update_all()?;

            if self.edit {
                // Update current states
                self.charging = read_power_source()?;
                self.charge = read_battery_charge()?;
                self.lid_state = read_lid_state()?;

                // If we just daemonized then make sure the states are the opposite of what they should
                // The logic after this block will make sure that they are set to the correct state
                // but only if the previous states were incorrect
                if first_run == true {
                    already_charging = !self.charging;
                    already_under_20_percent = !(self.charge < 20);
                    already_closed = self.lid_state == LidState::Closed;
                    first_run = false;
                }

                // Lid close rule -> gov powersave
                // If the lid just closed, turn on powersave

                if self.lid_state == LidState::Closed && !already_closed {
                    self.logger.log(
                        "Governor set to powersave because lid closed",
                        logger::Severity::Log,
                    );
                    self.apply_to_cpus(&make_gov_powersave)?;
                    already_closed = true;
                }

                if self.lid_state == LidState::Open && already_closed {
                    // A few checks inorder to insure the computer should actually be in performance
                    if !(self.charge < 20) && self.charging {
                        self.logger.log(
                            "Governor set to performance because lid opened",
                            logger::Severity::Log,
                        );
                        self.apply_to_cpus(&make_gov_performance)?;
                    } else {
                        self.logger.log(
                            "Lid opened however the governor remains unchanged",
                            logger::Severity::Log,
                        );
                    }

                    already_closed = false;
                }

                // Under 20% rule -> gov powersave
                // If the battery life is below 20%, set gov to powersave

                if self.charge < 20 && !already_under_20_percent {
                    self.logger.log(
                        "Governor set to powersave because battery was less than 20",
                        logger::Severity::Log,
                    );
                    self.apply_to_cpus(&make_gov_powersave)?;
                    already_under_20_percent = true;
                    // Make sure to reset state
                }
                if self.charge >= 20 {
                    already_under_20_percent = false;
                }

                // Charging rule -> gov performance
                // If the battery is charging, set to performance

                if self.charging && !already_charging {
                    if self.lid_state == LidState::Closed || self.charge < 20 {
                        self.logger.log(
                            "Battery is charging however the governor remains unchanged",
                            logger::Severity::Log,
                        );
                    } else {
                        self.logger.log(
                            "Governor set to performance because battery is charging",
                            logger::Severity::Log,
                        );
                    }
                    self.apply_to_cpus(&make_gov_performance)?;
                    already_charging = true;
                }
                if !self.charging && already_charging {
                    self.logger.log(
                        "Governor set to powersave because battery is not charging",
                        logger::Severity::Log,
                    );
                    self.apply_to_cpus(&make_gov_powersave)?;
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
    fn update_all(&mut self) -> Result<(), Error> {
        for cpu in self.cpus.iter_mut() {
            cpu.update()?;
        }
        Ok(())
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
        match has_battery() {
            Ok(a) => {
                if a {
                    match read_battery_charge() {
                        Ok(bat) => {
                            println!("{}Battery: {}%", style::Bold, bat)
                        }
                        Err(_) => {
                            // Failed!
                        }
                    }
                } else {
                    println!("{}Battery: {}%", style::Bold, "N/A")
                }
            }
            Err(_) => {
                // Who knows what happened
            }
        }
        match check_turbo_enabled() {
            Ok(turbo) => {
                if turbo {
                    println!("{}Turbo: {}", style::Bold, "yes")
                } else {
                    println!("{}Turbo: {}", style::Bold, "no")
                }
            }
            Err(_) => {
                // Failed
            }
        }
        println!("\nctrl+c to stop running\n\n");
        if self.verbose {
            for log in &self.logger.logs {
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
        lid_state: LidState::Unknown,
        // If edit is still true, then there is definitely a bool result to read_power_source
        // otherwise, there is a real problem, because there should be a power source possible
        charging: if edit { read_power_source()? } else { false },
        charge: 100,
        logger: logger::Logger {
            logs: Vec::<logger::Log>::new(),
        },
    };

    // Make a cpu struct for each cpu listed
    for mut cpu in list_cpus()? {
        // Fill that value that were zero with real values
        cpu.init_cpu()?;
        daemon.cpus.push(cpu);
    }

    Ok(daemon)
}
