#![cfg(feature = "android")]

use crate::Client;
use crate::__riko_POOL_Client;
use jni::objects::JClass;
use jni::sys::jbyteArray;
use jni::JNIEnv;
use riko_runtime::heap::Handle;
use riko_runtime::heap::Heaped;
use riko_runtime::heap::Pool;
use riko_runtime::returned::Returned;
use riko_runtime::Marshaled;

#[no_mangle]
pub extern "C" fn Java_viska_core_Client__1_1riko_1drop(
    _env: JNIEnv,
    _class: JClass,
    handle: Handle,
) {
    __riko_POOL_Client.drop(handle);
}

#[no_mangle]
pub extern "C" fn Java_viska_core_Client__1_1riko_1create(
    _env: JNIEnv,
    _class: JClass,
    arg_1_jni: jbyteArray,
) -> jbyteArray {
    let result = Client::new_ffi(&Marshaled::from_jni(&_env, arg_1_jni));
    Marshaled::to_jni(&Heaped::into_handle(result), &_env)
}

#[no_mangle]
pub extern "C" fn Java_viska_core_Client__1_1riko_1account_1id_1display(
    _env: JNIEnv,
    _class: JClass,
    handle: Handle,
) -> jbyteArray {
    let action = |obj: &mut Client| obj.account_id_display();
    let result: Returned<String> = __riko_POOL_Client.peek(handle, action).into();
    Marshaled::to_jni(&result, &_env)
}
