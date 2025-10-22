// ABOUTME: HTTP request handlers for security and encryption management
// ABOUTME: Provides endpoints for password-based encryption setup and API key status

use axum::{extract::State, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::auth::CurrentUser;
use super::response::ok_or_internal_error;
use crate::db::DbState;
use crate::storage::StorageError;

/// Security status response
#[derive(Debug, Serialize)]
pub struct SecurityStatusResponse {
    pub encryption_mode: String,
    pub is_locked: bool,
    pub failed_attempts: Option<i32>,
    pub lockout_ends_at: Option<String>,
}

/// Get current encryption and security status
pub async fn get_security_status(State(db): State<DbState>) -> impl IntoResponse {
    info!("Getting security status");

    // Get encryption mode from database
    let encryption_mode_result: Result<Option<(String,)>, sqlx::Error> =
        sqlx::query_as("SELECT encryption_mode FROM encryption_settings WHERE id = 1")
            .fetch_optional(&db.pool)
            .await;

    let encryption_mode = match encryption_mode_result {
        Ok(Some((mode_str,))) => mode_str,
        Ok(None) => "machine".to_string(), // Default
        Err(_) => "machine".to_string(),   // Default on error
    };

    // For now, we don't implement password lockout checking here
    // This would require checking failed_attempts and lockout_until fields
    let is_locked = false;
    let failed_attempts = None;
    let lockout_ends_at = None;

    let response = SecurityStatusResponse {
        encryption_mode,
        is_locked,
        failed_attempts,
        lockout_ends_at,
    };

    ok_or_internal_error::<SecurityStatusResponse, sqlx::Error>(
        Ok(response),
        "Failed to get security status",
    )
}

/// Key status for a single API key
#[derive(Debug, Serialize)]
pub struct KeyStatus {
    pub key: String,
    pub configured: bool,
    pub source: String, // "database", "environment", or "none"
    pub last_updated: Option<String>,
}

/// Keys status response
#[derive(Debug, Serialize)]
pub struct KeysStatusResponse {
    pub keys: Vec<KeyStatus>,
}

/// Get status of all API keys (sources and configuration state)
pub async fn get_keys_status(
    State(db): State<DbState>,
    current_user: CurrentUser,
) -> impl IntoResponse {
    info!("Getting API keys status");

    // Get user to check database keys
    let user = match db.user_storage.get_user(&current_user.id).await {
        Ok(u) => u,
        Err(e) => {
            return ok_or_internal_error::<KeysStatusResponse, StorageError>(
                Err(e),
                "Failed to get user data",
            );
        }
    };

    let mut keys = Vec::new();

    // Check each key type
    // OpenAI
    let has_openai_db = user.openai_api_key.is_some();
    let has_openai_env = std::env::var("OPENAI_API_KEY").is_ok();
    keys.push(KeyStatus {
        key: "openai".to_string(),
        configured: has_openai_db || has_openai_env,
        source: if has_openai_env {
            "environment".to_string()
        } else if has_openai_db {
            "database".to_string()
        } else {
            "none".to_string()
        },
        last_updated: if has_openai_db {
            Some(user.updated_at.to_rfc3339())
        } else {
            None
        },
    });

    // Anthropic
    let has_anthropic_db = user.anthropic_api_key.is_some();
    let has_anthropic_env = std::env::var("ANTHROPIC_API_KEY").is_ok();
    keys.push(KeyStatus {
        key: "anthropic".to_string(),
        configured: has_anthropic_db || has_anthropic_env,
        source: if has_anthropic_env {
            "environment".to_string()
        } else if has_anthropic_db {
            "database".to_string()
        } else {
            "none".to_string()
        },
        last_updated: if has_anthropic_db {
            Some(user.updated_at.to_rfc3339())
        } else {
            None
        },
    });

    // Google AI
    let has_google_db = user.google_api_key.is_some();
    let has_google_env = std::env::var("GOOGLE_API_KEY").is_ok();
    keys.push(KeyStatus {
        key: "google".to_string(),
        configured: has_google_db || has_google_env,
        source: if has_google_env {
            "environment".to_string()
        } else if has_google_db {
            "database".to_string()
        } else {
            "none".to_string()
        },
        last_updated: if has_google_db {
            Some(user.updated_at.to_rfc3339())
        } else {
            None
        },
    });

    // xAI
    let has_xai_db = user.xai_api_key.is_some();
    let has_xai_env = std::env::var("XAI_API_KEY").is_ok();
    keys.push(KeyStatus {
        key: "xai".to_string(),
        configured: has_xai_db || has_xai_env,
        source: if has_xai_env {
            "environment".to_string()
        } else if has_xai_db {
            "database".to_string()
        } else {
            "none".to_string()
        },
        last_updated: if has_xai_db {
            Some(user.updated_at.to_rfc3339())
        } else {
            None
        },
    });

    // AI Gateway Key
    let has_gateway_db = user.ai_gateway_key.is_some();
    let has_gateway_env = std::env::var("AI_GATEWAY_KEY").is_ok();
    keys.push(KeyStatus {
        key: "ai_gateway".to_string(),
        configured: has_gateway_db || has_gateway_env,
        source: if has_gateway_env {
            "environment".to_string()
        } else if has_gateway_db {
            "database".to_string()
        } else {
            "none".to_string()
        },
        last_updated: if has_gateway_db {
            Some(user.updated_at.to_rfc3339())
        } else {
            None
        },
    });

    let response = KeysStatusResponse { keys };

    ok_or_internal_error::<KeysStatusResponse, StorageError>(
        Ok(response),
        "Failed to get keys status",
    )
}

/// Request body for setting password
#[derive(Deserialize)]
pub struct SetPasswordRequest {
    pub password: String,
}

/// Set password for password-based encryption
pub async fn set_password(
    State(_db): State<DbState>,
    Json(_request): Json<SetPasswordRequest>,
) -> impl IntoResponse {
    info!("Setting password for encryption");

    // TODO: Implement password-based encryption setup
    // This requires more complex database operations
    let response = serde_json::json!({
        "message": "Password management not yet implemented in API",
        "encryption_mode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Password management not implemented",
    )
}

/// Request body for changing password
#[derive(Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Change encryption password
pub async fn change_password(
    State(_db): State<DbState>,
    _current_user: CurrentUser,
    Json(_request): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    info!("Changing encryption password");

    // TODO: Implement password change functionality
    let response = serde_json::json!({
        "message": "Password management not yet implemented in API",
        "encryption_mode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Password management not implemented",
    )
}

/// Remove password-based encryption (downgrade to machine-based)
pub async fn remove_password(State(_db): State<DbState>) -> impl IntoResponse {
    info!("Removing password-based encryption");

    // TODO: Implement password removal functionality
    let response = serde_json::json!({
        "message": "Password management not yet implemented in API",
        "encryption_mode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Password management not implemented",
    )
}
