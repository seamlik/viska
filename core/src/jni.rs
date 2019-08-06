#![cfg(feature = "android")]

use crate::Client;
use jni::objects::JClass;
use jni::objects::JString;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use riko_runtime::Handle;
use riko_runtime::HeapObject;
use riko_runtime::MarshaledAsByteArray;
use std::path::Path;
use std::path::PathBuf;

#[cfg(feature = "mock_profiles")]
#[no_mangle]
pub unsafe extern "C" fn Java_viska_mock_1profile_Module_Rust_1new_1mock_1profile(
    env: JNIEnv,
    _: JClass,
    profile_path_java: JString,
) {
    let profile_path: String = env.get_string(profile_path_java).unwrap().into();
    crate::mock_profiles::new_mock_profile(&Path::new(&profile_path));
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_android_Module_Rust_1initialize(_: JNIEnv, _: JClass) {
    crate::android::initialize()
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_Client_Rust_1drop(_: JNIEnv, _: JClass, handle: Handle) {
    riko_runtime::heap::drop::<crate::Client>(&crate::HEAP, &handle);
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_Client_Rust_1new(
    env: JNIEnv,
    class: JClass,
    profile_path: JString,
) -> Handle {
    let profile_path_rust: String = env.get_string(profile_path).unwrap().into();
    let result = crate::Client::new(PathBuf::from(profile_path_rust));
    HeapObject::into_handle_jni(result, &env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_Client_Rust_1account_1id(
    env: JNIEnv,
    class: JClass,
    handle: Handle,
) -> jbyteArray {
    let action = |obj: &mut Client| obj.account_id();
    let result = riko_runtime::heap::peek(&crate::HEAP, &handle, action);
    MarshaledAsByteArray::to_jni(&result, &env)
}
