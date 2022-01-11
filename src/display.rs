use super::cpu::CPU;
use super::power::LidState;
use std::fmt::Display;
use termion::{color, style};

#[macro_export]
macro_rules! bold_color_text {
    ($a:expr, $b:expr) => {{
        format!(
            "{}{}{}{}{}",
            termion::style::Bold,
            termion::color::Fg($b),
            $a,
            termion::color::Fg(termion::color::Reset),
            termion::style::Reset,
        )
    }};
}

#[macro_export]
macro_rules! warn_user {
    ($a:expr) => {{
        println!(
            "{}{}WARN:{}{} {}",
            termion::style::Bold,
            termion::color::Fg(termion::color::Yellow),
            termion::color::Fg(termion::color::Reset),
            termion::style::Reset,
            $a,
        );
    }};
}

pub fn print_freq(f: i32, raw: bool) {
    if raw {
        println!("{}", f);
    } else {
        println!("CPU freq is {} MHz", f / 1000)
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
        println!("Name:{}", name);
        for x in cpus {
            println!("{} is currently @ {} MHz", x.name, x.cur_freq / 1000);
        }
    }
}

pub fn print_cpu(cpu: &CPU) {
    let mut temp_color: String = color::Fg(color::Green).to_string();

    if cpu.cur_temp / 1000 > 60 {
        temp_color = color::Fg(color::Red).to_string();
    } else if cpu.cur_temp / 1000 > 40 {
        temp_color = color::Fg(color::Yellow).to_string();
    }

    println!(
        "{}{}:{} {}Hz\t{}Hz\t{}{}Hz{}\t{}C{}\t{}",
        style::Bold,
        cpu.name,
        style::Reset,
        cpu.max_freq / 1000,
        cpu.min_freq / 1000,
        color::Fg(color::Green),
        cpu.cur_freq / 1000,
        temp_color,
        cpu.cur_temp / 1000,
        style::Reset,
        cpu.gov
    );
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
