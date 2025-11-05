// ABOUTME: HTTP request handlers for sandbox settings operations
// ABOUTME: Handles sandbox and provider-specific configuration management

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::info;

use super::response::ok_or_internal_error;
use orkee_projects::DbState;
use orkee_sandbox::{ProviderSettings, SandboxSettings};

/// Get sandbox settings
pub async fn get_sandbox_settings(State(db): State<DbState>) -> impl IntoResponse {
    info!("Getting sandbox settings");

    let result = db.sandbox_settings.get_sandbox_settings().await;
    ok_or_internal_error(result, "Failed to get sandbox settings")
}

/// Request body for updating sandbox settings
#[derive(Deserialize)]
pub struct UpdateSandboxSettingsRequest {
    #[serde(flatten)]
    pub settings: SandboxSettings,
}

/// Update sandbox settings
pub async fn update_sandbox_settings(
    State(db): State<DbState>,
    Json(request): Json<UpdateSandboxSettingsRequest>,
) -> impl IntoResponse {
    info!("Updating sandbox settings");

    // For now, we don't track which user made the change
    // In production, this should be extracted from authentication context
    let updated_by = Some("system");

    let result = db
        .sandbox_settings
        .update_sandbox_settings(&request.settings, updated_by)
        .await
        .map(|_| serde_json::json!({"message": "Sandbox settings updated successfully"}));

    ok_or_internal_error(result, "Failed to update sandbox settings")
}

/// List all provider settings
pub async fn list_provider_settings(State(db): State<DbState>) -> impl IntoResponse {
    info!("Listing all provider settings");

    let result = db.sandbox_settings.list_provider_settings().await;
    ok_or_internal_error(result, "Failed to list provider settings")
}

/// Get provider settings by provider ID
pub async fn get_provider_settings(
    State(db): State<DbState>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    info!("Getting provider settings for: {}", provider);

    let result = db.sandbox_settings.get_provider_settings(&provider).await;
    ok_or_internal_error(result, "Failed to get provider settings")
}

/// Request body for updating provider settings
#[derive(Deserialize)]
pub struct UpdateProviderSettingsRequest {
    #[serde(flatten)]
    pub settings: ProviderSettings,
}

/// Update provider settings
pub async fn update_provider_settings(
    State(db): State<DbState>,
    Path(provider): Path<String>,
    Json(request): Json<UpdateProviderSettingsRequest>,
) -> impl IntoResponse {
    info!("Updating provider settings for: {}", provider);

    // Validate that the provider in the path matches the request body
    if request.settings.provider != provider {
        let error: Result<(), orkee_storage::StorageError> = Err(orkee_storage::StorageError::InvalidInput(
            "Provider in path does not match provider in request body".to_string(),
        ));
        return ok_or_internal_error(error, "Provider mismatch");
    }

    // For now, we don't track which user made the change
    // In production, this should be extracted from authentication context
    let updated_by = Some("system");

    let result = db
        .sandbox_settings
        .update_provider_settings(&request.settings, updated_by)
        .await
        .map(|_| serde_json::json!({"message": "Provider settings updated successfully"}));

    ok_or_internal_error(result, "Failed to update provider settings")
}

/// Delete provider settings
pub async fn delete_provider_settings(
    State(db): State<DbState>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    info!("Deleting provider settings for: {}", provider);

    let result = db
        .sandbox_settings
        .delete_provider_settings(&provider)
        .await
        .map(|_| serde_json::json!({"message": "Provider settings deleted successfully"}));

    ok_or_internal_error(result, "Failed to delete provider settings")
}
