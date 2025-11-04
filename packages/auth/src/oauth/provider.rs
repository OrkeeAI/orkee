// ABOUTME: OAuth provider definitions and configurations for AI services
// ABOUTME: Supports Claude, OpenAI, Google, and xAI with provider-specific URLs and scopes

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

use crate::error::{AuthError, AuthResult};

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Claude,
    OpenAI,
    Google,
    XAI,
}

impl OAuthProvider {
    /// Get authorization URL for this provider
    pub fn auth_url(&self) -> &str {
        match self {
            Self::Claude => "https://console.anthropic.com/oauth/authorize",
            Self::OpenAI => "https://platform.openai.com/oauth/authorize",
            Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            Self::XAI => "https://x.ai/oauth/authorize",
        }
    }

    /// Get token exchange URL for this provider
    pub fn token_url(&self) -> &str {
        match self {
            Self::Claude => "https://api.anthropic.com/oauth/token",
            Self::OpenAI => "https://api.openai.com/oauth/token",
            Self::Google => "https://oauth2.googleapis.com/token",
            Self::XAI => "https://api.x.ai/oauth/token",
        }
    }

    /// Get default scopes for this provider
    pub fn scopes(&self) -> &[&str] {
        match self {
            Self::Claude => &["model:claude", "account:read"],
            Self::OpenAI => &["model.read", "model.request"],
            Self::Google => &["https://www.googleapis.com/auth/cloud-platform"],
            Self::XAI => &["models:read", "models:write"],
        }
    }

    /// Get client ID for this provider
    /// Note: This should be configured via environment or database
    pub fn default_client_id(&self) -> &str {
        match self {
            Self::Claude => "orkee-cli-claude",
            Self::OpenAI => "orkee-cli-openai",
            Self::Google => "orkee-cli-google",
            Self::XAI => "orkee-cli-xai",
        }
    }

    /// Get all supported providers
    pub fn all() -> Vec<Self> {
        vec![Self::Claude, Self::OpenAI, Self::Google, Self::XAI]
    }
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

    fn from_str(s: &str) -> AuthResult<Self> {
        match s.to_lowercase().as_str() {
            "claude" => Ok(Self::Claude),
            "openai" => Ok(Self::OpenAI),
            "google" => Ok(Self::Google),
            "xai" => Ok(Self::XAI),
            _ => Err(AuthError::Configuration(format!(
                "Unknown provider: {}. Supported: claude, openai, google, xai",
                s
            ))),
        }
    }
}

impl TryFrom<String> for OAuthProvider {
    type Error = AuthError;

    fn try_from(s: String) -> AuthResult<Self> {
        s.parse()
    }
}

impl TryFrom<&str> for OAuthProvider {
    type Error = AuthError;

    fn try_from(s: &str) -> AuthResult<Self> {
        s.parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_parsing() {
        assert_eq!("claude".parse::<OAuthProvider>().unwrap(), OAuthProvider::Claude);
        assert_eq!("CLAUDE".parse::<OAuthProvider>().unwrap(), OAuthProvider::Claude);
        assert_eq!("openai".parse::<OAuthProvider>().unwrap(), OAuthProvider::OpenAI);
        assert!("invalid".parse::<OAuthProvider>().is_err());
    }

    #[test]
    fn test_provider_urls() {
        let claude = OAuthProvider::Claude;
        assert!(claude.auth_url().contains("anthropic"));
        assert!(claude.token_url().contains("api.anthropic.com"));
    }

    #[test]
    fn test_provider_display() {
        assert_eq!(OAuthProvider::Claude.to_string(), "claude");
        assert_eq!(OAuthProvider::OpenAI.to_string(), "openai");
    }
}
