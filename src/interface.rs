use super::config::Config;
use super::daemon::{daemon_init, Checker};
use super::display::{
    print_available_governors, print_bat_cond, print_cpu_governors, print_cpu_speeds,
    print_cpu_temp, print_cpus, print_freq, print_power, print_turbo,
};
use super::power::battery::Battery;
use super::power::lid::read_lid_state;
use super::power::{Power, PowerRetriever};
use super::settings::Settings;
use super::system::{
    check_available_governors, check_cpu_freq, check_cpu_name, check_turbo_enabled,
    get_cpu_percent, list_cpu_governors, list_cpu_speeds, list_cpu_temp, list_cpus,
};
use super::thermal::read_thermal_zones;
use crate::network::send::query_one;

pub struct DaemonControl {}

pub trait DaemonController {
    fn disable(&self);
    fn enable(&self);
    fn status(&self);
    fn toggle(&self);
}

impl DaemonController for DaemonControl {
    fn disable(&self) {
        match query_one(
            "/tmp/acs.sock",
            crate::network::Packet::DaemonDisableRequest(),
        ) {
            Ok(packet) => match packet {
                crate::network::Packet::DaemonDisableResponse(success) => match success {
                    true => println!("The running daemon has been disabled"),
                    false => println!("The running daemon is already disabled"),
                },
                _ => println!("Failed: Unexpected response packet"),
            },
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }

    fn enable(&self) {
        match query_one(
            "/tmp/acs.sock",
            crate::network::Packet::DaemonEnableRequest(),
        ) {
            Ok(packet) => match packet {
                crate::network::Packet::DaemonEnableResponse(success) => match success {
                    true => println!("The running daemon has been enabled"),
                    false => println!("The running daemon is already enabled"),
                },
                _ => println!("Failed: Unexpected response packet"),
            },
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }

    fn status(&self) {
        match query_one(
            "/tmp/acs.sock",
            crate::network::Packet::DaemonStatusRequest(),
        ) {
            Ok(packet) => match packet {
                crate::network::Packet::DaemonStatusResponse(status) => match status {
                    true => println!("The daemon is currently enabled"),
                    false => println!("The daemon is currently disabled"),
                },
                _ => println!("Failed: Unexpected response packet"),
            },
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }

    fn toggle(&self) {
        match query_one(
            "/tmp/acs.sock",
            crate::network::Packet::DaemonStatusRequest(),
        ) {
            Ok(packet) => match packet {
                crate::network::Packet::DaemonStatusResponse(status) => {
                    match query_one(
                        "/tmp/acs.sock",
                        match status {
                            true => crate::network::Packet::DaemonDisableRequest(),
                            false => crate::network::Packet::DaemonEnableRequest(),
                        },
                    ) {
                        Ok(packet) => match packet {
                            crate::network::Packet::DaemonDisableResponse(_) => {
                                println!("The running daemon has been disabled")
                            }
                            crate::network::Packet::DaemonEnableResponse(_) => {
                                println!("The running daemon has been enabled")
                            }
                            _ => println!("Failed: Unexpected response packet"),
                        },
                        Err(e) => {
                            println!("): {:?}", e)
                        }
                    }
                }
                _ => println!("Failed: Unexpected response packet"),
            },
            Err(e) => {
                println!("{:?}", e)
            }
        }
    }
}

pub struct Get {}

pub trait Getter {
    fn freq(&self, raw: bool);
    fn power(&self, raw: bool);
    fn usage(&self, raw: bool);
    fn thermal(&self, raw: bool);
    fn turbo(&self, raw: bool);
    fn available_govs(&self, raw: bool);
    fn cpus(&self, raw: bool);
    fn speeds(&self, raw: bool);
    fn temp(&self, raw: bool);
    fn govs(&self, raw: bool);
    fn bat_cond(&self, raw: bool);
}

impl Getter for Get {
    fn freq(&self, raw: bool) {
        let f = check_cpu_freq(&list_cpus());
        print_freq(f, raw);
    }

    fn power(&self, raw: bool) {
        let mut battery = match Battery::new() {
            Ok(plugged) => plugged,
            Err(e) => {
                eprintln!("Failed to get battery, an error occured: {:?}", e);
                return;
            }
        };
        match battery.update() {
            Ok(plugged) => plugged,
            Err(e) => {
                eprintln!("Failed to update battery, an error occured: {:?}", e);
                return;
            }
        };
        let power = Power::default();
        power.set_best_path();

        let plugged = match power.read_power_source() {
            Ok(plugged) => plugged,
            Err(e) => {
                eprintln!("Failed to get read power source, an error occured: {:?}", e);
                return;
            }
        };

        let lid = match read_lid_state() {
            Ok(lid) => lid,
            Err(e) => {
                eprintln!("Failed to get read lid state, an error occured: {:?}", e);
                return;
            }
        };

        print_power(lid, battery.capacity, plugged, raw);
    }

    fn usage(&self, raw: bool) {
        if !raw {
            println!("Calculating cpu percentage over 1 second.");
        }
        let percent = get_cpu_percent();

        if raw {
            println!("{}", percent)
        } else {
            println!("CPU is at {}%", percent)
        }
    }

    fn thermal(&self, raw: bool) {
        let zones = match read_thermal_zones() {
            Ok(zones) => zones,
            Err(error) => {
                println!("Error: {:?}", error);
                return;
            }
        };
        if raw {
            println!("{:?}", zones)
        } else {
            for zone in zones {
                println!("{}", zone);
            }
        }
    }

    fn turbo(&self, raw: bool) {
        match check_turbo_enabled() {
            Ok(turbo_enabled) => print_turbo(turbo_enabled, raw),
            Err(_) => println!("Failed to get turbo status"),
        };
    }

    fn available_govs(&self, raw: bool) {
        match check_available_governors() {
            Ok(available_governors) => print_available_governors(available_governors, raw),
            Err(_) => println!("Failed to get available governors"),
        };
    }

    fn cpus(&self, raw: bool) {
        let cpus = list_cpus();
        match check_cpu_name() {
            Ok(name) => print_cpus(cpus, name, raw),
            Err(_) => println!("Failed get list of cpus"),
        };
    }

    fn speeds(&self, raw: bool) {
        let speeds = list_cpu_speeds();
        print_cpu_speeds(speeds, raw);
    }

    fn temp(&self, raw: bool) {
        let cpu_temp = list_cpu_temp();
        print_cpu_temp(cpu_temp, raw);
    }

    fn govs(&self, raw: bool) {
        let govs = list_cpu_governors();
        print_cpu_governors(govs, raw);
    }

    fn bat_cond(&self, raw: bool) {
        let battery = match Battery::new() {
            Ok(plugged) => plugged,
            Err(_) => {
                eprintln!("Failed to get battery");
                return;
            }
        };
        print_bat_cond(battery.condition, raw)
    }
}

pub struct Set {}

pub trait Setter {
    fn gov(&self, value: String, config: Config, settings: Settings);
}

impl Setter for Set {
    fn gov(&self, value: String, config: Config, settings: Settings) {
        // Create the daemon to set the gov
        match daemon_init(settings, config) {
            Ok(d) => match d.lock().unwrap().set_govs(value) {
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
    pub dec: DaemonControl,
}
