// ABOUTME: OAuth module providing authentication flows for AI providers
// ABOUTME: Includes PKCE, callback server, token storage, and provider configurations

pub mod manager;
pub mod pkce;
pub mod provider;
pub mod server;
pub mod storage;
pub mod types;

pub use manager::{OAuthManager, ProviderStatus};
pub use provider::OAuthProvider;
pub use server::CallbackServer;
pub use storage::OAuthStorage;
pub use types::{OAuthProviderConfig, OAuthToken, PkceChallenge, TokenResponse};
