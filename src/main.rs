use error::Error;
use structopt::StructOpt;
use system::{check_cpu_freq, check_turbo_enabled};

pub mod error;
pub mod system;

#[derive(StructOpt)]
#[structopt(
    name = "autoclockspeed",
    about = "Automatic CPU frequency scaler and power saver"
)]
enum Command {
    #[structopt(name = "get-freq")]
    GetFreq,

    #[structopt(name = "get-turbo")]
    GetTurbo,
}

fn main() {
    match Command::from_args() {
        Command::GetFreq {} => match check_cpu_freq() {
            Ok(f) => println!("CPU freq is {} MHz", f),
            Err(_) => println!("Failed"),
        },
        Command::GetTurbo {} => match check_turbo_enabled() {
            Ok(a) => println!(
                "{}",
                if a {
                    "Turbo is enabled"
                } else {
                    "Turbo is not enabled"
                }
            ),
            Err(_) => println!("Failed"),
        },
    }
}
