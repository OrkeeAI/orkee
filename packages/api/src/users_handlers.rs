// ABOUTME: HTTP request handlers for user operations
// ABOUTME: Handles user settings and preferences management

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::auth::CurrentUser;
use super::response::ok_or_internal_error;
use orkee_projects::DbState;
use orkee_security::users::{MaskedUser, UserUpdateInput};

/// Get current user (with masked credentials)
pub async fn get_current_user(State(db): State<DbState>) -> impl IntoResponse {
    info!("Getting current user");

    let result = db.user_storage.get_current_user().await.map(|user| {
        let masked: MaskedUser = user.into();
        masked
    });

    ok_or_internal_error(result, "Failed to get current user")
}

/// Get user by ID
pub async fn get_user(State(db): State<DbState>, Path(user_id): Path<String>) -> impl IntoResponse {
    info!("Getting user: {}", user_id);

    let result = db.user_storage.get_user(&user_id).await;
    ok_or_internal_error(result, "Failed to get user")
}

/// Request body for setting default agent
#[derive(Deserialize)]
pub struct SetDefaultAgentRequest {
    #[serde(rename = "agentId")]
    pub agent_id: String,
}

/// Set user's default agent
pub async fn set_default_agent(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
    Json(request): Json<SetDefaultAgentRequest>,
) -> impl IntoResponse {
    info!(
        "Setting default agent {} for user {}",
        request.agent_id, user_id
    );

    let result = db
        .user_storage
        .set_default_agent(&user_id, &request.agent_id)
        .await
        .map(|_| serde_json::json!({"message": "Default agent updated successfully"}));

    ok_or_internal_error(result, "Failed to set default agent")
}

/// Request body for updating theme
#[derive(Deserialize)]
pub struct UpdateThemeRequest {
    pub theme: String,
}

/// Update user's theme preference
pub async fn update_theme(
    State(db): State<DbState>,
    Path(user_id): Path<String>,
    Json(request): Json<UpdateThemeRequest>,
) -> impl IntoResponse {
    info!("Updating theme for user {}", user_id);

    let result = db
        .user_storage
        .update_theme(&user_id, &request.theme)
        .await
        .map(|_| serde_json::json!({"message": "Theme updated successfully"}));

    ok_or_internal_error(result, "Failed to update theme")
}

/// Request body for updating credentials
#[derive(Deserialize)]
pub struct UpdateCredentialsRequest {
    #[serde(rename = "openaiApiKey")]
    pub openai_api_key: Option<String>,
    #[serde(rename = "anthropicApiKey")]
    pub anthropic_api_key: Option<String>,
    #[serde(rename = "googleApiKey")]
    pub google_api_key: Option<String>,
    #[serde(rename = "xaiApiKey")]
    pub xai_api_key: Option<String>,
    #[serde(rename = "aiGatewayEnabled")]
    pub ai_gateway_enabled: Option<bool>,
    #[serde(rename = "aiGatewayUrl")]
    pub ai_gateway_url: Option<String>,
    #[serde(rename = "aiGatewayKey")]
    pub ai_gateway_key: Option<String>,
}

/// Update user's API credentials and gateway configuration
pub async fn update_credentials(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(request): Json<UpdateCredentialsRequest>,
) -> impl IntoResponse {
    info!("Updating user credentials");

    // Check if any API keys are being saved
    let has_api_keys = request.openai_api_key.is_some()
        || request.anthropic_api_key.is_some()
        || request.google_api_key.is_some()
        || request.xai_api_key.is_some()
        || request.ai_gateway_key.is_some();

    if has_api_keys {
        // Check current encryption mode and log appropriate warning
        // Query encryption_settings table directly
        let encryption_mode_result: Result<Option<(String,)>, sqlx::Error> =
            sqlx::query_as("SELECT encryption_mode FROM encryption_settings WHERE id = 1")
                .fetch_optional(&db.pool)
                .await;

        match encryption_mode_result {
            Ok(Some((mode_str,))) if mode_str == "password" => {
                info!("✓ API keys being saved with PASSWORD-BASED encryption (at-rest encryption)");
            }
            Ok(Some((mode_str,))) if mode_str == "machine" => {
                tracing::warn!(
                    "⚠️  API keys being saved with MACHINE-BASED encryption (transport encryption only)"
                );
                tracing::warn!("   Machine-based encryption does NOT protect API keys at-rest on local machine");
                tracing::warn!(
                    "   Anyone with database file access can decrypt keys on this machine"
                );
                tracing::warn!(
                    "   RECOMMENDATION: Upgrade to password-based encryption with 'orkee security set-password'"
                );
            }
            Ok(None) => {
                // No encryption settings found - using default machine-based
                tracing::warn!(
                    "⚠️  API keys being saved with MACHINE-BASED encryption (transport encryption only)"
                );
                tracing::warn!("   Machine-based encryption does NOT protect API keys at-rest on local machine");
                tracing::warn!(
                    "   Anyone with database file access can decrypt keys on this machine"
                );
                tracing::warn!(
                    "   RECOMMENDATION: Upgrade to password-based encryption with 'orkee security set-password'"
                );
            }
            Err(e) => {
                tracing::warn!("Failed to check encryption mode: {}", e);
            }
            _ => {
                tracing::warn!("Unknown encryption mode - assuming machine-based");
            }
        }
    }

    let input = UserUpdateInput {
        openai_api_key: request.openai_api_key,
        anthropic_api_key: request.anthropic_api_key,
        google_api_key: request.google_api_key,
        xai_api_key: request.xai_api_key,
        ai_gateway_enabled: request.ai_gateway_enabled,
        ai_gateway_url: request.ai_gateway_url,
        ai_gateway_key: request.ai_gateway_key,
    };

    let result = db
        .user_storage
        .update_credentials(&current_user.id, input)
        .await
        .map(|user| {
            let masked: MaskedUser = user.into();
            masked
        });

    ok_or_internal_error(result, "Failed to update credentials")
}

/// Get user's Anthropic API key (decrypted)
/// This endpoint returns the actual API key for use in the frontend AI service.
/// Security: Only accessible from localhost (Tauri/web dashboard), protected by same auth as other endpoints.
pub async fn get_anthropic_key(
    State(db): State<DbState>,
    current_user: CurrentUser,
) -> impl IntoResponse {
    info!("Getting Anthropic API key for user {}", current_user.id);

    let result = db
        .user_storage
        .get_user(&current_user.id)
        .await
        .map(|user| {
            // Return the API key if present, otherwise return error message
            match user.anthropic_api_key {
                Some(key) => serde_json::json!({"apiKey": key}),
                None => serde_json::json!({
                    "apiKey": null,
                    "error": "No Anthropic API key configured. Please add your API key in Settings."
                }),
            }
        });

    ok_or_internal_error(result, "Failed to get Anthropic API key")
}
