# Auto Clock Speed (acs) ![Rust](https://img.shields.io/github/workflow/status/jakeroggenbuck/auto-clock-speed/Rust?style=for-the-badge)
 A utility to check stats about your CPU, and auto regulate clock speeds to help with either performance or battery life.
 
![image](https://user-images.githubusercontent.com/35516367/132371565-4ff64d03-bff2-4e46-9a73-165e933c460c.png)


## Goals
- First and foremost, this is a project to learn about Rust and Linux
- Secondly, try to improve upon AdnanHodzic's already amazing [auto-cpufreq](https://github.com/AdnanHodzic/auto-cpufreq)
    - Add options to display raw output of governors, clockspeed, turbo, battery, etc. for use in scripts or display panels like polybar.

## Install
If you have cargo on your machine, skip to step 3

1. Install [`rustup.rs`](https://rustup.rs/).

2. Setup compiler
   ```sh
   rustup override set stable
   rustup update stable
   ```

3. Clone the project and install
   ```
   git clone https://github.com/JakeRoggenbuck/auto-clock-speed
   cargo install --path auto-clock-speed
   ```

4. Copy the binary to /bin
   ```
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

### Get
```sh
# Get information about the system

# View all of get's subcommands
acs get --help

acs get temp
acs get freq
acs get cpu-speeds
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
    get        Get a specific value or status
    help       Prints this message or the help of the given subcommand(s)
    monitor    Monitor each cpu, it's min, max, and current speed, along with the governor
    run        Run the daemon, this checks and edit your cpu's speed
```
