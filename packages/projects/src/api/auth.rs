// ABOUTME: Authentication context for API requests
// ABOUTME: Provides user identification for request handlers
//
// IMPORTANT USAGE NOTE:
//
// This extractor should be used consistently across ALL user-related API endpoints.
// Currently there is an inconsistency in the codebase:
//
// - `update_credentials` (users_handlers.rs) correctly uses CurrentUser extractor
// - `get_current_user` (users_handlers.rs) does NOT use CurrentUser extractor
//
// For localhost-only APIs in desktop mode, this inconsistency has no security impact
// since all requests are treated as the default user. However, for consistency and
// future-proofing (e.g., if authentication is added), ALL user endpoints should use
// the CurrentUser extractor pattern.
//
// Recommended pattern:
// ```rust
// pub async fn handler(
//     State(db): State<DbState>,
//     current_user: CurrentUser,  // <-- Always use this
// ) -> impl IntoResponse {
//     // Use current_user.id instead of hardcoded "default-user"
// }
// ```

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
