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

#[cfg(test)]
mod tests {
    use super::*;

    /// Create a test token with custom expiry
    fn create_test_token(expires_in_seconds: i64) -> OAuthToken {
        OAuthToken {
            id: "test-id".to_string(),
            user_id: "test-user".to_string(),
            provider: "claude".to_string(),
            access_token: "test-access-token".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_at: Utc::now().timestamp() + expires_in_seconds,
            token_type: "Bearer".to_string(),
            scope: Some("model:claude account:read".to_string()),
            subscription_type: Some("pro".to_string()),
            account_email: Some("test@example.com".to_string()),
        }
    }

    #[test]
    fn test_token_valid_within_buffer() {
        // Token expires in 10 minutes (well beyond 5-minute buffer)
        let token = create_test_token(600);
        assert!(token.is_valid());
        assert!(!token.is_expired());
        assert!(!token.needs_refresh());
    }

    #[test]
    fn test_token_needs_refresh_within_buffer() {
        // Token expires in 4 minutes (within 5-minute buffer)
        let token = create_test_token(240);
        assert!(!token.is_valid());
        assert!(token.is_expired());
        assert!(token.needs_refresh());
    }

    #[test]
    fn test_token_needs_refresh_at_buffer_edge() {
        // Token expires in exactly 5 minutes (at buffer edge)
        // Note: < comparison means exactly at buffer is still valid
        let token = create_test_token(300);
        assert!(token.is_valid());
        assert!(!token.is_expired());
        assert!(!token.needs_refresh());
    }

    #[test]
    fn test_token_expired_in_past() {
        // Token expired 1 minute ago
        let token = create_test_token(-60);
        assert!(!token.is_valid());
        assert!(token.is_expired());
        assert!(token.needs_refresh());
    }

    #[test]
    fn test_token_just_outside_buffer() {
        // Token expires in 6 minutes (just outside 5-minute buffer)
        let token = create_test_token(360);
        assert!(token.is_valid());
        assert!(!token.is_expired());
        assert!(!token.needs_refresh());
    }

    #[test]
    fn test_token_far_future() {
        // Token expires in 1 hour
        let token = create_test_token(3600);
        assert!(token.is_valid());
        assert!(!token.is_expired());
        assert!(!token.needs_refresh());
    }

    #[test]
    fn test_token_expired_consistency() {
        // is_expired() and is_valid() should be opposite
        let valid_token = create_test_token(600);
        assert_eq!(valid_token.is_expired(), !valid_token.is_valid());

        let expired_token = create_test_token(240);
        assert_eq!(expired_token.is_expired(), !expired_token.is_valid());
    }

    #[test]
    fn test_token_needs_refresh_matches_expired() {
        // needs_refresh() and is_expired() should always match
        // (both use same logic with 5-minute buffer)
        let token1 = create_test_token(600);
        assert_eq!(token1.needs_refresh(), token1.is_expired());

        let token2 = create_test_token(240);
        assert_eq!(token2.needs_refresh(), token2.is_expired());

        let token3 = create_test_token(300);
        assert_eq!(token3.needs_refresh(), token3.is_expired());
    }

    #[test]
    fn test_token_refresh_buffer_seconds() {
        // Verify the buffer is exactly 5 minutes (300 seconds)
        let buffer_seconds = Duration::minutes(5).num_seconds();
        assert_eq!(buffer_seconds, 300);

        // Token expiring at buffer_seconds + 1 should be valid
        let token_valid = create_test_token(buffer_seconds + 1);
        assert!(token_valid.is_valid());

        // Token expiring at exactly buffer_seconds is still valid (< not <=)
        let token_at_edge = create_test_token(buffer_seconds);
        assert!(token_at_edge.is_valid());

        // Token expiring at buffer_seconds - 1 should need refresh
        let token_expired = create_test_token(buffer_seconds - 1);
        assert!(token_expired.needs_refresh());
    }
}
