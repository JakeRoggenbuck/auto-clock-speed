use std::mem;
use nix::libc::{c_short, c_ushort, ioctl, STDOUT_FILENO, TIOCGWINSZ};

pub struct TermSize {
    pub row: c_short,
    pub col: c_ushort,
}

pub fn terminal_width() -> usize {
    unsafe {
        let mut size: TermSize = mem::zeroed();
        ioctl(STDOUT_FILENO, TIOCGWINSZ.into(), &mut size as *mut _);
        return size.col as usize;
    }
}
