use std::convert::TryInto;
use std::time::SystemTime;
use std::{thread, time};

use colored::*;
use nix::unistd::Uid;

use super::config::Config;
use super::cpu::{Speed, CPU};
use super::graph::{Graph, Grapher};
use super::logger;
use super::logger::Interface;
use super::power::{has_battery, read_battery_charge, read_lid_state, read_power_source, LidState};
use super::system::{
    check_available_governors, check_cpu_temperature, check_cpu_freq, check_cpu_usage, check_turbo_enabled, get_highest_temp, list_cpus,
    parse_proc_file, read_proc_stat_file, ProcStat,
};
use super::terminal::terminal_width;
use super::Error;
use super::settings::{GraphType, Settings};
use crate::display::print_turbo_animation;
use crate::warn_user;

#[derive(Debug, PartialEq)]
pub enum State {
    Normal,
    BatteryLow,
    LidClosed,
    Charging,
    CpuUsageHigh,
    Unknown,
}

// Return governor string based on current state
fn get_governor(current_state: &State) -> Result<&'static str, Error> {
    Ok(match current_state {
        State::Normal => "powersave",
        State::BatteryLow => "powersave",
        State::LidClosed => "powersave",
        State::Charging => "performance",
        State::CpuUsageHigh => "performance",
        State::Unknown => "powersave",
    })
}

pub trait Checker {
    fn apply_to_cpus(
        &mut self,
        operation: &dyn Fn(&mut CPU) -> Result<(), Error>,
    ) -> Result<(), Error>;

    fn run(&mut self) -> Result<(), Error>;
    fn init(&mut self);

    fn start_loop(&mut self) -> Result<(), Error>;
    fn end_loop(&mut self);

    fn single_edit(&mut self) -> Result<(), Error>;
    fn single_monit(&mut self) -> Result<(), Error>;

    fn update_all(&mut self) -> Result<(), Error>;

    fn run_state_machine(&mut self) -> Result<State, Error>;

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
    pub already_charging: bool,
    pub already_closed: bool,
    pub already_under_powersave_under_percent: bool,
    pub already_high_temp: bool,
    pub already_high_usage: bool,
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

// TODO Figure out how to make generic governor work
//fn make_gov_generic(cpu: &mut CPU) -> Result<(), Error> {
//    cpu.set_gov(generic_gov.to_string())?;
//    Ok(())
//}

fn get_battery_status(charging: bool) -> String {
    if has_battery() {
        match read_battery_charge() {
            Ok(bat) => {
                format!(
                    "Battery: {}",
                    if charging {
                        format!("{}%", bat).green()
                    } else {
                        format!("{}%", bat).red()
                    },
                )
            }
            Err(e) => format!("Battery charge could not be read\n{:?}", e),
        }
    } else {
        format!("Battery: {}", "N/A".bold())
    }
}

fn print_turbo_status(cores: usize, no_animation: bool, term_width: usize, delay: u64) {
    let mut turbo_y_pos: usize = 7;
    let title_width = 94;

    if term_width > title_width {
        turbo_y_pos = 6
    }

    match check_turbo_enabled() {
        Ok(turbo) => {
            let enabled_message = if turbo { "yes" } else { "no" };

            println!("{} {}", "  Turbo:", enabled_message.bold(),);

            if !no_animation {
                print_turbo_animation(cores, turbo_y_pos, delay);
            }
        }
        Err(e) => eprintln!("Could not check turbo\n{:?}", e),
    }
}

fn calculate_average_usage(cpus: &Vec<CPU>) -> Result<f32, Error> {
    let mut sum = 0.0;
    for cpu in cpus {
        sum += cpu.cur_usage;
    }
    Ok((sum / (cpus.len() as f32)) as f32)
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

    fn run_state_machine(&mut self) -> Result<State, Error> {
        let mut state = State::Normal;

        if self.usage > 70.0 {
            state = State::CpuUsageHigh;
        }

        if self.lid_state == LidState::Closed {
            state = State::LidClosed;
        }

        if self.charging {
            state = State::Charging;
        }

        if self.charge < self.config.powersave_under {
            state = State::BatteryLow;
        }

        Ok(state)
    }

    fn run(&mut self) -> Result<(), Error> {
        self.init();

        if self.settings.testing {
            // Choose which mode acs runs in
            if self.settings.edit {
                let mut reps = 4;
                while reps > 0 {
                    self.single_edit()?;
                    reps -= 1;
                }
            } else {
                let mut reps = 4;
                while reps > 0 {
                    self.single_monit()?;
                    reps -= 1;
                }
            }
        } else {
            // Choose which mode acs runs in
            if self.settings.edit {
                loop {
                    self.single_edit()?;
                }
            } else {
                loop {
                    self.single_monit()?;
                }
            }
        }

        Ok(())
    }

    fn init(&mut self) {
        // Get the commit hash from the compile time env variable
        if self.settings.commit {
            self.commit_hash = env!("GIT_HASH").to_string();
        }

        self.timeout_battery = time::Duration::from_millis(self.settings.delay_battery);
        self.timeout = time::Duration::from_millis(self.settings.delay);

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

        // Update current states
        self.charging = read_power_source()?;
        self.charge = read_battery_charge()?;
        self.lid_state = read_lid_state()?;
        self.usage = calculate_average_usage(&self.cpus)? * 100.0;

        Ok(())
    }

    fn end_loop(&mut self) {
        // Print the each cpu, each iteration
        if self.settings.verbose {
            self.print();
        }

        if self.charging {
            thread::sleep(self.timeout);
        } else {
            thread::sleep(self.timeout_battery);
        }
    }

    fn single_edit(&mut self) -> Result<(), Error> {
        self.start_loop()?;

        let state = self.run_state_machine()?;

        // Check if the state has changed since the last time we checked
        if self.state != state {
            // Log the state change
            self.logger.log(
                &format!("State changed: {:?} -> {:?}", self.state, state,),
                logger::Severity::Log,
            );

            // Change the cpu governor based on the state
            self.set_govs(get_governor(&state)?.to_string())?;
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
            self.grapher.vals.push((check_cpu_temperature(&self.cpus) / 1000.0) as f64);
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

        format!("{}\n\n{}\n\n{}\n{}", graph, stop_message, logs, commit)
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

        // Clear screen
        println!("{}", termion::clear::All);

        // Goto top
        print!("{}", termion::cursor::Goto(1, 1));

        // Print all pre-rendered items
        print!("{}", preprint);

        // Shows if turbo is enabled with an amazing turbo animation
        let mut effective_delay = self.timeout_battery;
        if self.charging {
            effective_delay = self.timeout;
        }
        print_turbo_status(
            cores,
            self.settings.no_animation,
            term_width,
            effective_delay.as_millis().try_into().unwrap(),
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

pub fn daemon_init(settings: Settings, config: Config) -> Result<Daemon, Error> {
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
            println!(
                "{}{}",
                "In order to properly run the daemon in edit mode you must give the executable root privileges.\n",
                "Continuing anyway in 5 seconds...".red()
            );

            if !settings.testing {
                let timeout = time::Duration::from_millis(5000);
                thread::sleep(timeout);
            }

            edit = false;
            forced_reason = "acs was not run as root".to_string();
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
        already_charging: false,
        already_closed: false,
        already_under_powersave_under_percent: false,
        already_high_temp: false,
        already_high_usage: false,
        last_below_cpu_usage_percent: None,
        graph: String::new(),
        grapher: Graph { vals: vec![0.0] },
        temp_max: 0,
        commit_hash: String::new(),
        timeout: time::Duration::from_millis(1),
        timeout_battery: time::Duration::from_millis(2),
        state: State::Unknown,
        settings: new_settings,
    };

    // Make a cpu struct for each cpu listed
    for mut cpu in list_cpus() {
        // Fill that value that were zero with real values
        cpu.init_cpu()?;
        daemon.cpus.push(cpu);
    }

    Ok(daemon)
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
            should_graph: false,
            commit: false,
            testing: true,
        };

        let config = default_config();

        let daemon = daemon_init(settings, config).unwrap();
        assert_eq!(daemon.settings.edit, false);
    }

    #[test]
    fn preprint_render_test_edit_integration_test() {
        let settings = Settings {
            verbose: true,
            delay: 1,
            delay_battery: 2,
            edit: true,
            no_animation: false,
            should_graph: false,
            commit: false,
            testing: true,
        };

        let config = default_config();

        let mut daemon = daemon_init(settings, config).unwrap();
        let preprint = Checker::preprint_render(&mut daemon);
        assert!(preprint.contains("Auto Clock Speed daemon has been initialized in \u{1b}[31medit\u{1b}[0m mode with a delay of 1ms normally and 2ms when on battery\n"));
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
            should_graph: false,
            commit: false,
            testing: true,
        };

        let config = default_config();

        let mut daemon = daemon_init(settings, config).unwrap();
        let preprint = Checker::preprint_render(&mut daemon);
        assert!(preprint.contains("Auto Clock Speed daemon has been initialized in \u{1b}[33mmonitor\u{1b}[0m mode with a delay of 1ms normally and 2ms when on battery\n"));
        assert!(preprint.contains("Name  Max\tMin\tFreq\tTemp\tUsage\tGovernor\n"));
        assert!(preprint.contains("Hz"));
        assert!(preprint.contains("cpu"));
        assert!(preprint.contains("C"));
        assert!(preprint.contains("Battery: "));
    }
}
