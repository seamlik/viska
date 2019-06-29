#![cfg(target_os="android")]

use android_logger::Config;
use jni::objects::JClass;
use jni::JNIEnv;
use log::Level;

#[no_mangle]
pub unsafe extern "C" fn Java_viska_LibViska_initialize(env: JNIEnv, _: JClass) {
    let config = Config::default().with_min_level(Level::max());
    android_logger::init_once(config);
    log::error!("Holy cow!");
}
