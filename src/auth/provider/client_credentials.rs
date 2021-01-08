use crate::auth::{AccessToken, AuthProvider, RefreshableAuthProvider};
use futures::future::BoxFuture;
use crate::auth::provider::provider::AuthProviderError;
use std::borrow::Borrow;
use futures::FutureExt;
use crate::util::Result;
use crate::api::ApiClient;

pub struct ClientCredentialsAuthProvider {
    client_id: String,
    client_secret: String,
    current_token: Option<AccessToken>,
}

impl ClientCredentialsAuthProvider {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>) -> ClientCredentialsAuthProvider {
        ClientCredentialsAuthProvider {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
            current_token: None,
        }
    }
}

impl AuthProvider for ClientCredentialsAuthProvider {
    fn client_id(&self) -> &str {
        self.client_id.as_str()
    }

    fn current_scopes(&self) -> &[String] {
        &[]
    }

    fn access_token(&'a mut self) -> BoxFuture<Result<AccessToken>> {
        async move {
            match self.current_token.borrow() {
                Some(token) => {
                    if token.is_expired() {
                        return self.refresh().await;
                    }
                    Ok(token.clone())
                }
                None => self.refresh().await
            }
        }.boxed()
    }

    fn access_token_with_scopes(&'a mut self, scopes: Vec<&'a str>) -> BoxFuture<'a, Result<AccessToken>> {
        async move {
            if scopes.len() > 0 {
                return Err(Box::new(AuthProviderError::new("The client credentials flow does not support scopes")) as Box<dyn std::error::Error + Send + Sync>)
            }
            self.access_token().await
        }.boxed()
    }

    fn set_access_token(&mut self, token: AccessToken) {
        self.current_token = Some(token);
    }
}

impl RefreshableAuthProvider for ClientCredentialsAuthProvider {
    fn refresh(&'a mut self) -> BoxFuture<'a, Result<AccessToken>> {
        async move {
            let token = ApiClient::get_app_access_token(self.client_id.clone(), self.client_secret.clone()).await?;
            self.current_token = Some(token.clone());
            Ok(token)
        }.boxed()
    }
}