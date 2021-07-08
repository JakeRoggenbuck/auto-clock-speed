use display::{
    print_available_governors, print_cpu_governors, print_cpu_speeds, print_cpus, print_freq,
    print_turbo,
};
use error::Error;
use structopt::StructOpt;
use system::{
    check_available_governors, check_cpu_freq, check_turbo_enabled, list_cpu_governors,
    list_cpu_speeds, list_cpus,
};

pub mod display;
pub mod error;
pub mod system;

const GOVERNORS: [&str; 6] = [
    "performance",
    "powersave",
    "userspace",
    "ondemand",
    "conservative",
    "schedutil",
];

#[derive(StructOpt)]
#[structopt(
    name = "autoclockspeed",
    about = "Automatic CPU frequency scaler and power saver"
)]
enum Command {
    /// The overall frequency of your cpu
    #[structopt(name = "get-freq")]
    GetFreq {
        #[structopt(short, long)]
        raw: bool,
    },

    #[structopt(name = "get-turbo")]
    GetTurbo {
        #[structopt(short, long)]
        raw: bool,
    },

    #[structopt(name = "get-available-governors")]
    GetAvailableGovernors {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The names of the core
    #[structopt(name = "get-cpus")]
    GetCPUS {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The speed of the individual cores
    #[structopt(name = "get-cpu-speeds")]
    GetSpeeds {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The governors of the individual cores
    #[structopt(name = "get-cpu-governors")]
    GetGovernors {
        #[structopt(short, long)]
        raw: bool,
    },

    /// The possible governors
    #[structopt(name = "list-possible-governors")]
    GetPossibleGovernorsList { },
}

fn main() {
    match Command::from_args() {
        Command::GetFreq { raw } => match check_cpu_freq() {
            Ok(f) => print_freq(f, raw),
            Err(_) => eprintln!("Faild to get cpu frequency"),
        },
        Command::GetTurbo { raw } => match check_turbo_enabled() {
            Ok(turbo_enabled) => print_turbo(turbo_enabled, raw),
            Err(_) => println!("Failed to get turbo status"),
        },
        Command::GetAvailableGovernors { raw } => match check_available_governors() {
            Ok(available_governors) => print_available_governors(available_governors, raw),
            Err(_) => println!("Failed to get available governors"),
        },
        Command::GetCPUS { raw } => match list_cpus() {
            Ok(cpus) => print_cpus(cpus, raw),
            Err(_) => println!("Failed get list of cpus"),
        },
        Command::GetSpeeds { raw } => match list_cpu_speeds() {
            Ok(cpu_speeds) => print_cpu_speeds(cpu_speeds, raw),
            Err(_) => println!("Failed to get list of cpu speeds"),
        },
        Command::GetGovernors { raw } => match list_cpu_governors() {
            Ok(cpu_governors) => print_cpu_governors(cpu_governors, raw),
            Err(_) => println!("Failed to get list of cpu governors"),
        },
        Command::GetPossibleGovernorsList { } => {
            for governor in GOVERNORS.iter() {
                println!("{}", governor);
            }
        },
    }
}
