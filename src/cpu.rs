use efcl::{bold, color, Color};
use rand::Rng;
use std::fmt;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

use crate::proc::ProcStat;

use crate::gov::Gov;
use crate::system::{calculate_cpu_percent, read_int, read_str};
use crate::error::Error;

/// Any trait relating to a CPU Core
pub trait Speed {
    fn read_temp(&mut self, sub_path: &str) -> Result<i32, Error>;
    fn write_value(&mut self, value: WritableValue) -> Result<(), Error>;
    fn update(&mut self) -> Result<(), Error>;
    fn update_usage(&mut self, last_proc: &ProcStat, current_proc: &ProcStat) -> Result<(), Error>;
    fn init_cpu(&mut self) -> Result<(), Error>;
    fn set_max(&mut self, max: i32) -> Result<(), Error>;
    fn set_min(&mut self, min: i32) -> Result<(), Error>;
    fn get_max(&mut self);
    fn get_min(&mut self);
    fn get_cur(&mut self);
    fn get_temp(&mut self) -> Result<(), Error>;
    fn get_gov(&mut self) -> Result<(), Error>;
    fn set_gov(&mut self, gov: Gov) -> Result<(), Error>;
    fn random() -> CPU;
}

/// Data relating to the CPU
#[derive(Debug, Clone, Default)]
pub struct CPU {
    pub name: String,
    pub number: i8,
    pub max_freq: i32,
    pub min_freq: i32,
    pub cur_freq: i32,
    pub cur_temp: i32,
    pub cur_usage: f32,
    pub gov: String,
}

/// Paths that can be written to
/// This is an enum that keeps that values that are allowed
#[derive(PartialEq, Eq)]
pub enum WritableValue {
    Min,
    Max,
    Gov,
}

impl Speed for CPU {
    /// Read the temperature of a cpu
    fn read_temp(&mut self, sub_path: &str) -> Result<i32, Error> {
        let mut cpu_info_path: String = format!(
            "/sys/class/thermal/{}/{}",
            self.name.replace("cpu", "thermal_zone"),
            sub_path
        );

        // If the thermal path does not exist, use the first thermal path only if it exists
        if !Path::new(&cpu_info_path).exists() {
            let first_core_path = format!("/sys/class/thermal/thermal_zone0/{}", sub_path);
            if Path::new(&first_core_path).exists() {
                cpu_info_path = first_core_path;
            } else {
                return Ok(-1);
            }
        }

        let mut info = fs::read_to_string(cpu_info_path)?;

        // Remove the last character (the newline)
        info.pop();

        Ok(info
            .parse::<i32>()
            .unwrap_or_else(|e| panic!("Could not parse {}\n{}", sub_path, e)))
    }

    /// Write a specific value to a specific path
    fn write_value(&mut self, value: WritableValue) -> Result<(), Error> {
        let sub_path: &str;
        let to_write: String;

        match value {
            WritableValue::Max => {
                sub_path = "cpufreq/scaling_max_freq";
                to_write = self.max_freq.to_string();
            }
            WritableValue::Min => {
                sub_path = "cpufreq/scaling_min_freq";
                to_write = self.min_freq.to_string();
            }
            WritableValue::Gov => {
                sub_path = "cpufreq/scaling_governor";
                to_write = self.gov.to_string();
            }
        }

        let path: String = format!("/sys/devices/system/cpu/{}/{}", self.name, sub_path);
        let mut buffer = File::create(path)?;
        buffer.write_all(to_write.as_bytes())?;

        Ok(())
    }

    /// Pull and update some the attributes of the cpu
    /// These get methods write the value to the actual cpu object
    /// These methods are only the ones that have values that are expected to change
    fn update(&mut self) -> Result<(), Error> {
        self.get_cur();
        self.get_temp()?;
        self.get_gov()?;
        Ok(())
    }

    /// Updating usage takes more timing data it doesn't just work instantly
    fn update_usage(&mut self, last_proc: &ProcStat, current_proc: &ProcStat) -> Result<(), Error> {
        self.cur_usage = calculate_cpu_percent(last_proc, current_proc);
        Ok(())
    }

    /// Do the first update and write values from methods that are expected not to change
    fn init_cpu(&mut self) -> Result<(), Error> {
        // Add function calls in the init
        self.get_max();
        self.get_min();
        self.update()?;
        Ok(())
    }

    /// Set the max value
    fn set_max(&mut self, max: i32) -> Result<(), Error> {
        self.max_freq = max;
        self.write_value(WritableValue::Max)?;
        Ok(())
    }

    /// Set the min value
    fn set_min(&mut self, min: i32) -> Result<(), Error> {
        self.min_freq = min;
        self.write_value(WritableValue::Min)?;
        Ok(())
    }

    /// Get the max value from the cpu
    fn get_max(&mut self) {
        self.max_freq = read_int(&format!(
            "/sys/devices/system/cpu/{}/{}",
            self.name, "cpufreq/scaling_max_freq"
        ))
        .unwrap_or(0);
    }

    /// Get the min value from the cpu
    fn get_min(&mut self) {
        self.min_freq = read_int(&format!(
            "/sys/devices/system/cpu/{}/{}",
            self.name, "cpufreq/scaling_min_freq"
        ))
        .unwrap_or(0);
    }

    /// Get the current cpu frequency
    fn get_cur(&mut self) {
        self.cur_freq = read_int(&format!(
            "/sys/devices/system/cpu/{}/{}",
            self.name, "cpufreq/scaling_cur_freq"
        ))
        .unwrap_or(0);
    }

    /// Get the current cpu temp
    fn get_temp(&mut self) -> Result<(), Error> {
        self.cur_temp = self.read_temp("temp")?;
        Ok(())
    }

    /// Get the current governor
    fn get_gov(&mut self) -> Result<(), Error> {
        self.gov = read_str(&format!(
            "/sys/devices/system/cpu/{}/{}",
            self.name, "cpufreq/scaling_governor"
        ))
        .unwrap_or_else(|_| "unknown".to_string());
        Ok(())
    }

    /// Set the governor
    fn set_gov(&mut self, gov: Gov) -> Result<(), Error> {
        self.gov = format!("{gov}");
        self.write_value(WritableValue::Gov)?;
        Ok(())
    }

    /// Randomly generate cpu objects with somewhat realistic values
    fn random() -> CPU {
        let mut rng = rand::thread_rng();
        CPU {
            name: "TEST__0".to_string(),
            number: rng.gen_range(0..100),
            max_freq: rng.gen_range(0..100000),
            min_freq: rng.gen_range(0..10000),
            cur_freq: rng.gen_range(0..100000),
            cur_temp: rng.gen_range(0..100000),
            cur_usage: rng.gen::<f32>(),
            gov: if rng.gen_bool(0.5) {
                "powersave".to_string()
            } else {
                "performance".to_string()
            },
        }
    }
}

impl fmt::Display for CPU {
    /// Display any information about the cpu in a human readable and simple format
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let reduced_cpu_cur_temp = self.cur_temp / 1000;

        // Pick color based on temp
        let temp_color = if reduced_cpu_cur_temp > 60 {
            Color::RED
        } else if reduced_cpu_cur_temp > 40 {
            Color::YELLOW
        } else if reduced_cpu_cur_temp == 1 || reduced_cpu_cur_temp == 0 {
            Color::LIGHTGRAY
        } else {
            Color::GREEN
        };

        let temp: String = color!(temp_color, format!("{}C", reduced_cpu_cur_temp).as_str());

        let scaled_cpus_cur_usage = self.cur_usage * 100.0;

        // Pick color based on usage
        let usage_color = if self.cur_usage > 0.9 {
            Color::RED
        } else if self.cur_usage > 0.5 {
            Color::YELLOW
        } else if self.cur_usage > 0.0000 {
            Color::GREEN
        } else {
            Color::LIGHTGRAY
        };

        let usage: String = color!(
            usage_color,
            format!("{:.2}%", scaled_cpus_cur_usage).as_str()
        );

        writeln!(
            f,
            "{}:\t{}MHz\t{}MHz\t{}\t{}\t{}\t{}",
            bold!(&self.name),
            self.max_freq / 1000,
            self.min_freq / 1000,
            color!(
                Color::GREEN,
                format!("{}MHz", self.cur_freq / 1000).as_str()
            ),
            temp,
            usage,
            self.gov,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csv::Writable;

    #[test]
    fn cpu_random_unit_test() {
        let cpu_1 = CPU::random();
        let cpu_2 = CPU::random();

        assert_ne!(cpu_1.cur_temp, cpu_2.cur_temp);
        assert_ne!(cpu_1.max_freq, cpu_2.max_freq);
    }

    #[test]
    fn cpu_empt_unit_test() {
        let cpu_1 = CPU::default();
        let cpu_2 = CPU::random();

        assert_ne!(cpu_1.cur_temp, cpu_2.cur_temp);
        assert_ne!(cpu_1.max_freq, cpu_2.max_freq);

        assert_eq!(cpu_1.cur_temp, 0);
        assert_eq!(cpu_1.max_freq, 0);
    }

    #[test]
    fn cpu_to_csv_unit_test() {
        let mut cpu = CPU::default();

        let out = cpu.to_csv();
        assert!(out.contains(",,0,0,0,0,0,0,\n"));

        cpu.cur_temp = 5;

        let out = cpu.to_csv();
        assert!(out.contains(",,0,0,0,0,5,0,\n"));
    }
}
