use super::system::inside_docker;
use super::warn_user;
use wsl;

pub fn setup() {
    if is_wsl() {
        warn_user!("ACS is intended to run on an actual linux distribution, \
        the program will NOT work inside of Windows Subsystem for Linux \
        Please install an actual distribution of Linux");
    }
    if inside_docker() {
        warn_user!("Stats may be incorrect if running inside docker.");
    }
}
