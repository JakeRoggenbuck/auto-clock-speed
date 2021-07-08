use display::{print_available_governors, print_cpus, print_freq, print_turbo};
use error::Error;
use structopt::StructOpt;
use system::{check_available_governors, check_cpu_freq, check_turbo_enabled, list_cpus};

pub mod display;
pub mod error;
pub mod system;

#[derive(StructOpt)]
#[structopt(
    name = "autoclockspeed",
    about = "Automatic CPU frequency scaler and power saver"
)]
enum Command {
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

    #[structopt(name = "get-governors")]
    GetAvailableGovernors {
        #[structopt(short, long)]
        raw: bool,
    },

    #[structopt(name = "get-cpus")]
    GetCPUS {
        #[structopt(short, long)]
        raw: bool,
    },
}

fn main() {
    match Command::from_args() {
        Command::GetFreq { raw } => match check_cpu_freq() {
            Ok(f) => print_freq(f, raw),
            Err(_) => eprintln!("Failed"),
        },
        Command::GetTurbo { raw } => match check_turbo_enabled() {
            Ok(a) => print_turbo(a, raw),
            Err(_) => println!("Failed"),
        },
        Command::GetAvailableGovernors { raw } => match check_available_governors() {
            Ok(a) => print_available_governors(a, raw),
            Err(_) => println!("Failed"),
        },
        Command::GetCPUS { raw } => match list_cpus() {
            Ok(a) => print_cpus(a, raw),
            Err(_) => println!("Failed"),
        },
    }
}
