use super::interface::{Get, Getter, Interface, Set};
use std::io::{stdin, stdout, Write};

pub fn help() {
    println!("- get");
    println!("  - freq");
    println!("  - cpus");
    println!("  - temp");
    println!("  - govs");
    println!("  - power");
    println!("  - usage");
    println!("  - turbo");
    println!("  - available_governors");
    println!("\nE.g. 'get cpus'");
}

pub fn interactive() {
    let int = Interface {
        set: Set {},
        get: Get {},
    };

    let mut input;

    println!("Auto Clock Speed Interactive Mode:");

    loop {
        print!("\n> ");
        stdout().flush().unwrap();

        input = String::new();

        match stdin().read_line(&mut input) {
            Ok(_) => {
                input.pop();
                let new = input.as_str();
                match new {
                    "help" => help(),
                    "get freq" => int.get.freq(false),
                    "get power" => int.get.power(false),
                    "get usage" => int.get.usage(false),
                    "get turbo" => int.get.turbo(false),
                    "get available_governors" => int.get.available_govs(false),
                    "get cpus" => int.get.cpus(false),
                    "get speeds" => int.get.speeds(false),
                    "get temp" => int.get.temp(false),
                    "get govs" => int.get.govs(false),

                    "exit" => {
                        println!("Bye!");
                        return;
                    }
                    _ => println!("Command '{new}' not found. Use help."),
                };
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
