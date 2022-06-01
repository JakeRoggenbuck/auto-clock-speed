use log::debug;
use structopt::StructOpt;
use std::{thread, time};

use config::{config_dir_exists, get_config};
use daemon::{daemon_init, Checker};
use display::show_config;
use error::Error;
use interactive::interactive;
use interface::{Get, Getter, Interface, Set, Setter};
use settings::{Settings, GraphType, get_graph_type};

pub mod config;
pub mod cpu;
pub mod daemon;
pub mod display;
pub mod error;
pub mod graph;
pub mod interactive;
pub mod interface;
pub mod logger;
pub mod power;
pub mod settings;
pub mod state;
pub mod system;
pub mod terminal;

#[derive(StructOpt)]
enum GetType {
    /// Get the power
    #[structopt(name = "power")]
    Power {
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
    about = "Automatic CPU frequency scaler and power saver"
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

    /// Run the daemon, this checks and edit your cpu's speed
    #[structopt(name = "run")]
    Run {
        /// Show the information the monitor sub-command outputs
        #[structopt(short, long)]
        quiet: bool,

        /// Milliseconds between update
        #[structopt(short, long, default_value = "1000")]
        delay: u64,

        /// Milliseconds between update
        #[structopt(short = "b", long = "delay-battery", default_value = "5000")]
        delay_battery: u64,

        /// No animations, for systemctl updating issue
        #[structopt(short, long)]
        no_animation: bool,

        /// Graph
        #[structopt(short = "g", long = "--graph", default_value = "none")]
        should_graph: String,

        /// Commit hash
        #[structopt(short, long)]
        commit: bool,
    },

    /// Monitor each cpu, it's min, max, and current speed, along with the governor
    #[structopt(name = "monitor", alias = "monit")]
    Monitor {
        /// Milliseconds between update when on AC
        #[structopt(short, long, default_value = "1000")]
        delay: u64,

        /// Milliseconds between update
        #[structopt(short = "b", long = "delay-battery", default_value = "5000")]
        delay_battery: u64,

        /// No animations, for systemctl updating issue
        #[structopt(short, long)]
        no_animation: bool,

        /// Graph
        #[structopt(short = "g", long = "--graph", default_value = "none")]
        should_graph: String,

        /// Commit hash
        #[structopt(short, long)]
        commit: bool,
    },
}

fn parse_args(config: config::Config) {
    let mut daemon: daemon::Daemon;

    let set_settings = Settings {
        verbose: true,
        delay_battery: 0,
        delay: 0,
        edit: false,
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
        },

        ACSCommand::Set { set } => match set {
            SetType::Gov { value } => {
                int.set.gov(value, config, set_settings);
            }
        },

        ACSCommand::ShowConfig {} => show_config(),
        ACSCommand::Interactive {} => interactive(),

        // Run command
        ACSCommand::Run {
            quiet,
            delay,
            delay_battery,
            no_animation,
            should_graph,
            commit,
        } => {
            if !config_dir_exists() {
                warn_user!("Config directory '/etc/acs' does not exist!");
                thread::sleep(time::Duration::from_millis(5000));
            }

            let parsed_graph_type = match get_graph_type(&should_graph) {
                Some(graph_type) => graph_type,
                None => {
                    warn_user!("Graph type is not set! Can be hidden, frequency, frequency_individual, usage, usage_individual, temperature, temperature_individual. Continuing in 5 seconds...");
                    thread::sleep(time::Duration::from_millis(5000));
                    GraphType::Hidden
                }
            };

            let mut effective_delay_battery = delay_battery;
            if parsed_graph_type != GraphType::Hidden || delay != 1000 {
                effective_delay_battery = delay;
            }

            let settings = Settings {
                verbose: !quiet,
                delay_battery: effective_delay_battery,
                delay,
                edit: true,
                no_animation,
                graph: parsed_graph_type,
                commit,
                testing: false,
            };

            match daemon_init(settings, config) {
                Ok(d) => {
                    daemon = d;
                    daemon.run().unwrap_err();
                }
                Err(_) => eprint!("Could not run daemon in edit mode"),
            }
        }

        // Monitor command
        ACSCommand::Monitor {
            delay,
            delay_battery,
            no_animation,
            should_graph,
            commit,
        } => {
            if !config_dir_exists() {
                warn_user!("Config directory '/etc/acs' does not exist!");
            }
            

            let parsed_graph_type = match get_graph_type(&should_graph) {
                Some(graph_type) => graph_type,
                None => {
                    warn_user!("Graph type is not set! Can be hidden, frequency, frequency_individual, usage, usage_individual, temperature, temperature_individual. Continuing in 5 seconds...");
                    thread::sleep(time::Duration::from_millis(5000));
                    GraphType::Hidden
                }
            };

            let mut effective_delay_battery = delay_battery;
            if parsed_graph_type != GraphType::Hidden || delay != 1000 {
                effective_delay_battery = delay;
            }

            let settings = Settings {
                verbose: true,
                delay,
                delay_battery: effective_delay_battery,
                edit: false,
                no_animation,
                graph: parsed_graph_type,
                commit,
                testing: false,
            };

            match daemon_init(settings, config) {
                Ok(d) => {
                    daemon = d;
                    daemon.run().unwrap_err();
                }
                Err(_) => eprint!("Could not run daemon in monitor mode"),
            }
        }
    }
}

fn main() {
    env_logger::init();

    let config: config::Config = get_config();

    parse_args(config);
}
