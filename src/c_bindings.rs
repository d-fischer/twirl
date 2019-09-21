#![allow(non_snake_case)]

use std::ffi::CString;
use ffi_support::FfiStr;
use std::os::raw::c_char;

use crate::{TwitchClient, User};

#[repr(C)]
pub struct CUser {
    id: *mut c_char,
    login: *mut c_char,
}

impl From<User> for CUser {
    fn from(user: User) -> CUser {
        CUser {
            id: CString::new(user.id).expect("Could not create CString").into_raw(),
            login: CString::new(user.login).expect("Could not create CString").into_raw(),
        }
    }
}

lazy_static! {
    static ref RUNTIME: tokio::runtime::Runtime = tokio::runtime::Runtime::new().expect("Could not create runtime for FFI");
}

#[no_mangle]
pub extern fn createTwitchClient(client_id: FfiStr) -> *mut TwitchClient {
    let client_ptr = Box::into_raw(Box::new(TwitchClient::new(client_id.into_string())));
    client_ptr
}

#[no_mangle]
pub extern fn createTwitchClientWithCredentials(client_id: FfiStr, access_token: FfiStr) -> *mut TwitchClient {
    let client_ptr = Box::into_raw(Box::new(TwitchClient::with_credentials(client_id.into_string(), access_token.into_string(), None)));
    client_ptr
}

#[no_mangle]
pub extern fn getMe(client_ptr: *mut TwitchClient) -> *mut CUser {
    let client = unsafe { client_ptr.as_mut().expect("Got NULL ptr") };
    match RUNTIME.block_on(client.get_me()) {
        Ok(me) => Box::into_raw(Box::new(me.into())),
        Err(_) => std::ptr::null::<CUser>() as *mut CUser
    }
}

#[no_mangle]
pub extern fn destroyTwitchClient(client_ptr: *mut TwitchClient) {
    let _client: Box<TwitchClient> = unsafe{ Box::from_raw(client_ptr) };
    // do nothing with it
}