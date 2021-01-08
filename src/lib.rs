#![feature(in_band_lifetimes)]
#![allow(dead_code)]
#![warn(unused_imports)]

#[macro_use]
extern crate simple_error;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate connect;

mod api;
mod auth;
mod util;

#[derive(Deserialize, Debug)]
struct UserResponse {
    data: Vec<User>,
}

#[derive(Deserialize, Debug)]
pub struct User {
    id: String,
    login: String,
}

#[cfg(test)]
mod tests {
    use crate::util::Result;
    use crate::auth::StaticAuthProvider;
    use crate::api::ApiClient;
    use ffi_support::{FfiStr, rust_string_to_c};

    #[tokio::test]
    async fn a_test() -> Result<()> {
        let client_id = match std::env::var("TWITCH_CLIENT_ID") {
            Ok(val) => val,
            Err(_e) => panic!("TWITCH_CLIENT_ID was not set")
        };

        let access_token = match std::env::var("TWITCH_ACCESS_TOKEN") {
            Ok(val) => val,
            Err(_e) => panic!("TWITCH_ACCESS_TOKEN was not set")
        };
        let mut auth = StaticAuthProvider::new(client_id, access_token);
        let mut client = ApiClient::new(Box::new(auth));
        let user = client.get_me().await?;
        println!("user: {:#?}", user);
        Ok(())
    }

    #[test]
    fn c_test() -> Result<()> {
        unsafe {
            let client_id = match std::env::var("TWITCH_CLIENT_ID") {
                Ok(val) => val,
                Err(_e) => panic!("TWITCH_CLIENT_ID was not set")
            };
            let access_token = match std::env::var("TWITCH_ACCESS_TOKEN") {
                Ok(val) => val,
                Err(_e) => panic!("TWITCH_ACCESS_TOKEN was not set")
            };

            let client_id_ffi = FfiStr::from_raw(rust_string_to_c(client_id));
            let access_token_ffi = FfiStr::from_raw(rust_string_to_c(access_token));
            let mut auth = crate::c_bindings::createStaticAuthProvider(client_id_ffi, access_token_ffi);
            let mut client = crate::c_bindings::createApiClient(auth);

            let user = crate::c_bindings::getMe(client);
            print!("user: {} {} {:p} {:#?}", (*user).id(), (*user).login(), user, *user);

            Ok(())
        }
    }
}

mod c_bindings;

#[cfg(crate_type = "cdylib")]
pub use c_bindings::*;