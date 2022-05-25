use super::config::{config_dir_exists, get_config};
use super::daemon::{daemon_init, Checker};
use super::display::{
    print_available_governors, print_cpu_governors, print_cpu_speeds, print_cpu_temp, print_cpus,
    print_freq, print_power, print_turbo, show_config,
};
use super::error::Error;
use super::interactive::interactive;
use super::power::{read_battery_charge, read_lid_state, read_power_source};
use super::system::{
    check_available_governors, check_cpu_freq, check_cpu_name, check_turbo_enabled,
    get_cpu_percent, list_cpu_governors, list_cpu_speeds, list_cpu_temp, list_cpus,
};

pub struct Get {}

pub trait Getter {
    fn freq(self, raw: bool);
    fn power(self, raw: bool);
    fn usage(self, raw: bool);
    fn turbo(self, raw: bool);
    fn available_govs(self, raw: bool);
    fn cpus(self, raw: bool);
    fn speeds(self, raw: bool);
    fn temp(self, raw: bool);
    fn govs(self, raw: bool);
}

impl Getter for Get {
    fn freq(self, raw: bool) {
        let f = check_cpu_freq();
        print_freq(f, raw);
    }

    fn power(self, raw: bool) {
        match read_lid_state() {
            Ok(lid) => match read_battery_charge() {
                Ok(bat) => match read_power_source() {
                    Ok(plugged) => {
                        print_power(lid, bat, plugged, raw);
                    }
                    Err(_) => eprintln!("Failed to get read power source"),
                },
                Err(_) => eprintln!("Failed to get read battery charger"),
            },
            Err(_) => eprintln!("Failed to get read lid state"),
        };
    }

    fn usage(self, raw: bool) {
        if !raw {
            println!("Calculating cpu percentage over 1 second.");
        }
        match get_cpu_percent() {
            Ok(content) => {
                if raw {
                    println!("{}", content)
                } else {
                    println!("CPU is at {}%", content)
                }
            }
            Err(_) => println!("Unable to usage status"),
        }
    }

    fn turbo(self, raw: bool) {
        match check_turbo_enabled() {
            Ok(turbo_enabled) => print_turbo(turbo_enabled, raw),
            Err(_) => println!("Failed to get turbo status"),
        };
    }

    fn available_govs(self, raw: bool) {
        match check_available_governors() {
            Ok(available_governors) => print_available_governors(available_governors, raw),
            Err(_) => println!("Failed to get available governors"),
        };
    }

    fn cpus(self, raw: bool) {
        let cpus = list_cpus();
        match check_cpu_name() {
            Ok(name) => print_cpus(cpus, name, raw),
            Err(_) => println!("Failed get list of cpus"),
        };
    }

    fn speeds(self, raw: bool) {
        let speeds = list_cpu_speeds();
        print_cpu_speeds(speeds, raw);
    }

    fn temp(self, raw: bool) {
        let cpu_temp = list_cpu_temp();
        print_cpu_temp(cpu_temp, raw);
    }

    fn govs(self, raw: bool) {
        let govs = list_cpu_governors();
        print_cpu_governors(govs, raw);
    }
}

pub struct Set {}

pub trait Setter {
    fn gov(self, value);
}

impl Setter for Set {
    fn gov(self, value) {
        match daemon_init(set_settings, config) {
            Ok(mut d) => match d.set_govs(value.clone()) {
                Ok(_) => {}
                Err(e) => eprint!("Could not set gov, {:?}", e),
            },
            Err(_) => eprint!("Could not run daemon in edit mode"),
        }
    }
}

pub struct Interface {
    pub get: Get,
    pub set: Set,
}
