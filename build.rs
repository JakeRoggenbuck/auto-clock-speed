extern crate cmake;
use std::process::Command;
use cmake::Config;

fn main() {
    let output = Command::new("git")
        .args(&["rev-parse", "HEAD"])
        .output()
        .unwrap();

    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash);

    println!("cargo:rustc-link-search=all=libmsrtools");
    println!("cargo:rustc-link-lib=dylib=msrr.o");
}
