use std::any::Any;
use std::fs;
use std::path::{Path,PathBuf};
use globset::Glob;

use super::create_issue;
use super::Error;

pub mod battery;
pub mod lid;

const POWER_SOURCE_PATH: [&str; 4] = [
    "/sys/class/power_supply/AC/online",
    "/sys/class/power_supply/AC0/online",
    "/sys/class/power_supply/AC1/online",
    "/sys/class/power_supply/ACAD/online",
];

// Lookup a Hdw based on its Parent Path & a glob
pub fn get_sysfs_path_by_glob(sysfs_parent_path: &str, hdw_glob: &str) -> Result<PathBuf, Error> {
    let mut glob_path = sysfs_parent_path.to_string();
    glob_path.push_str(hdw_glob);

    let glob = Glob::new(&glob_path).unwrap().compile_matcher();
    let entries = fs::read_dir(sysfs_parent_path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let pathbuf = entry.path();
        if glob.is_match(&pathbuf) {
            return Ok(pathbuf)
        }
    }
    Err(Error::HdwNotFound)
}

pub fn get_best_path(paths: [&'static str; 4]) -> Result<&str, Error> {
    for path in paths.iter() {
        if Path::new(path).exists() {
            return Ok(path);
        }
    }

    Err(Error::Unknown)
}

pub fn read_power_source() -> Result<bool, Error> {
    let path: &str = match get_best_path(POWER_SOURCE_PATH) {
        Ok(path) => path,
        Err(error) => {
            if error.type_id() == Error::IO.type_id() {
                // Make sure to return IO error if one occurs
                return Err(error);
            }
            eprintln!("We could not detect your AC power source.");
            create_issue!("If you have a power source");
            return Ok(true);
        }
    };

    let mut pwr_str = fs::read_to_string(path)?;

    // Remove the \n char
    pwr_str.pop();

    Ok(pwr_str == "1")
}
