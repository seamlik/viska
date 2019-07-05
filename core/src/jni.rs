#![cfg(feature = "android")]

use jni::objects::JClass;
use jni::objects::JString;
use jni::JNIEnv;
use std::path::Path;

#[cfg(feature = "mock-profiles")]
#[no_mangle]
pub unsafe extern "C" fn Java_viska_mock_1profile_Module_newMockProfile(
    env: JNIEnv,
    _: JClass,
    profile_path_java: JString,
) {
    let profile_path: String = env.get_string(profile_path_java).unwrap().into();
    crate::mock_profiles::new_mock_profile(&Path::new(&profile_path));
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_android_Module_initialize(_: JNIEnv, _: JClass) {
    crate::android::initialize()
}
