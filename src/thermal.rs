use super::system::{read_int, read_str};
use super::Error;
use colored::*;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::read_dir;

const THERMAL_ZONE_DIR: &str = "/sys/class/thermal/";

#[derive(Debug)]
pub struct ThermalZone {
    pub name: String,
    pub path: String,
    pub temp: i32,
    pub enabled: bool,
}

pub trait Thermal {
    fn update(&mut self) -> Result<(), Error>;
}

impl Default for ThermalZone {
    fn default() -> Self {
        ThermalZone {
            name: "unknown".to_string(),
            path: "unknown".to_string(),
            temp: 0,
            enabled: false,
        }
    }
}

impl Display for ThermalZone {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(
            f,
            "{} {}{} {}",
            if self.enabled {
                self.name.green()
            } else {
                self.name.red()
            },
            (self.temp / 1000).to_string().yellow(),
            "C°".yellow(),
            self.path
        )
    }
}

pub fn read_thermal_zones() -> Vec<ThermalZone> {
    let mut zones = Vec::<ThermalZone>::new();

    for a in read_dir(THERMAL_ZONE_DIR).expect("Could not read thermal directory") {
        let path_string: String = format!("{}", a.unwrap().path().to_string_lossy());
        if !path_string.starts_with(&[THERMAL_ZONE_DIR, "thermal_zone"].concat()) {
            continue;
        }

        let mut zone = ThermalZone::default();

        zone.temp = read_int(&[&path_string, "/temp"].concat()).unwrap_or(0);
        zone.name = read_str(&[&path_string, "/type"].concat()).unwrap_or("unknown".to_string());
        zone.enabled = read_str(&[&path_string, "/mode"].concat())
            .unwrap_or("disable".to_string()) == "enabled";
        zone.path = path_string;

        zones.push(zone);
    }
    zones
}
