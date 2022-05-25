use std::io;

pub fn interactive() {
    loop {
        let mut input = String::new();

        match io::stdin().read_line(&mut input) {
            Ok(n) => {
                if input == "exit\n" || input == "e\n" {
                    break;
                }

                println!("{n} bytes read");
                println!("{input}");
            }
            Err(error) => println!("error: {error}"),
        }
    }
}
