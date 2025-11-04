// ABOUTME: Core type definitions for OAuth authentication
// ABOUTME: Includes OAuth tokens, provider configurations, and PKCE challenge types

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// OAuth token information stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub access_token: String,  // Encrypted in database
    pub refresh_token: Option<String>,  // Encrypted in database
    pub expires_at: i64,  // Unix timestamp
    pub token_type: String,
    pub scope: Option<String>,
    pub subscription_type: Option<String>,
    pub account_email: Option<String>,
}

impl OAuthToken {
    /// Check if token is expired with 5-minute buffer
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        let buffer = Duration::minutes(5).num_seconds();
        self.expires_at < now + buffer
    }

    /// Check if token is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Check if token needs refresh (within 5-minute buffer)
    pub fn needs_refresh(&self) -> bool {
        let now = Utc::now().timestamp();
        let buffer = Duration::minutes(5).num_seconds();
        self.expires_at < now + buffer
    }
}

/// OAuth provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProviderConfig {
    pub provider: String,
    pub client_id: String,
    pub client_secret: Option<String>,  // Encrypted in database
    pub auth_url: String,
    pub token_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub enabled: bool,
}

/// PKCE challenge for OAuth flow
#[derive(Debug, Clone)]
pub struct PkceChallenge {
    pub code_verifier: String,
    pub code_challenge: String,
    pub code_challenge_method: String,  // Usually "S256"
}

/// OAuth state for CSRF protection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub state: String,
    pub provider: String,
    pub pkce_verifier: String,
    pub created_at: DateTime<Utc>,
}

/// OAuth authorization code exchange request
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenExchangeRequest {
    pub code: String,
    pub code_verifier: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub grant_type: String,  // Usually "authorization_code"
}

/// OAuth token response from provider
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,  // Seconds
    pub token_type: String,
    pub scope: Option<String>,
}

/// OAuth refresh token request
#[derive(Debug, Serialize, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
    pub client_id: String,
    pub grant_type: String,  // "refresh_token"
}
