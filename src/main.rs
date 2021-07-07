use std::fs::File;
use std::io::{Read};
use structopt::StructOpt;

enum Error {
    IO(std::io::Error),
    Unknown,
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Error {
        Error::IO(e)
    }
}

#[derive(StructOpt)]
#[structopt(name = "autoclockspeed", about = "Automatic CPU frequency scaler and power saver")]
enum Command {
    #[structopt(name = "get-freq")]
    Get {
            // Any way to not need to repeat this for each command
            #[structopt(short, long)]
            verbose: bool,
    }
}

fn main() {
    match Command::from_args() {
        Command::Get {verbose} => {
            if verbose {
                println!("Verbose mode is turned on");
            }
            match get_cpu_freq() {
                Ok(f) => println!("CPU freq is {} MHz", f),
                Err(_) => println!("Failed")
            }
        }
    }
}

//https://docs.rs/sys-info/0.7.0/src/sys_info/lib.rs.html#367-406
// TODO: Move this to it's own file
fn get_cpu_freq() -> Result<i32, Error> {
    let mut cpu_info = String::new();
    File::open("/proc/cpuinfo")?.read_to_string(&mut cpu_info)?;

    // Find all lines that begin with cpu MHz
    let find_cpu_mhz = cpu_info.split('\n').find(|line|
        line.starts_with("cpu MHz\t") ||
        line.starts_with("BogoMIPS") ||
        line.starts_with("clock\t") ||
        line.starts_with("bogomips per cpu")
    );

    // For each line that starts with the clock speed identifier return the number after : as a 32
    // bit integer
    find_cpu_mhz.and_then(|line| line.split(':').last())
            .and_then(|val| val.replace("MHz", "").trim().parse::<f64>().ok())
            .map(|speed| speed as i32)
            .ok_or(Error::Unknown)
    
}
