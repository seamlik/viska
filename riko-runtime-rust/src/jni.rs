//! Utilities for interfacing with JNI.
//!
//! These methods should only be used by generated wrapper code.

use jni::JNIEnv;
use std::error::Error;

/// Throws a Java exception based on a Rust `Error`.
pub fn throw(env: &JNIEnv, err: &impl Error) {
    env.throw_new("java/lang/RuntimeException", ToString::to_string(err))
        .expect("Failed to raise an Exception!");
}
