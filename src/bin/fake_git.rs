//! A mock implementation of git.
//!
//! Used to facilitate testing scenarios where prescribed behavior is required, and would be too
//! cumbersome to obtain from "real git". Should only be used in unit testing; integration tests
//! should still run against an actual git binary.
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
