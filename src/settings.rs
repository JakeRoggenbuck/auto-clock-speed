use std::fmt;

#[derive(PartialEq, Eq, Clone)]
pub enum GraphType {
    Hidden,
    Frequency,
    Usage,
    Temperature,
    Unknown,
}

impl fmt::Display for GraphType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GraphType::Hidden => write!(f, "hidden"),
            GraphType::Frequency => write!(f, "frequency"),
            GraphType::Usage => write!(f, "usage"),
            GraphType::Temperature => write!(f, "temperature"),
            GraphType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Parse from graph_type parameter which type of graph will be displayed
pub fn get_graph_type(graph_type: &str) -> GraphType {
    match graph_type.to_lowercase().as_str() {
        "hidden" => GraphType::Hidden,
        "freq" => GraphType::Frequency,
        "usage" => GraphType::Usage,
        "temp" => GraphType::Temperature,
        _ => GraphType::Unknown,
    }
}

#[derive(Clone)]
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
}
