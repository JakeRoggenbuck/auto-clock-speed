#[derive(PartialEq, Clone)]
pub enum GraphType {
    Hidden,
    Frequency,
    FrequencyIndividual,
    Usage,
    UsageIndividual,
    Temperature,
    TemperatureIndividual,
}

pub fn get_graph_type(graph_type: &str) -> Option<GraphType> {
    match graph_type.to_lowercase().as_str() {
        "hidden" => Some(GraphType::Hidden),
        "frequency" => Some(GraphType::Frequency),
        "frequency_individual" => Some(GraphType::FrequencyIndividual),
        "usage" => Some(GraphType::Usage),
        "usage_individual" => Some(GraphType::UsageIndividual),
        "temperature" => Some(GraphType::Temperature),
        "temperature_individual" => Some(GraphType::TemperatureIndividual),
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
