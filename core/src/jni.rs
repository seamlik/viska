#![cfg(feature = "android")]

use crate::Client;
use crate::__RIKO_POOL_Client;
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use riko_runtime::heap::Handle;
use riko_runtime::returned::Returned;
use riko_runtime::Heap;
use riko_runtime::Marshaled;

#[cfg(feature = "mock_profiles")]
#[no_mangle]
pub unsafe extern "C" fn Java_viska_core__1_1Riko_1Module__1_1riko_1new_1mock_1profile(
    _env: JNIEnv,
    _class: JClass,
    arg_1_jni: jbyteArray,
) {
    crate::mock_profiles::new_mock_profile(&Marshaled::from_jni(&_env, arg_1_jni))
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
    ::riko_runtime::heap::drop::<Client>(&__RIKO_POOL_Client, &handle);
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_core_Client__1_1riko_1create(
    _env: JNIEnv,
    _class: JClass,
    arg_1_jni: jbyteArray,
) -> jbyteArray {
    let result = Client::new_ffi(&Marshaled::from_jni(&_env, arg_1_jni));
    Marshaled::to_jni(&Heap::into_handle(result), &_env)
}

#[no_mangle]
pub unsafe extern "C" fn Java_viska_core_Client__1_1riko_1account_1id(
    _env: JNIEnv,
    _class: JClass,
    handle: Handle,
) -> jbyteArray {
    let action = |obj: &mut Client| obj.account_id();
    let result: Returned<Vec<u8>> =
        ::riko_runtime::heap::peek(&__RIKO_POOL_Client, &handle, action).into();
    Marshaled::to_jni(&result, &_env)
}
