// ABOUTME: Orkee authentication library providing OAuth flows for AI providers
// ABOUTME: Supports Claude, OpenAI, Google, and xAI with PKCE and secure token storage

pub mod error;
pub mod oauth;

// Re-export main types
pub use error::{AuthError, AuthResult};
pub use oauth::{
    CallbackServer, OAuthManager, OAuthProvider, OAuthProviderConfig, OAuthStorage, OAuthToken,
    PkceChallenge, ProviderStatus, TokenResponse,
};
