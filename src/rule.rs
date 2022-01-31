pub trait Actions {
    fn run(&self);
    fn check(&self);
}

pub struct Rule {
    pub name: String,
    pub docs: String,
}

impl Actions for Rule {
    fn run(&self) {
        panic!("Not implemented");
    }
    fn check(&self) {
        panic!("Not implemented");
    }
}
