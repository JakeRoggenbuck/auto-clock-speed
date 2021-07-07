use std::path::PathBuf;
use structopt::StructOpt;

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
            // TODO: Make it return current CPU freq
            // Pls let me do this -Cameron
            println!("{:?}", verbose);
        }
    }
}
