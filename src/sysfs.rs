use crate::Error;
use std::fs;
use std::path::{Path,PathBuf};
use std::str::FromStr;
use globset::Glob;

pub fn read<T>(val: &mut T, path: &Path) -> Result<(), Error>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: std::fmt::Debug,
{
    let mut file_content = fs::read_to_string(path)?;
    // Remove the \n char
    file_content.pop();

    // Convert String to expected value
    *val = file_content.parse::<T>().unwrap();

    Ok(())
}

// Lookup a Hdw based on its Parent Path & a glob
pub fn get_path_by_glob(sysfs_parent_path: &str, hdw_glob: &str) -> Result<PathBuf, Error> {
    let mut glob_path = sysfs_parent_path.to_string();
    glob_path.push_str(hdw_glob);

    let glob = Glob::new(&glob_path).unwrap().compile_matcher();
    let entries = fs::read_dir(sysfs_parent_path).unwrap();

    for entry in entries {
        let entry = entry.unwrap();
        let pathbuf = entry.path();
        if glob.is_match(&pathbuf) {
            return Ok(pathbuf);
        }
    }
    Err(Error::HdwNotFound)
}
