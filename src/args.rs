use std::{thread, time};
use structopt::StructOpt;

use super::config;
use super::config::{config_dir_exists, init_config};
use super::daemon;
use super::daemon::daemon_init;
use super::display::show_config;
use super::interactive::interactive;
use super::interface::{Get, Getter, Interface, Set, Setter};
use super::settings::{get_graph_type, GraphType, Settings};
use super::warn_user;

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
    CPUS {
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
    #[structopt(name = "bat_cond")]
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

        /// Milliseconds between update
        #[structopt(short, long)]
        delay: Option<u64>,

        /// Milliseconds between update
        #[structopt(short = "b", long = "delay-battery")]
        delay_battery: Option<u64>,

        /// No animations, for systemctl updating issue
        #[structopt(short, long)]
        no_animation: bool,

        /// Graph
        #[structopt(short = "g", long = "--graph")]
        graph_type: Option<String>,

        /// Commit hash
        #[structopt(short, long)]
        commit: bool,
    },

    /// Monitor each cpu, it's min, max, and current speed, along with the governor
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

        /// Hook
        #[structopt(short = "h", long = "--hook")]
        hook: bool,

        /// Graph
        #[structopt(short = "g", long = "--graph")]
        graph_type: Option<String>,

        /// Commit hash
        #[structopt(short, long)]
        commit: bool,
    },
}

pub fn parse_args(config: config::Config) {
    let set_settings = Settings {
        verbose: true,
        delay_battery: 0,
        delay: 0,
        edit: false,
        hook: false,
        no_animation: false,
        graph: GraphType::Hidden,
        commit: false,
        testing: false,
    };

    let int = Interface {
        set: Set {},
        get: Get {},
    };

    match ACSCommand::from_args() {
        ACSCommand::Get { get } => match get {
            GetType::Freq { raw } => {
                int.get.freq(raw);
            }

            GetType::Power { raw } => {
                int.get.power(raw);
            }

            GetType::Usage { raw } => {
                int.get.usage(raw);
            }

            GetType::Thermal { raw } => {
                int.get.thermal(raw);
            }

            GetType::Turbo { raw } => {
                int.get.turbo(raw);
            }
            GetType::AvailableGovs { raw } => {
                int.get.available_govs(raw);
            }
            GetType::CPUS { raw } => {
                int.get.cpus(raw);
            }

            GetType::Speeds { raw } => {
                int.get.speeds(raw);
            }

            GetType::Temp { raw } => {
                int.get.temp(raw);
            }

            GetType::Govs { raw } => {
                int.get.govs(raw);
            }
            GetType::BatCond { raw } => {
                int.get.bat_cond(raw);
            }
        },

        ACSCommand::Set { set } => match set {
            SetType::Gov { value } => {
                int.set.gov(value, config, set_settings);
            }
        },

        ACSCommand::ShowConfig {} => show_config(),
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
        } => {
            if !config_dir_exists() {
                warn_user!("Config directory '/etc/acs' does not exist!");
                thread::sleep(time::Duration::from_millis(5000));
            }

            let mut parsed_graph_type = GraphType::Hidden;

            match graph_type {
                Some(graph_name) => {
                    parsed_graph_type = get_graph_type(&graph_name);
                    if parsed_graph_type == GraphType::Unknown {
                        warn_user!("Graph type does not exist! Can be freq, usage, or temp Continuing in 5 seconds...");
                        thread::sleep(time::Duration::from_millis(5000));
                    }
                }
                None => {}
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
            };

            match daemon_init(settings, config) {
                Ok(d) => {
                    daemon::run(d).unwrap_err();
                }
                Err(_) => eprint!("Could not run daemon in edit mode"),
            }
        }

        // Monitor command
        ACSCommand::Monitor {
            delay,
            delay_battery,
            no_animation,
            graph_type,
            hook,
            commit,
        } => {
            if !config_dir_exists() {
                warn_user!("Config directory '/etc/acs' does not exist!");
            }

            let mut parsed_graph_type = GraphType::Hidden;

            match graph_type {
                Some(graph_name) => {
                    parsed_graph_type = get_graph_type(&graph_name);
                    if parsed_graph_type == GraphType::Unknown {
                        warn_user!("Graph type does not exist! Can be freq, usage, or temp Continuing in 5 seconds...");
                        thread::sleep(time::Duration::from_millis(5000));
                    }
                }
                None => {}
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
            };

            match daemon_init(settings, config) {
                Ok(d) => {
                    daemon::run(d).unwrap_err();
                }
                Err(_) => eprint!("Could not run daemon in monitor mode"),
            }
        }
    }
}
