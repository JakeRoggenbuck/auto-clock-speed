use std::fs;
use std::path::Path;

use super::debug;

/// Return true if local path exists
/// path_name -> check_for_local -> ~/path_name
pub fn check_for_local(path_name: &str) -> bool {
    match home::home_dir() {
        Some(path) => {
            let local_path =
                Path::new(format!("{}/{}/", path.display(), &path_name).as_str()).to_owned();

            if local_path.exists() {
                return true;
            } else {
                debug!("Could not find ~/{}", path_name);
            }
        }
        None => eprintln!("Could not get home directory"),
    }

    return false;
}

/// Create local directory(s)
pub fn create_local(path_name: &str) {
    match home::home_dir() {
        Some(path) => {
            let local_path =
                Path::new(&format!("{}/{}", path.display(), &path_name).as_str()).to_owned();

            // Create each directory, e.g.
            // test/project -> create 'test' and 'test/project'
            match fs::create_dir_all(local_path.clone()) {
                Ok(_) => debug!("Created {:?}", local_path),
                Err(e) => eprintln!("{}", e),
            }
        }
        None => eprintln!("Could not get home directory"),
    }
}

/// Return the local config path
pub fn config_path() -> String {
    match home::home_dir() {
        Some(path) => {
            format!("{}/{}", path.display(), ".config/acs/acs.toml")
        }
        None => {
            eprintln!("Could not get home directory");
            String::new()
        }
    }
}

/// Check if the local config file exists
/// ~/.config/acs/acs.toml
pub fn local_config_file_exists() -> bool {
    Path::new(&config_path()).exists()
}

/// Check if the local config directory exists
/// ~/.config/acs/
pub fn local_config_dir_exists() -> bool {
    check_for_local(".config/acs/")
}

/// Create the local config directory
/// .config/acs/
pub fn create_local_config_dir() {
    create_local(".config/acs/")
}
