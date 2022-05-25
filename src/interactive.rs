use super::interface::{Get, Getter, Interface, Set};
use std::io::{stdin, stdout, Write};

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
                    "get usage" => int.get.usage(false),
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
