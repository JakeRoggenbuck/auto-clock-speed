use std::fs::File;
use std::io::Read;
use super::Error;

// https://docs.rs/sys-info/0.7.0/src/sys_info/lib.rs.html#367-406
pub fn get_cpu_freq() -> Result<i32, Error> {
    let mut cpu_info = String::new();
    File::open("/proc/cpuinfo")?.read_to_string(&mut cpu_info)?;

    // Find all lines that begin with cpu MHz
    let find_cpu_mhz = cpu_info.split('\n').find(|line| {
        line.starts_with("cpu MHz\t")
            || line.starts_with("BogoMIPS")
            || line.starts_with("clock\t")
            || line.starts_with("bogomips per cpu")
    });

    // For each line that starts with the clock speed identifier return the number after : as a 32
    // bit integer
    find_cpu_mhz
        .and_then(|line| line.split(':').last())
        .and_then(|val| val.replace("MHz", "").trim().parse::<f64>().ok())
        .map(|speed| speed as i32)
        .ok_or(Error::Unknown)
}
