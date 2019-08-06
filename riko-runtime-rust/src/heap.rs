//! Operations for handling heap-allocated objects.
//!
//! These methods should only be used by generated wrapper code.

use crate::Handle;

/// Returns the object back to the heap and destroys the `Box`.
///
/// Must be used after an FFI method finishes using the object.
pub fn shelf<T>(obj: Box<T>) {
    Box::into_raw(obj);
}

/// Dereferences a `Handle`.
pub unsafe fn deref<T>(handle: Handle) -> Box<T> {
    Box::from_raw(handle as *mut T)
}

/// Drops the object pointed by the `handle`.
pub unsafe fn drop<T>(handle: Handle) {
    deref::<T>(handle);
}
