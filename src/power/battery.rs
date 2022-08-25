use crate::create_issue;
use crate::power::get_best_path;
use crate::Error;
use std::any::Any;
use std::fs;
use std::fs::read_dir;
use std::path::Path;

/// A list containing each potential path used for gathering battery status from the kernel
const BATTERY_CHARGE_PATH: [&str; 4] = [
    "/sys/class/power_supply/BAT/",
    "/sys/class/power_supply/BAT0/",
    "/sys/class/power_supply/BAT1/",
    "/sys/class/power_supply/BAT2/",
];

/// Returns if this system has a battery or not
pub fn has_battery() -> bool {
    let power_dir = Path::new("/sys/class/power_supply/");
    let dir_count = read_dir(power_dir).into_iter().len();
    dir_count > 0
}

/// Describes how the battery condition was obtained
pub enum BatteryConditionType {
    Energy,
    Charge,
    None,
}

/// Describes the current status of the battery
#[derive(PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
    Unknown,
}

/// A structure for holding information about a battery
/// This structure follows an update model where information within the structure gets updated upon
/// calling the update method
pub struct Battery {
    pub sys_parent_path: String,
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
        let mut obj = Battery {
            sys_parent_path: "unknown".to_string(),
            capacity: 0_i8,
            condition_type: BatteryConditionType::None,
            condition: 0_i8,
            charge_full: 0_i32,
            charge_full_design: 0_i32,
            energy_full: 0_i32,
            energy_full_design: 0_i32,
            status: BatteryStatus::Unknown,
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

    /// Get the battery charge on this device then updates the struct
    fn read_charge(&mut self) -> Result<(), Error> {
        let charge_path = self.sys_parent_path.to_string() + "capacity";
        let mut cap_str = fs::read_to_string(charge_path)?;

        // Remove the \n char
        cap_str.pop();

        let charge = cap_str.parse::<i8>().unwrap();
        self.capacity = charge;

        Ok(())
    }

    /// Checks the file system for the proper battery condition type for this system then updates
    /// the struct
    /// BatteryConditionType::Charge = charge_full
    /// BatteryConditionType::Energy = energy_full
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

    /// Reads that charge_full and charge_full_design values and saves them to the struct
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

    /// Updates all values in this struct from the battery drivers
    pub fn update(&mut self) -> Result<(), Error> {
        self.get_condition()?;
        self.read_charge()?;

        Ok(())
    }
}
