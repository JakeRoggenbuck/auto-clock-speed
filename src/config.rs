use super::local::config_path;
use super::warn_user;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub powersave_under: i8,
    // Future variables
    // pub charging_powersave_under: i32,
}

pub fn default_config() -> Config {
    Config {
        powersave_under: 20,
    }
}

pub fn open_config() -> Result<Config, std::io::Error> {
    // Open config file
    let mut config_file = match File::open(config_path().as_str()) {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    // Read it to new string
    let mut config: String = String::new();
    config_file.read_to_string(&mut config).unwrap();

    // Try parsing as string, warn user if broken
    // e.g. WARN: missing field `charging_powersave_under` at line 1 column 1
    let config_toml: Config = match toml::from_str(config.as_str()) {
        Ok(a) => a,
        Err(e) => {
            warn_user!(format!("{}", e));
            panic!("{}", e);
        }
    };

    Ok(config_toml)
}
