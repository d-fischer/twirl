use crate::auth::{AuthProvider, AccessToken};
use crate::api::{TwitchApiCall, TwitchApiCallType, TokenInfo, TokenInfoData};
use crate::util::Result;
use hyper::{Client, Body};
use hyper_tls::HttpsConnector;
use hyper::client::HttpConnector;
use http::{Method, Request, Response};
use crate::{User, UserResponse};

#[repr(C)]
pub struct ApiClient {
    auth: Box<dyn AuthProvider + Sync + Send>,
}

impl ApiClient {
    pub fn new(auth: Box<dyn AuthProvider + Sync + Send>) -> ApiClient {
        ApiClient {
            auth
        }
    }

    pub async fn call_api<T, B>(&mut self, call: TwitchApiCall<'_, B>) -> Result<T>
        where T: serde::de::DeserializeOwned {
        let token = match call.scope() {
            Some(scope) => self.auth.access_token_with_scopes(vec![scope]),
            None => self.auth.access_token()
        }.await?;
        if token.is_expired() {
            // TODO
        }
        Self::call_api_with_credentials(call, self.auth.client_id(), token.access_token().to_string()).await
    }

    fn get_http_client() -> &'static Client<HttpsConnector<HttpConnector>> {
        lazy_static! {
            static ref CLIENT: Client<HttpsConnector<HttpConnector>> = Client::builder()
                .build(HttpsConnector::new());
        }

        &CLIENT
    }

    async fn transform_response<T>(url_str: String, res: Response<Body>) -> Result<T>
        where T: serde::de::DeserializeOwned {
        match res.status().is_success() {
            true => {
                let body = res.into_body();
                let chunk = hyper::body::to_bytes(body).await?;
                let data: T = serde_json::from_slice(chunk.as_ref())?;
                Ok(data)
            }
            false => bail!(format!("request to {} failed with status {}", url_str, res.status()))
        }
    }

    pub async fn call_api_without_credentials<T, B>(call: TwitchApiCall<'_, B>) -> Result<T>
        where T: serde::de::DeserializeOwned {
        let url = call.full_url();
        let url_str = url.into_string();
        let uri: hyper::Uri = url_str.parse()?;

        let req = Request::builder()
            .uri(uri)
            .method(call.method());

        let res = Self::get_http_client().request(req.body(Body::empty())?).await?;

        Self::transform_response(url_str, res).await
    }

    pub async fn call_api_with_credentials<T, B>(call: TwitchApiCall<'_, B>, client_id: impl ToString, access_token: impl ToString) -> Result<T>
        where T: serde::de::DeserializeOwned {
        let url = call.full_url();
        let url_str = url.into_string();
        let uri: hyper::Uri = url_str.parse()?;

        let req = Request::builder()
            .uri(uri)
            .header("Client-ID", client_id.to_string())
            .header("Authorization", format!("Bearer {}", access_token.to_string()))
            .method(call.method());

        let res = Self::get_http_client().request(req.body(Body::empty())?).await?;

        Self::transform_response(url_str, res).await
    }

    pub async fn get_app_access_token(client_id: impl ToString, client_secret: impl ToString) -> Result<AccessToken> {
        let call = TwitchApiCall::builder_empty()
            .with_call_type(TwitchApiCallType::Auth)
            .with_url("token")
            .with_method(Method::POST)
            .with_param("grant_type", "client_credentials")
            .with_param("client_id", client_id.to_string())
            .with_param("client_secret", client_secret.to_string())
            .build()?;

        let response = Self::call_api_without_credentials(call).await?;
        Ok(AccessToken::new(response))
    }

    pub async fn refresh_access_token(client_id: impl ToString, client_secret: impl ToString, refresh_token: impl ToString) -> Result<AccessToken> {
        let call = TwitchApiCall::builder_empty()
            .with_call_type(TwitchApiCallType::Auth)
            .with_url("token")
            .with_method(Method::POST)
            .with_param("grant_type", "refresh_token")
            .with_param("client_id", client_id.to_string())
            .with_param("client_secret", client_secret.to_string())
            .with_param("refresh_token", refresh_token.to_string())
            .build()?;

        let response = Self::call_api_without_credentials(call).await?;
        Ok(AccessToken::new(response))
    }

    pub async fn get_token_info_for_access_token(client_id: impl ToString, access_token: impl ToString) -> Result<TokenInfo> {
        let call = TwitchApiCall::builder_empty()
            .with_call_type(TwitchApiCallType::Auth)
            .with_url("validate")
            .build()?;
        let response: TokenInfoData = Self::call_api_with_credentials(call, client_id.to_string(), access_token.to_string()).await?;
        Ok(TokenInfo::new(response))
    }

    pub async fn get_me(&mut self) -> Result<User> {
        let call = TwitchApiCall::builder_empty()
            .with_call_type(TwitchApiCallType::Helix)
            .with_url("users")
            .build()?;
        let mut response: UserResponse = self.call_api(call).await?;

        Ok(response.data.swap_remove(0))
    }

    pub async fn get_user_by_login(&mut self, login: impl ToString) -> Result<Option<User>> {
        let login = login.to_string();
        let call = TwitchApiCall::builder_empty()
            .with_call_type(TwitchApiCallType::Helix)
            .with_url("users")
            .with_param("login", login)
            .build()?;
        let mut response: UserResponse = self.call_api(call).await?;

        match response.data.is_empty() {
            true => Ok(None),
            false => Ok(Some(response.data.swap_remove(0)))
        }
    }
}