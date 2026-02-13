// ABOUTME: HTTP request handlers for OAuth authentication
// ABOUTME: Provides endpoints for managing OAuth tokens and provider authentication

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::auth::CurrentUser;
use super::response::{bad_request, ok_or_internal_error};
use orkee_auth::oauth::types::OAuthToken;
use orkee_auth::oauth::OAuthProvider;
use orkee_auth::OAuthManager;
use orkee_projects::DbState;

/// 1-year token validity for imported Claude tokens
const CLAUDE_TOKEN_VALIDITY_SECONDS: i64 = 365 * 24 * 60 * 60;

/// Response for authentication status
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatusResponse {
    pub providers: Vec<ProviderStatusResponse>,
}

/// Provider authentication status
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProviderStatusResponse {
    pub provider: String,
    pub authenticated: bool,
    pub expires_at: Option<i64>,
    pub account_email: Option<String>,
    pub subscription_type: Option<String>,
}

/// Response for token endpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenResponse {
    pub token: String,
    pub expires_at: i64,
}

/// List available OAuth providers
pub async fn list_providers() -> impl IntoResponse {
    info!("Listing available OAuth providers");

    let providers = vec![
        serde_json::json!({
            "id": "claude",
            "name": "Claude",
            "description": "Anthropic Claude (Pro/Max subscriptions)",
            "scopes": ["model:claude", "account:read"],
        }),
        serde_json::json!({
            "id": "openai",
            "name": "OpenAI",
            "description": "OpenAI GPT models",
            "scopes": ["model.read", "model.request"],
        }),
        serde_json::json!({
            "id": "google",
            "name": "Google AI",
            "description": "Google AI Platform",
            "scopes": ["https://www.googleapis.com/auth/cloud-platform"],
        }),
        serde_json::json!({
            "id": "xai",
            "name": "xAI",
            "description": "xAI Grok models",
            "scopes": ["models:read", "models:write"],
        }),
    ];

    ok_or_internal_error::<Vec<serde_json::Value>, std::convert::Infallible>(
        Ok(providers),
        "Failed to list providers",
    )
}

/// Get authentication status for all providers
pub async fn get_auth_status(
    State(db): State<DbState>,
    CurrentUser { id }: CurrentUser,
) -> impl IntoResponse {
    info!("Getting authentication status for user: {}", id);

    let manager = match OAuthManager::new(db.pool.clone()) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to initialize OAuth manager: {}", e);
            return ok_or_internal_error(
                Err::<AuthStatusResponse, _>(e),
                "Failed to initialize OAuth manager",
            );
        }
    };

    let result = manager.get_status(&id).await.map(|statuses| {
        let providers: Vec<ProviderStatusResponse> = statuses
            .into_iter()
            .map(|s| ProviderStatusResponse {
                provider: s.provider.to_string(),
                authenticated: s.authenticated,
                expires_at: s.expires_at,
                account_email: s.account_email,
                subscription_type: s.subscription_type,
            })
            .collect();

        AuthStatusResponse { providers }
    });

    ok_or_internal_error(result, "Failed to get authentication status")
}

/// Get current access token for a provider
pub async fn get_token(
    State(db): State<DbState>,
    Path(provider): Path<String>,
    CurrentUser { id }: CurrentUser,
) -> impl IntoResponse {
    info!("Getting token for provider: {} (user: {})", provider, id);

    let provider = match parse_provider(&provider) {
        Ok(p) => p,
        Err(e) => {
            error!("Invalid provider: {}", e);
            return ok_or_internal_error(Err::<TokenResponse, _>(e), "Invalid provider");
        }
    };

    let manager = match OAuthManager::new(db.pool.clone()) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to initialize OAuth manager: {}", e);
            return ok_or_internal_error(
                Err::<TokenResponse, _>(e),
                "Failed to initialize OAuth manager",
            );
        }
    };

    let result = manager
        .get_token(&id, provider)
        .await
        .and_then(|token_opt| {
            token_opt.ok_or_else(|| {
                orkee_auth::AuthError::TokenNotFound(format!(
                    "No valid token found for {}",
                    provider
                ))
            })
        })
        .map(|token| TokenResponse {
            token: token.access_token,
            expires_at: token.expires_at,
        });

    ok_or_internal_error(result, "Failed to get token")
}

/// Refresh token for a provider
pub async fn refresh_token(
    State(_db): State<DbState>,
    Path(provider): Path<String>,
    CurrentUser { id }: CurrentUser,
) -> impl IntoResponse {
    info!(
        "Refresh token requested for provider: {} (user: {})",
        provider, id
    );

    // Token refresh is not yet implemented
    // Claude tokens cannot be refreshed - they must be re-imported
    // Other providers would need OAuth implementation
    error!("Token refresh not yet implemented");
    ok_or_internal_error(
        Err::<TokenResponse, _>("Token refresh not yet implemented".to_string()),
        "Token refresh not yet implemented",
    )
}

/// Logout from a provider (delete stored token)
pub async fn logout(
    State(db): State<DbState>,
    Path(provider): Path<String>,
    CurrentUser { id }: CurrentUser,
) -> impl IntoResponse {
    info!("Logging out from provider: {} (user: {})", provider, id);

    let provider = match parse_provider(&provider) {
        Ok(p) => p,
        Err(e) => {
            error!("Invalid provider: {}", e);
            return ok_or_internal_error(Err::<serde_json::Value, _>(e), "Invalid provider");
        }
    };

    let manager = match OAuthManager::new(db.pool.clone()) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to initialize OAuth manager: {}", e);
            return ok_or_internal_error(
                Err::<serde_json::Value, _>(e),
                "Failed to initialize OAuth manager",
            );
        }
    };

    let result = manager.logout(&id, provider).await.map(
        |_| serde_json::json!({ "message": format!("Successfully logged out from {}", provider) }),
    );

    ok_or_internal_error(result, "Failed to logout")
}

/// Request body for token import
#[derive(Debug, Deserialize)]
pub struct ImportTokenRequest {
    pub token: String,
}

/// Import an OAuth token for a provider
pub async fn import_token(
    State(db): State<DbState>,
    Path(provider): Path<String>,
    CurrentUser { id }: CurrentUser,
    Json(body): Json<ImportTokenRequest>,
) -> impl IntoResponse {
    info!("Importing token for provider: {} (user: {})", provider, id);

    let provider_enum = match parse_provider(&provider) {
        Ok(p) => p,
        Err(e) => {
            error!("Invalid provider: {}", e);
            return bad_request(e, "Invalid provider");
        }
    };

    let token = body.token.trim().to_string();
    if token.is_empty() {
        return bad_request("Token cannot be empty", "Invalid token");
    }

    // Generate a stable ID from the token prefix
    let token_id = if token.len() >= 21 {
        token[..21].to_string()
    } else {
        token.clone()
    };

    let oauth_token = OAuthToken {
        id: token_id,
        user_id: id.clone(),
        provider: provider.to_lowercase(),
        access_token: token,
        refresh_token: None,
        expires_at: Utc::now().timestamp() + CLAUDE_TOKEN_VALIDITY_SECONDS,
        token_type: "Bearer".to_string(),
        scope: match provider_enum {
            OAuthProvider::Claude => Some("model:claude account:read".to_string()),
            _ => None,
        },
        subscription_type: None,
        account_email: None,
    };

    let manager = match OAuthManager::new(db.pool.clone()) {
        Ok(m) => m,
        Err(e) => {
            error!("Failed to initialize OAuth manager: {}", e);
            return ok_or_internal_error(
                Err::<serde_json::Value, _>(e),
                "Failed to initialize OAuth manager",
            );
        }
    };

    let result = manager.import_token(oauth_token).await.map(|_| {
        serde_json::json!({ "message": format!("Successfully imported {} token", provider) })
    });

    ok_or_internal_error(result, "Failed to import token")
}

/// Parse provider string into OAuthProvider enum
fn parse_provider(provider: &str) -> Result<OAuthProvider, orkee_auth::AuthError> {
    match provider.to_lowercase().as_str() {
        "claude" => Ok(OAuthProvider::Claude),
        "openai" => Ok(OAuthProvider::OpenAI),
        "google" => Ok(OAuthProvider::Google),
        "xai" => Ok(OAuthProvider::XAI),
        _ => Err(orkee_auth::AuthError::Configuration(format!(
            "Invalid provider: {}. Valid providers are: claude, openai, google, xai",
            provider
        ))),
    }
}
