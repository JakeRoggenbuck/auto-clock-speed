#[derive(PartialEq, Clone)]
pub enum GraphType {
    Hidden,
    Frequency,
    Usage,
    Temperature,
}

pub fn get_graph_type(graph_type: &str) -> Option<GraphType> {
    match graph_type.to_lowercase().as_str() {
        "hidden" => Some(GraphType::Hidden),
        "freq" => Some(GraphType::Frequency),
        "usage" => Some(GraphType::Usage),
        "temp" => Some(GraphType::Temperature),
        _ => None,
    }
}

#[derive(Clone)]
pub struct Settings {
    pub verbose: bool,
    pub delay: u64,
    pub delay_battery: u64,
    pub edit: bool,
    pub no_animation: bool,
    pub graph: GraphType,
    pub commit: bool,
    pub testing: bool,
}
