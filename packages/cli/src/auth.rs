// ABOUTME: Authentication context for API requests
// ABOUTME: Provides user identification for request handlers

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// Current authenticated user
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
    pub authenticated: bool,
}

impl CurrentUser {
    /// Get the default user ID for single-user desktop application
    fn default_user() -> Self {
        Self {
            id: "default-user".to_string(),
            authenticated: false,
        }
    }

    /// Get authenticated user
    fn authenticated_user() -> Self {
        Self {
            id: "default-user".to_string(),
            authenticated: true,
        }
    }
}

impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        // Check if API token authentication was successful
        // The api_token_middleware sets a boolean extension when auth succeeds
        let authenticated = parts.extensions.get::<bool>().copied().unwrap_or(false);

        if authenticated {
            Ok(Self::authenticated_user())
        } else {
            Ok(Self::default_user())
        }
    }
}
