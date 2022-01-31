use std::fs::File;
use std::io::Read;

use serde::{Deserialize, Serialize};

use super::local::config_path;
use super::warn_user;

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

fn open_config_file(conf_path: &str) -> Result<File, std::io::Error> {
    File::open(conf_path)
}

fn read_as_string(config_file: &mut File) -> String {
    // Read it to new string
    let mut config: String = String::new();
    config_file.read_to_string(&mut config).unwrap();
    return config;
}

fn parse_as_toml(config: String) -> Config {
    // Try parsing as string, warn user if broken
    // e.g. WARN: missing field `charging_powersave_under` at line 1 column 1
    match toml::from_str(config.as_str()) {
        Ok(a) => a,
        Err(e) => {
            warn_user!(format!("{}", e));
            panic!("{}", e);
        }
    }
}

pub fn open_config() -> Result<Config, std::io::Error> {
    let conf_path = config_path();
    let mut config_file: File = match open_config_file(&conf_path) {
        Ok(a) => a,
        Err(e) => return Err(e),
    };

    let config_string = read_as_string(&mut config_file);
    let config_toml = parse_as_toml(config_string);

    Ok(config_toml)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_test() {
        let config: Config = default_config();
        assert!(config.powersave_under > 0 && config.powersave_under < 100);
    }

    #[test]
    fn open_config_file_test() -> Result<(), std::io::Error> {
        let conf_file = "acs.toml";
        open_config_file(conf_file)?;
        Ok(())
    }

    #[test]
    fn read_as_string_test() -> Result<(), std::io::Error> {
        let conf_file = "acs.toml";
        let conf_str: String = read_as_string(&mut File::open(conf_file)?);

        assert!(conf_str.contains("# acs.toml\n"));
        assert!(conf_str.contains("powersave_under = 20\n"));
        Ok(())
    }

    #[test]
    fn parse_as_toml_test() -> Result<(), std::io::Error> {
        let conf_file = "acs.toml";
        let conf_str: String = read_as_string(&mut File::open(conf_file)?);
        let toml = parse_as_toml(conf_str);
        assert_eq!(toml.powersave_under, 20);
        Ok(())
    }
}
