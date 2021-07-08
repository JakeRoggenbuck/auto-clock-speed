use super::Error;
use std::fs::{read_dir, File};
use std::io::Read;

pub struct Cpu {
    pub name: String,
    max_freq: i32,
    min_freq: i32,
    cur_freq: i32,
    base_freq: i32,
    turbo: bool,
}

impl Cpu {
    pub fn new(name: &str) -> Cpu {
        Cpu {
            name: name.to_string(),
            max_freq: 0,
            min_freq: 0,
            cur_freq: 0,
            base_freq: 0,
            turbo: false
        }
    }

    /// A generic function to take a path and a single cpu (single core) and get an i32
    pub fn read_value(&self, sub_path: String) -> Result<i32, Error> {
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
}
