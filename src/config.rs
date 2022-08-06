use crate::print_error;

use super::daemon::State;
use super::warn_user;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::fs::File;
use std::io::{ErrorKind, Read, Write};
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
        high_cpu_threshold: 50,
        active_rules: vec![
            State::BatteryLow,
            State::LidClosed,
            State::Charging,
            State::CpuUsageHigh,
        ],
    }
}
/// Creates a config file at the default location if it doesn't exist
pub fn init_config() {
    if !config_file_exists() {
        // If the config directory doesn't exist, create it
        if !config_dir_exists() {
            let acs_dir = std::fs::create_dir_all("/etc/acs/");
            match acs_dir {
                Ok(_) => {}
                Err(error) => match error.kind() {
                    ErrorKind::PermissionDenied => {
                        print_error!("Could not create config directory '/etc/acs/'. Permission denied. Try running as root or use sudo.");
                        return;
                    }
                    _ => {
                        print_error!(format!("Failed to create config directory: {}", error));
                        return;
                    }
                },
            }
        }
        let config_file = File::create(&config_path());
        let mut config = match config_file {
            Ok(file) => file,
            Err(error) => match error.kind() {
                ErrorKind::PermissionDenied => {
                    print_error!("Looks like you don't have permission to write to /etc/acs/acs.toml. Try running this program as root or using sudo.");
                    return;
                }
                _ => {
                    print_error!(format!("Failed to create config file: {}", error));
                    return;
                }
            },
        };
        let default_config = default_config();
        let serialized = toml::to_string(&default_config).unwrap();
        config.write_all(serialized.as_bytes()).unwrap();
        println!("Created config file at '/etc/acs/acs.toml'");
    } else {
        warn_user!("Config file already exists at '/etc/acs/acs.toml'. No changes made.");
    }
}

#[derive(Debug, Serialize)]
pub struct Config {
    pub powersave_under: i8,
    pub overheat_threshold: i8,
    pub high_cpu_threshold: i8,
    pub active_rules: Vec<State>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct SafeConfig {
    pub powersave_under: Option<i8>,
    pub overheat_threshold: Option<i8>,
    pub high_cpu_threshold: Option<i8>,
    pub active_rules: Option<Vec<String>>,
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

        if self.high_cpu_threshold.is_some() {
            base.high_cpu_threshold = self.high_cpu_threshold.unwrap();
        }

        if self.active_rules.is_some() {
            base.active_rules.clear();
            for rule in self.active_rules.clone().unwrap() {
                base.active_rules.push(match rule.as_str() {
                    "battery_percent_rule" => State::BatteryLow,
                    "lid_open_rule" => State::LidClosed,
                    "ac_charging_rule" => State::Charging,
                    "cpu_usage_rule" => State::CpuUsageHigh,
                    _ => State::Unknown,
                });
            }
        }

        base
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> fmt::Result {
        // TODO: We need to make a better way to loop through everything. It might be to make
        // config iterable. This would also make safe_fill_config a lot easier as well.
        write!(
            f,
            "powersave_under = {}\noverheat_threshold = {}\nhigh_cpu_threshold = {}\nactive_rules = {:?}",
            self.powersave_under, self.overheat_threshold, self.high_cpu_threshold, self.active_rules,
        )
    }
}

fn read_as_string(config_file: &mut File) -> String {
    // Read it to new string
    let mut config: String = String::new();
    config_file.read_to_string(&mut config).unwrap();
    config
}

fn parse_as_toml(config: String) -> Config {
    let mut safe_config: SafeConfig =
        // Read the config from config string and if it fails, give the base config with undefined
        // variables so that the defined variables can be swapped in
        toml::from_str(config.as_str()).unwrap_or(SafeConfig {
            powersave_under: None,
            overheat_threshold: None,
            high_cpu_threshold: None,
            active_rules: None,
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
            warn_user!("Using default config. Create file '/etc/acs/acs.toml' for custom config or run 'acs initconfig' to setup default config automatically.");
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
