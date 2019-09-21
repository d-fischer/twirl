use futures::future::{BoxFuture, FutureExt};
use std::{
    borrow::{Borrow, Cow},
    error::Error,
    fmt::Display,
    option::Option,
};

use crate::auth::AccessToken;
use crate::util::Result;
use crate::TwitchClient;

pub trait AuthProvider {
    fn client_id(&self) -> &str;
    fn current_scopes(&self) -> &[String];
    fn access_token(&'a mut self, scope: Option<Vec<&'a str>>) -> BoxFuture<'a, Result<AccessToken>>;
    fn set_access_token(&mut self, token: AccessToken);
}

pub trait RefreshableAuthProvider: AuthProvider {
    fn refresh(&'a mut self) -> BoxFuture<'a, Result<AccessToken>>;
}

#[derive(Clone)]
pub struct StaticAuthProvider {
    client_id: String,
    access_token: Option<AccessToken>,
    scopes: Option<Vec<String>>,
}

impl StaticAuthProvider {
    pub fn new(client_id: String) -> Self {
        Self {
            client_id,
            access_token: None,
            scopes: None,
        }
    }

    pub fn with_access_token(client_id: String, access_token: String, scopes: Option<Vec<String>>) -> Self {
        Self {
            client_id,
            access_token: Some(AccessToken::with_access_token_and_scope(
                access_token,
                scopes.borrow().as_ref().map(|sc| sc.join(" ")),
            )),
            scopes,
        }
    }
}

impl AuthProvider for StaticAuthProvider {
    fn client_id(&self) -> &str {
        self.client_id.as_str()
    }

    fn current_scopes(&self) -> &[String] {
        self.scopes.as_ref().map_or(&[], Vec::as_slice)
    }

    fn access_token(&'a mut self, scope: Option<Vec<&'a str>>) -> BoxFuture<'a, Result<AccessToken>> {
        async move {
            if let Some(scopes) = scope {
                if !scopes.is_empty() {
                    if self.scopes.is_none() {
                        if self.access_token.is_none() {
                            return Err(Box::new(AuthProviderError::new(
                                "Auth provider has not been initialized with a token yet and is requesting scopes"
                            )) as Box<dyn std::error::Error + Send + Sync>);
                        }
                        let token_info = TwitchClient::get_token_info_for_access_token(
                            self.client_id.clone(),
                            self.access_token.borrow().as_ref().unwrap().access_token(),
                        ).await?;
                        self.scopes = Some(token_info.scopes().to_owned())
                    }
                    let current_scopes = self.scopes.borrow().as_ref().unwrap();
                    if scopes.iter().any(|scope| !current_scopes.iter().any(|inner_scope| inner_scope == scope)) {
                        return Err(Box::new(AuthProviderError::new(format!(
                            "This token does not have the requested scopes ({}) and can not be upgraded",
                            scopes.join(", ")))) as Box<dyn std::error::Error + Send + Sync>
                        );
                    }
                }
            }

            Ok(self.access_token.clone().unwrap_or_else(|| AccessToken::empty()))
        }.boxed()
    }

    fn set_access_token(&mut self, token: AccessToken) {
        self.access_token = Some(token);
    }
}

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

    fn access_token(&'a mut self, scope: Option<Vec<&'a str>>) -> BoxFuture<'a, Result<AccessToken>> {
        async move {
            if let Some(s) = scope {
                if s.len() > 0 {
                    return Err(Box::new(AuthProviderError::new("The client credentials flow does not support scopes")) as Box<dyn std::error::Error + Send + Sync>);
                }
            }

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

    fn set_access_token(&mut self, token: AccessToken) {
        self.current_token = Some(token);
    }
}

impl RefreshableAuthProvider for ClientCredentialsAuthProvider {
    fn refresh(&'a mut self) -> BoxFuture<'a, Result<AccessToken>> {
        async move {
            let token = TwitchClient::get_app_access_token(self.client_id.clone(), self.client_secret.clone()).await?;
            self.current_token = Some(token.clone());
            Ok(token)
        }.boxed()
    }
}

#[derive(Debug)]
pub struct AuthProviderError<'a> {
    description: Cow<'a, str>,
}

impl<'a> AuthProviderError<'a> {
    pub fn new(description: impl Into<Cow<'a, str>>) -> Self {
        Self {
            description: description.into()
        }
    }
}

impl<'a> Error for AuthProviderError<'a> {
    fn description(&self) -> &str {
        self.description.borrow()
    }
}

impl<'a> Display for AuthProviderError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Error::description(self).fmt(f)
    }
}