// ABOUTME: OAuth module providing authentication flows for AI providers
// ABOUTME: Includes PKCE, callback server, token storage, and provider configurations

pub mod pkce;
pub mod provider;
pub mod server;
pub mod storage;
pub mod types;

pub use provider::OAuthProvider;
pub use server::CallbackServer;
pub use storage::OAuthStorage;
pub use types::{OAuthToken, OAuthProviderConfig, PkceChallenge, TokenResponse};
