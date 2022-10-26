use super::system::inside_docker;
use super::warn_user;

pub fn setup() {
    if inside_docker() {
        warn_user!("Stats may be incorrect if running inside docker.");
    }
}
