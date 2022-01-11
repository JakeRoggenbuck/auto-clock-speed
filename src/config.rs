use super::local::config_path;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io;
use std::io::Read;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub powersave_under: i32,
    pub charging_powersave_under: i32,
}

pub fn open_config() -> Result<Config, io::Error> {
    let mut config_file = File::open(config_path().as_str())?;
    let mut config: String = String::new();
    config_file.read_to_string(&mut config)?;
    let config_toml: Config = toml::from_str(config.as_str()).unwrap();
    Ok(config_toml)
}
