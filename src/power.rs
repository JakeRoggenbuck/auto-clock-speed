use colored::Colorize;
use std::any::Any;
use std::cmp::PartialEq;
use std::fmt;
use std::fs::{self, read_dir};
use std::path::Path;

use super::create_issue;
use super::Error;

const LID_STATUS_PATH: [&str; 4] = [
    "/proc/acpi/button/lid/LID/state",
    "/proc/acpi/button/lid/LID0/state",
    "/proc/acpi/button/lid/LID1/state",
    "/proc/acpi/button/lid/LID2/state",
];

const BATTERY_CHARGE_PATH: [&str; 4] = [
    "/sys/class/power_supply/BAT/",
    "/sys/class/power_supply/BAT0/",
    "/sys/class/power_supply/BAT1/",
    "/sys/class/power_supply/BAT2/",
];

const POWER_SOURCE_PATH: [&str; 4] = [
    "/sys/class/power_supply/AC/online",
    "/sys/class/power_supply/AC0/online",
    "/sys/class/power_supply/AC1/online",
    "/sys/class/power_supply/ACAD/online",
];

#[derive(PartialEq, Eq)]
pub enum LidState {
    Open,
    Closed,
    Unapplicable,
    Unknown,
}

impl fmt::Display for LidState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LidState::Open => write!(f, "open"),
            LidState::Closed => write!(f, "closed"),
            LidState::Unapplicable => write!(f, "unapplicable"),
            LidState::Unknown => write!(f, "unknown"),
        }
    }
}

pub fn has_battery() -> bool {
    let power_dir = Path::new("/sys/class/power_supply/");
    let dir_count = read_dir(power_dir).into_iter().len();
    dir_count > 0
}

pub fn get_best_path(paths: [&'static str; 4]) -> Result<&str, Error> {
    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(path);
        }
    }

    Err(Error::Unknown)
}

pub fn read_lid_state() -> Result<LidState, Error> {
    let path: &str = match get_best_path(LID_STATUS_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("Could not detect your lid state.");
            create_issue!("If you are on a laptop");
            return Ok(LidState::Unapplicable);
        }
    };

    let lid_str = fs::read_to_string(path)?;

    let state = if lid_str.contains("open") {
        LidState::Open
    } else if lid_str.contains("closed") {
        LidState::Closed
    } else {
        LidState::Unknown
    };

    Ok(state)
}

pub fn read_power_source() -> Result<bool, Error> {
    let path: &str = match get_best_path(POWER_SOURCE_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("We could not detect your AC power source.");
            create_issue!("If you have a power source");
            return Ok(true);
        }
    };

    let mut pwr_str = fs::read_to_string(path)?;

    // Remove the \n char
    pwr_str.pop();

    Ok(pwr_str == "1")
}

enum BatteryConditionType {
    Energy,
    Charge,
    None,
}
pub struct Battery {
    sys_parent_path: String,
    capacity: i8,
    condition_type: BatteryConditionType,
    condition: f32,
    charge_full: i32,
    charge_full_design: i32,
    energy_full: i32,
    energy_full_design: i32,
}

impl Battery {
    pub fn new() -> Result<Battery, Error> {
        let mut obj = Battery {
            sys_parent_path: "unknown".to_string(),
            capacity: 0_i8,
            condition_type: BatteryConditionType::None,
            condition: 0_f32,
            charge_full: 0_i32,
            charge_full_design: 0_i32,
            energy_full: 0_i32,
            energy_full_design: 0_i32,
        };
        let path: &str = match get_best_path(BATTERY_CHARGE_PATH) {
            Ok(path) => path,
            Err(error) => {
                if error.type_id() == Error::IO.type_id() {
                    // Make sure to return IO error if one occurs
                    return Err(error);
                }
                // If it doesn't exist then it is plugged in so make it 100% percent capacity
                eprintln!("We could not detect your battery.");
                create_issue!("If you are on a laptop");
                return Err(Error::Unknown);
            }
        };
        obj.sys_parent_path = path.to_string();
        obj.check_condition_type();
        Ok(obj)
    }

    pub fn read_charge(&mut self) -> Result<i8, Error> {
        let charge_path = self.sys_parent_path.to_string() + "capacity";
        let mut cap_str = fs::read_to_string(charge_path)?;

        // Remove the \n char
        cap_str.pop();

        let charge = cap_str.parse::<i8>().unwrap();
        self.capacity = charge;

        Ok(charge)
    }

    pub fn print_status(&mut self, charging: bool) -> String {
        if has_battery() {
            match self.read_charge() {
                Ok(bat) => {
                    format!(
                        "Battery: {}",
                        if charging {
                            format!("{}%", bat).green()
                        } else {
                            format!("{}%", bat).red()
                        },
                    )
                }
                Err(e) => format!("Battery charge could not be read\n{:?}", e),
            }
        } else {
            format!("Battery: {}", "N/A".bold())
        }
    }

    fn check_condition_type(&mut self) {
        let path = self.sys_parent_path.to_string() + "charge_full";
        if Path::new(&path).is_file() {
            self.condition_type = BatteryConditionType::Charge
        }
        let path = self.sys_parent_path.to_string() + "energy_full";
        if Path::new(&path).is_file() {
            self.condition_type = BatteryConditionType::Energy
        }
    }

    pub fn get_condition(&mut self) -> Result<f32, Error> {
        match self.condition_type {
            BatteryConditionType::Energy => {
                self.read_energy_full()?;
                self.condition = self.energy_full as f32 / self.energy_full_design as f32
            }
            BatteryConditionType::Charge => {
                self.read_charge_full()?;
                self.condition = self.charge_full as f32 / self.charge_full_design as f32
            }
            BatteryConditionType::None => {
                return Err(Error::Unknown);
            }
        }
        let mut bat_cond = self.condition * 100.0;
        if bat_cond >= 100.0 {
            bat_cond = 100.00;
        } else if bat_cond <= 0.0 {
            bat_cond = 0.0;
        }

        Ok(bat_cond.round())
    }

    fn read_energy_full(&mut self) -> Result<(), Error> {
        let mut energy_path: String;
        let mut value: String;

        energy_path = self.sys_parent_path.to_string() + "energy_full_design";
        value = fs::read_to_string(energy_path)?;
        value.pop();
        self.energy_full_design = value.parse::<i32>().unwrap();

        energy_path = self.sys_parent_path.to_string() + "energy_full";
        value = fs::read_to_string(energy_path)?;
        value.pop();
        self.energy_full = value.parse::<i32>().unwrap();
        Ok(())
    }

    fn read_charge_full(&mut self) -> Result<(), Error> {
        let mut charge_path: String;
        let mut value: String;

        charge_path = self.sys_parent_path.to_string() + "charge_full_design";
        value = fs::read_to_string(charge_path)?;
        value.pop();
        self.charge_full_design = value.parse::<i32>().unwrap();

        charge_path = self.sys_parent_path.to_string() + "charge_full";
        value = fs::read_to_string(charge_path)?;
        value.pop();
        self.charge_full = value.parse::<i32>().unwrap();
        Ok(())
    }
}
