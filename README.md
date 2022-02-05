# Auto Clock Speed (acs) ![Rust](https://img.shields.io/github/workflow/status/jakeroggenbuck/auto-clock-speed/Rust?style=for-the-badge)
A utility to check stats about your CPU, and auto regulate clock speeds to help with either performance or battery life.
 
![image](https://user-images.githubusercontent.com/35516367/151893537-1ed4241d-9e3c-4e02-a620-568820ce13d0.png)

## Goals
- First and foremost, this is a project to learn about Rust and Linux
- Secondly, try to improve upon AdnanHodzic's already amazing [auto-cpufreq](https://github.com/AdnanHodzic/auto-cpufreq)
    - Add options to display raw output of governors, clockspeed, turbo, battery, etc. for use in scripts or display panels like polybar.

## Install Latest Release
If you have cargo on your machine, skip to step 3

1. Install [`rustup.rs`](https://rustup.rs/).

2. Setup rust
   ```sh
   rustup override set stable
   rustup update stable
   ```

3. Install from crates
   ```
   cargo install autoclockspeed
   ```

## Install from github
Do steps 1 and 2 from other install if you don't have rust installed, then do this next step.

3. Clone the project and install

   ```
   git clone https://github.com/JakeRoggenbuck/auto-clock-speed
   ```
   ```
   cargo install --path auto-clock-speed
   ```

## In Action
[![image](https://user-images.githubusercontent.com/35516367/151716685-a3ed3c53-07f4-459f-a3ae-e1de1ba16429.png)](https://www.youtube.com/watch?v=T9nN_rQOYsg)

## Systemd
In order to have auto-clock-speed start when you restart your computer you must follow these instruction
```sh
# IMPORTANT: Modify the service file (acs.service) in the
# project directory to include the path to the binary file 
# (usually /home/username/.cargo/bin/acs)
```

```sh
# In the auto clock speed directory run this command to
# move the service file into your systemd directory
sudo cp acs.service /etc/systemd/system/
```

```sh
# Start and enable the service
sudo systemctl start acs
sudo systemctl enable acs

# Check service is up and running
systemctl status acs
```
## Systemctl command
The line after `[Service]` in `acs.service` is the command that will be run. You may want to add or remove arguments, mainly `--quiet`.
```
[Unit]
Description=Manages Clock Speed

[Service]
ExecStart=/home/your-user-here/.cargo/bin/acs run --no-animation

[Install]
WantedBy=multi-user.target
```

# Config

### Using default config
```sh
WARN: Using default config. Create file ~/.config/acs/acs.toml for custom config.
```
This warning recommends creating a config file, use the following example and install at `~/.config/acs/acs.toml`

```sh
mkdir -p ~/.config/acs
cp ./acs.toml ~/.config/acs/acs.toml
```

### This is an example config
also the default settings if not config is provided

```yaml
# acs.toml
powersave_under = 20
```

## Turn Off
```sh
# Temporarily stop (only lasts until reboot)
sudo systemctl stop acs

# Perminatly stop until turned on
sudo systemctl disable acs
```

## Uninstall
```sh
# Remove binary
cargo uninstall autoclockspeed

# Remove systemctl entry
rm /etc/systemd/system/acs.service
```

# Example Usage
```sh
# Monitor mode
acs monitor

# Run as root
sudo acs run

# Get all speeds
acs get speeds

# Select gov from dmenu
sudo acs set gov $(acs get available-govs --raw | dmenu)
```

## Help
```sh
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
    set
```
