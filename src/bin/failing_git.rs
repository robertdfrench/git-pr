//! A program that always returns an error.
//!
//! Used to facilitate testing scenarios where git should immediately fail.
use std::process::exit;

fn main() {
    exit(1)
}
