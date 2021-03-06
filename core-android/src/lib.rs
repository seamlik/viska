#![feature(proc_macro_hygiene)]

#[path = "../../target/riko/viska_android.rs"]
#[riko::ignore]
pub mod bridge;

/// To re-export the symbols in the core crate
pub use viska::bridge as bridge_core;

use android_logger::Config;
use log::Level;
use log::ParseLevelError;
use std::str::FromStr;

/// Initializes the whole library.
///
/// Must be used by a Java client loading this crate.
#[riko::fun]
pub fn initialize(log_level: String) -> Result<(), ParseLevelError> {
    let config = Config::default().with_min_level(Level::from_str(&log_level)?);
    android_logger::init_once(config);
    Ok(())
}
