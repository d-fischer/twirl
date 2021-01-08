use crate::auth::{AuthProvider, AccessToken};
use crate::util::Result;
use futures::future::BoxFuture;
use crate::api::ApiClient;
use crate::auth::provider::provider::AuthProviderError;
use futures::FutureExt;

#[derive(Clone)]
pub struct StaticAuthProvider {
    client_id: String,
    access_token: AccessToken,
    scopes: Option<Vec<String>>,
}

impl StaticAuthProvider {
    pub fn new(client_id: String, access_token: String) -> Self {
        Self {
            client_id,
            access_token: AccessToken::with_access_token_and_scopes(
                access_token,
                vec![],
            ),
            scopes: None,
        }
    }

    pub fn with_scopes(client_id: String, access_token: String, scopes: Vec<String>) -> Self {
        Self {
            client_id,
            access_token: AccessToken::with_access_token_and_scopes(
                access_token,
                scopes.clone(),
            ),
            scopes: Some(scopes),
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

    fn access_token(&'a mut self) -> BoxFuture<'a, Result<AccessToken>> {
        async move { Ok(self.access_token.clone()) }.boxed()
    }

    fn access_token_with_scopes(&'a mut self, scopes: Vec<&'a str>) -> BoxFuture<'a, Result<AccessToken>> {
        async move {
            if !scopes.is_empty() {
                if self.scopes.is_none() {
                    let token_info = ApiClient::get_token_info_for_access_token(
                        self.client_id.clone(),
                        self.access_token.access_token(),
                    ).await?;
                    self.scopes = Some(token_info.scopes().to_owned())
                }
                let current_scopes = self.scopes.as_ref().unwrap();
                if scopes.iter().any(|scope| !current_scopes.iter().any(|inner_scope| inner_scope == scope)) {
                    return Err(Box::new(AuthProviderError::new(format!(
                        "This token does not have the requested scopes ({}) and can not be upgraded",
                        scopes.join(", ")))) as Box<dyn std::error::Error + Send + Sync>
                    );
                }
            }

            Ok(self.access_token.clone())
        }.boxed()
    }

    fn set_access_token(&mut self, token: AccessToken) {
        self.access_token = token;
    }
}