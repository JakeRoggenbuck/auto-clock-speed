#![forbid(unsafe_code)]
use super::config::config_dir_exists;
use super::system::{inside_docker, inside_wsl};
use super::warn_user;
use std::{thread, time};

pub fn inside_wsl_message() -> String {
    String::from(
        "ACS is intended to run on an actual linux distribution, \
        the program will NOT work inside of Windows Subsystem for Linux, \
        please install an actual distribution of Linux.",
    )
}

pub fn inside_docker_message() -> String {
    String::from("Stats may be incorrect if running inside docker.")
}

pub fn setup() {
    if inside_wsl() {
        warn_user!(inside_wsl_message());
    }
    if inside_docker() {
        warn_user!(inside_docker_message());
    }
}

pub fn check_config_dir_exists() {
    if !config_dir_exists() {
        warn_user!("Config directory '/etc/acs' does not exist!");
        thread::sleep(time::Duration::from_millis(5000));
    }
}
