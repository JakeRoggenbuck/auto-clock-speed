use super::cpu;
use super::Error;
use crate::cpu::Speed;
use regex::Regex;
use std::fs::{read_dir, File};
use std::io::Read;
use std::string::String;

/// Check the frequency of the cpu
pub fn check_cpu_freq() -> Result<i32, Error> {
    let mut total = 0;
    let mut count = 0;
    for cpu in list_cpus()? {
        count+=1;
        total+=cpu.cur_freq;
    }
    Ok((total as f32/count as f32) as i32)
}

pub fn check_cpu_name() -> Result<String, Error> {
    let mut cpu_info: String = String::new();
    File::open("/proc/cpuinfo")?.read_to_string(&mut cpu_info)?;

    // Find all lines that begin with cpu MHz
    let find_cpu_mhz = cpu_info.split('\n').find(|line| {
        line.starts_with("model name\t")
    });

    // For each line that starts with the clock speed identifier return the number after : as a 32
    // bit integer
    find_cpu_mhz
        .and_then(|line| line.split(':').last())
        .map(|x| x.to_owned())
        .ok_or(Error::Unknown)
}

/// Check if turbo is enabled for the machine, (enabled in bios)
pub fn check_turbo_enabled() -> Result<bool, Error> {
    let mut is_turbo: String = String::new();
    let turbo_path: &str = "/sys/devices/system/cpu/intel_pstate/no_turbo";
    File::open(turbo_path)?.read_to_string(&mut is_turbo)?;

    // Remove the last character (the newline)
    is_turbo.pop();
    // The file will be something like 0 or 1, parse this into an int
    match is_turbo.parse::<i8>() {
        Err(e) => panic!("{}", e),
        // Zero means turbo is enabled, so return true
        Ok(a) => Ok(a == 0),
    }
}

/// Check the governors available for the cpu
pub fn check_available_governors() -> Result<Vec<String>, Error> {
    let mut governors_string: String = String::new();
    let governors_path: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    File::open(governors_path)?.read_to_string(&mut governors_string)?;

    // Remove the newline at the end
    governors_string.pop();
    let governors: Vec<String> = governors_string
        // Governors are in the file separated by a space
        .split(" ")
        .into_iter()
        .map(|x| x.to_owned())
        .collect();
    return Ok(governors);
}

/// Get all the cpus (cores), returns cpus from 0 to the (amount of cores -1) the machine has
pub fn list_cpus() -> Result<Vec<cpu::CPU>, Error> {
    let mut cpus: Vec<String> = Vec::<String>::new();
    // The string "cpu" followed by a digit
    let cpu = Regex::new(r"cpu\d").unwrap();

    // Get each item in the cpu directory
    for a in read_dir("/sys/devices/system/cpu")? {
        let path_string: String = format!("{:?}", a?.path()).to_string();
        let path: String = path_string
            .chars()
            // Skip the characters that are before the cpu name
            .skip(25)
            // Take only the characters that are apart of the name
            .take(path_string.len() - 26)
            .collect::<String>();

        cpus.push(path);
    }

    cpus = cpus
        .iter()
        // Check if the file is actually a cpu, meaning it matches that regex
        .filter(|x| cpu.is_match(x))
        .map(|x| x.to_owned())
        .collect();

    let mut to_return: Vec<cpu::CPU> = Vec::<cpu::CPU>::new();

    for cpu in cpus {
        let mut new = cpu::CPU {
            name: cpu,
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
            // Temporary initial value
            gov: "Unknown".to_string(),
        };

        new.update();

        to_return.push(new)
    }

    Ok(to_return)
}

/// Get a vector of speeds reported from each cpu from list_cpus
pub fn list_cpu_speeds() -> Result<Vec<i32>, Error> {
    let cpus = list_cpus()?;
    let mut speeds = Vec::<i32>::new();

    for cpu in cpus {
        let speed = cpu.cur_freq;
        speeds.push(speed)
    }
    Ok(speeds)
}

/// Get a vector of the governors that the cpus from list_cpus
pub fn list_cpu_governors() -> Result<Vec<String>, Error> {
    let cpus = list_cpus()?;
    let mut governors = Vec::<String>::new();

    for cpu in cpus {
        governors.push(cpu.gov)
    }
    Ok(governors)
}
