use std::convert::TryInto;
use std::io::Write;
use std::os::unix::net::{UnixListener};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use std::{thread, time};

use colored::*;
use nix::unistd::Uid;
use serde::Serialize;

use super::config::Config;
use super::cpu::{Speed, CPU};
use super::graph::{Graph, Grapher};
use super::logger;
use super::logger::Interface;
use super::network::Packet;
use super::power::{
    get_battery_status, has_battery, read_battery_charge, read_lid_state, read_power_source,
    LidState,
};
use super::settings::{GraphType, Settings};
use super::system::{
    check_available_governors, check_cpu_freq, check_cpu_temperature, check_cpu_usage,
    get_highest_temp, list_cpus, parse_proc_file, read_proc_stat_file, ProcStat,
};
use super::terminal::terminal_width;
use super::Error;
use crate::display::print_turbo_status;
use crate::warn_user;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub enum State {
    Normal,
    #[serde(rename = "battery_percent_rule")]
    BatteryLow,
    #[serde(rename = "lid_open_rule")]
    LidClosed,
    #[serde(rename = "ac_charging_rule")]
    Charging,
    #[serde(rename = "cpu_usage_rule")]
    CpuUsageHigh,
    Unknown,
}

/// Return governor string based on current state
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

    fn start_loop(&mut self) -> Result<(), Error>;
    fn end_loop(&mut self);

    fn single_edit(&mut self) -> Result<(), Error>;
    fn single_monit(&mut self) -> Result<(), Error>;

    fn update_all(&mut self) -> Result<(), Error>;

    fn run_state_machine(&mut self) -> State;

    fn preprint_render(&mut self) -> String;
    fn postprint_render(&mut self) -> String;
    fn print(&mut self);

    fn set_govs(&mut self, gov: String) -> Result<(), Error>;
}

pub struct Daemon {
    pub cpus: Vec<CPU>,
    pub last_proc: Vec<ProcStat>,
    pub message: String,
    pub lid_state: LidState,
    pub charging: bool,
    pub charge: i8,
    pub usage: f32,
    pub logger: logger::Logger,
    pub config: Config,
    pub last_below_cpu_usage_percent: Option<SystemTime>,
    pub state: State,
    pub graph: String,
    pub grapher: Graph,
    pub temp_max: i8,
    pub commit_hash: String,
    pub timeout: time::Duration,
    pub timeout_battery: time::Duration,
    pub settings: Settings,
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

        if self.config.active_rules.contains(&State::LidClosed) {
            if self.lid_state == LidState::Closed {
                state = State::LidClosed;
            }
        }

        if self.config.active_rules.contains(&State::Charging) {
            if self.charging {
                state = State::Charging;
            }
        }

        if self.config.active_rules.contains(&State::BatteryLow) {
            if self.charge < self.config.powersave_under {
                state = State::BatteryLow;
            }
        }

        state
    }

    fn init(&mut self) {
        // Get the commit hash from the compile time env variable
        if self.settings.commit {
            self.commit_hash = env!("GIT_HASH").to_string();
        }

        self.timeout_battery = time::Duration::from_millis(self.settings.delay_battery);
        self.timeout = time::Duration::from_millis(self.settings.delay);
    }

    fn start_loop(&mut self) -> Result<(), Error> {
        // Update all the values for each cpu before they get used
        self.update_all()?;

        // Update current states
        self.charging = read_power_source()?;
        self.charge = read_battery_charge()?;
        self.lid_state = read_lid_state()?;
        self.usage = calculate_average_usage(&self.cpus) * 100.0;

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

        let state = self.run_state_machine();

        // Check if the state has changed since the last time we checked
        if self.state != state {
            // Log the state change
            self.logger.log(
                &format!("State changed: {:?} -> {:?}", self.state, state,),
                logger::Severity::Log,
            );

            // Change the cpu governor based on the state
            self.set_govs(get_governor(&state).to_string())?;
        }

        self.state = state;

        self.end_loop();
        Ok(())
    }

    fn single_monit(&mut self) -> Result<(), Error> {
        self.start_loop()?;
        self.end_loop();
        Ok(())
    }

    /// Calls update on each cpu to update the state of each one
    fn update_all(&mut self) -> Result<(), Error> {
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
        let title = "Name  Max\tMin\tFreq\tTemp\tUsage\tGovernor\n".bold();
        // Render each line of cpu core
        let cpus = &self.cpus.iter().map(|c| c.render()).collect::<String>();

        // Prints batter percent or N/A if not
        let battery_status = get_battery_status(self.charging);

        format!("{}{}{}\n{}\n", message, title, cpus, battery_status)
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
            "{}\n{}\n\n{}\n\n{}\n{}",
            graph_type, graph, stop_message, logs, commit
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

        // Shows if turbo is enabled with an amazing turbo animation
        let mut effective_delay = self.timeout_battery;
        if self.charging {
            effective_delay = self.timeout;
        }
        let delay_in_millis = effective_delay.as_millis().try_into().unwrap();

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
        if gov == "performance".to_string() {
            return self.apply_to_cpus(&make_gov_performance);
        } else if gov == "powersave".to_string() {
            return self.apply_to_cpus(&make_gov_powersave);
        } else if gov == "schedutil".to_string() {
            warn_user!("schedutil governor not officially supported");
            return self.apply_to_cpus(&make_gov_schedutil);
        } else if check_available_governors().is_ok() {
            if check_available_governors().unwrap().contains(&gov.into()) {
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
                "{}{}",
                "In order to properly run the daemon in edit mode you must give the executable root privileges.\n",
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
        graph: settings.graph,
        commit: settings.commit,
        testing: settings.testing,
    };

    // Create a new Daemon
    let mut daemon: Daemon = Daemon {
        cpus: Vec::<CPU>::new(),
        last_proc: Vec::<ProcStat>::new(),
        message,
        lid_state: LidState::Unknown,
        // If edit is still true, then there is definitely a bool result to read_power_source
        // otherwise, there is a real problem, because there should be a power source possible
        charging: if settings.edit {
            read_power_source()?
        } else {
            false
        },
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
    };

    // Make a cpu struct for each cpu listed
    for cpu in list_cpus() {
        daemon.cpus.push(cpu);
    }

    let daemon_mutex = Arc::new(Mutex::new(daemon));
    let c_daemon_mutex = Arc::clone(&daemon_mutex);

    thread::spawn(move || {
        println!("Handling connections");
        // Try to handle sock connections then
        let listener = UnixListener::bind("/tmp/acs.sock").unwrap();

        // Spawn a new thread to listen for commands
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(mut stream) => {
                        /* connection succeeded */
                        let mut daemon = c_daemon_mutex.lock().unwrap();
                        daemon.logger.log(
                            &format!(
                                "Received connection from socket on {:?}",
                                stream.peer_addr().expect("Couldn't get local addr")
                            ),
                            logger::Severity::Log,
                        );

                        // Broadcast hello packet
                        let hello_packet = Packet::HelloResponse("Hello from acs".to_string(), 0);
                        stream
                            .write_all(format!("{}", hello_packet).as_bytes())
                            .unwrap();
                    }
                    Err(err) => {
                        /* connection failed */
                        let mut daemon = c_daemon_mutex.lock().unwrap();
                        daemon
                            .logger
                            .log(&format!("Failed to connect from socket with error: {}", err), logger::Severity::Error);
                        break;
                    }
                }
            }
        });
    });

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
        let mode = daemon.settings.edit.clone();

        let effective_timeout = if daemon.charging {
            daemon.timeout.clone()
        } else {
            daemon.timeout_battery.clone()
        };

        drop(daemon);

        // Choose which mode acs runs in
        if mode {
            loop {
                let mut daemon = daemon_mutex.lock().unwrap();
                daemon.single_edit()?;
                drop(daemon);
                thread::sleep(effective_timeout);
            }
        } else {
            loop {
                let mut daemon = daemon_mutex.lock().unwrap();
                daemon.single_monit()?;
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
            edit: true,
            no_animation: false,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
        };

        let config = default_config();

        let daemon_mutex = daemon_init(settings, config).unwrap();
        let daemon = daemon_mutex.lock().unwrap();


        if Uid::effective().is_root() {
            assert_eq!(daemon.settings.edit, true);
        } else {
            assert_eq!(daemon.settings.edit, false);
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
            edit: true,
            no_animation: false,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
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
        assert!(preprint.contains("Name  Max\tMin\tFreq\tTemp\tUsage\tGovernor\n"));
        assert!(preprint.contains("Hz"));
        assert!(preprint.contains("cpu"));
        assert!(preprint.contains("C"));
        assert!(preprint.contains("Battery: "));
    }

    #[test]
    fn preprint_render_test_monit_integration_test() {
        let settings = Settings {
            verbose: true,
            delay: 1,
            delay_battery: 2,
            edit: false,
            no_animation: false,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
        };

        let config = default_config();

        let daemon_mutex = daemon_init(settings, config).unwrap();
        let mut daemon = daemon_mutex.lock().unwrap();
        let preprint = daemon.preprint_render();
        assert!(preprint.contains("Auto Clock Speed daemon has been initialized in \u{1b}[33mmonitor\u{1b}[0m mode with a delay of 1ms normally and 2ms when on battery\n"));
        assert!(preprint.contains("Name  Max\tMin\tFreq\tTemp\tUsage\tGovernor\n"));
        assert!(preprint.contains("Hz"));
        assert!(preprint.contains("cpu"));
        assert!(preprint.contains("C"));
        assert!(preprint.contains("Battery: "));
    }
}
