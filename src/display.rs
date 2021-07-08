use std::fmt::Display;

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

fn print_vec<T: Display>(t: Vec<T>, raw: bool) {
    if raw {
        for x in t {
            println!("{}", x);
        }
    } else {
        print!("[ ");
        for x in t {
            print!("\"{}\" ", x);
        }
        print!("]");
    }
}

pub fn print_available_governors(a: Vec<String>, raw: bool) {
    print_vec(a, raw);
}

pub fn print_cpus(a: Vec<String>, raw: bool) {
    print_vec(a, raw);
}

pub fn print_cpu_speeds(a: Vec<i32>, raw: bool) {
    print_vec(a, raw);
}
