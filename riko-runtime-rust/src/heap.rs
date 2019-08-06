//! Operations for handling heap-allocated objects.
//!
//! These methods should only be used by generated wrapper code.

use crate::Handle;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;

/// Thread-safe and type-safe collection of `HeapObject`s.
pub type Heap<T> = RwLock<HashMap<Handle, Arc<Mutex<T>>>>;

/// Applies a closure on a `HeapObject`.
pub fn peek<T, R>(heap: &Heap<T>, handle: &Handle, action: impl FnOnce(&mut T) -> R) -> R {
    let heap_guard = heap.read().expect("Failed to read-lock the heap!");
    let raw = heap_guard.get(handle).expect("Invalid handle!").clone();
    std::mem::drop(heap_guard);

    let mut obj = raw.lock().expect("Failed to lock the object!");
    action(&mut *obj)
}

/// Drops the object pointed by the `handle`.
pub fn drop<T>(heap: &Heap<T>, handle: &Handle) {
    heap.write()
        .expect("Failed to write-lock the heap!")
        .remove(handle);
}
