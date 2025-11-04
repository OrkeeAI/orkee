// ABOUTME: OAuth manager orchestrating complete authentication flows
// ABOUTME: Handles login, logout, refresh, and status for all AI providers

use chrono::Utc;
use reqwest::Client;
use sqlx::SqlitePool;
use tracing::{debug, error, info};
use url::Url;

use crate::{
    error::{AuthError, AuthResult},
    oauth::{
        pkce::generate_pkce_challenge,
        provider::OAuthProvider,
        server::CallbackServer,
        storage::OAuthStorage,
        types::{OAuthToken, RefreshTokenRequest, TokenExchangeRequest, TokenResponse},
    },
};

/// OAuth manager for handling authentication flows
pub struct OAuthManager {
    storage: OAuthStorage,
    client: Client,
}

impl OAuthManager {
    /// Create a new OAuth manager with database pool
    pub fn new(pool: SqlitePool) -> AuthResult<Self> {
        let storage = OAuthStorage::new(pool)?;
        Ok(Self {
            storage,
            client: Client::new(),
        })
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

    /// Authenticate with a provider using OAuth flow
    ///
    /// This will:
    /// 1. Generate PKCE challenge
    /// 2. Generate CSRF state parameter
    /// 3. Build authorization URL
    /// 4. Open browser for user consent
    /// 5. Start callback server
    /// 6. Validate state parameter (CSRF protection)
    /// 7. Exchange authorization code for token
    /// 8. Store encrypted token
    pub async fn authenticate(
        &self,
        provider: OAuthProvider,
        user_id: &str,
    ) -> AuthResult<OAuthToken> {
        info!("Starting OAuth authentication for provider: {}", provider);

        // Generate PKCE challenge
        let pkce = generate_pkce_challenge()?;
        debug!("Generated PKCE challenge");

        // Generate state parameter for CSRF protection
        let expected_state = nanoid::nanoid!();
        debug!("Generated state parameter for CSRF protection");

        // Get or create provider config
        let config = self
            .storage
            .get_provider_config(provider)
            .await?
            .unwrap_or_else(|| self.default_provider_config(provider));

        // Build authorization URL with state parameter
        let auth_url = self.build_auth_url(&config, &pkce, &expected_state)?;
        info!("Opening browser for authentication: {}", auth_url);

        // Open browser
        if let Err(e) = open::that(&auth_url) {
            error!("Failed to open browser: {}", e);
            return Err(AuthError::BrowserOpen(format!(
                "Failed to open browser. Please manually visit: {}",
                auth_url
            )));
        }

        // Start callback server and wait for authorization code and state
        let server = CallbackServer::new();
        let (auth_code, returned_state) = server.wait_for_callback().await?;

        // Validate state parameter (CSRF protection)
        if returned_state != expected_state {
            error!(
                "State mismatch: expected {}, got {}",
                expected_state, returned_state
            );
            return Err(AuthError::StateMismatch);
        }

        info!("✅ State validated successfully");
        info!("Received authorization code, exchanging for token");

        // Exchange code for token
        let token_response = self
            .exchange_code_for_token(&config, &auth_code, &pkce.code_verifier)
            .await?;

        // Get subscription type (if supported by provider)
        let subscription_type = self
            .detect_subscription_type(provider, &token_response)
            .await;

        // Get account email (if available from token info)
        let account_email = self.get_account_email(provider, &token_response).await;

        // Create token record
        let token = OAuthToken {
            id: nanoid::nanoid!(),
            user_id: user_id.to_string(),
            provider: provider.to_string(),
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token,
            expires_at: Utc::now().timestamp() + token_response.expires_in,
            token_type: token_response.token_type,
            scope: token_response.scope,
            subscription_type,
            account_email,
        };

        // Store token (encrypted)
        self.storage.store_token(&token).await?;

        info!("✅ Successfully authenticated with {}", provider);
        Ok(token)
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
            Some(token) if token.needs_refresh() => {
                // Token needs refresh
                debug!("Token needs refresh");
                match self.refresh_token(user_id, provider).await {
                    Ok(refreshed) => Ok(Some(refreshed)),
                    Err(e) => {
                        error!("Failed to refresh token: {}", e);
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// Refresh an expired token
    pub async fn refresh_token(
        &self,
        user_id: &str,
        provider: OAuthProvider,
    ) -> AuthResult<OAuthToken> {
        info!("Refreshing token for provider: {}", provider);

        // Get existing token
        let existing_token = self
            .storage
            .get_token(user_id, provider)
            .await?
            .ok_or_else(|| AuthError::TokenNotFound(format!("No token found for {}", provider)))?;

        // Check if refresh token exists
        let refresh_token = existing_token
            .refresh_token
            .ok_or_else(|| AuthError::RefreshFailed("No refresh token available".to_string()))?;

        // Get provider config
        let config = self
            .storage
            .get_provider_config(provider)
            .await?
            .unwrap_or_else(|| self.default_provider_config(provider));

        // Build refresh request
        let request = RefreshTokenRequest {
            refresh_token: refresh_token.clone(),
            client_id: config.client_id.clone(),
            grant_type: "refresh_token".to_string(),
        };

        // Exchange refresh token for new access token
        let response = self
            .client
            .post(&config.token_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::TokenExchange(format!("Failed to refresh token: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            // Don't leak full response body - only log status for security
            error!("Token refresh failed with status {}", status);
            return Err(AuthError::TokenExchange(format!(
                "Token refresh failed with status {}",
                status
            )));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            AuthError::TokenExchange(format!("Failed to parse token response: {}", e))
        })?;

        // Create new token record
        let new_token = OAuthToken {
            id: existing_token.id,
            user_id: user_id.to_string(),
            provider: provider.to_string(),
            access_token: token_response.access_token,
            refresh_token: token_response.refresh_token.or(Some(refresh_token)),
            expires_at: Utc::now().timestamp() + token_response.expires_in,
            token_type: token_response.token_type,
            scope: token_response.scope,
            subscription_type: existing_token.subscription_type,
            account_email: existing_token.account_email,
        };

        // Store refreshed token
        self.storage.store_token(&new_token).await?;

        info!("✅ Successfully refreshed token for {}", provider);
        Ok(new_token)
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

    /// Build authorization URL with PKCE challenge and state parameter
    fn build_auth_url(
        &self,
        config: &crate::oauth::types::OAuthProviderConfig,
        pkce: &crate::oauth::types::PkceChallenge,
        state: &str,
    ) -> AuthResult<String> {
        let mut url = Url::parse(&config.auth_url)
            .map_err(|e| AuthError::Configuration(format!("Invalid auth URL: {}", e)))?;

        url.query_pairs_mut()
            .append_pair("client_id", &config.client_id)
            .append_pair("redirect_uri", &config.redirect_uri)
            .append_pair("response_type", "code")
            .append_pair("scope", &config.scopes.join(" "))
            .append_pair("code_challenge", &pkce.code_challenge)
            .append_pair("code_challenge_method", &pkce.code_challenge_method)
            .append_pair("state", state);

        Ok(url.to_string())
    }

    /// Exchange authorization code for access token
    async fn exchange_code_for_token(
        &self,
        config: &crate::oauth::types::OAuthProviderConfig,
        code: &str,
        code_verifier: &str,
    ) -> AuthResult<TokenResponse> {
        let request = TokenExchangeRequest {
            code: code.to_string(),
            code_verifier: code_verifier.to_string(),
            redirect_uri: config.redirect_uri.clone(),
            client_id: config.client_id.clone(),
            grant_type: "authorization_code".to_string(),
        };

        let response = self
            .client
            .post(&config.token_url)
            .json(&request)
            .send()
            .await
            .map_err(|e| AuthError::TokenExchange(format!("Failed to exchange code: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            // Don't leak full response body - only log status for security
            error!("Token exchange failed with status {}", status);
            return Err(AuthError::TokenExchange(format!(
                "Token exchange failed with status {}",
                status
            )));
        }

        let token_response: TokenResponse = response.json().await.map_err(|e| {
            AuthError::TokenExchange(format!("Failed to parse token response: {}", e))
        })?;

        Ok(token_response)
    }

    /// Detect subscription type for provider (if supported)
    async fn detect_subscription_type(
        &self,
        _provider: OAuthProvider,
        _token_response: &TokenResponse,
    ) -> Option<String> {
        None
    }

    /// Get account email from token info (if available)
    async fn get_account_email(
        &self,
        _provider: OAuthProvider,
        _token_response: &TokenResponse,
    ) -> Option<String> {
        None
    }

    /// Get default provider configuration
    fn default_provider_config(
        &self,
        provider: OAuthProvider,
    ) -> crate::oauth::types::OAuthProviderConfig {
        let server = CallbackServer::new();
        crate::oauth::types::OAuthProviderConfig {
            provider: provider.to_string(),
            client_id: provider.default_client_id().to_string(),
            client_secret: None,
            auth_url: provider.auth_url().to_string(),
            token_url: provider.token_url().to_string(),
            redirect_uri: server.callback_url(),
            scopes: provider.scopes().iter().map(|s| s.to_string()).collect(),
            enabled: true,
        }
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
