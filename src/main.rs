use daemon::{daemon_init, Checker};
use display::{
    print_available_governors, print_cpu_governors, print_cpu_speeds, print_cpu_temp, print_cpus,
    print_freq, print_power, print_turbo,
};
use error::Error;
use power::{read_battery_charge, read_lid_state, read_power_source};
use std::process::exit;
use structopt::StructOpt;
use system::{
    check_available_governors, check_cpu_freq, check_cpu_name, check_turbo_enabled,
    list_cpu_governors, list_cpu_speeds, list_cpu_temp, list_cpus,
};

pub mod cpu;
pub mod daemon;
pub mod display;
pub mod error;
pub mod logger;
pub mod power;
pub mod system;

#[derive(StructOpt)]
enum GetType {
    /// Get the power
    #[structopt(name = "power")]
    Power {
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
enum Command {
    /// Get a specific value or status
    #[structopt(name = "get")]
    Get {
        /// The type of value to request
        #[structopt(subcommand)]
        get: GetType,
    },

    #[structopt(name = "set")]
    Set {
        #[structopt(subcommand)]
        set: SetType,
    },

    /// Run the daemon, this checks and edit your cpu's speed
    #[structopt(name = "run")]
    Run {
        /// Show the information the monitor sub-command outputs
        #[structopt(short, long)]
        quiet: bool,

        /// Milliseconds between update
        #[structopt(short, long, default_value = "1000")]
        delay: u64,
    },

    /// Monitor each cpu, it's min, max, and current speed, along with the governor
    #[structopt(name = "monitor", alias = "monit")]
    Monitor {
        /// Milliseconds between update
        #[structopt(short, long, default_value = "1000")]
        delay: u64,
    },
}

fn main() {
    let mut main_daemon: daemon::Daemon;

    match Command::from_args() {
        // Everything starting with "get"
        Command::Get { get } => match get {
            GetType::Freq { raw } => match check_cpu_freq() {
                Ok(f) => print_freq(f, raw),
                Err(_) => eprintln!("Faild to get cpu frequency"),
            },
            GetType::Power { raw } => match read_lid_state() {
                Ok(lid) => match read_battery_charge() {
                    Ok(bat) => match read_power_source() {
                        Ok(plugged) => {
                            print_power(lid, bat, plugged, raw);
                        }
                        Err(_) => eprintln!("Faild to get read power source"),
                    },
                    Err(_) => eprintln!("Faild to get read battery charger"),
                },
                Err(_) => eprintln!("Faild to get read lid state"),
            },
            GetType::Turbo { raw } => match check_turbo_enabled() {
                Ok(turbo_enabled) => print_turbo(turbo_enabled, raw),
                Err(_) => println!("Failed to get turbo status"),
            },
            GetType::AvailableGovs { raw } => match check_available_governors() {
                Ok(available_governors) => print_available_governors(available_governors, raw),
                Err(_) => println!("Failed to get available governors"),
            },
            GetType::CPUS { raw } => match list_cpus() {
                Ok(cpus) => match check_cpu_name() {
                    Ok(name) => print_cpus(cpus, name, raw),
                    Err(_) => println!("Failed get list of cpus"),
                },
                Err(_) => println!("Failed get list of cpus"),
            },
            GetType::Speeds { raw } => match list_cpu_speeds() {
                Ok(cpu_speeds) => print_cpu_speeds(cpu_speeds, raw),
                Err(_) => println!("Failed to get list of cpu speeds"),
            },
            GetType::Temp { raw } => match list_cpu_temp() {
                Ok(cpu_temp) => print_cpu_temp(cpu_temp, raw),
                Err(_) => println!("Failed to get list of cpu temperature"),
            },
            GetType::Govs { raw } => match list_cpu_governors() {
                Ok(cpu_governors) => print_cpu_governors(cpu_governors, raw),
                Err(_) => println!("Failed to get list of cpu governors"),
            },
        },

        // Everything starting with "set"
        Command::Set { set } => match set {
            SetType::Gov { value } => match daemon_init(true, 0, false) {
                Ok(mut d) => match d.set_govs(value.clone()) {
                    Ok(_) => {}
                    Err(e) => eprint!("Could not set gov, {:?}", e),
                },
                Err(_) => eprint!("Could not run daemon in edit mode"),
            },
        },

        // Run command
        Command::Run { quiet, delay } => match daemon_init(!quiet, delay, true) {
            Ok(d) => {
                main_daemon = d;
                main_daemon.run().unwrap_err();
            }
            Err(_) => eprint!("Could not run daemon in edit mode"),
        },

        // Monitor command
        Command::Monitor { delay } => match daemon_init(true, delay, false) {
            Ok(d) => {
                main_daemon = d;
                main_daemon.run().unwrap_err();
            }
            Err(_) => eprint!("Could not run daemon in monitor mode"),
        },
    }
}
