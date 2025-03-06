#![forbid(unsafe_code)]
use super::graph::GraphType;
use std::default::Default;

pub trait DefaultTesting {
    fn default_testing() -> Settings;
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub verbose: bool,
    /// The delay between rerenders when connected to a power source
    pub delay: u64,
    /// The delay between rerenders when using battery power only
    pub delay_battery: u64,
    /// If the daemon should edit the governor based on the rules
    pub edit: bool,
    /// If the daemon should hook on to another client daemon
    pub hook: bool,
    /// If ACS should show an ASCII animation when turbo is enabled
    pub animation: bool,
    /// The type of graph that should be displayed with ACS monit
    pub graph: GraphType,
    /// If it should show the latest commit (used for debugging mostly)
    pub commit: bool,
    /// If the daemon is running in testing mode
    pub testing: bool,
    pub testing_logging: bool,
    /// The output csv filepath
    pub csv_file: String,
    /// If ACS should log to a file
    pub log_csv: bool,
    /// The size in MB that should be written before the file gets overwritten
    pub log_size_cutoff: i32,
    /// If ACS should show the settings and exit (used for debugging)
    pub show_settings: bool,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            verbose: true,
            delay: 1000,
            delay_battery: 1000,
            edit: false,
            hook: false,
            animation: true,
            graph: GraphType::default(),
            commit: false,
            testing: false,
            testing_logging: false,
            csv_file: String::default(),
            log_csv: false,
            log_size_cutoff: 0,
            show_settings: false,
        }
    }
}

impl DefaultTesting for Settings {
    fn default_testing() -> Settings {
        Settings {
            verbose: true,
            delay: 1,
            delay_battery: 2,
            edit: false,
            hook: false,
            animation: true,
            graph: GraphType::Hidden,
            commit: false,
            testing: true,
            testing_logging: false,
            csv_file: String::default(),
            log_csv: false,
            log_size_cutoff: 0,
            show_settings: false,
        }
    }
}
