#[derive(PartialEq)]
pub enum GraphType {
    Hidden,
    Frequency,
    FrequencyIndividual,
    Usage,
    UsageIndividual,
    Temperature,
    TemperatureIndividual,
}

pub fn get_graph_type(graph_type: &str) -> GraphType {
    match graph_type.to_lowercase().as_str() {
        "hidden" => GraphType::Hidden,
        "frequency" => GraphType::Frequency,
        "frequency_individual" => GraphType::FrequencyIndividual,
        "usage" => GraphType::Usage,
        "usage_individual" => GraphType::UsageIndividual,
        "temperature" => GraphType::Temperature,
        "temperature_individual" => GraphType::TemperatureIndividual,
        _ => GraphType::Hidden,
    }
}

#[derive(Clone)]
pub struct Settings {
    pub verbose: bool,
    pub delay: u64,
    pub delay_battery: u64,
    pub edit: bool,
    pub no_animation: bool,
    pub should_graph: GraphType,
    pub commit: bool,
    pub testing: bool,
}
