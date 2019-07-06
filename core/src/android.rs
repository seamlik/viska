#![cfg(feature = "android")]

use android_logger::Config;
use log::Level;

pub fn initialize() {
    let config = Config::default().with_min_level(Level::max());
    android_logger::init_once(config);
    log::error!("Holy cow!");
}
