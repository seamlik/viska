//! ‎Miscellaneous utilities that make our lives easier.

use data_encoding::HEXUPPER_PERMISSIVE;
use std::error::Error;

/// Join multiple `String`s with a space as the delimiter.
pub fn join_strings(strings: impl Iterator<Item = String>) -> String {
    let mut result = String::default();
    for it in strings {
        if !result.is_empty() {
            result.push(' ');
        }
        result.push_str(&it)
    }

    result
}

/// The simplest `Result` that supports polymorphism in error handling.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

pub fn display_id(id: &[u8]) -> String {
    HEXUPPER_PERMISSIVE.encode(id)
}
