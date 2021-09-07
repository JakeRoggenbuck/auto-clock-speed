use super::display::print_cpu;
use super::exit;
use super::{Error, GovGetError, TempGetError, GovSetError, SpeedGetError, SpeedSetError};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub trait Speed {
    fn read_int(&mut self, sub_path: String) -> Result<i32, Error>;
    fn read_str(&mut self, sub_path: String) -> Result<String, Error>;
    fn read_temp(&mut self, sub_path: String) -> Result<i32, Error>;
    fn write_value(&mut self, value: WritableValue) -> Result<(), Error>;
    fn update(&mut self);
    fn init_cpu(&mut self);
    fn set_max(&mut self, max: i32);
    fn set_min(&mut self, min: i32);
    fn get_max(&mut self);
    fn get_min(&mut self);
    fn get_cur(&mut self);
    fn get_temp(&mut self);
    fn get_gov(&mut self);
    fn set_gov(&mut self, gov: String);
    fn print(&self);
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
    fn read_int(&mut self, sub_path: String) -> Result<i32, Error> {
        let mut info: String = String::new();
        let cpu_info_path: String = format!("/sys/devices/system/cpu/{}/{}", self.name, sub_path);

        File::open(cpu_info_path)?.read_to_string(&mut info)?;

        // Remove the last character (the newline)
        info.pop();
        match info.parse::<i32>() {
            Err(_) => {
                eprintln!("Could not read {}", sub_path);
                exit(1);
            }
            Ok(a) => Ok(a),
        }
    }

    fn read_temp(&mut self, sub_path: String) -> Result<i32, Error> {
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

        match info.parse::<i32>() {
            Err(_) => {
                eprintln!("Could not read {}", sub_path);
                exit(1);
            }
            Ok(a) => Ok(a),
        }
    }

    fn read_str(&mut self, sub_path: String) -> Result<String, Error> {
        let mut info: String = String::new();
        let cpu_info_path: String = format!("/sys/devices/system/cpu/{}/{}", self.name, sub_path);

        File::open(cpu_info_path)?.read_to_string(&mut info)?;

        // Remove the last character (the newline)
        info.pop();
        Ok(info)
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
    fn update(&mut self) {
        self.get_max();
        self.get_min();
        self.get_cur();
        self.get_temp();
        self.get_gov();
    }

    fn init_cpu(&mut self) {
        self.update();
    }

    fn set_max(&mut self, max: i32) {
        self.max_freq = max;
        match self.write_value(WritableValue::Max) {
            Err(_) => {
                eprint!("{}", SpeedSetError);
                exit(1);
            }
            Ok(_) => (),
        };
    }

    fn set_min(&mut self, min: i32) {
        self.min_freq = min;
        match self.write_value(WritableValue::Min) {
            Err(_) => {
                eprint!("{}", SpeedSetError);
                exit(1);
            }
            Ok(_) => (),
        };
    }

    fn get_max(&mut self) {
        let path = "cpufreq/scaling_max_freq";
        match self.read_int(path.to_string()) {
            Ok(a) => {
                self.max_freq = a;
            }
            Err(_) => {
                eprint!("{}", SpeedGetError);
                exit(1);
            }
        }
    }

    fn get_min(&mut self) {
        let path = "cpufreq/scaling_min_freq";
        match self.read_int(path.to_string()) {
            Ok(a) => {
                self.min_freq = a;
            }
            Err(_) => {
                eprint!("{}", SpeedGetError);
                exit(1)
            }
        }
    }

    fn get_cur(&mut self) {
        let path = "cpufreq/scaling_cur_freq";
        match self.read_int(path.to_string()) {
            Ok(a) => {
                self.cur_freq = a;
            }
            Err(_) => {
                eprint!("{}", SpeedGetError);
                exit(1)
            }
        }
    }

    fn get_temp(&mut self) {
        let path = "temp";
        match self.read_temp(path.to_string()) {
            Ok(a) => {
                self.cur_temp = a;
            }
            Err(_) => {
                eprint!("{}", TempGetError);
                exit(1)
            }
        }
    }

    fn get_gov(&mut self) {
        let path = "cpufreq/scaling_governor";
        match self.read_str(path.to_string()) {
            Ok(a) => {
                self.gov = a;
            }
            Err(_) => {
                eprint!("{}", GovGetError);
                exit(1)
            }
        }
    }

    fn set_gov(&mut self, gov: String) {
        self.gov = gov.clone();
        match self.write_value(WritableValue::Gov) {
            Err(_) => {
                eprint!("{}", GovSetError);
                exit(1)
            }
            Ok(_) => (),
        };
    }

    fn print(&self) {
        print_cpu(self);
    }
}
