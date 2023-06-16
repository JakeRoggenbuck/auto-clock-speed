use std::{thread, time};
use structopt::StructOpt;

use super::config;
use super::config::init_config;
use super::daemon;
use super::daemon::daemon_init;
use super::display::show_config;
use super::graph::{get_graph_type, GraphType};
use super::interactive::interactive;
use super::interface::{DaemonControl, DaemonController, Get, Getter, Interface, Set, Setter};
use super::settings::Settings;
use super::setup::check_config_dir_exists;
use super::warn_user;

#[derive(StructOpt)]
enum DaemonControlType {
    #[structopt(name = "disable")]
    Disable,
    #[structopt(name = "enable")]
    Enable,
    #[structopt(name = "status")]
    Status,
    #[structopt(name = "toggle")]
    Toggle,
}

#[derive(StructOpt)]
enum GetType {
    /// Get the power
    #[structopt(name = "power")]
    Power {
        #[structopt(short, long)]
        raw: bool,
    },

    /// Get the thermal zones
    #[structopt(name = "thermal")]
    Thermal {
        #[structopt(short, long)]
        raw: bool,
    },

    /// Get the power
    #[structopt(name = "usage")]
    Usage {
        #[structopt(short, long)]
        raw: bool,

        #[structopt(short, long)]
        delay: Option<u64>,
    },

    /// The overall frequency of your cpu
    #[structopt(name = "freq")]
    Freq {
        #[structopt(short, long)]
        raw: bool,
    },

    /// Get whether turbo is enabled or not
    #[structopt(name = "turbo")]
    Turbo {
        #[structopt(short, long)]
        raw: bool,
    },

    /// Get the available governor
    #[structopt(name = "available-govs")]
    AvailableGovs {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The names of the core
    #[structopt(name = "cpus")]
    CPUs {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The speed of the individual cores
    #[structopt(name = "speeds")]
    Speeds {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The temperature of the individual cores
    #[structopt(name = "temp")]
    Temp {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The governors of the individual cores
    #[structopt(name = "govs")]
    Govs {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The battery condition in percentage
    #[structopt(name = "bat-cond")]
    BatCond {
        #[structopt(short, long)]
        raw: bool,
    },
}

#[derive(StructOpt)]
enum SetType {
    #[structopt(name = "gov")]
    Gov {
        #[structopt()]
        value: String,
    },
}

#[derive(StructOpt)]
#[structopt(
    name = "autoclockspeed",
    about = "\
_
         %%%@%%
      %%%%%%@%%%%
     %%%                   @@      @@@@   @@@@@
    %%%     %             @  @   @@       @@
    %%%    %%%           @@@@@@  @@          @@
     %%%                @@    @@   @@@@   @@@@@
      %%%%%%@%%%%%
         %%%@%%
_
    \
    Automatic CPU frequency scaler and power saver"
)]

enum ACSCommand {
    /// Controls interaction with a running daemon
    #[structopt(name = "daemon", alias = "d")]
    Daemon {
        /// The type of data to request or control
        #[structopt(subcommand)]
        control: DaemonControlType,
    },

    /// Get a specific value or status
    #[structopt(name = "get", alias = "g")]
    Get {
        /// The type of value to request
        #[structopt(subcommand)]
        get: GetType,
    },

    /// Set a specific value
    #[structopt(name = "set", alias = "s")]
    Set {
        #[structopt(subcommand)]
        set: SetType,
    },

    /// Interactive mode for auto clock speed commands
    #[structopt(name = "interactive", alias = "i")]
    Interactive {},

    /// Show the current config in use
    #[structopt(name = "showconfig", alias = "conf")]
    ShowConfig {},

    /// Initialize config
    #[structopt(name = "initconfig")]
    InitConfig {},

    /// Run the daemon, this checks and edit your cpu's speed
    #[structopt(name = "run")]
    Run {
        /// Show the information the monitor sub-command outputs
        #[structopt(short, long)]
        quiet: bool,

        /// Output the settings and quit
        #[structopt(short, long)]
        show_settings: bool,

        /// Milliseconds between update (when charging)
        #[structopt(short, long)]
        delay: Option<u64>,

        /// Milliseconds between update (when on battery)
        #[structopt(short = "b", long = "delay-battery")]
        delay_battery: Option<u64>,

        /// No animations, for systemctl updating issue
        #[structopt(short, long)]
        no_animation: bool,

        /// Graph
        #[structopt(short = "g", long = "graph")]
        graph_type: Option<String>,

        /// Commit hash
        #[structopt(short, long)]
        commit: bool,

        /// Write to a csv file
        #[structopt(long = "csv")]
        csv_file: Option<String>,

        /// Log file size cutoff in MB
        #[structopt(long = "log-size-cutoff", default_value = "20")]
        log_size_cutoff: i32,
    },

    /// Monitor each cpu, its min, max, and current speed, along with the governor
    #[structopt(name = "monitor", alias = "monit")]
    Monitor {
        /// Milliseconds between update
        #[structopt(short, long)]
        delay: Option<u64>,

        /// Milliseconds between update
        #[structopt(short = "b", long = "delay-battery")]
        delay_battery: Option<u64>,

        /// No animations, for systemctl updating issue
        #[structopt(short, long)]
        no_animation: bool,

        /// Output the settings and quit
        #[structopt(short, long)]
        show_settings: bool,

        /// Hook
        #[structopt(short = "h", long = "--hook")]
        hook: bool,

        /// Graph "freq", "usage", or "temp"
        #[structopt(short = "g", long = "--graph")]
        graph_type: Option<String>,

        /// Commit hash
        #[structopt(short, long)]
        commit: bool,

        /// Write to a csv file
        #[structopt(long = "csv")]
        csv_file: Option<String>,

        /// Log file size cutoff in MB
        #[structopt(long = "log-size-cutoff", default_value = "20")]
        log_size_cutoff: i32,
    },
}

pub fn parse_args(config: config::Config) {
    let set_settings = Settings::default();

    let int = Interface {
        set: Set {},
        get: Get {},
        dec: DaemonControl {},
    };

    match ACSCommand::from_args() {
        ACSCommand::Daemon { control } => match control {
            DaemonControlType::Disable => int.dec.disable(),
            DaemonControlType::Enable => int.dec.enable(),
            DaemonControlType::Status => int.dec.status(),
            DaemonControlType::Toggle => int.dec.toggle(),
        },

        ACSCommand::Get { get } => match get {
            GetType::Freq { raw } => int.get.freq(raw),
            GetType::Power { raw } => int.get.power(raw),
            GetType::Usage { raw, delay } => int.get.usage(raw, delay),
            GetType::Thermal { raw } => int.get.thermal(raw),
            GetType::Turbo { raw } => int.get.turbo(raw),
            GetType::AvailableGovs { raw } => int.get.available_govs(raw),
            GetType::CPUs { raw } => int.get.cpus(raw),
            GetType::Speeds { raw } => int.get.speeds(raw),
            GetType::Temp { raw } => int.get.temp(raw),
            GetType::Govs { raw } => int.get.govs(raw),
            GetType::BatCond { raw } => int.get.bat_cond(raw),
        },

        ACSCommand::Set { set } => match set {
            SetType::Gov { value } => int.set.gov(value, config, set_settings),
        },

        ACSCommand::ShowConfig {} => show_config(&config),
        ACSCommand::InitConfig {} => init_config(),
        ACSCommand::Interactive {} => interactive(),

        // Run command
        ACSCommand::Run {
            quiet,
            delay,
            delay_battery,
            no_animation,
            graph_type,
            commit,
            csv_file,
            log_size_cutoff,
            show_settings,
        } => {
            check_config_dir_exists();

            let mut parsed_graph_type = GraphType::Hidden;
            if let Some(gt) = graph_type {
                parsed_graph_type = get_graph_type(&gt);
                if parsed_graph_type == GraphType::Unknown {
                    warn_user!("Graph type does not exist! Can be freq, usage, or temp Continuing in 5 seconds...");
                    thread::sleep(time::Duration::from_millis(5000));
                }
            }

            let mut effective_delay_battery = delay_battery.unwrap_or(5000);
            let regular_delay = delay.unwrap_or(1000);
            if parsed_graph_type != GraphType::Hidden {
                effective_delay_battery = regular_delay;
            }

            let settings = Settings {
                verbose: !quiet,
                delay_battery: effective_delay_battery,
                delay: regular_delay,
                edit: true,
                no_animation,
                hook: false,
                graph: parsed_graph_type,
                commit,
                testing: false,
                log_csv: csv_file.is_some(),
                csv_file: csv_file.unwrap_or_else(|| "/tmp/acs/".to_string()),
                log_size_cutoff,
                show_settings,
            };

            let d = daemon_init(settings, config);
            daemon::run(d).expect("Daemon did not run successfully.");
        }

        // Monitor command
        ACSCommand::Monitor {
            delay,
            delay_battery,
            no_animation,
            graph_type,
            hook,
            commit,
            csv_file,
            log_size_cutoff,
            show_settings,
        } => {
            check_config_dir_exists();

            let mut parsed_graph_type = GraphType::Hidden;

            if let Some(gt) = graph_type {
                parsed_graph_type = get_graph_type(&gt);
                if parsed_graph_type == GraphType::Unknown {
                    warn_user!("Graph type does not exist! Can be freq, usage, or temp Continuing in 5 seconds...");
                    thread::sleep(time::Duration::from_millis(5000));
                }
            }

            let mut effective_delay_battery = delay_battery.unwrap_or(5000);
            let regular_delay = delay.unwrap_or(1000);
            if parsed_graph_type != GraphType::Hidden {
                effective_delay_battery = regular_delay;
            }

            let settings = Settings {
                verbose: true,
                delay: regular_delay,
                delay_battery: effective_delay_battery,
                edit: false,
                hook,
                no_animation,
                graph: parsed_graph_type,
                commit,
                testing: false,
                log_csv: csv_file.is_some(),
                csv_file: csv_file.unwrap_or_else(|| "/tmp/acs/".to_string()),
                log_size_cutoff,
                show_settings,
            };

            let d = daemon_init(settings, config);
            daemon::run(d).expect("Daemon did not run successfully.");
        }
    }
}
