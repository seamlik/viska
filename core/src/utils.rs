//! ‎Miscellaneous utilities that make our lives easier.

use std::error::Error;

pub fn join_strings(strings: impl Iterator<Item = String>) -> String {
    strings.fold("".to_owned(), |acc, x| {
        if acc.is_empty() {
            x
        } else {
            format!("{} {}", acc, x).to_owned()
        }
    })
}

/// The simplest `Result` that supports polymorphism in error handling.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;
