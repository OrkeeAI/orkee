// ABOUTME: HTTP request handlers for user operations
// ABOUTME: Handles user settings and preferences management

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::ApiResponse;
use crate::db::DbState;
use crate::users::{MaskedUser, UserUpdateInput};

/// Get current user (with masked credentials)
pub async fn get_current_user(State(db): State<DbState>) -> impl IntoResponse {
    info!("Getting current user");

    match db.user_storage.get_current_user().await {
        Ok(user) => {
            let masked: MaskedUser = user.into();
            (StatusCode::OK, ResponseJson(ApiResponse::success(masked))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Get user by ID
pub async fn get_user(State(db): State<DbState>, Path(user_id): Path<String>) -> impl IntoResponse {
    info!("Getting user: {}", user_id);

    match db.user_storage.get_user(&user_id).await {
        Ok(user) => (StatusCode::OK, ResponseJson(ApiResponse::success(user))).into_response(),
        Err(e) => e.into_response(),
    }
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

    match db
        .user_storage
        .set_default_agent(&user_id, &request.agent_id)
        .await
    {
        Ok(()) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success(serde_json::json!({
                "message": "Default agent updated successfully"
            }))),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
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

    match db.user_storage.update_theme(&user_id, &request.theme).await {
        Ok(()) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success(serde_json::json!({
                "message": "Theme updated successfully"
            }))),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
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
    Json(request): Json<UpdateCredentialsRequest>,
) -> impl IntoResponse {
    info!("Updating user credentials");

    let input = UserUpdateInput {
        openai_api_key: request.openai_api_key,
        anthropic_api_key: request.anthropic_api_key,
        google_api_key: request.google_api_key,
        xai_api_key: request.xai_api_key,
        ai_gateway_enabled: request.ai_gateway_enabled,
        ai_gateway_url: request.ai_gateway_url,
        ai_gateway_key: request.ai_gateway_key,
    };

    match db
        .user_storage
        .update_credentials("default-user", input)
        .await
    {
        Ok(user) => {
            let masked: MaskedUser = user.into();
            (StatusCode::OK, ResponseJson(ApiResponse::success(masked))).into_response()
        }
        Err(e) => e.into_response(),
    }
}
