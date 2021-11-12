//! A mock implementation of git.
//!
//! Used to facilitate testing scenarios where prescribed behavior is required, and would be too
//! cumbersome to obtain from "real git". Should only be used in unit testing; integration tests
//! should still run against an actual git binary.
use std::process::exit;


// Grab a command line argument by its index
//
// ## Example: `./fake_git a --b sea`
// * argv!(1) => "a"
// * argv!(2) => "--b"
// * argv!(3) => "sea"
macro_rules! argv {
    ($n:expr) => {
        std::env::args().nth($n).as_deref()
    };
}


fn main() {
    match argv!(1) {

        // No input given
        None => exit(1),

        Some("-C") => match argv!(2) {
            None => exit(1),
            Some(_) => match argv!(3) {
                None => exit(1),

                // git --version
                Some("--version") => println!("fake_git version 1"),

                // unrecognized input
                Some(_) => exit(1)
            }
        },

        // git checkout -b <anything>
        Some("checkout") => match argv!(2) {
            None => exit(1),
            Some("-b") => match argv!(3) {
                None => exit(1),
                Some(_) => exit(0) // Any argument will do, return 0
            },
            Some(_) => exit(1)
        },

        // git push -u origin <anything>
        Some("push") => match argv!(2) {
            None => exit(1),
            Some("-u") => match argv!(3) {
                None => exit(1),
                Some("origin") => match argv!(4) {
                    None => exit(1),
                    Some(_) => exit(0) // Any argument will do, return 0
                },
                Some(_) => exit(1)
            },
            Some(_) => exit(1)
        },

        // git rev-parse --short HEAD
        Some("rev-parse") => match argv!(2) {
            None => exit(1),
            Some("--short") => match argv!(3) {
                None => exit(1),
                Some("HEAD") => println!("1234567"),
                Some(_) => exit(1)
            },
            Some(_) => exit(1)
        },

        Some("branch") => match argv!(2) {
            None => exit(1),
            Some("-d") => match argv!(3) {
                None => exit(1),
                Some("already-been-merged") => exit(0),
                Some(_) => exit(1)
            },
            Some("--merged") => println!("* trunk\nalready-been-merged"),
            Some(_) => exit(1)
        }
        // unrecognized input
        Some(_) => exit(1)
    };

    exit(0);
}
