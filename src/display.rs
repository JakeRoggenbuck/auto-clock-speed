use std::fmt::Display;
use std::thread;

use super::config::get_config;
use super::cpu::CPU;
use super::power::LidState;
use super::system::check_turbo_enabled;
use colored::*;

#[macro_export]
macro_rules! warn_user {
    ($a:expr) => {{
        use colored::Colorize;
        println!("{}: {}", "WARN".bold().yellow(), $a,);
    }};
}

#[macro_export]
macro_rules! print_error {
    ($a:expr) => {{
        use colored::Colorize;
        println!("{}: {}", "ERROR".bold().red(), $a,);
        std::process::exit(-1);
    }};
}

#[macro_export]
macro_rules! create_issue {
    ($a:expr) => {{
        eprintln!(
            "{}, {}",
            $a,
            "please create an issue at https://github.com/JakeRoggenbuck/auto-clock-speed/issues/new/choose",
        );
    }};
}

pub fn show_config() {
    println!("{}", get_config());
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

pub fn print_turbo_animation(cpu: usize, y_pos: usize, delay: u64) {
    let frames = ['◷', '◶', '◵', '◴'];
    let y_pos = cpu + y_pos;
    let mut current = 0;
    let count = delay / 100;

    thread::spawn(move || {
        for _ in 0..count {
            termion::cursor::Goto(3, 7);
            println!("{}[{};1H{}", 27 as char, y_pos, frames[current]);
            current += 1;
            if current == 4 {
                current = 0;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    });
}

pub fn print_turbo_status(cores: usize, no_animation: bool, term_width: usize, delay: u64) {
    let mut turbo_y_pos: usize = 7;
    let title_width = 94;

    if term_width > title_width {
        turbo_y_pos = 6
    }

    match check_turbo_enabled() {
        Ok(turbo) => {
            let enabled_message = if turbo { "yes" } else { "no" };

            println!("{} {}", "  Turbo:", enabled_message.bold(),);

            if !no_animation {
                print_turbo_animation(cores, turbo_y_pos, delay);
            }
        }
        Err(e) => eprintln!("Could not check turbo\n{:?}", e),
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
        print!("\n")
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

pub fn print_cpu(cpu: &CPU) {
    print!("{}", render_cpu(cpu));
}

pub fn render_cpu(cpu: &CPU) -> String {
    let temp: colored::ColoredString;
    let reduced_cpu_cur_temp = cpu.cur_temp / 1000;

    if reduced_cpu_cur_temp > 60 {
        temp = format!("{}C", reduced_cpu_cur_temp).red();
    } else if reduced_cpu_cur_temp > 40 {
        temp = format!("{}C", reduced_cpu_cur_temp).yellow();
    } else if reduced_cpu_cur_temp == 1 || reduced_cpu_cur_temp == 0 {
        temp = format!("{}C*", reduced_cpu_cur_temp).white();
    } else {
        temp = format!("{}C", reduced_cpu_cur_temp).green();
    }

    let usage: colored::ColoredString;
    let scaled_cpus_cur_usage = cpu.cur_usage * 100.0;

    if cpu.cur_usage > 0.9 {
        usage = format!("{:.2}%", scaled_cpus_cur_usage).red();
    } else if cpu.cur_usage > 0.5 {
        usage = format!("{:.2}%", scaled_cpus_cur_usage).yellow();
    } else if cpu.cur_usage > 0.2 {
        usage = format!("{:.2}%", scaled_cpus_cur_usage).white();
    } else if cpu.cur_usage > 0.0000 {
        usage = format!("{:.2}%", scaled_cpus_cur_usage).green();
    } else {
        usage = format!("{:.2}%", scaled_cpus_cur_usage).purple();
    }

    format!(
        "{}: {}MHz\t{}MHz\t{}\t{}\t{}\t{}\n",
        cpu.name.bold(),
        cpu.max_freq / 1000,
        cpu.min_freq / 1000,
        format!("{}MHz", cpu.cur_freq / 1000).green(),
        temp,
        usage,
        cpu.gov
    )
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

        let out = render_cpu(&new);
        assert!(out.contains("Unknown"));
        assert!(out.contains("cpu1"));
    }
}
