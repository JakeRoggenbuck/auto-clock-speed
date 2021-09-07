use super::cpu::CPU;
use std::fmt::Display;
use termion::{color, style};

pub fn print_freq(f: i32, raw: bool) {
    if raw {
        println!("{}", f);
    } else {
        println!("CPU freq is {} MHz", f / 1000)
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
        print!("[ ");
        for x in t {
            print!("\"{}\" ", x);
        }
        print!("]");
    }
}

pub fn print_available_governors(available_governors: Vec<String>, raw: bool) {
    print_vec(available_governors, raw);
}

pub fn print_cpus(cpus: Vec<CPU>, name: String) {
    println!("Name:{}", name);
    for x in cpus {
        println!("{} is currently @ {} MHz", x.name, x.cur_freq / 1000);
    }
}

pub fn print_cpu(cpu: &CPU) {
    println!(
        "{}{}:{} {}Hz\t{}Hz\t{}{}Hz{}\t{}{}C\t{}",
        style::Bold,
        cpu.name,
        style::Reset,
        cpu.max_freq / 1000,
        cpu.min_freq / 1000,
        color::Fg(color::Green),
        cpu.cur_freq / 1000,
        style::Reset,
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
