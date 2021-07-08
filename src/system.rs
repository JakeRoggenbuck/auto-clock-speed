use super::Error;
use regex::Regex;
use std::fs::{read_dir, File};
use std::io::Read;
use std::string::String;

// https://docs.rs/sys-info/0.7.0/src/sys_info/lib.rs.html#367-406
/// Check the frequency of the cpu
pub fn check_cpu_freq() -> Result<i32, Error> {
    let mut cpu_info: String = String::new();
    File::open("/proc/cpuinfo")?.read_to_string(&mut cpu_info)?;

    // Find all lines that begin with cpu MHz
    let find_cpu_mhz = cpu_info.split('\n').find(|line| {
        line.starts_with("cpu MHz\t")
            || line.starts_with("BogoMIPS")
            || line.starts_with("clock\t")
            || line.starts_with("bogomips per cpu")
    });

    // For each line that starts with the clock speed identifier return the number after : as a 32
    // bit integer
    find_cpu_mhz
        .and_then(|line| line.split(':').last())
        .and_then(|val| val.replace("MHz", "").trim().parse::<f64>().ok())
        .map(|speed| speed as i32)
        .ok_or(Error::Unknown)
}

/// Check the speed for a single cpu (single core)
pub fn check_speed_by_cpu(cpu: String) -> Result<i32, Error> {
    let mut speed: String = String::new();
    let cpu_speed_path: String =
        format!("/sys/devices/system/cpu/{}/cpufreq/scaling_cur_freq", cpu);

    File::open(cpu_speed_path)?.read_to_string(&mut speed)?;

    // Remove the last character (the newline)
    speed.pop();
    match speed.parse::<i32>() {
        Err(e) => panic!("{}", e),
        // Zero means turbo is enabled, so return true
        Ok(a) => Ok(a),
    }
}

/// Check the max speed for a single cpu (single core)
pub fn check_max_speed_by_cpu(cpu: String) -> Result<i32, Error> {
    let mut speed: String = String::new();
    let cpu_speed_path: String =
        format!("/sys/devices/system/cpu/{}/cpufreq/scaling_max_freq", cpu);

    File::open(cpu_speed_path)?.read_to_string(&mut speed)?;

    // Remove the last character (the newline)
    speed.pop();
    match speed.parse::<i32>() {
        Err(e) => panic!("{}", e),
        // Zero means turbo is enabled, so return true
        Ok(a) => Ok(a),
    }
}

/// Check the min speed for a single cpu (single core)
pub fn check_min_speed_by_cpu(cpu: String) -> Result<i32, Error> {
    let mut speed: String = String::new();
    let cpu_speed_path: String =
        format!("/sys/devices/system/cpu/{}/cpufreq/scaling_min_freq", cpu);

    File::open(cpu_speed_path)?.read_to_string(&mut speed)?;

    // Remove the last character (the newline)
    speed.pop();
    match speed.parse::<i32>() {
        Err(e) => panic!("{}", e),
        // Zero means turbo is enabled, so return true
        Ok(a) => Ok(a),
    }
}

/// Check the governor of a single cpu (single core)
pub fn check_governor_by_cpu(cpu: String) -> Result<String, Error> {
    let mut governor: String = String::new();
    let cpu_governor_path: String =
        format!("/sys/devices/system/cpu/{}/cpufreq/scaling_governor", cpu);

    File::open(cpu_governor_path)?.read_to_string(&mut governor)?;

    // Remove the last character (the newline)
    governor.pop();
    match governor.parse::<String>() {
        Err(e) => panic!("{}", e),
        Ok(a) => Ok(a),
    }
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
pub fn list_cpus() -> Result<Vec<String>, Error> {
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

        cpus.push(path)
    }

    cpus = cpus
        .iter()
        // Check if the file is actually a cpu, meaning it matches that regex
        .filter(|x| cpu.is_match(x))
        .map(|x| x.to_owned())
        .collect();

    Ok(cpus)
}

/// Get a vector of speeds reported from each cpu from list_cpus
pub fn list_cpu_speeds() -> Result<Vec<i32>, Error> {
    let cpus = list_cpus()?;
    let mut speeds = Vec::<i32>::new();

    for cpu in cpus {
        let speed = check_speed_by_cpu(cpu)?;
        speeds.push(speed)
    }
    Ok(speeds)
}

/// Get a vector of the governors that the cpus from list_cpus
pub fn list_cpu_governors() -> Result<Vec<String>, Error> {
    let cpus = list_cpus()?;
    let mut governors = Vec::<String>::new();

    for cpu in cpus {
        let governor = check_governor_by_cpu(cpu)?;
        governors.push(governor)
    }
    Ok(governors)
}
