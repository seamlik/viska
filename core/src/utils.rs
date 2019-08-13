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

/// A placeholder type for being returned by a trait method.
///
/// This type shall be removed after `impl Trait` is supported in trait definitions.
pub struct GenericIterator<T>(Box<dyn Iterator<Item = T>>);

impl<T> GenericIterator<T> {
    pub fn new(src: Box<dyn Iterator<Item = T>>) -> Self {
        Self(src)
    }
}

impl<T> Iterator for GenericIterator<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
