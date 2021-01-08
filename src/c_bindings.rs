#![allow(non_snake_case)]

use std::ffi::{CString, CStr};
use ffi_support::FfiStr;
use std::os::raw::c_char;

use crate::User;
use crate::api::ApiClient;
use crate::auth::StaticAuthProvider;
use crate::auth::poly::{CAuthProvider, OwnedAuthProvider};

#[derive(Debug)]
#[repr(C)]
pub struct CUser {
    id: *mut c_char,
    login: *mut c_char,
}

impl CUser {
    pub unsafe fn id(&self) -> &str {
        CStr::from_ptr(self.id).to_str().unwrap()
    }

    pub unsafe fn login(&self) -> &str {
        CStr::from_ptr(self.login).to_str().unwrap()
    }
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
pub unsafe extern "C" fn createStaticAuthProvider(client_id: FfiStr, access_token: FfiStr) -> *mut CAuthProvider {
    let provider_ptr = CAuthProvider::for_auth_provider(StaticAuthProvider::new(client_id.into_string(), access_token.into_string()));
    provider_ptr
}

#[no_mangle]
pub unsafe extern fn createApiClient(provider_ptr: *mut CAuthProvider) -> *mut ApiClient {
    let client_ptr = Box::into_raw(Box::new(ApiClient::new(Box::new(OwnedAuthProvider::from_raw(provider_ptr)))));
    client_ptr
}

#[no_mangle]
pub extern fn getMe(client_ptr: *mut ApiClient) -> *mut CUser {
    let client = unsafe { client_ptr.as_mut().expect("Got NULL ptr") };
    match RUNTIME.block_on(client.get_me()) {
        Ok(me) => Box::into_raw(Box::new(me.into())),
        Err(_) => {
            std::ptr::null::<CUser>() as *mut CUser
        }
    }
}

#[no_mangle]
pub extern fn destroyApiClient(client_ptr: *mut ApiClient) {
    let _client: Box<ApiClient> = unsafe { Box::from_raw(client_ptr) };
    // do nothing with it
}