pub mod android;
pub mod database;
pub mod mock_profiles;
pub mod pki;

mod ffi;
mod jni;
mod utils;

use std::error::Error;

/// The simplest `Result` that supports polymorphism in error handling.
pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

