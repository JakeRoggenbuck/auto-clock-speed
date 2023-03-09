use super::system::{read_int, read_str};
use crate::error::Error;
use efcl::{color, Color};
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
                color!(Color::GREEN, &self.name)
            } else {
                color!(Color::RED, &self.name)
            },
            color!(Color::YELLOW, (self.temp / 1000).to_string().as_str()),
            color!(Color::YELLOW, "C°"),
            self.path
        )
    }
}

pub fn read_thermal_zones() -> Result<Vec<ThermalZone>, Error> {
    let mut zones = Vec::<ThermalZone>::new();

    for a in read_dir(THERMAL_ZONE_DIR).expect("Could not read thermal directory") {
        let path_string: String = format!("{}", a?.path().to_string_lossy());
        if !path_string.starts_with(&[THERMAL_ZONE_DIR, "thermal_zone"].concat()) {
            continue;
        }

        let zone = ThermalZone {
            temp: read_int(&[&path_string, "/temp"].concat())?,
            name: read_str(&[&path_string, "/type"].concat())?,
            enabled: read_str(&[&path_string, "/mode"].concat())? == "enabled",
            path: path_string,
        };

        zones.push(zone);
    }
    Ok(zones)
}
