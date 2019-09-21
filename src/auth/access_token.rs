use std::option::Option;
use std::time::SystemTime;
use std::borrow::Borrow;

#[derive(Clone, Deserialize, Debug)]
pub struct AccessTokenData {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
    scope: Option<String>,
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
            obtainment_date: SystemTime::now()
        }
    }

    pub fn empty() -> Self {
        Self {
            data: AccessTokenData {
                access_token: "".to_string(),
                refresh_token: None,
                expires_in: None,
                scope: None
            },
            obtainment_date: SystemTime::now()
        }
    }

    pub(crate) fn with_access_token_and_scope(access_token: String, scope: Option<String>) -> Self {
        Self {
            data: AccessTokenData {
                access_token,
                scope,
                expires_in: None,
                refresh_token: None
            },
            obtainment_date: SystemTime::now()
        }
    }

    pub fn with_obtainment_date(data: AccessTokenData, obtainment_date: SystemTime) -> AccessToken {
        AccessToken {
            data,
            obtainment_date
        }
    }

    pub fn access_token(&self) -> String {
        self.data.access_token.clone()
    }

    pub fn refresh_token(&self) -> Option<String> {
        self.data.refresh_token.clone()
    }

    pub fn scope(&self) -> Vec<String> {
        match self.data.scope.borrow() {
            Some(scope) => scope.split(' ').map(|s| s.to_string()).collect(),
            None => Vec::new()
        }
    }

    pub fn is_expired(&self) -> bool {
        match self.data.expires_in {
            Some(secs) => SystemTime::now().duration_since(self.obtainment_date).unwrap().as_secs() > secs,
            None => false
        }
    }
}