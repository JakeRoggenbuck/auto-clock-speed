# Auto Clock Speed ![Rust](https://img.shields.io/github/workflow/status/jakeroggenbuck/auto-clock-speed/Rust?style=for-the-badge)

## Goals
- First and foremost, this is a project to learn about Rust and Linux
- Secondly, try to improve upon AdnanHodzic's already amazing [auto-cpufreq](https://github.com/AdnanHodzic/auto-cpufreq)
    - Add options to display raw output of governors, clockspeed, turbo, battery, etc. for use in scripts or display panels like polybar.

## Install
```
git clone https://github.com/JakeRoggenbuck/auto-clock-speed
cargo install --path auto-clock-speed
```

## Usage
```
Automatic CPU frequency scaler and power saver

USAGE:
    clockspeed <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    get-freq
    get-governors
    get-turbo
    help             Prints this message or the help of the given subcommand(s)
```
