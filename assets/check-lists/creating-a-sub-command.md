## Creating a sub command

- [ ] Create a new function in another file, usually system.rs, power.rs, or thermal.rs
- [ ] Possibly add unit tests for this new function, especially if parsing
- [ ] Add a function to print it with `print_{thing}()` as the function name...
```rs
fn print_{thing}(raw: bool) {
    // ...
}
```
- [ ] Add it as a method to either the Getter or Setter trait in interface.rs (Should be very short)
```rs
fn speeds(&self, raw: bool) {
    let speeds = list_cpu_speeds();
    print_cpu_speeds(speeds, raw);
}
```
- [ ] Add it as an option in structopt in main.rs
```rs
/// The speed of the individual cores
#[structopt(name = "speeds")]
Speeds {
    #[structopt(short, long)]
    raw: bool,
},
```
- [ ] Add a call to the interface from the parse of the structopt like so...
```rs
GetType::Speeds { raw } => {
    int.get.speeds(raw);
}
```
- [ ] Add a match for it in the interactive mode
```rs
"get speeds" => int.get.speeds(false),
```
- [ ] Add it as a help option in the help function in interactive.rs
```rs
pub fn help() {
    const HELP_TEXT: &str = "\
    - get
      - freq
      - cpus
      etc.
    ";
}
```
