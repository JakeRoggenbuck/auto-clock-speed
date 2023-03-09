use std::fmt;

pub enum Gov {
    Powersave,
    Performance,
    Schedutil,
}

impl fmt::Display for Gov {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match &self {
            Gov::Powersave => "powersave",
            Gov::Performance => "performance",
            Gov::Schedutil => "schedutil",
        };

        write!(f, "{}", name)
    }
}
