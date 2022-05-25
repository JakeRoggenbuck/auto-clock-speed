use super::config::{get_config, Config};
use super::interface::{Get, Getter, Interface, Set, Setter};
use super::settings::Settings;
use std::io::{stdin, stdout, Write};

pub fn interactive() {
    let int = Interface {
        set: Set {},
        get: Get {},
    };

    let mut input;

    // let set_settings = Settings {
    //     verbose: true,
    //     delay_battery: 0,
    //     delay: 0,
    //     edit: false,
    //     no_animation: false,
    //     should_graph: false,
    //     commit: false,
    //     testing: false,
    // };

    // let config: Config = get_config();

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
                    "get freq" => int.get.freq(false),
                    "get power" => int.get.power(false),
                    "get usage" => int.get.usage(false),
                    "get turbo" => int.get.turbo(false),
                    "get available_governors" => int.get.available_govs(false),
                    "get cpus" => int.get.cpus(false),
                    "get speeds" => int.get.speeds(false),
                    "get temp" => int.get.temp(false),
                    "get govs" => int.get.govs(false),

                    // "set gov performance" => {
                    //     int.set.gov("performance".to_string(), config, set_settings)
                    // }
                    // "set gov powersave" => {
                    //     int.set.gov("powersave".to_string(), config, set_settings)
                    // }
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
