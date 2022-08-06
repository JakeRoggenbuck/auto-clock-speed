use nix::libc::{c_short, c_ushort, ioctl, STDOUT_FILENO, TIOCGWINSZ};
use std::mem;

pub struct TermSize {
    pub row: c_short,
    pub col: c_ushort,
}

pub fn terminal_width() -> usize {
    unsafe {
        let mut size: TermSize = mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ, &mut size as *mut _);
        size.col as usize
    }
}
