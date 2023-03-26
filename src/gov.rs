use std::fmt;

/// Governor
///
/// https://www.kernel.org/doc/html/v4.14/admin-guide/pm/cpufreq.html#generic-scaling-governors
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
