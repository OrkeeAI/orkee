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

/// Get current user
pub async fn get_current_user(State(db): State<DbState>) -> impl IntoResponse {
    info!("Getting current user");

    match db.user_storage.get_current_user().await {
        Ok(user) => (StatusCode::OK, ResponseJson(ApiResponse::success(user))).into_response(),
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
