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
    GetFreq {
        #[structopt(short, long)]
        raw: bool,
    },

    #[structopt(name = "get-turbo")]
    GetTurbo {
        #[structopt(short, long)]
        raw: bool,
    },
}

fn main() {
    match Command::from_args() {
        Command::GetFreq { raw } => match check_cpu_freq() {
            Ok(f) => {
                if raw {
                    println!("{}", f);
                } else {
                    println!("CPU freq is {} MHz", f)
                }
            }
            Err(_) => println!("Failed"),
        },
        Command::GetTurbo { raw } => match check_turbo_enabled() {
            Ok(a) => {
                if raw {
                    println!("{}", a);
                    return;
                }

                println!(
                    "{}",
                    if a {
                        "Turbo is enabled"
                    } else {
                        "Turbo is not enabled"
                    }
                )
            }
            Err(_) => println!("Failed"),
        },
    }
}
