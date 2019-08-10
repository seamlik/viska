#![cfg(feature = "android")]

use crate::Client;
use crate::HEAP;
use jni::objects::JClass;
use jni::objects::JString;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use riko_runtime::heap::Handle;
use riko_runtime::HeapObject;
use riko_runtime::MarshaledAsBytes;
use riko_runtime::MarshaledAsString;

#[cfg(feature = "mock_profiles")]
#[no_mangle]
pub unsafe extern "C" fn Java_viska_core__1_1Riko_1Module__1_1riko_1new_1mock_1profile(
    _env: JNIEnv,
    _class: JClass,
    arg_1_jni: JString,
) {
    let arg_1_rust = MarshaledAsString::from_jni(&_env, arg_1_jni);

    crate::mock_profiles::new_mock_profile(&arg_1_rust)
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_core__1_1Riko_1Module__1_1riko_1initialize(
    _env: JNIEnv,
    _class: JClass,
) {
    crate::android::initialize()
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_core_Client__1_1riko_1drop(
    _env: JNIEnv,
    _class: JClass,
    handle: Handle,
) {
    ::riko_runtime::heap::drop::<Client>(&HEAP, &handle);
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_core_Client__1_1riko_1new(
    _env: JNIEnv,
    _class: JClass,
    arg_1_jni: JString,
) -> Handle {
    let arg_1_rust = MarshaledAsString::from_jni(&_env, arg_1_jni);

    let result = Client::new_ffi(&arg_1_rust);
    HeapObject::into_handle_jni(result, &_env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_core_Client__1_1riko_1account_1id(
    _env: JNIEnv,
    _class: JClass,
    handle: Handle,
) -> jbyteArray {
    let action = |obj: &mut Client| obj.account_id();

    let result = ::riko_runtime::heap::peek(&HEAP, &handle, action);
    MarshaledAsBytes::to_jni(&result, &_env)
}
