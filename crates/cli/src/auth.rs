use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use async_trait::async_trait;
use rust_mcp_sdk::auth::{AuthInfo, AuthProvider, AuthenticationError, OauthEndpoint};
use rust_mcp_sdk::mcp_http::{GenericBody, GenericBodyExt, McpAppState, McpHttpError};
use subtle::ConstantTimeEq;

/// Static API keys have no expiry, but the SDK auth middleware requires an
/// `expires_at` value on `AuthInfo`, so report a far-future expiration.
const STATIC_API_KEY_LIFETIME: Duration = Duration::from_secs(100 * 365 * 24 * 60 * 60);

/// Validates `Authorization: Bearer <API_KEY>` for the HTTP transport.
pub struct ApiKeyAuthProvider {
    expected: Vec<u8>,
}

impl ApiKeyAuthProvider {
    pub fn new(key: String) -> Self {
        Self {
            expected: key.into_bytes(),
        }
    }
}

#[async_trait]
impl AuthProvider for ApiKeyAuthProvider {
    async fn verify_token(&self, access_token: String) -> Result<AuthInfo, AuthenticationError> {
        let provided = access_token.as_bytes();

        if bool::from(provided.ct_eq(&self.expected)) {
            Ok(AuthInfo {
                token_unique_id: "api-key".to_string(),
                client_id: None,
                user_id: None,
                scopes: None,
                expires_at: Some(SystemTime::now() + STATIC_API_KEY_LIFETIME),
                audience: None,
                extra: None,
            })
        } else {
            Err(AuthenticationError::InvalidToken {
                description: "Invalid API key",
            })
        }
    }

    fn auth_endpoints(&self) -> Option<&HashMap<String, OauthEndpoint>> {
        None
    }

    async fn handle_request(
        &self,
        _request: http::Request<&str>,
        _state: Arc<McpAppState>,
    ) -> Result<http::Response<GenericBody>, McpHttpError> {
        Ok(GenericBody::create_404_response())
    }

    fn protected_resource_metadata_url(&self) -> Option<&str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn accepts_correct_key() {
        let provider = ApiKeyAuthProvider::new("secret-key".to_string());
        let info = provider
            .verify_token("secret-key".to_string())
            .await
            .expect("valid API key");

        assert_eq!(info.token_unique_id, "api-key");
    }

    #[tokio::test]
    async fn rejects_wrong_key() {
        let provider = ApiKeyAuthProvider::new("secret-key".to_string());

        assert!(matches!(
            provider.verify_token("wrong-key".to_string()).await,
            Err(AuthenticationError::InvalidToken { .. })
        ));
    }

    #[tokio::test]
    async fn rejects_empty_token() {
        let provider = ApiKeyAuthProvider::new("secret-key".to_string());

        assert!(matches!(
            provider.verify_token(String::new()).await,
            Err(AuthenticationError::InvalidToken { .. })
        ));
    }
}
