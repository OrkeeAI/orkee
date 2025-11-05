// ABOUTME: OAuth manager for token management and storage
// ABOUTME: Handles token import, logout, retrieval, and status for all AI providers

use sqlx::SqlitePool;
use tracing::{debug, info};

use crate::{
    error::{AuthError, AuthResult},
    oauth::{
        storage::OAuthStorage,
        types::{OAuthProvider, OAuthToken},
    },
};

/// OAuth manager for handling token storage and retrieval
pub struct OAuthManager {
    storage: OAuthStorage,
}

impl OAuthManager {
    /// Create a new OAuth manager with database pool
    pub fn new(pool: SqlitePool) -> AuthResult<Self> {
        let storage = OAuthStorage::new(pool)?;
        Ok(Self { storage })
    }

    /// Create a new OAuth manager with default database connection
    pub async fn new_default() -> AuthResult<Self> {
        // Connect to default database location
        let db_path = dirs::home_dir()
            .ok_or_else(|| {
                AuthError::Configuration("Could not determine home directory".to_string())
            })?
            .join(".orkee")
            .join("orkee.db");

        let database_url = format!("sqlite:{}", db_path.display());

        let pool = sqlx::SqlitePool::connect(&database_url)
            .await
            .map_err(|e| AuthError::Storage(format!("Failed to connect to database: {}", e)))?;

        Self::new(pool)
    }

    /// Import a token directly without OAuth flow (for Claude session keys)
    pub async fn import_token(&self, token: OAuthToken) -> AuthResult<()> {
        info!(
            "Importing {} token for user {}",
            token.provider, token.user_id
        );

        // Validate token format for known providers
        match token.provider.as_str() {
            "claude" => {
                if !token.access_token.starts_with("sk-ant-") {
                    return Err(AuthError::InvalidToken(
                        "Claude tokens should start with 'sk-ant-'".into(),
                    ));
                }
                // Specifically check for OAuth token format, not API key
                if !token.access_token.starts_with("sk-ant-oat01-") {
                    info!(
                        "Token appears to be a Claude API key (not OAuth token). \
                         OAuth tokens start with 'sk-ant-oat01-'. \
                         API keys should be stored in Settings instead."
                    );
                }
            }
            "openai" | "google" | "xai" => {
                // These providers would need their own validation once implemented
                info!("{} token import not yet fully implemented", token.provider);
            }
            _ => {
                return Err(AuthError::InvalidProvider(format!(
                    "Unknown provider: {}",
                    token.provider
                )));
            }
        }

        // Store the encrypted token
        self.storage.store_token(&token).await?;

        info!("Successfully imported {} token", token.provider);
        Ok(())
    }

    /// Logout from a provider (delete stored token)
    pub async fn logout(&self, user_id: &str, provider: OAuthProvider) -> AuthResult<()> {
        info!("Logging out from provider: {}", provider);
        self.storage.delete_token(user_id, provider).await?;
        info!("✅ Successfully logged out from {}", provider);
        Ok(())
    }

    /// Get token for user and provider (if exists and valid)
    pub async fn get_token(
        &self,
        user_id: &str,
        provider: OAuthProvider,
    ) -> AuthResult<Option<OAuthToken>> {
        let token = self.storage.get_token(user_id, provider).await?;

        match token {
            Some(token) if token.is_valid() => Ok(Some(token)),
            Some(_token) => {
                // Token is expired or needs refresh - return None
                debug!("Token expired or needs refresh");
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Check authentication status for all providers
    pub async fn get_status(&self, user_id: &str) -> AuthResult<Vec<ProviderStatus>> {
        let mut statuses = Vec::new();

        for provider in OAuthProvider::all() {
            let token = self.storage.get_token(user_id, provider).await?;

            let status = match token {
                Some(token) if token.is_valid() => ProviderStatus {
                    provider,
                    authenticated: true,
                    expires_at: Some(token.expires_at),
                    account_email: token.account_email,
                    subscription_type: token.subscription_type,
                },
                Some(token) => ProviderStatus {
                    provider,
                    authenticated: false, // Expired
                    expires_at: Some(token.expires_at),
                    account_email: token.account_email,
                    subscription_type: token.subscription_type,
                },
                None => ProviderStatus {
                    provider,
                    authenticated: false,
                    expires_at: None,
                    account_email: None,
                    subscription_type: None,
                },
            };

            statuses.push(status);
        }

        Ok(statuses)
    }
}

/// Provider authentication status
#[derive(Debug, Clone)]
pub struct ProviderStatus {
    pub provider: OAuthProvider,
    pub authenticated: bool,
    pub expires_at: Option<i64>,
    pub account_email: Option<String>,
    pub subscription_type: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_status() {
        let status = ProviderStatus {
            provider: OAuthProvider::Claude,
            authenticated: true,
            expires_at: Some(1234567890),
            account_email: Some("test@example.com".to_string()),
            subscription_type: Some("pro".to_string()),
        };

        assert_eq!(status.provider, OAuthProvider::Claude);
        assert!(status.authenticated);
        assert_eq!(status.account_email, Some("test@example.com".to_string()));
    }
}
