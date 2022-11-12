//! The daemon handles the running auto clock speed instance
//!
//! # Modes
//!
//! The auto clock speed daemon has two different modes
//! - **Edit mode**
//!     - Modifies the system cpu governor based on information such as battery state and cpu usage
//!     - Requires sudo to run
//! - **Monitor Mode**
//!     - Displays information about the system to the user
//!     - Runs in without sudo
//!
//! The selected mode is passed to the daemon through the settings object
//!
//! # Updating
//!
//! Data within the daemon struct gets updated every `daemon.settings.delay` millis or every
//! `daemon.settings.delay_battery` millis when on battery.
//!
//! The data gets updated in the `update_all` method that gets called periodically from the `run`
//! method.
//!
//! # Extra Features
//!
//! When not disabled the by user, the daemon will print out pretty printed data to stdout. The creation of this
//! print string is controlled by `preprint_render` and `postprint_render`.
//!
//! When enabled by the user the daemon will log all of the cpu data to a csv file.

use std::convert::TryInto;
use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{thread, time};

use colored::Colorize;
use nix::unistd::Uid;
use serde::Serialize;

use super::config::Config;
use super::cpu::{Speed, CPU};
use super::graph::{Graph, Grapher};
use super::logger;
use super::logger::Interface;
use super::network::{hook, listen};
use super::power::battery::{has_battery, Battery};
use super::power::lid::{Lid, LidRetriever, LidState};
use super::power::{Power, PowerRetriever};
use super::settings::{GraphType, Settings};
use super::setup::{inside_docker_message, inside_wsl_message};
use super::system::{
    check_available_governors, check_cpu_freq, check_cpu_temperature, check_cpu_usage,
    get_highest_temp, inside_docker, inside_wsl, list_cpus, parse_proc_file, read_proc_stat_file,
    ProcStat,
};
use super::terminal::terminal_width;
use super::Error;
use crate::display::{print_battery_status, print_turbo_status};
use crate::warn_user;

/// Describes the state of the machine
///
/// - The state is stored in the Daemon
/// - The state value is updated in the run_state_machine method.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub enum State {
    /// System will be in powersave mode unless it gets plugged in
    Normal,
    #[serde(rename = "battery_percent_rule")]
    /// System will always be in powersave mode
    BatteryLow,
    #[serde(rename = "lid_open_rule")]
    /// System will always be in powersave mode
    LidClosed,
    #[serde(rename = "ac_charging_rule")]
    /// The system will be in performance mode unless the battery is low
    Charging,
    #[serde(rename = "cpu_usage_rule")]
    /// The cpu usage has been high for a certain amount of time
    /// The cpu will enter performance mode until the usage goes down
    CpuUsageHigh,
    /// We down know what state the system is in
    Unknown,
}

/// Returns the expected governor string based on current state
///
/// Switches through each state and returns the specified governor string
/// Currently returns "powersave" or "performance"
fn get_governor(current_state: &State) -> &'static str {
    match current_state {
        State::Normal => "powersave",
        State::BatteryLow => "powersave",
        State::LidClosed => "powersave",
        State::Charging => "performance",
        State::CpuUsageHigh => "performance",
        State::Unknown => "powersave",
    }
}

pub trait Checker {
    fn apply_to_cpus(
        &mut self,
        operation: &dyn Fn(&mut CPU) -> Result<(), Error>,
    ) -> Result<(), Error>;

    fn init(&mut self);
    fn setup_csv_logging(&mut self);

    fn start_loop(&mut self) -> Result<(), Error>;
    fn end_loop(&mut self);

    fn single_edit(&mut self) -> Result<(), Error>;
    fn single_monit(&mut self) -> Result<(), Error>;

    fn update_all(&mut self) -> Result<(), Error>;

    fn run_state_machine(&mut self) -> State;
    fn write_csv(&mut self);

    fn preprint_render(&mut self) -> String;
    fn postprint_render(&mut self) -> String;
    fn print(&mut self);

    fn set_govs(&mut self, gov: String) -> Result<(), Error>;
}

/// The daemon structure which contains information about the auto clock speed instance
pub struct Daemon {
    pub battery: Battery,
    pub power: Power,
    pub lid: Lid,
    pub lid_state: LidState,

    pub config: Config,
    pub settings: Settings,

    pub state: State,

    pub logger: logger::Logger,
    pub grapher: Graph,

    pub cpus: Vec<CPU>,
    pub last_proc: Vec<ProcStat>,
    pub message: String,
    pub charging: bool,
    pub charge: i8,
    pub usage: f32,
    pub last_below_cpu_usage_percent: Option<SystemTime>,
    pub graph: String,
    pub temp_max: i8,
    pub commit_hash: String,
    pub paused: bool,
    pub do_update_battery: bool,

    pub timeout: time::Duration,
    pub timeout_battery: time::Duration,
}

fn make_gov_powersave(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("powersave".to_string())?;
    Ok(())
}

fn make_gov_performance(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("performance".to_string())?;
    Ok(())
}

fn make_gov_schedutil(cpu: &mut CPU) -> Result<(), Error> {
    cpu.set_gov("schedutil".to_string())?;
    Ok(())
}

/// Finds the average cpu usage based on a vector of CPUs
fn calculate_average_usage(cpus: &Vec<CPU>) -> f32 {
    let mut sum = 0.0;
    for cpu in cpus {
        sum += cpu.cur_usage;
    }
    (sum / (cpus.len() as f32)) as f32
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

    /// Returns the wanted `State` of the machine based on a set of rules
    /// See `State` enum for information about the individual state.
    fn run_state_machine(&mut self) -> State {
        let mut state = State::Normal;

        if self.config.active_rules.contains(&State::CpuUsageHigh) {
            if self.usage > self.config.high_cpu_threshold.into()
                && self.last_below_cpu_usage_percent.is_none()
            {
                self.last_below_cpu_usage_percent = Some(SystemTime::now());
            }

            if self.usage <= self.config.high_cpu_threshold.into() {
                self.last_below_cpu_usage_percent = None;
            }

            match self.last_below_cpu_usage_percent {
                Some(last) => {
                    if SystemTime::now()
                        .duration_since(last)
                        .expect("Could not compare times")
                        .as_secs()
                        >= 15
                    {
                        state = State::CpuUsageHigh;
                    }
                }
                None => {}
            }
        }

        if self.config.active_rules.contains(&State::LidClosed)
            && self.lid_state == LidState::Closed
        {
            state = State::LidClosed;
        }

        if self.config.active_rules.contains(&State::Charging) && self.charging {
            state = State::Charging;
        }

        if self.config.active_rules.contains(&State::BatteryLow)
            && self.charge < self.config.powersave_under
        {
            state = State::BatteryLow;
        }

        state
    }

    /// Writes out all the cpu data from the daemon to the csv file
    ///
    /// This method gets called every `daemon.settings.delay` millis or every `daemon.settings.delay_battery` millis when on battery
    ///
    /// Each time this method gets called it creates a new row in the csv file. If the csv file
    /// gets larger than `self.settings.log_size_cutoff` MB it will cease logging.
    ///
    /// If an error occurs it will log the error to the daemon logger.
    fn write_csv(&mut self) {
        let lines = &self.cpus.iter().map(|c| c.to_csv()).collect::<String>();

        if let Some(name) = &self.settings.csv_file {
            // Open file in append mode
            // future additions may keep this file open
            let mut file = OpenOptions::new()
                .write(true)
                .append(true) // This is needed to append to file
                .open(name)
                .unwrap();

            // If file is smaller than log_size_cutoff
            if file.metadata().unwrap().len() < (self.settings.log_size_cutoff * 1000_000) as u64 {
                // Try to write the cpus
                match write!(file, "{}", lines) {
                    Ok(_) => {}
                    Err(..) => {
                        self.logger
                            .log("Could not write to CSV file.", logger::Severity::Warning);
                    }
                };
            } else {
                self.logger.log(
                    &format!(
                        "Max log file size reached of {}MB",
                        self.settings.log_size_cutoff
                    ),
                    logger::Severity::Warning,
                );
                // Deactivate csv logging after file size max
                self.settings.csv_file = None;
            }
        }
    }

    /// Initializes a new csv file. If ones currently exists it will keep it. If not it will
    /// generate a new file.
    ///
    /// # Generating a new file
    ///
    /// The file will be created and the column titles will be filled in
    /// If an error occurs while generating a file it will be logged to the daemon
    fn setup_csv_logging(&mut self) {
        // If csv log mode is on
        if let Some(name) = &self.settings.csv_file {
            // If file does not exist
            if !Path::new(name).exists() {
                // Try to create file
                match File::create(name) {
                    Ok(a) => {
                        // Write header and show error if broken
                        match write!(
                            &a,
                            "epoch,name,number,max_freq,min_freq,cur_freq,cur_temp,cur_usage,gov\n"
                        ) {
                            Ok(_) => {}
                            Err(..) => {
                                self.logger
                                    .log("Could not write to CSV file.", logger::Severity::Warning);
                            }
                        };
                    }
                    // File did not get created
                    Err(..) => {
                        self.logger.log(
                            "Could not create file. Turning csv log mode off and continuing.",
                            logger::Severity::Warning,
                        );
                        // Turn log mode off
                        self.settings.csv_file = None;
                    }
                }
            } else {
                // File did exist, use it
                self.logger.log(
                    &format!(
                        "File \"{}\" already exists, continuing in append mode.",
                        name
                    ),
                    logger::Severity::Warning,
                );
            }
        }
    }

    fn init(&mut self) {
        // Get the commit hash from the compile time env variable
        if self.settings.commit {
            self.commit_hash = env!("GIT_HASH").to_string();
        }

        self.timeout_battery = time::Duration::from_millis(self.settings.delay_battery);
        self.timeout = time::Duration::from_millis(self.settings.delay);

        if self.settings.csv_enabled {
            self.setup_csv_logging();
        }

        if inside_wsl() {
            self.logger
                .log(&inside_wsl_message(), logger::Severity::Warning);
        }
        if inside_docker() {
            self.logger
                .log(&inside_docker_message(), logger::Severity::Warning);
        }
    }

    fn start_loop(&mut self) -> Result<(), Error> {
        // Update all the values for each cpu before they get used
        self.update_all()?;

        // Update current states
        self.charging = self.power.read_power_source().unwrap_or(true);
        self.charge = self.battery.capacity;
        self.lid_state = self.lid.read_lid_state()?;
        self.usage = calculate_average_usage(&self.cpus) * 100.0;

        if self.settings.csv_enabled {
            self.write_csv();
        }

        Ok(())
    }

    fn end_loop(&mut self) {
        // Print the each cpu, each iteration
        if self.settings.verbose {
            self.print();
        }
    }

    fn single_edit(&mut self) -> Result<(), Error> {
        self.start_loop()?;

        if !self.paused {
            let state = self.run_state_machine();

            // Check if the state has changed since the last time we checked
            if self.state != state {
                self.logger.log(
                    &format!("State changed: {:?} -> {:?}", self.state, state,),
                    logger::Severity::Log,
                );

                // Change the cpu governor based on the state
                self.set_govs(get_governor(&state).to_string())?;
            }

            self.state = state;
        }

        self.end_loop();
        Ok(())
    }

    fn single_monit(&mut self) -> Result<(), Error> {
        self.start_loop()?;
        self.end_loop();
        Ok(())
    }

    /// Calls update on each cpu to update the state of each one
    /// Also updates battery
    fn update_all(&mut self) -> Result<(), Error> {
        if self.do_update_battery {
            match self.battery.update() {
                Ok(_) => {}
                Err(e) => {
                    if !matches!(e, Error::HdwNotFound) {
                        self.do_update_battery = false;
                        self.logger
                            .log(&format!("Battery error: {:?}", e), logger::Severity::Error)
                    }
                }
            }
        }

        let cur_proc = parse_proc_file(read_proc_stat_file()?)?;
        for cpu in self.cpus.iter_mut() {
            cpu.update()?;
            for (i, proc) in self.last_proc.iter().enumerate() {
                if cpu.name == proc.cpu_name {
                    cpu.update_usage(proc, &cur_proc[i])?;
                }
            }
        }

        self.last_proc = cur_proc;

        self.temp_max = (get_highest_temp(&self.cpus) / 1000) as i8;

        // Update the data in the graph and render it
        if self.settings.graph == GraphType::Usage {
            self.grapher.vals.push(check_cpu_usage(&self.cpus) as f64);
        }
        if self.settings.graph == GraphType::Frequency {
            self.grapher.vals.push(check_cpu_freq(&self.cpus) as f64);
        }
        if self.settings.graph == GraphType::Temperature {
            self.grapher
                .vals
                .push((check_cpu_temperature(&self.cpus) / 1000.0) as f64);
        }

        Ok(())
    }

    fn preprint_render(&mut self) -> String {
        let message = format!("{}\n", self.message);
        let title = "Name\tMax\tMin\tFreq\tTemp\tUsage\tGovernor\n".bold();
        // Render each line of cpu core
        let cpus = &self.cpus.iter().map(|c| format!("{c}")).collect::<String>();

        // Prints battery percent or N/A if not
        let battery_status = print_battery_status(&self.battery);
        let battery_condition = format!("Condition: {}%", self.battery.condition);

        format!(
            "{}{}{}\n{}\n{}\n",
            message, title, cpus, battery_status, battery_condition
        )
    }

    fn postprint_render(&mut self) -> String {
        // Display the current graph type
        let graph_type = if self.settings.graph != GraphType::Hidden {
            format!("Graphing: {}", self.settings.graph)
        } else {
            "".to_string()
        };

        // Render the graph if should_graph
        let graph = if self.settings.graph != GraphType::Hidden {
            self.graph.clone()
        } else {
            String::from("")
        };

        let stop_message = String::from("ctrl+c to stop running");

        // render all of the logs, e.g.
        // notice: 2022-01-13 00:02:17 -> Governor set to performance because battery is charging
        let logs = if self.settings.verbose {
            self.logger
                .logs
                .iter()
                .map(|l| format!("{}\n", l))
                .collect::<String>()
        } else {
            String::from("")
        };

        // Render the commit hash and label
        let commit = if self.settings.commit {
            format!("Commit hash: {}", self.commit_hash.clone())
        } else {
            String::from("")
        };

        format!(
            "{}\n{}\n\n{}\n\n{}\n\n{}",
            graph_type, graph, commit, stop_message, logs
        )
    }

    /// Output the values from each cpu
    fn print(&mut self) {
        let cores = self.cpus.len();

        // Compute graph before screen is cleared
        if self.settings.graph != GraphType::Hidden {
            self.graph = self.grapher.update_one(&mut self.grapher.vals.clone());
        }

        let term_width = terminal_width();

        // Render two sections of the output
        // Rendering before screen is cleared reduces the time between clear and print
        // This reduces and completely avoids all flickering
        let preprint = self.preprint_render();
        let postprint = self.postprint_render();

        let mut effective_delay = self.timeout_battery;
        if self.charging {
            effective_delay = self.timeout;
        }
        let delay_in_millis = effective_delay
            .as_millis()
            .try_into()
            .expect("Delay too large. Should have broken in structopt first.");

        // Clear screen
        println!("{}", termion::clear::All);

        // Goto top
        print!("{}", termion::cursor::Goto(1, 1));

        // Print all pre-rendered items
        print!("{}", preprint);

        print_turbo_status(
            cores,
            self.settings.no_animation,
            term_width,
            delay_in_millis,
        );

        // Print more pre-rendered items
        print!("{}", postprint);
    }

    fn set_govs(&mut self, gov: String) -> Result<(), Error> {
        if gov == *"performance" {
            return self.apply_to_cpus(&make_gov_performance);
        } else if gov == *"powersave" {
            return self.apply_to_cpus(&make_gov_powersave);
        } else if gov == *"schedutil" {
            warn_user!("schedutil governor not officially supported");
            return self.apply_to_cpus(&make_gov_schedutil);
        } else if check_available_governors().is_ok() {
            if check_available_governors().unwrap().contains(&gov) {
                self.logger
                    .log("Other governors not supported yet", logger::Severity::Log);
            } else {
                eprintln!("Governor not available",);
            }
        } else {
            eprintln!("Error checking \"{}\" governor", gov);
        }
        Ok(())
    }
}

fn format_message(
    edit: bool,
    started_as_edit: bool,
    forced_reason: String,
    delay: u64,
    delay_battery: u64,
) -> String {
    // Format the original message with mode and delay, along with the forced message if it
    // was forced to switched modes
    format!(
        "Auto Clock Speed daemon has been initialized in {} mode with a delay of {}ms normally and {}ms when on battery{}\n",
        if edit {
            "edit".red()
        } else {
            "monitor".yellow()
        },
        delay,
        delay_battery,
        if started_as_edit != edit { format!("\nForced to monitor mode because {}!", forced_reason).red() } else { "".normal() }
    )
}

pub fn daemon_init(settings: Settings, config: Config) -> Result<Arc<Mutex<Daemon>>, Error> {
    let started_as_edit: bool = settings.edit;
    let mut edit = settings.edit;
    let mut forced_reason: String = String::new();

    // Check if the device has a battery, otherwise force it to monitor mode
    if !has_battery() {
        edit = false;
        forced_reason = "the device has no battery".to_string();
    }

    // Check if effective permissions are enough for edit
    if edit {
        // If not running as root, tell the user and force to monitor
        if !Uid::effective().is_root() {
            if !settings.testing {
                println!(
                "In order to properly run the daemon in edit mode you must give the executable root privileges.\n{}",
                "Continuing anyway in 5 seconds...".red()
            );

                let timeout = time::Duration::from_millis(5000);
                thread::sleep(timeout);
            }
            forced_reason = "acs was not run as root".to_string();
            edit = false;
        }
    }

    let message = format_message(
        edit, // Use current edit because settings.edit has not changed
        started_as_edit,
        forced_reason,
        settings.delay,
        settings.delay_battery,
    );

    let new_settings = Settings {
        verbose: settings.verbose,
        delay: settings.delay,
        delay_battery: settings.delay_battery,
        edit, // Use new edit for new settings
        no_animation: settings.no_animation,
        hook: settings.hook,
        graph: settings.graph,
        commit: settings.commit,
        testing: settings.testing,
        csv_file: settings.csv_file,
        csv_enabled: settings.csv_enabled,
        log_size_cutoff: settings.log_size_cutoff,
    };

    // Attempt to create battery object
    let battery_present;
    let ac_present;

    let power = Power::new();
    let lid = Lid::new();

    // Create a new Daemon
    let mut daemon: Daemon = Daemon {
        battery: {
            let battery = Battery::new();
            battery_present = battery.is_ok();
            battery.unwrap_or_default()
        },
        cpus: Vec::<CPU>::new(),
        last_proc: Vec::<ProcStat>::new(),
        message,
        lid_state: LidState::Unknown,
        lid,
        // If edit is still true, then there is definitely a bool result to read_power_source
        // otherwise, there is a real problem, because there should be a power source possible
        charging: {
            let source = power.read_power_source();
            ac_present = source.is_ok();
            source.unwrap_or(true)
        },
        power,
        charge: 100,
        usage: 0.0,
        logger: logger::Logger {
            logs: Vec::<logger::Log>::new(),
        },
        config,
        last_below_cpu_usage_percent: None,
        graph: String::new(),
        grapher: Graph {
            vals: Vec::<f64>::new(),
        },
        temp_max: 0,
        commit_hash: String::new(),
        timeout: time::Duration::from_millis(1),
        timeout_battery: time::Duration::from_millis(2),
        state: State::Unknown,
        settings: new_settings,
        paused: false,
        do_update_battery: true,
    };

    if !battery_present {
        daemon.do_update_battery = false;
        daemon.logger.log(
            "Failed to detect a laptop battery",
            logger::Severity::Warning,
        )
    }

    if !ac_present {
        daemon.logger.log(
            "Failed to detect AC power source",
            logger::Severity::Warning,
        )
    }

    // Make a cpu struct for each cpu listed
    for cpu in list_cpus() {
        daemon.cpus.push(cpu);
    }

    let daemon_mutex = Arc::new(Mutex::new(daemon));

    let c_daemon_mutex = Arc::clone(&daemon_mutex);
    if settings.edit {
        // Listen for acs clients
        listen::listen("/tmp/acs.sock", c_daemon_mutex);
    } else {
        // Broadcast hello message
        if settings.hook {
            hook::hook("/tmp/acs.sock", c_daemon_mutex);
        }
    }

    Ok(daemon_mutex)
}

pub fn run(daemon_mutex: Arc<Mutex<Daemon>>) -> Result<(), Error> {
    // Aquire the lock for a bit
    let mut daemon = daemon_mutex.lock().unwrap();

    daemon.init();

    if daemon.settings.testing {
        // Choose which mode acs runs in
        if daemon.settings.edit {
            let mut reps = 4;
            while reps > 0 {
                daemon.single_edit()?;
                reps -= 1;
            }
        } else {
            let mut reps = 4;
            while reps > 0 {
                daemon.single_monit()?;
                reps -= 1;
            }
        }
    } else {
        // Before runnig the loop drop the lock and aquire it again later within the loop
        let mode = daemon.settings.edit;

        drop(daemon);

        // Choose which mode acs runs in
        if mode {
            loop {
                let mut daemon = daemon_mutex.lock().unwrap();
                daemon.single_edit()?;
                let effective_timeout = if daemon.charging {
                    daemon.timeout
                } else {
                    daemon.timeout_battery
                };
                drop(daemon);
                thread::sleep(effective_timeout);
            }
        } else {
            loop {
                let mut daemon = daemon_mutex.lock().unwrap();
                daemon.single_monit()?;
                let effective_timeout = if daemon.charging {
                    daemon.timeout
                } else {
                    daemon.timeout_battery
                };
                drop(daemon);
                thread::sleep(effective_timeout);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_config;

    #[test]
    fn daemon_init_force_to_monit_integration_test() {
        let settings = Settings {
            verbose: true,
            delay: 1,
            delay_battery: 2,
            hook: false,
            edit: true,
            no_animation: false,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
            csv_file: None,
            csv_enabled: false,
            log_size_cutoff: 20,
        };

        let config = default_config();

        let daemon_mutex = daemon_init(settings, config).unwrap();
        let daemon = daemon_mutex.lock().unwrap();

        if Uid::effective().is_root() {
            assert!(daemon.settings.edit);
        } else {
            assert!(!daemon.settings.edit);
        }
    }

    #[test]
    fn preprint_render_test_edit_integration_test() {
        // It should be possible to skip tests ):<
        // https://github.com/Camerooooon/dev-log/blob/main/logs/2022-06-13.md
        let settings = Settings {
            verbose: true,
            delay: 1,
            delay_battery: 2,
            hook: false,
            edit: true,
            no_animation: false,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
            csv_file: None,
            csv_enabled: false,
            log_size_cutoff: 20,
        };

        let config = default_config();

        let daemon_mutex = daemon_init(settings, config).unwrap();
        let mut daemon = daemon_mutex.lock().unwrap();
        let preprint = daemon.preprint_render();
        if Uid::effective().is_root() {
            assert!(preprint.contains("Auto Clock Speed daemon has been initialized in \u{1b}[31medit\u{1b}[0m mode with a delay of 1ms normally and 2ms when on battery"));
        } else {
            assert!(preprint.contains("Auto Clock Speed daemon has been initialized in \u{1b}[33mmonitor\u{1b}[0m mode with a delay of 1ms normally and 2ms when on battery"));
        }
        assert!(preprint.contains("Name\tMax\tMin\tFreq\tTemp\tUsage\tGovernor\n"));
        assert!(preprint.contains("Hz"));
        assert!(preprint.contains("cpu"));
        assert!(preprint.contains('C'));
        assert!(preprint.contains("Battery: "));
    }

    #[test]
    fn preprint_render_test_monit_integration_test() {
        let settings = Settings {
            verbose: true,
            delay: 1,
            delay_battery: 2,
            hook: false,
            edit: false,
            no_animation: false,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
            csv_file: None,
            csv_enabled: false,
            log_size_cutoff: 20,
        };

        let config = default_config();

        let daemon_mutex = daemon_init(settings, config).unwrap();
        let mut daemon = daemon_mutex.lock().unwrap();
        let preprint = daemon.preprint_render();
        assert!(preprint.contains("Auto Clock Speed daemon has been initialized in \u{1b}[33mmonitor\u{1b}[0m mode with a delay of 1ms normally and 2ms when on battery\n"));
        assert!(preprint.contains("Name\tMax\tMin\tFreq\tTemp\tUsage\tGovernor\n"));
        assert!(preprint.contains("Hz"));
        assert!(preprint.contains("cpu"));
        assert!(preprint.contains('C'));
        assert!(preprint.contains("Battery: "));
    }
}
