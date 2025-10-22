// ABOUTME: HTTP request handlers for security and encryption management
// ABOUTME: Provides endpoints for password-based encryption setup and API key status

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::auth::CurrentUser;
use super::response::{ok_or_internal_error, ApiResponse};
use crate::db::DbState;
use crate::storage::StorageError;

// Password validation constants
const MIN_PASSWORD_LENGTH: usize = 8;

/// Validates password meets security requirements
fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        ));
    }
    Ok(())
}

/// Security status response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
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
        Ok(None) => {
            tracing::warn!("No encryption mode found in database, defaulting to 'machine'");
            "machine".to_string()
        }
        Err(e) => {
            tracing::error!("Failed to read encryption mode from database: {}", e);
            "machine".to_string()
        }
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
            let is_locked = locked_until.is_some_and(|until| until > now);

            (
                is_locked,
                Some(attempt_count as i32),
                locked_until.map(|dt| dt.to_rfc3339()),
            )
        }
        Ok(None) => {
            tracing::debug!("No password attempt records found, user is not locked out");
            (false, Some(0), None)
        }
        Err(e) => {
            tracing::error!(
                "Failed to read password lockout status from database: {}",
                e
            );
            (false, None, None)
        }
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
}

/// Keys status response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KeysStatusResponse {
    pub keys: Vec<KeyStatus>,
}

/// Helper function to check status of a single API key
fn check_key_status(key_name: &str, db_key: Option<&String>, env_var_name: &str) -> KeyStatus {
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
        check_key_status("openai", user.openai_api_key.as_ref(), "OPENAI_API_KEY"),
        check_key_status(
            "anthropic",
            user.anthropic_api_key.as_ref(),
            "ANTHROPIC_API_KEY",
        ),
        check_key_status("google", user.google_api_key.as_ref(), "GOOGLE_API_KEY"),
        check_key_status("xai", user.xai_api_key.as_ref(), "XAI_API_KEY"),
        check_key_status("ai_gateway", user.ai_gateway_key.as_ref(), "AI_GATEWAY_KEY"),
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
    _current_user: CurrentUser,
    Json(request): Json<SetPasswordRequest>,
) -> impl IntoResponse {
    info!("Setting password for encryption");

    // Validate password
    if let Err(error_message) = validate_password(&request.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        )
            .into_response();
    }

    // TODO: Implement password-based encryption setup
    // This requires more complex database operations
    let response = serde_json::json!({
        "message": "Password management not yet implemented in API",
        "encryptionMode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Password management not implemented",
    )
}

/// Request body for changing password
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

/// Change encryption password
pub async fn change_password(
    State(_db): State<DbState>,
    _current_user: CurrentUser,
    Json(request): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    info!("Changing encryption password");

    // Validate new password
    if let Err(error_message) = validate_password(&request.new_password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        )
            .into_response();
    }

    // TODO: Implement password change functionality
    let response = serde_json::json!({
        "message": "Password management not yet implemented in API",
        "encryptionMode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Password management not implemented",
    )
}

/// Remove password-based encryption (downgrade to machine-based)
pub async fn remove_password(
    State(_db): State<DbState>,
    _current_user: CurrentUser,
) -> impl IntoResponse {
    info!("Removing password-based encryption");

    // TODO: Implement password removal functionality
    let response = serde_json::json!({
        "message": "Password management not yet implemented in API",
        "encryptionMode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Password management not implemented",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_password_valid() {
        assert!(validate_password("password123").is_ok());
        assert!(validate_password("12345678").is_ok());
        assert!(validate_password("very_long_password_that_is_secure").is_ok());
    }

    #[test]
    fn test_validate_password_too_short() {
        let result = validate_password("short");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Password must be at least 8 characters long"
        );
    }

    #[test]
    fn test_validate_password_empty() {
        let result = validate_password("");
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            "Password must be at least 8 characters long"
        );
    }

    #[test]
    fn test_validate_password_exactly_min_length() {
        // Exactly 8 characters should be valid
        assert!(validate_password("12345678").is_ok());
    }

    #[test]
    fn test_validate_password_one_char_less_than_min() {
        // 7 characters should be invalid
        let result = validate_password("1234567");
        assert!(result.is_err());
    }
}
