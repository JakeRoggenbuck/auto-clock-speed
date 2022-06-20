use std::fs::{read_dir};

pub struct ThermalZone {
    pub name: String,
    pub location: String,
    pub temp: i32,
    pub enabled: bool,
}

impl Default for ThermalZone {
    fn default() -> Self {
        ThermalZone {
            name: "unknown".to_string(),
            location: "unknown".to_string(),
            temp: 0,
            enabled: false,
        }
    }
}

pub fn read_thermal_zones() -> Vec::<ThermalZone> {
    let mut zones = Vec::<ThermalZone>::new();

    for a in read_dir("/sys/class/thermal/").expect("Could not read thermal directory") {
        let path_string: String = format!("{:?}", a.unwrap().path());
        println!("{}", path_string);

    }
    zones
}
