pub fn print_freq(f: i32, raw: bool) {
    if raw {
        println!("{}", f);
    } else {
        println!("CPU freq is {} MHz", f)
    }
}

pub fn print_turbo(t: bool, raw: bool) {
    if raw {
        println!("{}", t);
    } else {
        println!(
            "{}",
            if t {
                "Turbo is enabled"
            } else {
                "Turbo is not enabled"
            }
        )
    }
}

fn print_vec(a: Vec<String>, raw: bool) {
    if raw {
        for x in a.iter() {
            println!("{}", x);
        }
    } else {
        println!("{:?}", a);
    }
}

pub fn print_available_governors(a: Vec<String>, raw: bool) {
    print_vec(a, raw);
}

pub fn print_cpus(a: Vec<String>, raw: bool) {
    print_vec(a, raw);
}
