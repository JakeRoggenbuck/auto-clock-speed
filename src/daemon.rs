use std::{mem, thread, time};

use nix::libc::{c_short, c_ushort, ioctl, STDOUT_FILENO, TIOCGWINSZ};
use nix::unistd::Uid;
use termion::{color, style};

use crate::display::print_turbo_animation;

use super::config::Config;
use super::cpu::{Speed, CPU};
use super::debug;
use super::graph::{Graph, Grapher};
use super::logger;
use super::logger::Interface;
use super::power::{has_battery, read_battery_charge, read_lid_state, read_power_source, LidState};
use super::system::{check_cpu_freq, check_turbo_enabled, list_cpus};
use super::Error;

pub trait Checker {
    fn apply_to_cpus(
        &mut self,
        operation: &dyn Fn(&mut CPU) -> Result<(), Error>,
    ) -> Result<(), Error>;

    // Start Charging Rule
    fn lid_closed_or_charge_under(&mut self);
    fn lid_open_and_charge_over(&mut self) -> Result<(), Error>;
    fn start_charging_rule(&mut self) -> Result<(), Error>;

    // End Charging Rule
    fn end_charging_rule(&mut self) -> Result<(), Error>;

    // Lid Close Rule
    fn lid_close_rule(&mut self) -> Result<(), Error>;

    // Lid Open Rule
    fn not_charging_or_charge_under(&mut self) -> Result<(), Error>;
    fn charging_and_charge_over(&mut self) -> Result<(), Error>;
    fn lid_open_rule(&mut self) -> Result<(), Error>;

    // Under Powersave Under Rule
    fn under_powersave_under_rule(&mut self) -> Result<(), Error>;

    // Other methods
    fn run(&mut self) -> Result<(), Error>;
    fn init(&mut self);

    fn start_loop(&mut self) -> Result<(), Error>;
    fn end_loop(&mut self);

    fn loop_edit(&mut self) -> Result<(), Error>;
    fn loop_monit(&mut self) -> Result<(), Error>;

    fn update_all(&mut self) -> Result<(), Error>;
    fn print(&mut self);
    fn set_govs(&mut self, gov: String) -> Result<(), Error>;
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub message: String,
    pub verbose: bool,
    pub delay: u64,
    pub edit: bool,
    pub lid_state: LidState,
    pub charging: bool,
    pub charge: i8,
    pub logger: logger::Logger,
    pub config: Config,
    pub no_animation: bool,
    pub already_charging: bool,
    pub already_closed: bool,
    pub already_under_powersave_under_percent: bool,
    pub should_graph: bool,
    pub graph: String,
    pub grapher: Graph,
    pub commit: bool,
    pub commit_hash: String,
    pub timeout: time::Duration,
}

fn make_gov_powersave(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("powersave".to_string())?;
    Ok(())
}

fn make_gov_performance(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("performance".to_string())?;
    Ok(())
}

fn green_or_red(boolean: bool) -> String {
    if boolean {
        color::Fg(color::Green).to_string()
    } else {
        color::Fg(color::Green).to_string()
    }
}

fn get_battery_status(charging: bool) -> String {
    match has_battery() {
        Ok(has) => {
            if has {
                match read_battery_charge() {
                    Ok(bat) => {
                        format!(
                            "Battery: {}{}{}%{}",
                            style::Bold,
                            green_or_red(charging),
                            bat,
                            style::Reset
                        )
                    }
                    Err(e) => format!("Battery charge could not be read\n{:?}", e),
                }
            } else {
                format!("Battery: {}{}{}", style::Bold, "N/A", style::Reset)
            }
        }
        Err(e) => format!("Could not find battery\n{:?}", e),
    }
}

fn print_turbo_status(cores: usize, no_animation: bool, term_width: usize) {
    let mut turbo_y_pos: usize = 7;
    if term_width > 94 {
        turbo_y_pos = 6
    }
    match check_turbo_enabled() {
        Ok(turbo) => {
            let enabled_message = if turbo { "yes" } else { "no" };

            println!(
                "  Turbo: {}{}{}",
                style::Bold,
                enabled_message,
                style::Reset
            );

            // println!(" {}", term_width);

            if !no_animation {
                print_turbo_animation(turbo, cores, turbo_y_pos);
            }
        }
        Err(e) => eprintln!("Could not check turbo\n{:?}", e),
    }
}

struct TermSize {
    row: c_short,
    col: c_ushort,
}

fn terminal_width() -> usize {
    unsafe {
        let mut size: TermSize = mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut size as *mut _);
        return size.col as usize;
    }
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

    fn lid_closed_or_charge_under(&mut self) {
        debug!("Just started charging && (lid is closed || charge is lower than powersave_under)");
        self.logger.log(
            "Battery is charging however the governor remains unchanged",
            logger::Severity::Log,
        );
    }

    fn lid_open_and_charge_over(&mut self) -> Result<(), Error> {
        debug!("Just started charging && (lid is open && charge is higher than powersave_under)");
        self.logger.log(
            "Governor set to performance because battery is charging",
            logger::Severity::Log,
        );
        self.apply_to_cpus(&make_gov_performance)?;
        Ok(())
    }

    fn start_charging_rule(&mut self) -> Result<(), Error> {
        if self.charging && !self.already_charging {
            if self.lid_state == LidState::Closed || self.charge < self.config.powersave_under {
                self.lid_closed_or_charge_under();
            } else {
                self.lid_open_and_charge_over()?;
            }
            self.already_charging = true;
        }
        Ok(())
    }

    fn end_charging_rule(&mut self) -> Result<(), Error> {
        if !self.charging && self.already_charging {
            self.logger.log(
                "Governor set to powersave because battery is not charging",
                logger::Severity::Log,
            );
            self.apply_to_cpus(&make_gov_powersave)?;
            self.already_charging = false;
        }
        Ok(())
    }

    fn lid_close_rule(&mut self) -> Result<(), Error> {
        if self.lid_state == LidState::Closed && !self.already_closed {
            self.logger.log(
                "Governor set to powersave because lid closed",
                logger::Severity::Log,
            );
            self.apply_to_cpus(&make_gov_powersave)?;
            self.already_closed = true;
        }
        Ok(())
    }

    fn charging_and_charge_over(&mut self) -> Result<(), Error> {
        self.logger.log(
            "Governor set to performance because lid opened",
            logger::Severity::Log,
        );
        self.apply_to_cpus(&make_gov_performance)?;
        Ok(())
    }

    fn not_charging_or_charge_under(&mut self) -> Result<(), Error> {
        self.logger.log(
            "Lid opened however the governor remains unchanged",
            logger::Severity::Log,
        );
        Ok(())
    }

    fn lid_open_rule(&mut self) -> Result<(), Error> {
        if self.lid_state == LidState::Open && self.already_closed {
            // A few checks in order to insure the computer should actually be in performance
            if self.charging && !(self.charge < self.config.powersave_under) {
                self.charging_and_charge_over()?;
            } else {
                self.not_charging_or_charge_under()?;
            }
            self.already_closed = false;
        }

        Ok(())
    }

    fn under_powersave_under_rule(&mut self) -> Result<(), Error> {
        if self.charge < self.config.powersave_under && !self.already_under_powersave_under_percent
        {
            self.logger.log(
                &format!(
                    "Governor set to powersave because battery was less than {}",
                    self.config.powersave_under
                ),
                logger::Severity::Log,
            );
            self.apply_to_cpus(&make_gov_powersave)?;
            self.already_under_powersave_under_percent = true;
        }
        if self.charge >= self.config.powersave_under {
            self.already_under_powersave_under_percent = false;
        }
        Ok(())
    }

    fn init(&mut self) {
        if self.commit {
            self.commit_hash = env!("GIT_HASH").to_string();
        }

        self.timeout = time::Duration::from_millis(self.delay);

        // If we just daemonized then make sure the states are the opposite of what they should
        // The logic after this block will make sure that they are set to the correct state
        // but only if the previous states were incorrect
        self.already_charging = !self.charging;
        self.already_under_powersave_under_percent = !(self.charge < self.config.powersave_under);
        self.already_closed = self.lid_state == LidState::Closed;
    }

    fn start_loop(&mut self) -> Result<(), Error> {
        // Update all the values for each cpu before they get used
        self.update_all()?;
        Ok(())
    }

    fn end_loop(&mut self) {
        // Print the each cpu, each iteration
        if self.verbose {
            self.print();
        }

        thread::sleep(self.timeout);
    }

    fn loop_edit(&mut self) -> Result<(), Error> {
        loop {
            self.start_loop()?;

            // Update current states
            self.charging = read_power_source()?;
            self.charge = read_battery_charge()?;
            self.lid_state = read_lid_state()?;

            // Call all rules
            self.start_charging_rule()?;
            self.end_charging_rule()?;
            self.lid_close_rule()?;
            self.lid_open_rule()?;
            self.under_powersave_under_rule()?;

            self.end_loop();
        }
    }

    fn loop_monit(&mut self) -> Result<(), Error> {
        loop {
            self.start_loop()?;
            self.end_loop();
        }
    }

    fn run(&mut self) -> Result<(), Error> {
        self.init();

        if self.edit {
            self.loop_edit()?;
        } else {
            self.loop_monit()?;
        }

        Ok(())
    }

    /// Calls update on each cpu to update the state of each one
    fn update_all(&mut self) -> Result<(), Error> {
        for cpu in self.cpus.iter_mut() {
            cpu.update()?;
        }

        if self.should_graph {
            self.grapher.freqs.push(check_cpu_freq()? as f64);
        }

        Ok(())
    }

    /// Output the values from each cpu
    fn print(&mut self) {
        let cores = num_cpus::get();

        // Compute graph before screen is cleared
        if self.should_graph {
            self.graph = self.grapher.update_one(&mut self.grapher.freqs.clone());
        }

        // Prints batter percent or N/A if not
        let battery_status = get_battery_status(self.charging);

        // Clear screen
        println!("{}", termion::clear::All);

        // Print initial banner
        println!("{}{}", termion::cursor::Goto(1, 1), self.message);

        // Print cpu label banner
        println!("{}Name  Max\tMin\tFreq\tTemp\tGovernor", style::Bold);

        // Print each cpu
        for cpu in &self.cpus {
            cpu.print();
        }

        // Just need a little space
        println!("");

        println!("{}", battery_status);

        // Shows if turbo is enabled with an amazing turbo animation
        print_turbo_status(cores, self.no_animation, terminal_width());

        if self.should_graph {
            println!("{}", self.graph);
        }

        // Tells user how to stop
        println!("\nctrl+c to stop running\n\n");

        // Print all of the logs, e.g.
        // notice: 2022-01-13 00:02:17 -> Governor set to performance because battery is charging
        if self.verbose {
            for log in &self.logger.logs {
                println!("{}", log)
            }
        }

        if self.commit {
            println!("Commit hash: {}", self.commit_hash);
        }
    }
}

fn format_message(edit: bool, started_as_edit: bool, forced_reason: String, delay: u64) -> String {
    // Create the message for why it force switched to monitor mode
    let force: String = if started_as_edit != edit {
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

pub fn daemon_init(
    verbose: bool,
    delay: u64,
    mut edit: bool,
    config: Config,
    no_animation: bool,
    should_graph: bool,
    commit: bool,
) -> Result<Daemon, Error> {
    let started_as_edit: bool = edit;
    let mut forced_reason: String = String::new();

    // Check if the device has a battery, otherwise force it to monitor mode
    match has_battery() {
        Ok(has) => {
            if !has {
                edit = false;
                forced_reason = "the device has no battery".to_string();
            }
        }
        Err(e) => eprintln!("Could not find battery\n{:?}", e),
    }

    // Check if effective permissions are enough for edit
    if edit {
        // If not running as root, tell the user and force to monitor
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
        config,
        no_animation,
        already_charging: false,
        already_closed: false,
        already_under_powersave_under_percent: false,
        should_graph,
        graph: String::new(),
        grapher: Graph { freqs: vec![0.0] },
        commit,
        commit_hash: String::new(),
        timeout: time::Duration::from_millis(1),
    };

    // Make a cpu struct for each cpu listed
    for mut cpu in list_cpus()? {
        // Fill that value that were zero with real values
        cpu.init_cpu()?;
        daemon.cpus.push(cpu);
    }

    Ok(daemon)
}
