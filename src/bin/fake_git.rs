use std::env;
use std::process::exit;

fn main() {
    let first_arg = env::args().nth(1);
    match first_arg {
        None => exit(1),
        Some(val) => {
            if val == "--version" {
                println!("fake_git version 1");
            } else {
                exit(1);
            }
        }
    };

    exit(0);
}
