use super::config::{get_config, Config};
use super::interface::{DaemonControl, DaemonController, Get, Getter, Interface, Set, Setter};
use super::settings::{GraphType, Settings};
use colored::Colorize;
use std::io::{stdin, stdout, Write};

pub fn help() {
    const HELP_TEXT: &str = "\
- exit

- get
  - freq
  - cpus
  - temp
  - govs
  - power
  - usage
  - turbo
  - speeds
  - available_governors
  - battery_condition

- set
  - gov

- daemon
  - disable
  - enable
  - status
  - toggle

E.g. 'get cpus'
    ";

    println!("{}\n", "Help:".bold().green());
    println!("{HELP_TEXT}")
}

pub fn interactive() {
    let int = Interface {
        set: Set {},
        get: Get {},
        dec: DaemonControl {},
    };

    let mut input;

    println!("{}", "Auto Clock Speed".bold());
    println!("{}", "Interactive Mode".bold().blue());

    let set_settings = Settings {
        verbose: true,
        delay_battery: 0,
        delay: 0,
        edit: false,
        hook: false,
        no_animation: false,
        graph: GraphType::Hidden,
        commit: false,
        testing: false,
        csv_file: None,
        csv_enabled: false,
        log_size_cutoff: 20,
    };

    loop {
        print!("{}", "\n> ".bold().green());
        stdout().flush().expect("Failed to flush stdout");

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
                    "get battery_condition" => int.get.bat_cond(false),

                    "set gov performance" => {
                        let config: Config = get_config();

                        int.set
                            .gov("performance".to_string(), config, set_settings.clone());
                    }

                    "set gov powersave" => {
                        let config: Config = get_config();

                        int.set
                            .gov("powersave".to_string(), config, set_settings.clone());
                    }
                    "daemon disable" => int.dec.disable(),
                    "daemon enable" => int.dec.enable(),
                    "daemon status" => int.dec.status(),
                    "daemon toggle" => int.dec.toggle(),

                    "exit" => {
                        println!("Bye!");
                        return;
                    }
                    _ => println!(
                        "{}",
                        format!("Command '{}' not found. Use 'help'.", new).red()
                    ),
                };
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
