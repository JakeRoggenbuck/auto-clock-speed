use cached::proc_macro::once;
use std::fs::{read_dir, File};
use std::io::Read;
use std::string::String;
use std::{thread, time};

use crate::cpu::Speed;
use crate::debug;

use super::cpu::CPU;
use super::Error;

/// Find the average frequency of all cores
pub fn check_cpu_freq(cpus: &Vec<CPU>) -> f32 {
    let freqs: Vec<i32> = cpus.into_iter().map(|x| x.cur_freq).collect();
    let sum: i32 = Iterator::sum(freqs.iter());
    sum as f32 / freqs.len() as f32
}

/// Find the average usage of all cores
pub fn check_cpu_usage(cpus: &Vec<CPU>) -> f32 {
    let usage: Vec<i32> = cpus
        .into_iter()
        .map(|x| (x.cur_usage * 100.0) as i32)
        .collect();
    let sum: i32 = Iterator::sum(usage.iter());
    sum as f32 / usage.len() as f32
}

/// Find the average temperature of all cores
pub fn check_cpu_temperature(cpus: &Vec<CPU>) -> f32 {
    let usage: Vec<i32> = cpus.into_iter().map(|x| x.cur_temp).collect();
    let sum: i32 = Iterator::sum(usage.iter());
    sum as f32 / usage.len() as f32
}

pub fn get_highest_temp(cpus: &Vec<CPU>) -> i32 {
    let mut temp_max: i32 = 0;
    for cpu in cpus {
        if cpu.cur_temp > temp_max {
            temp_max = cpu.cur_temp;
        }
    }
    temp_max
}

fn open_cpu_info() -> String {
    let mut cpu_info: String = String::new();
    File::open("/proc/cpuinfo")
        .unwrap()
        .read_to_string(&mut cpu_info)
        .unwrap_or_else(|_| {
            panic!("Could not read /proc/cpuinfo");
        });
    cpu_info
}

fn get_name_from_cpu_info(cpu_info: String) -> Result<String, Error> {
    // Find all lines that begin with cpu MHz
    let find_cpu_mhz = cpu_info
        .split('\n')
        .find(|line| line.starts_with("model name\t"));

    // For each line that starts with the clock speed identifier return the number after : as a 32
    // bit integer
    find_cpu_mhz
        .and_then(|line| line.split(": ").last())
        .map(|x| x.to_owned())
        .ok_or(Error::Unknown)
}

pub fn check_cpu_name() -> Result<String, Error> {
    let cpu_info: String = open_cpu_info();
    let name: String = get_name_from_cpu_info(cpu_info)?;
    Ok(name)
}

pub fn read_proc_stat_file() -> Result<String, Error> {
    let mut is_turbo: String = String::new();
    let turbo_path: &str = "/proc/stat";
    File::open(turbo_path)?.read_to_string(&mut is_turbo)?;
    Ok(is_turbo)
}

#[derive(Debug)]
pub struct ProcStat {
    pub cpu_name: String,
    pub cpu_sum: f32,
    pub cpu_idle: f32,
}

impl Default for ProcStat {
    fn default() -> ProcStat {
        ProcStat {
            cpu_name: "cpu".to_string(),
            cpu_sum: 0.0,
            cpu_idle: 0.0,
        }
    }
}

pub fn parse_proc_file(proc: String) -> Result<Vec<ProcStat>, Error> {
    let lines: Vec<_> = proc.lines().collect();
    let mut procs: Vec<ProcStat> = Vec::<ProcStat>::new();
    for l in lines {
        if l.starts_with("cpu") {
            let mut columns: Vec<_> = l.split(" ").collect();

            // Remove first index if cpu starts with "cpu  " because the two spaces count as a
            // column
            if l.starts_with("cpu  ") {
                columns.remove(0);
            }
            let mut proc_struct: ProcStat = ProcStat::default();
            proc_struct.cpu_name = columns[0].to_string();
            for col in &columns {
                let parse = col.parse::<f32>();
                match parse {
                    Ok(num) => {
                        proc_struct.cpu_sum += num;
                    }
                    Err(_) => {}
                }
            }

            match columns[4].parse::<f32>() {
                Ok(num) => {
                    proc_struct.cpu_idle = num;
                }
                Err(_) => {}
            }
            procs.push(proc_struct);
        }
    }
    Ok(procs)
}

pub fn get_cpu_percent() -> String {
    let mut proc = read_proc_stat_file().unwrap();
    let avg_timing: &ProcStat = &parse_proc_file(proc).unwrap()[0];

    thread::sleep(time::Duration::from_millis(1000));
    proc = read_proc_stat_file().unwrap();

    let avg_timing_2: &ProcStat = &parse_proc_file(proc).unwrap()[0];

    format!(
        "{}",
        calculate_cpu_percent(&avg_timing, &avg_timing_2) * 100.0
    )
}

pub fn calculate_cpu_percent(timing_1: &ProcStat, timing_2: &ProcStat) -> f32 {
    debug!("{:?} -- {:?}", timing_1, timing_2);
    assert_eq!(
        timing_1.cpu_name, timing_2.cpu_name,
        "ProcStat object {:?} and {:?} do not belong to the same cpu",
        timing_1, timing_2
    );
    let cpu_delta: f32 = timing_2.cpu_sum - timing_1.cpu_sum;
    let cpu_delta_idle: f32 = timing_2.cpu_idle - timing_1.cpu_idle;
    let cpu_used: f32 = cpu_delta - cpu_delta_idle;
    cpu_used / cpu_delta
}

fn read_turbo_file() -> Result<String, Error> {
    let mut is_turbo: String = String::new();
    let turbo_path: &str = "/sys/devices/system/cpu/intel_pstate/no_turbo";
    File::open(turbo_path)?.read_to_string(&mut is_turbo)?;
    Ok(is_turbo)
}

fn interpret_turbo(is_turbo: &mut String) -> Result<bool, Error> {
    // Remove the last character (the newline)
    is_turbo.pop();
    // The file will be something like 0 or 1, parse this into an int
    match is_turbo.parse::<i8>() {
        Err(e) => panic!("{}", e),
        // Zero means turbo is enabled, so return true
        Ok(a) => Ok(a == 0),
    }
}

fn read_bat_energy_full(design: bool) -> Result<i32, Error> {
    let mut capacity_readings: String = String::new();
    let bat_energy_full_path: &str = "/sys/class/power_supply/BAT0/";
    if design == true {
        File::open(bat_energy_full_path.to_string() + "energy_full_design")?
            .read_to_string(&mut capacity_readings)?;
    } else {
        File::open(bat_energy_full_path.to_string() + "energy_full")?
            .read_to_string(&mut capacity_readings)?;
    }
    Ok(capacity_readings.parse::<i32>().unwrap())
}

/// Check if turbo is enabled for the machine, (enabled in bios)
pub fn check_turbo_enabled() -> Result<bool, Error> {
    let mut turbo_string = read_turbo_file()?;
    let is_turbo = interpret_turbo(&mut turbo_string)?;
    Ok(is_turbo)
}

pub fn check_bat_cond() -> Result<f32, Error> {
    let bat_cond_calc: f32 = (read_bat_energy_full(false)? / read_bat_energy_full(true)?) as f32;
    Ok(bat_cond_calc)
}

pub fn get_battery_condition(check_bat_cond: f32) -> Result<f32, Error> {
    let mut bat_cond = 0.0;
    if check_bat_cond * 100.0 >= 100.0 {
        bat_cond = 100.00;
    } else {
        bat_cond = check_bat_cond;
    }
    Ok(bat_cond.round())
}

fn read_govs_file() -> Result<String, Error> {
    let mut governors_string: String = String::new();
    let governors_path: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_available_governors";
    File::open(governors_path)?.read_to_string(&mut governors_string)?;
    Ok(governors_string)
}

fn interpret_govs(governors_string: &mut String) -> Result<Vec<String>, Error> {
    // Remove the newline at the end
    governors_string.pop();
    let governors: Vec<String> = governors_string
        // Governors are in the file separated by a space
        .split(" ")
        .into_iter()
        .map(|x| x.to_owned())
        .collect();
    Ok(governors)
}

/// Check the governors available for the cpu
pub fn check_available_governors() -> Result<Vec<String>, Error> {
    let mut govs_string = read_govs_file()?;
    let governors = interpret_govs(&mut govs_string)?;
    Ok(governors)
}

/// Get all the cpus (cores), returns cpus from 0 to the (amount of cores -1) the machine has
#[once]
pub fn list_cpus() -> Vec<CPU> {
    let mut cpus: Vec<String> = Vec::<String>::new();

    // Get each item in the cpu directory
    for a in read_dir("/sys/devices/system/cpu").unwrap_or_else(|_| {
        panic!("Could not read directory");
    }) {
        let path_string: String = format!("{:?}", a.unwrap().path());
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
        // Check if the file is actually a cpu, meaning it matches both having 'cpu' and a
        // character of index 3 is a number
        .filter(|x| x.find(r"cpu").is_some() && x.chars().nth(3).unwrap().is_numeric())
        .map(|x| x.to_owned())
        .collect();

    let mut to_return: Vec<CPU> = Vec::<CPU>::new();

    for cpu in cpus {
        let num: i8 = match cpu[3..].parse::<i8>() {
            Ok(a) => a,
            Err(_) => 0,
        };

        // Make a new cpu
        let mut new = CPU {
            name: cpu,
            number: num,
            // Temporary initial values
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
            cur_temp: 0,
            cur_usage: 0.0,
            gov: "Unknown".to_string(),
        };

        new.init_cpu().unwrap();

        new.update().unwrap();

        to_return.push(new)
    }

    to_return.sort_by(|a, b| a.number.cmp(&b.number));
    to_return
}

/// Get a vector of speeds reported from each cpu from list_cpus
pub fn list_cpu_speeds() -> Vec<i32> {
    list_cpus().into_iter().map(|x| x.cur_freq).collect()
}

/// Get a vector of temperatures reported from each cpu from list_cpus
pub fn list_cpu_temp() -> Vec<i32> {
    list_cpus().into_iter().map(|x| x.cur_temp).collect()
}

/// Get a vector of the governors that the cpus from list_cpus
pub fn list_cpu_governors() -> Vec<String> {
    list_cpus().into_iter().map(|x| x.gov).collect()
}

pub fn read_int(path: &str) -> Result<i32, Error> {
    let mut value: String = String::new();

    File::open(path)?.read_to_string(&mut value)?;

    // Remove trailing newline
    value.pop();
    Ok(value
        .parse::<i32>()
        .unwrap_or_else(|e| panic!("Could not parse {}\n{}", path, e)))
}

pub fn read_str(path: &str) -> Result<String, Error> {
    let mut value: String = String::new();

    File::open(path)?.read_to_string(&mut value)?;

    // Remove trailing newline
    value.pop();
    Ok(value)
}

#[cfg(test)]
mod tests {
    use std::any::type_name;

    use super::*;

    fn type_of<T>(_: T) -> &'static str {
        type_name::<T>()
    }

    #[test]
    fn check_cpu_freq_acs_test() {
        assert!(check_cpu_freq(&list_cpus()) > 0.0);
    }

    #[test]
    fn test_parse_proc_stat_file() {
        let cpu_percent = get_cpu_percent().parse::<f32>().unwrap();
        assert_eq!(type_of(cpu_percent), type_of(0.0_f32));
        assert!(cpu_percent > 0.0 && cpu_percent < 100.0);
    }

    #[test]
    fn get_name_from_cpu_info_unit_test() -> Result<(), Error> {
        let cpu_info = String::from(
            "
processor	: 0
vendor_id	: GenuineIntel
cpu family	: 6
model		: 158
model name	: Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz
stepping	: 9
microcode	: 0xea
processor	: 1
vendor_id	: GenuineIntel
cpu family	: 6
model		: 158
model name	: Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz
stepping	: 9
microcode	: 0xea
processor	: 2
vendor_id	: GenuineIntel
cpu family	: 6
model		: 158
model name	: Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz
stepping	: 9
microcode	: 0xea
processor	: 3
vendor_id	: GenuineIntel
cpu family	: 6
model		: 158
model name	: Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz
stepping	: 9
microcode	: 0xea
            ",
        );
        let name = get_name_from_cpu_info(cpu_info)?;
        assert_eq!(name, "Intel(R) Core(TM) i5-7600K CPU @ 3.80GHz");
        Ok(())
    }

    #[test]
    fn check_cpu_name_unit_test() -> Result<(), Error> {
        assert_eq!(type_of(check_cpu_name()?), type_of(String::new()));
        assert!(check_cpu_name()?.len() > 0);
        Ok(())
    }

    // Non-Platform dependent
    #[test]
    fn interpret_turbo_unit_test() -> Result<(), Error> {
        let mut is_turbo = String::from("0\n");
        assert!(interpret_turbo(&mut is_turbo)?);

        let mut is_turbo = String::from("2\n");
        assert!(!interpret_turbo(&mut is_turbo)?);
        Ok(())
    }

    #[test]
    fn check_turbo_enabled_acs_test() -> Result<(), Error> {
        assert_eq!(type_of(check_turbo_enabled()?), type_of(true));
        Ok(())
    }

    #[test]
    fn check_available_governors_acs_test() -> Result<(), Error> {
        assert_eq!(
            type_of(check_available_governors()?),
            type_of(Vec::<String>::new())
        );

        for x in check_available_governors()? {
            assert!(x == "powersave" || x == "performance");
        }
        Ok(())
    }

    // Non-Platform dependent
    #[test]
    fn interpret_govs_unit_test() -> Result<(), Error> {
        let mut governors_string = String::from("performance powersave\n");
        let govs = interpret_govs(&mut governors_string)?;
        assert_eq!(vec!["performance", "powersave"], govs);

        let mut governors_string = String::from("performance\n");
        let govs = interpret_govs(&mut governors_string)?;
        assert_eq!(vec!["performance"], govs);

        let mut governors_string = String::from("calvin hobbes\n");
        let govs = interpret_govs(&mut governors_string)?;
        assert_eq!(vec!["calvin", "hobbes"], govs);
        Ok(())
    }

    #[test]
    fn list_cpus_acs_test() {
        assert_eq!(type_of(list_cpus()), type_of(Vec::<CPU>::new()));

        for x in list_cpus() {
            assert!(x.name.len() > 0);
            assert!(x.max_freq > 0);
            assert!(x.min_freq > 0);

            assert!(x.cur_freq > 0);
            assert!(x.cur_temp > -100);

            assert!(x.gov == "powersave" || x.gov == "performance");
        }
    }

    #[test]
    fn list_cpu_speeds_acs_test() -> Result<(), Error> {
        // Type check
        assert_eq!(type_of(list_cpu_speeds()), type_of(Vec::<i32>::new()));

        for x in list_cpu_speeds() {
            assert!(x > 0);
        }
        Ok(())
    }

    #[test]
    fn list_cpu_temp_acs_test() {
        // Type check
        assert_eq!(type_of(list_cpu_temp()), type_of(Vec::<i32>::new()));

        for x in list_cpu_temp() {
            assert!(x > -100);
        }
    }

    #[test]
    fn list_cpu_governors_acs_test() {
        // Type check
        assert_eq!(type_of(list_cpu_governors()), type_of(Vec::<String>::new()));

        for x in list_cpu_governors() {
            assert!(x == "powersave" || x == "performance");
        }
    }
}
