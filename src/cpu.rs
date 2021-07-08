use super::system::get_some_cpu_int_by_path;

pub trait Speed {
    fn update(&mut self);
    fn init_cpu(&mut self);
    fn set_max(&mut self, max: i32);
    fn set_min(&mut self, min: i32);
    fn get_max(&mut self);
    fn get_min(&mut self);
    fn get_cur(&mut self);
}

#[derive(Debug)]
pub struct CPU {
    pub name: String,
    pub max_freq: i32,
    pub min_freq: i32,
    pub cur_freq: i32,
}

impl Speed for CPU {
    fn update(&mut self) {
        self.get_max();
        self.get_min();
        self.get_cur();
    }

    fn init_cpu(&mut self) {
        self.update();
    }

    fn set_max(&mut self, max: i32) {
        // TODO: change the file with the speed
        self.max_freq = max;
    }

    fn set_min(&mut self, min: i32) {
        // TODO: change the file with the speed
        self.min_freq = min;
    }

    fn get_max(&mut self) {
        match get_some_cpu_int_by_path(self.name.clone(), "cpufreq/scaling_max_freq".to_string()) {
            Ok(a) => {
                self.max_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }

    fn get_min(&mut self) {
        match get_some_cpu_int_by_path(self.name.clone(), "cpufreq/scaling_min_freq".to_string()) {
            Ok(a) => {
                self.min_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }

    fn get_cur(&mut self) {
        match get_some_cpu_int_by_path(self.name.clone(), "cpufreq/scaling_cur_freq".to_string()) {
            Ok(a) => {
                self.cur_freq = a;
            }
            Err(_) => eprint!("Failed"),
        }
    }
}
