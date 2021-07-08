use super::Error;
use std::fs::{read_dir, File};
use std::io::Read;

pub trait Speed {
    fn read_value(&mut self, sub_path: String) -> Result<i32, Error>;
    fn update(&mut self);
    fn init_cpu(&mut self);
    fn set_max(&mut self, max: i32);
    fn set_min(&mut self, min: i32);
    fn get_max(&mut self);
    fn get_min(&mut self);
    fn get_cur(&mut self);
    fn get_gov(&mut self);
    fn print(&self);
}

#[derive(Debug)]
pub struct CPU {
    pub name: String,
    pub max_freq: i32,
    pub min_freq: i32,
    pub cur_freq: i32,
}

impl Speed for CPU {
    /// A generic function to take a path and a single cpu (single core) and get an i32
    fn read_value(&mut self, sub_path: String) -> Result<i32, Error> {
        let mut info: String = String::new();
        let cpu_info_path: String =
            format!("/sys/devices/system/cpu/{}/{}", self.name, sub_path);
    
        File::open(cpu_info_path)?.read_to_string(&mut info)?;
    
        // Remove the last character (the newline)
        info.pop();
        match info.parse::<i32>() {
            Err(e) => panic!("{}", e),
            Ok(a) => Ok(a),
        }
    }

    fn update(&mut self) {
        self.get_max();
        self.get_min();
        self.get_cur();
    }

    fn init_cpu(&mut self) {
        self.update();
    }

    fn set_max(&mut self, max: i32) {
        // TODO: change the file with the speed
        self.max_freq = max;
    }

    fn set_min(&mut self, min: i32) {
        // TODO: change the file with the speed
        self.min_freq = min;
    }

    fn get_max(&mut self) {
        match self.read_value(
            "cpufreq/scaling_max_freq".to_string(),
        ) {
            Ok(a) => {
                self.max_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }

    fn get_min(&mut self) {
        match self.read_value(
            "cpufreq/scaling_min_freq".to_string(),
        ) {
            Ok(a) => {
                self.min_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }

    fn get_cur(&mut self) {
        match self.read_value(
            "cpufreq/scaling_cur_freq".to_string(),
        ) {
            Ok(a) => {
                self.cur_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }
    fn get_gov(&mut self) {
        match self.read_value(
            "cpufreq/scaling_governor".to_string(),
        ) {
            Ok(a) => {
                self.cur_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }

    fn print(&self) {
        println!("{:?}", self);
    }
}
