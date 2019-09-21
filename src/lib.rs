#![feature(in_band_lifetimes)]

#[macro_use]
extern crate simple_error;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

extern crate chrono;
extern crate connect;

use futures_util::TryStreamExt;
use hyper::{
    Body,
    Client,
    Method,
    Request,
    client::HttpConnector,
};
use hyper_tls::HttpsConnector;

mod api;
mod auth;
mod util;

use crate::api::{TokenInfo, TokenInfoData, TwitchAPICall, TwitchAPICallType};
use crate::auth::{AccessToken, AuthProvider, ClientCredentialsAuthProvider, StaticAuthProvider};
use crate::util::{Result, StringOption};

#[repr(C)]
pub struct TwitchClient {
    auth: Box<dyn AuthProvider + Sync + Send>,
}

impl TwitchClient {
    pub fn new(client_id: impl Into<String>) -> Self {
        Self {
            auth: Box::new(StaticAuthProvider::new(client_id.into()))
        }
    }

    pub fn with_credentials(client_id: impl Into<String>, access_token: impl Into<String>, scopes: Option<Vec<String>>) -> Self {
        Self {
            auth: Box::new(StaticAuthProvider::with_access_token(client_id.into(), access_token.into(), scopes))
        }
    }

    pub fn with_client_credentials(client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        Self {
            auth: Box::new(ClientCredentialsAuthProvider::new(client_id.into(), client_secret.into())),
        }
    }

    pub async fn call_api<T, B>(&mut self, call: TwitchAPICall<'_, B>) -> Result<T>
        where T: serde::de::DeserializeOwned {
        let token: AccessToken = self.auth.access_token(call.scope().map(|scope| vec![scope])).await?;
        if token.is_expired() {
            // TODO
        }
        Self::call_api_with_credentials(call, self.auth.client_id(), token.access_token().to_string()).await
    }

    pub async fn call_api_with_credentials<T, B>(call: TwitchAPICall<'_, B>, client_id: impl StringOption, access_token: impl StringOption) -> Result<T>
        where T: serde::de::DeserializeOwned {
        lazy_static! {
            static ref CLIENT: Client<HttpsConnector<HttpConnector>> = Client::builder()
                .build(HttpsConnector::new().expect("Failed initializing HTTPS connector"));
        }

        let url = call.full_url();
        let url_str = url.into_string();
        let uri: hyper::Uri = url_str.parse()?;

        let mut req = Request::builder();
        req.uri(uri);
        if let Some(id) = client_id.get() {
            req.header("Client-ID", id);
        }
        if let Some(tk) = access_token.get() {
            req.header("Authorization", format!("Bearer {}", tk));
        }
        req.method(call.method());

        let res = CLIENT.request(req.body(Body::empty())?).await?;

        match res.status().is_success() {
            true => {
                let body = res.into_body();
                let chunk = body.try_concat().await?;
                let data: T = serde_json::from_slice(&chunk)?;
                Ok(data)
            }
            false => bail!(format!("request to {} failed with status {}", url_str, res.status()))
        }
    }

    pub async fn get_app_access_token(client_id: impl Into<String>, client_secret: impl Into<String>) -> Result<AccessToken> {
        let call = TwitchAPICall::builder_empty()
            .with_call_type(TwitchAPICallType::Auth)
            .with_url("token")
            .with_method(Method::POST)
            .with_param("grant_type", "client_credentials")
            .with_param("client_id", client_id.into())
            .with_param("client_secret", client_secret.into())
            .build()?;

        let response = Self::call_api_with_credentials(call, None, None).await?;
        Ok(AccessToken::new(response))
    }

    pub async fn refresh_access_token(client_id: impl Into<String>, client_secret: impl Into<String>, refresh_token: impl Into<String>) -> Result<AccessToken> {
        let call = TwitchAPICall::builder_empty()
            .with_call_type(TwitchAPICallType::Auth)
            .with_url("token")
            .with_method(Method::POST)
            .with_param("grant_type", "refresh_token")
            .with_param("client_id", client_id.into())
            .with_param("client_secret", client_secret.into())
            .with_param("refresh_token", refresh_token.into())
            .build()?;

        let response = Self::call_api_with_credentials(call, None, None).await?;
        Ok(AccessToken::new(response))
    }

    pub async fn get_token_info_for_access_token(client_id: impl Into<String>, access_token: impl Into<String>) -> Result<TokenInfo> {
        let call = TwitchAPICall::builder_empty()
            .with_call_type(TwitchAPICallType::Auth)
            .with_url("validate")
            .build()?;
        let response: TokenInfoData = Self::call_api_with_credentials(call, client_id.into(), access_token.into()).await?;
        Ok(TokenInfo::new(response))
    }

    pub async fn get_me(&mut self) -> Result<User> {
        let call = TwitchAPICall::builder_empty()
            .with_call_type(TwitchAPICallType::Helix)
            .with_url("users")
            .build()?;
        let mut response: UserResponse = self.call_api(call).await?;

        Ok(response.data.swap_remove(0))
    }

    pub async fn get_user_by_login(&mut self, login: impl Into<String>) -> Result<Option<User>> {
        let login = login.into();
        let call = TwitchAPICall::builder_empty()
            .with_call_type(TwitchAPICallType::Helix)
            .with_url("users")
            .with_param("login", login.as_str())
            .build()?;
        let mut response: UserResponse = self.call_api(call).await?;

        match response.data.is_empty() {
            true => Ok(None),
            false => Ok(Some(response.data.swap_remove(0)))
        }
    }
}

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
    use super::TwitchClient;
    use super::Result;

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
        let mut client = TwitchClient::with_credentials(client_id, access_token, None);
        let user = client.get_me().await?;
        println!("user: {:#?}", user);
        Ok(())
    }
}

mod c_bindings;

#[cfg(crate_type = "cdylib")]
pub use c_bindings::*;