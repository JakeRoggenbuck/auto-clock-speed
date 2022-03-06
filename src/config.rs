use super::warn_user;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

/// Return the local config path
pub fn config_path() -> String {
    String::from("/etc/acs/acs.toml")
}

/// Check if the config file exists
/// /etc/acs/acs.toml
pub fn config_file_exists() -> bool {
    Path::new(&config_path()).exists()
}

/// Check if the config directory exists
/// /etc/acs/acs.toml
pub fn config_dir_exists() -> bool {
    Path::new("/etc/acs/").exists()
}

pub fn default_config() -> Config {
    Config {
        powersave_under: 20,
        overheat_threshold: 80,
        ignore_power: false,
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub powersave_under: i8,
    pub overheat_threshold: i8,
    pub ignore_power: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SafeConfig {
    pub powersave_under: Option<i8>,
    pub overheat_threshold: Option<i8>,
    pub ignore_power: Option<bool>,
}

trait SafeFillConfig {
    fn safe_fill_config(&mut self) -> Config;
}

impl SafeFillConfig for SafeConfig {
    fn safe_fill_config(&mut self) -> Config {
        // This function makes sure the config contains every value from Config,
        // even when this type is SafeConfig
        //
        // This could be done one of two ways.
        // The current implementation pulls the default_config to base
        // then checks each value in self (Config) and if it exists, or is_some,
        // then copy the value from self and overwrite the value in base
        //
        // if self.val.is_some => base.val = self.val
        //
        // This approach coincidentally happens to be more efficient when more than half
        // of the values are not defined. This means that if no config is present, then no work
        // will be done to modify base.
        let mut base = default_config();

        if self.powersave_under.is_some() {
            base.powersave_under = self.powersave_under.unwrap();
        }

        if self.overheat_threshold.is_some() {
            base.overheat_threshold = self.overheat_threshold.unwrap();
        }

        if self.ignore_power.is_some() {
            base.ignore_power = self.ignore_power.unwrap();
        }

        return base;
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        // TODO: We need to make a better way to loop through everything. It might be to make
        // config iterable. This would also make safe_fill_config a lot easier as well.
        write!(
            f,
            "powersave_under = {}\noverheat_threshold = {}\nignore_power = {}",
            self.powersave_under, self.overheat_threshold, self.ignore_power,
        )
    }
}

fn read_as_string(config_file: &mut File) -> String {
    // Read it to new string
    let mut config: String = String::new();
    config_file.read_to_string(&mut config).unwrap();
    return config;
}

fn parse_as_toml(config: String) -> Config {
    let mut safe_config: SafeConfig =
        // Read the config from config string and if it fails, give the base config with undefined
        // variables so that the defined variables can be swapped in
        toml::from_str(config.as_str()).unwrap_or_else(|_| SafeConfig {
            powersave_under: None,
            overheat_threshold: None,
            ignore_power: None,
        });

    safe_config.safe_fill_config()
}

pub fn open_config() -> Result<Config, std::io::Error> {
    let conf_path = config_path();
    let mut config_file: File = File::open(&conf_path)?;
    let config_string = read_as_string(&mut config_file);
    let config_toml = parse_as_toml(config_string);

    Ok(config_toml)
}

pub fn get_config() -> Config {
    // Config will always exist, default or otherwise
    match open_config() {
        Ok(conf) => conf,
        Err(_) => {
            warn_user!("Using default config. Create file '/etc/acs/acs.toml' for custom config.");
            // Use default config as config
            default_config()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_unit_test() {
        let config: Config = default_config();
        assert!(config.powersave_under > 0 && config.powersave_under < 100);
    }

    #[test]
    fn read_as_string_unit_test() -> Result<(), std::io::Error> {
        let conf_file = "acs.toml";
        let conf_str: String = read_as_string(&mut File::open(conf_file)?);

        assert!(conf_str.contains("# acs.toml\n"));
        assert!(conf_str.contains("powersave_under = 20\n"));
        Ok(())
    }

    #[test]
    fn parse_as_toml_unit_test() -> Result<(), std::io::Error> {
        let conf_file = "acs.toml";
        let conf_str: String = read_as_string(&mut File::open(conf_file)?);
        let toml = parse_as_toml(conf_str);
        assert_eq!(toml.powersave_under, 20);
        Ok(())
    }
}
