#![cfg(feature = "android")]

use android_logger::Config;
use jni::objects::JClass;
use jni::JNIEnv;
use log::Level;

#[no_mangle]
pub unsafe extern "C" fn Java_viska_LibViska_initialize(_: JNIEnv, _: JClass) {
    let config = Config::default().with_min_level(Level::max());
    android_logger::init_once(config);
    error!("Holy cow!");
}
