//! A mock implementation of git.
//!
//! Used to facilitate testing scenarios where prescribed behavior is required, and would be too
//! cumbersome to obtain from "real git". Should only be used in unit testing; integration tests
//! should still run against an actual git binary.
use std::env;
use std::process::exit;

fn main() {
    let first_arg = env::args().nth(1);
    match first_arg.as_deref() {
        None => exit(1),
        Some("--version") => println!("fake_git version 1"),
        Some("branch") => match env::args().nth(2).as_deref() {
            None => exit(1),
            Some("-d") => match env::args().nth(3).as_deref() {
                None => exit(1),
                Some("already-been-merged") => exit(0),
                Some(_) => exit(1)
            },
            Some("--merged") => println!("* trunk\nalready-been-merged"),
            Some(_) => exit(1)
        }
        Some(_) => exit(1)
    };

    exit(0);
}
