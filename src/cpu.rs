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
