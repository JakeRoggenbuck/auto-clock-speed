use structopt::StructOpt;
use error::Error;
use system::get_cpu_freq;

pub mod error;
pub mod system;


#[derive(StructOpt)]
#[structopt(
    name = "autoclockspeed",
    about = "Automatic CPU frequency scaler and power saver"
)]
enum Command {
    #[structopt(name = "get-freq")]
    Get {
        // Any way to not need to repeat this for each command
        #[structopt(short, long)]
        verbose: bool,
    },
}

fn main() {
    match Command::from_args() {
        Command::Get { verbose } => {
            if verbose {
                println!("Verbose mode is turned on");
            }
            match get_cpu_freq() {
                Ok(f) => println!("CPU freq is {} MHz", f),
                Err(_) => println!("Failed"),
            }
        }
    }
}
