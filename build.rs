#![allow(clippy::uninlined_format_args)]

use std::process::Command;

fn main() {
    // Get commit hash of last commit
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .output()
        .unwrap();

    let git_hash = String::from_utf8(output.stdout).unwrap();
    // Set commit hash to env to read later with --commit option
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
}
