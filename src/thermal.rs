use std::fs::{read_dir};

const THERMAL_ZONE_DIR: &str = "/sys/class/thermal/";

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

    for a in read_dir(THERMAL_ZONE_DIR).expect("Could not read thermal directory") {
        let path_string: String = format!("{}", a.unwrap().path().to_string_lossy());
        if !path_string.starts_with(&[THERMAL_ZONE_DIR, "thermal_zone"].concat()) {
            continue;
        }
        println!("{}", path_string);

    }
    zones
}
