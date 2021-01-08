use http::uri::{Scheme, Uri};
use hyper::Method;
use url::Url;
use std::{
    borrow::Cow,
    error::Error,
    fmt::Display
};

use crate::util::Result;

#[derive(Clone)]
pub enum TwitchApiCallType {
    Kraken,
    Helix,
    Auth,
    Custom,
}

#[derive(Clone)]
pub struct TwitchApiCall<'a, T = ()> {
    url: String,
    call_type: TwitchApiCallType,
    method: Method,
    params: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    body: Option<T>,
    scope: Option<String>,
}

impl<'a> TwitchApiCall<'a> {
    pub fn builder_empty() -> TwitchAPICallBuilder<'a> {
        TwitchAPICallBuilder::<'a>::new()
    }
}

impl<'a, T> TwitchApiCall<'a, T> {
    pub fn builder() -> TwitchAPICallBuilder<'a, T> {
        TwitchAPICallBuilder::<'a, T>::new()
    }

    pub fn full_url(&self) -> Url {
        let builder = Uri::builder()
            .scheme(Scheme::HTTPS)
            .authority("api.twitch.tv");

        let uri = match self.call_type {
            TwitchApiCallType::Kraken => {
                let path = format!("/kraken/{}", self.url.trim_start_matches('/'));
                builder.path_and_query(path.as_str()).build().unwrap()
            }
            TwitchApiCallType::Helix => {
                let path = format!("/helix/{}", self.url.trim_start_matches('/'));
                builder.path_and_query(path.as_str()).build().unwrap()
            }
            TwitchApiCallType::Auth => {
                let path = format!("/oauth2/{}", self.url.trim_start_matches('/'));
                builder.authority("id.twitch.tv").path_and_query(path.as_str()).build().unwrap()
            }
            TwitchApiCallType::Custom => self.url.parse().unwrap()
        };

        Url::parse_with_params(uri.to_string().as_str(), self.params.to_vec()).unwrap()
    }

    pub fn scope(&self) -> Option<&str> {
        self.scope.as_ref().map(String::as_str)
    }

    pub fn method(&self) -> Method {
        self.method.clone()
    }
}

#[derive(Clone)]
pub struct TwitchAPICallBuilder<'a, T = ()> {
    __url: Option<String>,
    __call_type: TwitchApiCallType,
    __method: Method,
    __params: Vec<(Cow<'a, str>, Cow<'a, str>)>,
    __body: Option<T>,
    __scope: Option<String>,
}

impl<'a, T> TwitchAPICallBuilder<'a, T> {
    pub fn new() -> Self {
        Self {
            __url: None,
            __call_type: TwitchApiCallType::Helix,
            __method: Method::GET,
            __params: Vec::new(),
            __body: None,
            __scope: None,
        }
    }

    pub fn with_url(mut self, url: impl Into<String>) -> Self {
        self.__url = Some(url.into());
        self
    }

    pub fn with_call_type(mut self, call_type: TwitchApiCallType) -> Self {
        self.__call_type = call_type;
        self
    }

    pub fn with_method(mut self, method: Method) -> Self {
        self.__method = method;
        self
    }

    pub fn with_body(mut self, body: impl Into<T>) -> Self {
        self.__body = Some(body.into());
        self
    }

    pub fn with_param(mut self, key: impl Into<Cow<'a, str>>, value: impl Into<Cow<'a, str>>) -> Self {
        self.__params.push((key.into(), value.into()));
        self
    }

    pub fn with_scope(mut self, scope: impl Into<String>) -> Self {
        self.__scope = Some(scope.into());
        self
    }

    pub fn build(self) -> Result<TwitchApiCall<'a, T>> {
        if self.__url.is_none() {
            return Err(Box::new(TwitchAPICallBuilderError::new("No URL given")));
        }
        if std::mem::size_of::<T>() > 0 && self.__body.is_none() {
            return Err(Box::new(TwitchAPICallBuilderError::new("No body given")));
        }
        Ok(TwitchApiCall {
            url: self.__url.unwrap(),
            call_type: self.__call_type,
            method: self.__method,
            params: self.__params,
            body: self.__body,
            scope: self.__scope,
        })
    }
}

#[derive(Debug)]
pub struct TwitchAPICallBuilderError<'a> {
    description: &'a str,
}

impl<'a> TwitchAPICallBuilderError<'a> {
    pub fn new(description: &'a str) -> Self {
        Self {
            description
        }
    }
}

impl<'a> Error for TwitchAPICallBuilderError<'a> {
    fn description(&self) -> &str {
        self.description
    }
}

impl<'a> Display for TwitchAPICallBuilderError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.to_string().fmt(f)
    }
}
