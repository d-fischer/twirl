use futures::future::{BoxFuture};
use std::{
    borrow::{Borrow, Cow},
    error::Error,
    fmt::Display,
};

use crate::auth::AccessToken;
use crate::util::Result;

pub trait AuthProvider {
    fn client_id(&self) -> &str;
    fn current_scopes(&self) -> &[String];
    fn access_token(&mut self) -> BoxFuture<Result<AccessToken>>;
    fn access_token_with_scopes(&'a mut self, scopes: Vec<&'a str>) -> BoxFuture<'a, Result<AccessToken>>;
    fn set_access_token(&mut self, token: AccessToken);
}

pub trait RefreshableAuthProvider: AuthProvider {
    fn refresh(&'a mut self) -> BoxFuture<'a, Result<AccessToken>>;
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
        self.to_string().fmt(f)
    }
}