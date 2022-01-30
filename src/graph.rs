use rasciigraph::{plot, Config};

pub trait Grapher {
    fn update_all(&mut self);
    fn update_one(&self, vec: &mut Vec<f64>);
    fn plot(&self, nums: Vec<f64>);
}

pub struct Graph {
    pub freqs: Vec<f64>,
}

impl Grapher for Graph {
    fn update_all(&mut self) {
        self.update_one(&mut self.freqs.clone());
    }

    fn update_one(&self, vec: &mut Vec<f64>) {
        while vec.len() > 40 {
            vec.remove(0);
        }

        self.plot(vec.clone());
    }

    fn plot(&self, nums: Vec<f64>) {
        println!(
            "\n{}",
            plot(nums, Config::default().with_offset(10).with_height(10))
        );
    }
}
