// ABOUTME: Error types for authentication and OAuth operations
// ABOUTME: Provides detailed error handling for OAuth flows, token management, and provider interactions

use thiserror::Error;

pub type AuthResult<T> = Result<T, AuthError>;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("OAuth authentication failed: {0}")]
    OAuthFailed(String),

    #[error("Token expired or invalid")]
    TokenExpired,

    #[error("Provider error: {0}")]
    Provider(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Invalid configuration: {0}")]
    Configuration(String),

    #[error("PKCE error: {0}")]
    Pkce(String),

    #[error("Callback server error: {0}")]
    CallbackServer(String),

    #[error("State mismatch: CSRF protection failed")]
    StateMismatch,

    #[error("Browser open failed: {0}")]
    BrowserFailed(String),

    #[error("Failed to open browser: {0}")]
    BrowserOpen(String),

    #[error("Token not found: {0}")]
    TokenNotFound(String),

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Invalid provider: {0}")]
    InvalidProvider(String),

    #[error("Token refresh failed: {0}")]
    RefreshFailed(String),

    #[error("Token exchange failed: {0}")]
    TokenExchange(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP request error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
