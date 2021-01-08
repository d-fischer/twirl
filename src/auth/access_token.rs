use std::option::Option;
use std::time::SystemTime;

#[derive(Clone, Deserialize, Debug)]
pub struct AccessTokenData {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    scope: Vec<String>,
}

#[derive(Clone)]
pub struct AccessToken {
    data: AccessTokenData,
    obtainment_date: SystemTime,
}

impl AccessToken {
    pub fn new(data: AccessTokenData) -> Self {
        Self {
            data,
            obtainment_date: SystemTime::now(),
        }
    }

    pub fn empty() -> Self {
        Self {
            data: AccessTokenData {
                access_token: "".to_string(),
                refresh_token: None,
                expires_in: None,
                scope: vec![],
            },
            obtainment_date: SystemTime::now(),
        }
    }

    pub(crate) fn with_access_token(access_token: String) -> Self {
        Self {
            data: AccessTokenData {
                access_token,
                refresh_token: None,
                expires_in: None,
                scope: vec![],
            },
            obtainment_date: SystemTime::now(),
        }
    }

    pub(crate) fn with_access_token_and_scopes(access_token: String, scopes: Vec<String>) -> Self {
        Self {
            data: AccessTokenData {
                access_token,
                refresh_token: None,
                expires_in: None,
                scope: scopes,
            },
            obtainment_date: SystemTime::now(),
        }
    }

    pub fn with_obtainment_date(data: AccessTokenData, obtainment_date: SystemTime) -> Self {
        Self {
            data,
            obtainment_date,
        }
    }

    pub fn access_token(&self) -> &str {
        self.data.access_token.as_str()
    }

    pub fn refresh_token(&self) -> Option<&str> {
        self.data.refresh_token.as_ref().map(String::as_str)
    }

    pub fn scopes(&self) -> &[String] {
        &self.data.scope
    }

    pub fn is_expired(&self) -> bool {
        match self.data.expires_in {
            Some(secs) => SystemTime::now().duration_since(self.obtainment_date).unwrap().as_secs() > secs,
            None => false
        }
    }
}