use super::graph::GraphType;
use std::default::Default;

pub trait DefaultTesting {
    fn default_testing() -> Settings;
}

#[derive(Clone, Debug)]
pub struct Settings {
    pub verbose: bool,
    pub delay: u64,
    pub delay_battery: u64,
    pub edit: bool,
    pub hook: bool,
    pub animation: bool,
    pub graph: GraphType,
    pub commit: bool,
    pub testing: bool,
    pub csv_file: String,
    pub log_csv: bool,
    pub log_size_cutoff: i32,
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
            csv_file: String::default(),
            log_csv: false,
            log_size_cutoff: 0,
            show_settings: false,
        }
    }
}
