use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use super::display::{print_cpu, render_cpu};
use super::Error;

pub trait Speed {
    fn read_int(&mut self, sub_path: &str) -> Result<i32, Error>;
    fn read_str(&mut self, sub_path: &str) -> Result<String, Error>;
    fn read_temp(&mut self, sub_path: &str) -> Result<i32, Error>;
    fn write_value(&mut self, value: WritableValue) -> Result<(), Error>;
    fn update(&mut self) -> Result<(), Error>;
    fn init_cpu(&mut self) -> Result<(), Error>;
    fn set_max(&mut self, max: i32) -> Result<(), Error>;
    fn set_min(&mut self, min: i32) -> Result<(), Error>;
    fn get_max(&mut self) -> Result<(), Error>;
    fn get_min(&mut self) -> Result<(), Error>;
    fn get_cur(&mut self) -> Result<(), Error>;
    fn get_temp(&mut self) -> Result<(), Error>;
    fn get_gov(&mut self) -> Result<(), Error>;
    fn set_gov(&mut self, gov: String) -> Result<(), Error>;
    fn print(&self);
    fn render(&self) -> String;
}

#[derive(Debug)]
pub struct CPU {
    pub name: String,
    pub max_freq: i32,
    pub min_freq: i32,
    pub cur_freq: i32,
    pub cur_temp: i32,
    pub gov: String,
}

#[derive(PartialEq)]
pub enum WritableValue {
    Min,
    Max,
    Gov,
}

impl Speed for CPU {
    /// A generic function to take a path and a single cpu (single core) and get an i32
    fn read_int(&mut self, sub_path: &str) -> Result<i32, Error> {
        let mut info: String = String::new();
        let cpu_info_path: String = format!("/sys/devices/system/cpu/{}/{}", self.name, sub_path);

        File::open(cpu_info_path)?.read_to_string(&mut info)?;

        // Remove newline
        info.pop();
        Ok(info
            .parse::<i32>()
            .unwrap_or_else(|e| panic!("Could not parse {}\n{}", sub_path, e)))
    }

    fn read_str(&mut self, sub_path: &str) -> Result<String, Error> {
        let mut info: String = String::new();
        let cpu_info_path: String = format!("/sys/devices/system/cpu/{}/{}", self.name, sub_path);

        File::open(cpu_info_path)?.read_to_string(&mut info)?;

        // Remove newline
        info.pop();
        Ok(info)
    }

    fn read_temp(&mut self, sub_path: &str) -> Result<i32, Error> {
        let mut info: String = String::new();
        let cpu_info_path: String = format!(
            "/sys/class/thermal/{}/{}",
            self.name.replace("cpu", "thermal_zone"),
            sub_path
        );

        if !Path::new(&cpu_info_path).exists() {
            return Ok(-1);
        }

        File::open(cpu_info_path)?.read_to_string(&mut info)?;

        // Remove the last character (the newline)
        info.pop();

        Ok(info
            .parse::<i32>()
            .unwrap_or_else(|e| panic!("Could not parse {}\n{}", sub_path, e)))
    }

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
        buffer.write(&to_write.as_bytes())?;

        Ok(())
    }

    /// Get all the attributes of a cpu
    /// These get methods write the value returned
    fn update(&mut self) -> Result<(), Error> {
        self.get_max()?;
        self.get_min()?;
        self.get_cur()?;
        self.get_temp()?;
        self.get_gov()?;
        Ok(())
    }

    fn init_cpu(&mut self) -> Result<(), Error> {
        self.update()?;
        Ok(())
    }

    fn set_max(&mut self, max: i32) -> Result<(), Error> {
        self.max_freq = max;
        self.write_value(WritableValue::Max)?;
        Ok(())
    }

    fn set_min(&mut self, min: i32) -> Result<(), Error> {
        self.min_freq = min;
        self.write_value(WritableValue::Min)?;
        Ok(())
    }

    fn get_max(&mut self) -> Result<(), Error> {
        self.max_freq = self.read_int("cpufreq/scaling_max_freq")?;
        Ok(())
    }

    fn get_min(&mut self) -> Result<(), Error> {
        self.min_freq = self.read_int("cpufreq/scaling_min_freq")?;
        Ok(())
    }

    fn get_cur(&mut self) -> Result<(), Error> {
        self.cur_freq = self.read_int("cpufreq/scaling_cur_freq")?;
        Ok(())
    }

    fn get_temp(&mut self) -> Result<(), Error> {
        self.cur_temp = self.read_temp("temp")?;
        Ok(())
    }

    fn get_gov(&mut self) -> Result<(), Error> {
        self.gov = self.read_str("cpufreq/scaling_governor")?;
        Ok(())
    }

    fn set_gov(&mut self, gov: String) -> Result<(), Error> {
        self.gov = gov.clone();
        self.write_value(WritableValue::Gov)?;
        Ok(())
    }

    fn print(&self) {
        print_cpu(self);
    }

    fn render(&self) -> String {
        render_cpu(self)
    }
}
