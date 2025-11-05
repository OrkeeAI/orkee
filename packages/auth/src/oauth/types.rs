// ABOUTME: Core type definitions for OAuth authentication
// ABOUTME: Includes OAuth tokens, provider configurations, and type definitions

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::AuthError;

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Claude,
    OpenAI,
    Google,
    XAI,
}

impl fmt::Display for OAuthProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Claude => write!(f, "claude"),
            Self::OpenAI => write!(f, "openai"),
            Self::Google => write!(f, "google"),
            Self::XAI => write!(f, "xai"),
        }
    }
}

impl FromStr for OAuthProvider {
    type Err = AuthError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "openai" => Ok(Self::OpenAI),
            "google" => Ok(Self::Google),
            "xai" => Ok(Self::XAI),
            _ => Err(AuthError::InvalidProvider(format!("Unknown provider: {}", s))),
        }
    }
}

impl OAuthProvider {
    /// Get all supported providers
    pub fn all() -> Vec<Self> {
        vec![Self::Claude, Self::OpenAI, Self::Google, Self::XAI]
    }
}

/// OAuth token information stored in database
#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthToken {
    pub id: String,
    pub user_id: String,
    pub provider: String,
    pub access_token: String,          // Encrypted in database
    pub refresh_token: Option<String>, // Encrypted in database
    pub expires_at: i64,               // Unix timestamp
    pub token_type: String,
    pub scope: Option<String>,
    pub subscription_type: Option<String>,
    pub account_email: Option<String>,
}

impl fmt::Debug for OAuthToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OAuthToken")
            .field("id", &self.id)
            .field("user_id", &self.user_id)
            .field("provider", &self.provider)
            .field("access_token", &"[REDACTED]")
            .field("refresh_token", &"[REDACTED]")
            .field("expires_at", &self.expires_at)
            .field("token_type", &self.token_type)
            .field("scope", &self.scope)
            .field("subscription_type", &self.subscription_type)
            .field("account_email", &self.account_email)
            .finish()
    }
}

impl OAuthToken {
    /// Check if token is expired (no buffer)
    pub fn is_expired(&self) -> bool {
        let now = Utc::now().timestamp();
        self.expires_at < now
    }

    /// Check if token is valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }

    /// Check if token needs refresh (within 5-minute buffer before expiry)
    pub fn needs_refresh(&self) -> bool {
        let now = Utc::now().timestamp();
        let buffer = Duration::minutes(5).num_seconds();
        self.expires_at < now + buffer
    }
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
        // Token is still valid but needs refresh
        let token = create_test_token(240);
        assert!(token.is_valid()); // Still valid
        assert!(!token.is_expired()); // Not expired yet
        assert!(token.needs_refresh()); // But should refresh soon
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
        // is_expired() and is_valid() should always be opposite
        let valid_token = create_test_token(600);
        assert_eq!(valid_token.is_expired(), !valid_token.is_valid());

        let expired_token = create_test_token(-60);
        assert_eq!(expired_token.is_expired(), !expired_token.is_valid());
    }

    #[test]
    fn test_token_needs_refresh_independent_of_expired() {
        // needs_refresh() uses buffer, is_expired() does not
        // Token with 4 minutes left: not expired but needs refresh
        let token1 = create_test_token(240);
        assert!(!token1.is_expired()); // Not expired
        assert!(token1.needs_refresh()); // But needs refresh

        // Token with 10 minutes left: not expired and no refresh needed
        let token2 = create_test_token(600);
        assert!(!token2.is_expired());
        assert!(!token2.needs_refresh());

        // Token expired 1 minute ago: expired and needs refresh
        let token3 = create_test_token(-60);
        assert!(token3.is_expired());
        assert!(token3.needs_refresh());
    }

    #[test]
    fn test_token_refresh_buffer_seconds() {
        // Verify the buffer is exactly 5 minutes (300 seconds)
        let buffer_seconds = Duration::minutes(5).num_seconds();
        assert_eq!(buffer_seconds, 300);

        // Token expiring at buffer_seconds + 1 should be valid and not need refresh
        let token_valid = create_test_token(buffer_seconds + 1);
        assert!(token_valid.is_valid());
        assert!(!token_valid.needs_refresh());

        // Token expiring at exactly buffer_seconds is still valid (< not <=)
        let token_at_edge = create_test_token(buffer_seconds);
        assert!(token_at_edge.is_valid());
        assert!(!token_at_edge.needs_refresh());

        // Token expiring at buffer_seconds - 1 should still be valid but need refresh
        let token_needs_refresh = create_test_token(buffer_seconds - 1);
        assert!(token_needs_refresh.is_valid());
        assert!(token_needs_refresh.needs_refresh());
    }
}
