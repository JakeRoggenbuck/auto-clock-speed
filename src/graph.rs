#![forbid(unsafe_code)]
use rasciigraph::{plot, Config};
use std::collections::VecDeque;
use std::default::Default;
use std::fmt;

pub trait Grapher {
    fn update_all(&mut self);
    fn update_one(&self, vec: &mut VecDeque<f64>) -> String;
    fn clear_before(&self, vec: &mut VecDeque<f64>);
    fn plot(&self, nums: VecDeque<f64>) -> String;
    fn new() -> Self;
}

pub struct Graph {
    /// The values that get graphed
    pub vals: VecDeque<f64>,
    max: usize,
}

impl Grapher for Graph {
    fn update_all(&mut self) {
        self.update_one(&mut self.vals.clone());
    }

    fn update_one(&self, vec: &mut VecDeque<f64>) -> String {
        self.clear_before(vec);
        self.plot(vec.clone())
    }

    fn clear_before(&self, vec: &mut VecDeque<f64>) {
        while vec.len() > self.max {
            vec.pop_front();
        }
    }

    fn plot(&self, nums: VecDeque<f64>) -> String {
        format!(
            "\n{}",
            plot(
                nums.into(),
                Config::default().with_offset(10).with_height(10)
            )
        )
    }

    fn new() -> Self {
        Graph {
            vals: VecDeque::<f64>::new(),
            max: 40,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum GraphType {
    Hidden,
    Frequency,
    Usage,
    Temperature,
    Unknown,
}

impl Default for GraphType {
    fn default() -> GraphType {
        GraphType::Hidden
    }
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
