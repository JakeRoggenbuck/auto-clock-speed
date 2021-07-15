# Auto Clock Speed (acs) ![Rust](https://img.shields.io/github/workflow/status/jakeroggenbuck/auto-clock-speed/Rust?style=for-the-badge)
 A utility to check stats about your CPU, and auto regulate clock speeds to help with either performance or battery life.
 
![image](https://user-images.githubusercontent.com/35516367/125746973-e8537607-f265-414f-be37-479d68ac1f7d.png)


## Goals
- First and foremost, this is a project to learn about Rust and Linux
- Secondly, try to improve upon AdnanHodzic's already amazing [auto-cpufreq](https://github.com/AdnanHodzic/auto-cpufreq)
    - Add options to display raw output of governors, clockspeed, turbo, battery, etc. for use in scripts or display panels like polybar.

## Install
```
git clone https://github.com/JakeRoggenbuck/auto-clock-speed
cargo install --path auto-clock-speed

# Move the excecutable so sudo can access it
sudo cp ~/.cargo/bin/acs /bin/acs
```

## Usage
### Monitor
```sh
# Show the min, max, and current cpu frequency along with the cpu governor
acs monitor

# A delay (in milliseconds) can be set for both monitor and run
acs monitor --delay 1000
```

### Run
```sh
# Run requires sudo because it edits the cpu's frequency

# Edit speeds and shows exactly what monitor does
sudo acs run

# Shows no output but still edits speeds
sudo acs run --quiet
```

## Help
```
Automatic CPU frequency scaler and power saver

USAGE:
    acs <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

SUBCOMMANDS:
    get-available-governors    Get the available governor
    get-cpu-governors          The governors of the individual cores
    get-cpu-speeds             The speed of the individual cores
    get-cpus                   The names of the core
    get-freq                   The overall frequency of your cpu
    get-turbo                  Get whether turbo is enabled or not
    help                       Prints this message or the help of the given subcommand(s)
    monitor                    Monitor each cpu, it's min, max, and current speed, along with the governor
    run                        Run the daemon, this checks and edit your cpu's speed
```
