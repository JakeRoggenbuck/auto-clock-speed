use super::graph::GraphType;

#[derive(Clone, Debug)]
pub struct Settings {
    pub verbose: bool,
    pub delay: u64,
    pub delay_battery: u64,
    pub edit: bool,
    pub hook: bool,
    pub no_animation: bool,
    pub graph: GraphType,
    pub commit: bool,
    pub testing: bool,
    pub csv_file: String,
    pub log_csv: bool,
    pub log_size_cutoff: i32,
    pub show_settings: bool,
}
