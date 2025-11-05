// ABOUTME: Orkee authentication library for AI provider token management
// ABOUTME: Provides secure token storage and management for Claude, OpenAI, Google, and xAI

pub mod error;
pub mod oauth;

// Re-export main types
pub use error::{AuthError, AuthResult};
pub use oauth::{OAuthManager, OAuthStorage, OAuthToken, ProviderStatus};
