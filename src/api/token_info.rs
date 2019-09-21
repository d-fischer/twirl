use std::time::{Duration, SystemTime};

#[derive(Clone, Deserialize, Debug)]
pub struct TokenInfoData {
    client_id: String,
    login: String,
    scopes: Vec<String>,
    user_id: String,
    expires_in: Option<u64>
}

pub struct TokenInfo {
    data: TokenInfoData,
    obtainment_date: SystemTime,
}

impl TokenInfo {
    pub fn new(data: TokenInfoData) -> Self {
        Self {
            data,
            obtainment_date: SystemTime::now()
        }
    }

    pub fn client_id(&self) -> String {
        self.data.client_id.clone()
    }

    pub fn scopes(&self) -> Vec<String> {
        self.data.scopes.clone()
    }

    pub fn expiry_date(&self) -> Option<SystemTime> {
        self.data.expires_in.map(|expires_in| self.obtainment_date + Duration::from_secs(expires_in))
    }
}