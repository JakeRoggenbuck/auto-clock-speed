use rasciigraph::{plot, Config};
use std::fmt;

pub trait Grapher {
    fn update_all(&mut self);
    fn update_one(&self, vec: &mut Vec<f64>) -> String;
    fn clear_before(&self, vec: &mut Vec<f64>);
    fn plot(&self, nums: Vec<f64>) -> String;
}

pub struct Graph {
    pub vals: Vec<f64>,
}

impl Grapher for Graph {
    fn update_all(&mut self) {
        self.update_one(&mut self.vals.clone());
    }

    fn update_one(&self, vec: &mut Vec<f64>) -> String {
        self.clear_before(vec);
        self.plot(vec.clone())
    }

    fn clear_before(&self, vec: &mut Vec<f64>) {
        while vec.len() > 40 {
            vec.remove(0);
        }
    }

    fn plot(&self, nums: Vec<f64>) -> String {
        format!(
            "\n{}",
            plot(nums, Config::default().with_offset(10).with_height(10))
        )
    }
}

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
