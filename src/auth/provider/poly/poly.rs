use std::any::TypeId;

use crate::util::Result;
use futures::future::BoxFuture;
use crate::auth::{AccessToken, AuthProvider};

#[repr(C)]
pub struct CAuthProvider {
    pub(crate) type_id: TypeId,
    pub(crate) access_token: unsafe fn(*mut CAuthProvider) -> BoxFuture<'static, Result<AccessToken>>,
    pub(crate) access_token_with_scopes: for<'a> unsafe fn(*mut CAuthProvider, Vec<&'a str>) -> BoxFuture<'a, Result<AccessToken>>,
    pub(crate) client_id: unsafe fn(*mut CAuthProvider) -> &'static str,
    pub(crate) current_scopes: unsafe fn(*mut CAuthProvider) -> &'static [String],
    pub(crate) set_access_token: unsafe fn(*mut CAuthProvider, AccessToken),
}

#[repr(C)]
struct CAuthProviderWrapper<A> {
    pub(crate) vt: CAuthProvider,
    pub(crate) provider: A,
}

impl CAuthProvider {
    pub fn for_auth_provider<A>(provider: A) -> *mut CAuthProvider
        where A: AuthProvider + Send + Sync + 'static, {
        let wrap = CAuthProviderWrapper {
            vt: CAuthProvider::vtable::<A>(),
            provider,
        };

        let boxed = Box::into_raw(Box::new(wrap));

        boxed as *mut _
    }

    fn vtable<A: AuthProvider + 'static>() -> CAuthProvider {
        let type_id = TypeId::of::<A>();

        unsafe fn client_id<A: AuthProvider + 'static>(provider: *mut CAuthProvider) -> &'static str {
            let wrap = &mut *(provider as *mut CAuthProviderWrapper<A>);
            wrap.provider.client_id()
        }

        unsafe fn access_token<A: AuthProvider + 'static>(provider: *mut CAuthProvider) -> BoxFuture<'static, Result<AccessToken>> {
            let wrap = &mut *(provider as *mut CAuthProviderWrapper<A>);
            wrap.provider.access_token()
        }

        unsafe fn access_token_with_scopes<'a, A: AuthProvider + 'static>(provider: *mut CAuthProvider, scopes: Vec<&'a str>) -> BoxFuture<'a, Result<AccessToken>> {
            let wrap = &mut *(provider as *mut CAuthProviderWrapper<A>);
            wrap.provider.access_token_with_scopes(scopes)
        }

        unsafe fn current_scopes<A: AuthProvider + 'static>(provider: *mut CAuthProvider) -> &'static [String] {
            let wrap = &mut *(provider as *mut CAuthProviderWrapper<A>);
            wrap.provider.current_scopes()
        }

        unsafe fn set_access_token<A: AuthProvider + 'static>(provider: *mut CAuthProvider, access_token: AccessToken) {
            let wrap = &mut *(provider as *mut CAuthProviderWrapper<A>);
            wrap.provider.set_access_token(access_token);
        }

        CAuthProvider {
            type_id,
            client_id: client_id::<A>,
            access_token: access_token::<A>,
            access_token_with_scopes: access_token_with_scopes::<A>,
            current_scopes: current_scopes::<A>,
            set_access_token: set_access_token::<A>
        }
    }
}