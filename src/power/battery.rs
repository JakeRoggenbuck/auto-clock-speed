use crate::sysfs;
use crate::error::Error;
use std::any::Any;
use std::fs::read_dir;
use std::path::{Path, PathBuf};

// SYSFS root path
const SYSFS_BATTERY_PATH: &str = "/sys/class/power_supply/";

/// Returns if this system has a battery or not
pub fn has_battery() -> bool {
    let power_dir = Path::new("/sys/class/power_supply/");
    let dir_count = read_dir(power_dir).into_iter().len();
    dir_count > 0
}

/// Describes how the battery condition was obtained
#[derive(Clone, Default)]
pub enum BatteryConditionType {
    Energy,
    Charge,
    #[default]
    None,
}

/// Describes the current status of the battery
#[derive(PartialEq, Eq, Clone, Default)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    #[default]
    Unknown,
}

/// A structure for holding information about a battery
/// This structure follows an update model where information within the structure gets updated upon
/// calling the update method
#[derive(Clone, Default)]
pub struct Battery {
    pub sys_parent_path: PathBuf,
    pub capacity: i8,
    pub condition_type: BatteryConditionType,
    pub condition: i8,
    pub charge_full: i32,
    pub charge_full_design: i32,
    pub energy_full: i32,
    pub energy_full_design: i32,
    pub status: BatteryStatus,
}

impl Battery {
    /// Creates a new instance of a battery
    /// This method also initialises the sys_parent_path variable with the correct path for the
    /// current system. It will also initialize the condition_type variable by checking in the file
    /// system.
    pub fn new() -> Result<Battery, Error> {
        let mut obj = Battery::default();
        let path: PathBuf = match sysfs::get_path_by_glob(SYSFS_BATTERY_PATH, "BAT*") {
            Ok(path) => path,
            Err(error) => {
                if error.type_id() == Error::IO.type_id() {
                    // Make sure to return IO error if one occurs
                    return Err(error);
                }
                return Err(Error::HdwNotFound);
            }
        };
        obj.sys_parent_path = path;
        obj.check_condition_type();
        Ok(obj)
    }

    /// Get the battery charge on this device then updates the struct
    fn read_charge(&mut self) -> Result<(), Error> {
        sysfs::read(
            &mut self.capacity,
            &self.sys_parent_path.clone().join("capacity"),
        )?;

        Ok(())
    }

    /// Checks the file system for the proper battery condition type for this system then updates
    /// the struct
    /// BatteryConditionType::Charge = charge_full
    /// BatteryConditionType::Energy = energy_full
    fn check_condition_type(&mut self) {
        let path = self.sys_parent_path.clone().join("charge_full");
        if path.is_file() {
            self.condition_type = BatteryConditionType::Charge
        }
        let path = self.sys_parent_path.clone().join("energy_full");
        if path.is_file() {
            self.condition_type = BatteryConditionType::Energy
        }
    }

    /// Gets the current battery condition of this device based on condition_type and updates the
    /// struct
    fn get_condition(&mut self) -> Result<(), Error> {
        match self.condition_type {
            BatteryConditionType::Energy => {
                self.read_energy_full()?;
                self.condition =
                    ((self.energy_full as f32 / self.energy_full_design as f32) * 100_f32) as i8
            }
            BatteryConditionType::Charge => {
                self.read_charge_full()?;
                self.condition =
                    ((self.charge_full as f32 / self.charge_full_design as f32) * 100_f32) as i8
            }
            BatteryConditionType::None => {
                return Err(Error::Unknown);
            }
        }

        Ok(())
    }

    /// Reads the energy_full and energy_full_design values and saves them to the struct
    fn read_energy_full(&mut self) -> Result<(), Error> {
        sysfs::read(
            &mut self.energy_full_design,
            &self.sys_parent_path.clone().join("energy_full_design"),
        )?;

        sysfs::read(
            &mut self.energy_full,
            &self.sys_parent_path.clone().join("energy_full"),
        )?;
        Ok(())
    }

    /// Reads that charge_full and charge_full_design values and saves them to the struct
    fn read_charge_full(&mut self) -> Result<(), Error> {
        sysfs::read(
            &mut self.charge_full_design,
            &self.sys_parent_path.clone().join("charge_full_design"),
        )?;

        sysfs::read(
            &mut self.charge_full,
            &self.sys_parent_path.clone().join("charge_full"),
        )?;
        Ok(())
    }

    /// Updates all values in this struct from the battery drivers
    pub fn update(&mut self) -> Result<(), Error> {
        self.get_condition()?;
        self.read_charge()?;

        Ok(())
    }
}
