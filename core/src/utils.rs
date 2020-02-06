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

pub trait ResultIterator<SrcIter, E, I> {
    fn unpack(self) -> Box<dyn Iterator<Item = Result<I, E>>>;
}

impl<SrcIter, E, I> ResultIterator<SrcIter, E, I> for Result<SrcIter, E>
where
    SrcIter: Iterator<Item = Result<I, E>> + 'static,
    E: 'static,
    I: 'static,
{
    fn unpack(self) -> Box<dyn Iterator<Item = Result<I, E>>> {
        match self {
            Err(err) => Box::new(std::iter::once(Err(err))),
            Ok(iter) => Box::new(iter),
        }
    }
}
