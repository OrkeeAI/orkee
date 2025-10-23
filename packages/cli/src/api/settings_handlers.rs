// ABOUTME: API handlers for system settings management
// ABOUTME: REST endpoints for runtime configuration

use crate::error::AppError;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use orkee_projects::{
    db::DbState,
    settings::{BulkSettingUpdate, SettingUpdate, SettingsResponse},
    storage::StorageError,
};
use serde_json::{json, Value};
use tracing::info;

/// Get all settings
pub async fn get_settings(State(db): State<DbState>) -> Result<Json<Value>, StatusCode> {
    info!("Getting all system settings");

    match db.settings_storage.get_all().await {
        Ok(settings) => {
            let requires_restart = settings.iter().any(|s| s.requires_restart);
            Ok(Json(json!({
                "success": true,
                "data": SettingsResponse {
                    settings,
                    requires_restart,
                },
                "error": null
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get settings: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get settings by category
pub async fn get_settings_by_category(
    State(db): State<DbState>,
    Path(category): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    info!("Getting settings for category: {}", category);

    match db.settings_storage.get_by_category(&category).await {
        Ok(settings) => {
            let requires_restart = settings.iter().any(|s| s.requires_restart);
            Ok(Json(json!({
                "success": true,
                "data": SettingsResponse {
                    settings,
                    requires_restart,
                },
                "error": null
            })))
        }
        Err(e) => {
            tracing::error!("Failed to get settings for category {}: {}", category, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Update a single setting
pub async fn update_setting(
    State(db): State<DbState>,
    Path(key): Path<String>,
    Json(update): Json<SettingUpdate>,
) -> Result<Json<Value>, AppError> {
    info!("Updating setting: {}", key);

    match db.settings_storage.update(&key, update, "user").await {
        Ok(setting) => Ok(Json(json!({
            "success": true,
            "data": setting,
            "error": null
        }))),
        Err(StorageError::Validation(msg)) => {
            // Check if this is an is_env_only error (should be 403 Forbidden)
            if msg.contains("environment-only") {
                Err(AppError::Forbidden { message: msg })
            } else {
                // Otherwise it's a validation error (400 Bad Request)
                Err(AppError::Validation(msg))
            }
        }
        Err(StorageError::NotFound) => Err(AppError::NotFound),
        Err(e) => {
            tracing::error!("Failed to update setting {}: {}", key, e);
            Err(AppError::internal(anyhow::anyhow!("Storage error: {}", e)))
        }
    }
}

/// Update multiple settings
pub async fn bulk_update_settings(
    State(db): State<DbState>,
    Json(updates): Json<BulkSettingUpdate>,
) -> Result<Json<Value>, AppError> {
    info!("Bulk updating {} settings", updates.settings.len());

    match db.settings_storage.bulk_update(updates, "user").await {
        Ok(settings) => {
            let requires_restart = settings.iter().any(|s| s.requires_restart);
            Ok(Json(json!({
                "success": true,
                "data": SettingsResponse {
                    settings,
                    requires_restart,
                },
                "error": null
            })))
        }
        Err(StorageError::Validation(msg)) => {
            // Check if this is an is_env_only error (should be 403 Forbidden)
            if msg.contains("environment-only") {
                Err(AppError::Forbidden { message: msg })
            } else {
                // Otherwise it's a validation error (400 Bad Request)
                Err(AppError::Validation(msg))
            }
        }
        Err(e) => {
            tracing::error!("Failed to bulk update settings: {}", e);
            Err(AppError::internal(anyhow::anyhow!("Storage error: {}", e)))
        }
    }
}
