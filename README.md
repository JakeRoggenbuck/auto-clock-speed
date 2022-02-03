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

# Detailed Usage
## Monitor
Monitor will not edit anything
```sh
# Show the min, max, and current cpu frequency
# along with the cpu governor
acs monitor

# A delay (in milliseconds) can be set for both monitor and run
acs monitor --delay 1000
```

<br>

## Run
Run will edit governor
```sh
# Run requires sudo because it edits the gov

# Edit speeds and shows exactly what monitor does
sudo acs run

# Shows no output but still edits speeds
sudo acs run --quiet
```

<br>

## Get

### Flags
`--raw` is the only used flag for the `get` command.

### Subcommands

### available-govs
<details><summary>more (click to expand)</summary>
<p>

Normal
```sh
~ λ acs get available-govs
performance powersave
```

Raw
```sh
~ λ acs get available-govs --raw
performance
powersave
```

</p>
</details>

### cpus
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get cpus
Name: Intel(R) Core(TM) i5-7300U CPU @ 2.60GHz
cpu0 is currently @ 589 MHz
cpu1 is currently @ 629 MHz
cpu2 is currently @ 594 MHz
cpu3 is currently @ 649 MHz
```

Raw
```sh
~ λ acs get cpus --raw
cpu0 628003
cpu1 601547
cpu2 590444
cpu3 627150
```

</p>
</details>

### freq
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get freq
CPU freq is 597 MHz
```

Raw
```sh
~ λ acs get freq --raw
597471
```

</p>
</details>

### govs
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get govs
powersave powersave powersave powersave
```

Raw
```sh
~ λ acs get govs --raw
powersave
powersave
powersave
powersave
```

</p>
</details>

### power
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get power
Lid: open Battery: 0 Plugged: false
```

Raw
```sh
~ λ acs get power --raw
open 0 false
```

</p>
</details>

### speeds
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get speeds
578444 578308 572217 579259
```

Raw
```sh
~ λ acs get speeds --raw
572773
580328
566880
579120
```

</p>
</details>

### temp
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get temp
25000 31050 20000 29050
```

Raw
```sh
~ λ acs get temp --raw
25000
32050
20000
29050
```

</p>
</details>

### turbo
<details><summary>more</summary>
<p>

Normal
```sh
~ λ acs get turbo
Turbo is enabled
```

Raw
```sh
~ λ acs get turbo --raw
true
```

</p>
</details>

<br>

## Set

### Perms
Note that all of the set commands require sudo.

### Subcommand

### available-govs
<details><summary>more (click to expand)</summary>
<p>

Normal use
```sh
sudo acs set gov performance
sudo acs set gov powersave
```

Fancy set script
```sh
sudo acs set gov $(acs get available-govs --raw | dmenu)
```

</p>
</details>

<br>

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
