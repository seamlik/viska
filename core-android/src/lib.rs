#![feature(proc_macro_hygiene)]

#[path = "../../target/riko/viska_android.rs"]
#[riko::ignore]
mod bridge_android;

#[path = "../../target/riko/viska.rs"]
#[riko::ignore]
mod bridge_core;

use android_logger::Config;
use log::Level;

/// Initializes the whole library.
///
/// Must be used by a Java client loading this crate.
#[riko::fun]
pub fn initialize() {
    let config = Config::default().with_min_level(Level::max());
    android_logger::init_once(config);
}
