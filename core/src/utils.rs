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

pub trait ResultOption<T, E> {
    /// Maps on the inner value `x` in `Ok(Some(x))`.
    fn map_deep<U, F: FnOnce(T) -> U>(self, op: F) -> Result<Option<U>, E>;
}

impl<T, E> ResultOption<T, E> for Result<Option<T>, E> {
    fn map_deep<U, F: FnOnce(T) -> U>(self, op: F) -> Result<Option<U>, E> {
        match self {
            Err(e) => Err(e),
            Ok(None) => Ok(None),
            Ok(Some(it)) => Ok(Some(op(it))),
        }
    }
}
