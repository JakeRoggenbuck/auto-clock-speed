use super::system::get_some_cpu_int_by_path;

trait Speed {
    fn set_max(&mut self, max: i32);
    fn set_min(&mut self, min: i32);
    fn get_max(&mut self);
    fn get_min(&mut self);
    fn get_cur(&mut self);
}

struct CPU {
    name: String,
    max_freq: i32,
    min_freq: i32,
    cur_freq: i32,
    base_freq: i32,
    turbo: bool,
}

impl Speed for CPU {
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
