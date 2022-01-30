use super::cpu::CPU;
use super::Error;
use crate::cpu::Speed;
use regex::Regex;
use std::fs::{read_dir, File};
use std::io::Read;
use std::string::String;

/// Check the frequency of the cpu
pub fn check_cpu_freq() -> Result<i32, Error> {
    let freqs: Vec<i32> = list_cpus()?.into_iter().map(|x| x.cur_freq).collect();
    let sum: i32 = Iterator::sum(freqs.iter());
    Ok((sum as f32 / freqs.len() as f32) as i32)
}

fn open_cpu_info() -> Result<String, Error> {
    let mut cpu_info: String = String::new();
    File::open("/proc/cpuinfo")?.read_to_string(&mut cpu_info)?;
    Ok(cpu_info)
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
    let cpu_info: String = open_cpu_info()?;
    let name: String = get_name_from_cpu_info(cpu_info)?;
    Ok(name)
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

/// Check if turbo is enabled for the machine, (enabled in bios)
pub fn check_turbo_enabled() -> Result<bool, Error> {
    let mut turbo_string = read_turbo_file()?;
    let is_turbo = interpret_turbo(&mut turbo_string)?;
    Ok(is_turbo)
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
pub fn list_cpus() -> Result<Vec<CPU>, Error> {
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

    cpus.sort();

    let mut to_return: Vec<CPU> = Vec::<CPU>::new();

    for cpu in cpus {
        // Make a new cpu
        let mut new = CPU {
            name: cpu,
            // Temporary initial values
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
            cur_temp: 0,
            gov: "Unknown".to_string(),
        };

        new.update()?;

        to_return.push(new)
    }

    Ok(to_return)
}

/// Get a vector of speeds reported from each cpu from list_cpus
pub fn list_cpu_speeds() -> Result<Vec<i32>, Error> {
    Ok(list_cpus()?.into_iter().map(|x| x.cur_freq).collect())
}

/// Get a vector of temperatures reported from each cpu from list_cpus
pub fn list_cpu_temp() -> Result<Vec<i32>, Error> {
    Ok(list_cpus()?.into_iter().map(|x| x.cur_temp).collect())
}

/// Get a vector of the governors that the cpus from list_cpus
pub fn list_cpu_governors() -> Result<Vec<String>, Error> {
    Ok(list_cpus()?.into_iter().map(|x| x.gov).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::type_name;

    fn type_of<T>(_: T) -> &'static str {
        type_name::<T>()
    }

    #[test]
    fn check_cpu_freq_test() -> Result<(), Error> {
        assert_eq!(type_of(check_cpu_freq()?), type_of(1));

        assert!(check_cpu_freq()? > 0);
        Ok(())
    }

    // Non-Platform dependent
    #[test]
    fn get_name_from_cpu_info_test() -> Result<(), Error> {
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
    fn check_cpu_name_test() -> Result<(), Error> {
        assert_eq!(type_of(check_cpu_name()?), type_of(String::new()));
        assert!(check_cpu_name()?.len() > 0);
        Ok(())
    }

    // Non-Platform dependent
    #[test]
    fn interpret_turbo_test() -> Result<(), Error> {
        let mut is_turbo = String::from("0\n");
        assert!(interpret_turbo(&mut is_turbo)?);

        let mut is_turbo = String::from("2\n");
        assert!(!interpret_turbo(&mut is_turbo)?);
        Ok(())
    }

    #[test]
    fn acs_check_turbo_enabled_test() -> Result<(), Error> {
        assert_eq!(type_of(check_turbo_enabled()?), type_of(true));
        Ok(())
    }

    #[test]
    fn check_available_governors_test() -> Result<(), Error> {
        assert_eq!(
            type_of(check_available_governors()?),
            type_of(Vec::<String>::new())
        );

        for x in check_available_governors()? {
            assert!(x == "powersave" || x == "performance");
        }
        Ok(())
    }

    #[test]
    fn list_cpus_test() -> Result<(), Error> {
        assert_eq!(type_of(list_cpus()?), type_of(Vec::<CPU>::new()));

        for x in list_cpus()? {
            assert!(x.name.len() > 0);
            assert!(x.max_freq > 0);
            assert!(x.min_freq > 0);

            assert!(x.cur_freq > 0);
            assert!(x.cur_temp > -100);

            assert!(x.gov == "powersave" || x.gov == "performance");
        }
        Ok(())
    }

    #[test]
    fn list_cpu_speeds_test() -> Result<(), Error> {
        // Type check
        assert_eq!(type_of(list_cpu_speeds()?), type_of(Vec::<i32>::new()));

        for x in list_cpu_speeds()? {
            assert!(x > 0);
        }
        Ok(())
    }

    #[test]
    fn list_cpu_temp_test() -> Result<(), Error> {
        // Type check
        assert_eq!(type_of(list_cpu_temp()?), type_of(Vec::<i32>::new()));

        for x in list_cpu_temp()? {
            assert!(x > -100);
        }
        Ok(())
    }

    #[test]
    fn list_cpu_governors_test() -> Result<(), Error> {
        // Type check
        assert_eq!(
            type_of(list_cpu_governors()?),
            type_of(Vec::<String>::new())
        );

        for x in list_cpu_governors()? {
            assert!(x == "powersave" || x == "performance");
        }
        Ok(())
    }
}
