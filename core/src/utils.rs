//! ‎Miscellaneous utilities that make our lives easier.

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
