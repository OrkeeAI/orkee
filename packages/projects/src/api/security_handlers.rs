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
pub async fn get_security_status(
    State(db): State<DbState>,
    _current_user: CurrentUser,
) -> impl IntoResponse {
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

    // Get password lockout status from database
    let lockout_result: Result<Option<(i64, Option<String>)>, sqlx::Error> =
        sqlx::query_as("SELECT attempt_count, locked_until FROM password_attempts WHERE id = 1")
            .fetch_optional(&db.pool)
            .await;

    let (is_locked, failed_attempts, lockout_ends_at) = match lockout_result {
        Ok(Some((attempt_count, locked_until_str))) => {
            let locked_until = locked_until_str
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc));

            let now = chrono::Utc::now();
            let is_locked = locked_until.map_or(false, |until| until > now);

            (
                is_locked,
                Some(attempt_count as i32),
                locked_until.map(|dt| dt.to_rfc3339()),
            )
        }
        Ok(None) => (false, Some(0), None),
        Err(_) => (false, None, None),
    };

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

/// Helper function to check status of a single API key
fn check_key_status(
    key_name: &str,
    db_key: Option<&String>,
    env_var_name: &str,
    updated_at: &chrono::DateTime<chrono::Utc>,
) -> KeyStatus {
    let has_db = db_key.is_some();
    let has_env = std::env::var(env_var_name).is_ok();

    KeyStatus {
        key: key_name.to_string(),
        configured: has_db || has_env,
        source: if has_env {
            "environment".to_string()
        } else if has_db {
            "database".to_string()
        } else {
            "none".to_string()
        },
        last_updated: if has_db {
            Some(updated_at.to_rfc3339())
        } else {
            None
        },
    }
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

    let keys = vec![
        check_key_status("openai", user.openai_api_key.as_ref(), "OPENAI_API_KEY", &user.updated_at),
        check_key_status("anthropic", user.anthropic_api_key.as_ref(), "ANTHROPIC_API_KEY", &user.updated_at),
        check_key_status("google", user.google_api_key.as_ref(), "GOOGLE_API_KEY", &user.updated_at),
        check_key_status("xai", user.xai_api_key.as_ref(), "XAI_API_KEY", &user.updated_at),
        check_key_status("ai_gateway", user.ai_gateway_key.as_ref(), "AI_GATEWAY_KEY", &user.updated_at),
    ];

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
