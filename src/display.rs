#![forbid(unsafe_code)]
use efcl::{color, Color};
use std::fmt::Display;
use std::thread;

use super::config::Config;
use super::cpu::CPU;
use super::power::lid::LidState;
use super::system::check_turbo_enabled;
use crate::power::battery::{has_battery, Battery, BatteryStatus};

#[macro_export]
macro_rules! warn_user {
    ($a:expr) => {{
        use efcl::{color, Color};
        println!("{}: {}", color!(Color::YELLOW, "WARN"), $a,);
    }};
}

#[macro_export]
macro_rules! print_error {
    ($a:expr) => {{
        use efcl::{color, Color};
        println!("{}: {}", color!(Color::RED, "ERROR"), $a,);
    }};
}

#[macro_export]
macro_rules! print_done {
    ($a:expr) => {{
        use efcl::{color, Color};
        println!("{}: {}", color!(Color::GREEN, "DONE"), $a,);
    }};
}

#[macro_export]
macro_rules! create_issue {
    ($a:expr) => {{
        eprintln!(
            "{}, {}",
            $a, "please create an issue at issue.autoclockspeed.org",
        );
    }};
}

pub fn show_config(config: &Config) {
    println!("{}", config);
}

pub fn print_freq(f: f32, raw: bool) {
    if raw {
        println!("{}", f);
    } else {
        println!("CPU freq is {} MHz", f / 1000.0)
    }
}

pub fn print_power(lid: LidState, bat: i8, plugged: bool, raw: bool) {
    if raw {
        println!("{} {} {}", lid, bat, plugged);
    } else {
        println!("Lid: {} Battery: {} Plugged: {}", lid, bat, plugged);
    }
}

pub fn print_bat_cond(c: i8, raw: bool) {
    if raw {
        println!("{}", c);
    } else {
        println!("{:.2}%", c)
    }
}

pub fn print_turbo(t: bool, raw: bool) {
    if raw {
        println!("{}", t);
    } else {
        println!(
            "{}",
            if t {
                "Turbo is enabled"
            } else {
                "Turbo is not enabled"
            }
        )
    }
}

pub fn print_battery_status(battery: &Battery) -> String {
    if has_battery() {
        format!(
            "Battery: {}",
            if battery.status == BatteryStatus::Charging {
                color!(Color::GREEN, format!("{}%", battery.capacity).as_str())
            } else {
                color!(Color::RED, format!("{}%", battery.capacity).as_str())
            },
        )
    } else {
        format!("Battery: {}", "N/A")
    }
}

pub fn print_turbo_animation(cpu: usize, y_pos: usize, delay: u64) {
    let frames = ['◷', '◶', '◵', '◴'];
    let y_pos = cpu + y_pos;
    let mut current = 0;
    let count = delay / 100;

    thread::spawn(move || {
        for _ in 0..count {
            println!("{}[{};1H{}", 27 as char, y_pos, frames[current]);
            current += 1;
            if current == 4 {
                current = 0;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

pub fn print_turbo_status(cores: usize, animation: bool, term_width: usize, delay: u64) {
    let mut turbo_y_pos: usize = 8;
    let title_width = 94;

    if term_width > title_width {
        turbo_y_pos = 7
    }

    match check_turbo_enabled() {
        Ok(turbo) => {
            let enabled_message = if turbo { "enabled" } else { "disabled" };

            if animation {
                println!("  Turbo: {}", enabled_message);
                print_turbo_animation(cores, turbo_y_pos, delay);
            } else {
                println!("Turbo: {}", enabled_message);
            }
        }
        Err(..) => eprintln!("Could not check turbo. Expected for AMD.\n"),
    }
}

fn print_vec<T: Display>(t: Vec<T>, raw: bool) {
    if raw {
        for x in t {
            println!("{}", x);
        }
    } else {
        for x in t {
            print!("{} ", x);
        }
        println!()
    }
}

pub fn print_available_governors(available_governors: Vec<String>, raw: bool) {
    print_vec(available_governors, raw);
}

pub fn print_cpus(cpus: Vec<CPU>, name: String, raw: bool) {
    if raw {
        for x in cpus {
            println!("{} {}", x.name, x.cur_freq);
        }
    } else {
        println!("Name: {}", name);
        for x in cpus {
            println!("{}:\t{} MHz", x.name, x.cur_freq / 1000);
        }
    }
}

pub fn print_cpu_speeds(cpu_speeds: Vec<i32>, raw: bool) {
    print_vec(cpu_speeds, raw);
}

pub fn print_cpu_temp(cpu_temp: Vec<i32>, raw: bool) {
    print_vec(cpu_temp, raw);
}

pub fn print_cpu_governors(cpu_governors: Vec<String>, raw: bool) {
    print_vec(cpu_governors, raw);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_cpu_unit_test() {
        let new = CPU {
            name: "cpu1".to_string(),
            cur_usage: 0.0,
            number: 1,
            // Temporary initial values
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
            cur_temp: 0,
            gov: "Unknown".to_string(),
        };

        let out = format!("{}", &new);
        assert!(out.contains("Unknown"));
        assert!(out.contains("cpu1"));
    }
}
