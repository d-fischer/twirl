use std::ptr::NonNull;
use crate::auth::{AuthProvider, AccessToken};
use crate::util::Result;
use futures::future::BoxFuture;
use crate::auth::provider::poly::CAuthProvider;

#[repr(transparent)]
pub struct OwnedAuthProvider(NonNull<CAuthProvider>);

impl OwnedAuthProvider {
    pub fn new<A>(provider: A) -> Self
        where A: AuthProvider + Send + Sync + 'static {
        unsafe {
            let provider_ptr = CAuthProvider::for_auth_provider(provider);
            assert!(!provider_ptr.is_null());
            OwnedAuthProvider::from_raw(provider_ptr)
        }
    }

    pub unsafe fn from_raw(provider_ptr: *mut CAuthProvider) -> Self {
        debug_assert!(!provider_ptr.is_null());
        OwnedAuthProvider(NonNull::new_unchecked(provider_ptr))
    }

    pub fn into_raw(self) -> *mut CAuthProvider {
        let ptr = self.0.as_ptr();
        std::mem::forget(self);
        ptr
    }
}

impl AuthProvider for OwnedAuthProvider {
    fn client_id(&self) -> &str {
        unsafe {
            let ptr = self.0.as_ptr();
            let CAuthProvider { client_id, .. } = *ptr;
            (client_id)(ptr)
        }
    }

    fn current_scopes(&self) -> &[String] {
        unsafe {
            let ptr = self.0.as_ptr();
            let CAuthProvider { current_scopes, .. } = *ptr;
            (current_scopes)(ptr)
        }
    }

    fn access_token(&mut self) -> BoxFuture<Result<AccessToken>> {
        unsafe {
            let ptr = self.0.as_ptr();
            let CAuthProvider { access_token, .. } = *ptr;
            (access_token)(ptr)
        }
    }

    fn access_token_with_scopes(&'a mut self, scopes: Vec<&'a str>) -> BoxFuture<'a, Result<AccessToken>> {
        unsafe {
            let ptr = self.0.as_ptr();
            let CAuthProvider { access_token_with_scopes, .. } = *ptr;
            (access_token_with_scopes)(ptr, scopes)
        }
    }

    fn set_access_token(&mut self, token: AccessToken) {
        unsafe {
            let ptr = self.0.as_ptr();
            let CAuthProvider { set_access_token, .. } = *ptr;
            (set_access_token)(ptr, token)
        }
    }
}

unsafe impl Send for OwnedAuthProvider {}
unsafe impl Sync for OwnedAuthProvider {}