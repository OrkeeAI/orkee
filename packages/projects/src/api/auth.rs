// ABOUTME: Authentication context for API requests
// ABOUTME: Provides user identification for request handlers

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

/// Current authenticated user
#[derive(Debug, Clone)]
pub struct CurrentUser {
    pub id: String,
}

impl CurrentUser {
    /// Get the default user ID for single-user desktop application
    fn default_user() -> Self {
        Self {
            id: "default-user".to_string(),
        }
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, &'static str);

    async fn from_request_parts(_parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self::default_user())
    }
}
