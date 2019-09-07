//! ‎Miscellaneous utilities that make our lives easier.

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
