// ABOUTME: HTTP request handlers for security and encryption management
// ABOUTME: Provides endpoints for password-based encryption setup and API key status

use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::auth::CurrentUser;
use super::response::{ok_or_internal_error, ApiResponse};
use crate::db::DbState;
use crate::security::ApiKeyEncryption;
use crate::storage::StorageError;

// Password validation constants
const MIN_PASSWORD_LENGTH: usize = 8;
const PASSWORD_MAX_ATTEMPTS: i64 = 5;
const PASSWORD_LOCKOUT_DURATION_MINUTES: i64 = 15;

/// Check account lockout status within a transaction (atomic read)
async fn check_lockout_status(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
) -> Result<(i64, Option<String>), (StatusCode, Json<ApiResponse<()>>)> {
    let lockout_check: Result<Option<(i64, Option<String>)>, sqlx::Error> =
        sqlx::query_as("SELECT attempt_count, locked_until FROM password_attempts WHERE id = 1")
            .fetch_optional(&mut **tx)
            .await;

    match lockout_check {
        Ok(Some((count, locked_str))) => {
            // Check if account is currently locked
            if let Some(ref locked_until_str) = locked_str {
                if let Ok(locked_time) = chrono::DateTime::parse_from_rfc3339(locked_until_str) {
                    let now = chrono::Utc::now();
                    if locked_time.with_timezone(&chrono::Utc) > now {
                        error!("Account locked due to too many failed password attempts");
                        return Err((
                            StatusCode::TOO_MANY_REQUESTS,
                            Json(ApiResponse::<()>::error(format!(
                                "Account locked until {}. Too many failed password attempts.",
                                locked_until_str
                            ))),
                        ));
                    }
                }
            }
            Ok((count, locked_str))
        }
        Ok(None) => Ok((0, None)),
        Err(e) => {
            error!("Failed to check lockout status: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to check account status".to_string(),
                )),
            ))
        }
    }
}

/// Update password attempt counter within a transaction
async fn update_attempt_counter(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    attempt_count: i64,
    success: bool,
) -> Result<Option<String>, sqlx::Error> {
    if success {
        // Reset counter on success
        sqlx::query(
            r#"
            UPDATE password_attempts
            SET attempt_count = 0,
                locked_until = NULL,
                last_attempt_at = datetime('now', 'utc'),
                updated_at = datetime('now', 'utc')
            WHERE id = 1
            "#,
        )
        .execute(&mut **tx)
        .await?;
        Ok(None)
    } else {
        // Increment counter on failure
        let new_attempt_count = attempt_count + 1;
        let locked_until_new = if new_attempt_count >= PASSWORD_MAX_ATTEMPTS {
            let lockout_time =
                chrono::Utc::now() + chrono::Duration::minutes(PASSWORD_LOCKOUT_DURATION_MINUTES);
            let locked_str = lockout_time.to_rfc3339();
            error!(
                "Too many failed password attempts. Account locked until {}",
                lockout_time
            );
            Some(locked_str)
        } else {
            None
        };

        sqlx::query(
            r#"
            UPDATE password_attempts
            SET attempt_count = ?,
                locked_until = ?,
                last_attempt_at = datetime('now', 'utc'),
                updated_at = datetime('now', 'utc')
            WHERE id = 1
            "#,
        )
        .bind(new_attempt_count)
        .bind(&locked_until_new)
        .execute(&mut **tx)
        .await?;

        Ok(locked_until_new)
    }
}

// Type alias for encryption settings query result
type EncryptionSettingsResult =
    Result<Option<(String, Option<Vec<u8>>, Option<Vec<u8>>)>, sqlx::Error>;

/// Validates password meets security requirements
/// - Minimum 8 characters (longer is better for Argon2)
/// - At least one uppercase letter
/// - At least one lowercase letter
/// - At least one digit
/// - At least one special character
fn validate_password_strength(password: &str) -> Result<(), String> {
    if password.len() < MIN_PASSWORD_LENGTH {
        return Err(format!(
            "Password must be at least {} characters long",
            MIN_PASSWORD_LENGTH
        ));
    }

    let has_uppercase = password.chars().any(|c| c.is_ascii_uppercase());
    let has_lowercase = password.chars().any(|c| c.is_ascii_lowercase());
    let has_digit = password.chars().any(|c| c.is_ascii_digit());
    let has_special = password.chars().any(|c| !c.is_alphanumeric());

    if !has_uppercase {
        return Err("Password must contain at least one uppercase letter".to_string());
    }
    if !has_lowercase {
        return Err("Password must contain at least one lowercase letter".to_string());
    }
    if !has_digit {
        return Err("Password must contain at least one digit".to_string());
    }
    if !has_special {
        return Err("Password must contain at least one special character".to_string());
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
    let has_env = std::env::var(env_var_name)
        .ok()
        .filter(|v| !v.is_empty())
        .is_some();

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
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(request): Json<SetPasswordRequest>,
) -> impl IntoResponse {
    info!("Setting password-based encryption");

    // SECURITY: Validate password strength (never log password)
    if let Err(error_message) = validate_password_strength(&request.password) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        )
            .into_response();
    }

    // Check if already using password-based encryption
    let current_mode: Result<Option<(String,)>, sqlx::Error> =
        sqlx::query_as("SELECT encryption_mode FROM encryption_settings WHERE id = 1")
            .fetch_optional(&db.pool)
            .await;

    let mode = match current_mode {
        Ok(Some((mode_str,))) if mode_str == "password" => {
            error!("Already using password-based encryption. Use change-password instead.");
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::<()>::error(
                    "Already using password-based encryption. Use change-password to update."
                        .to_string(),
                )),
            )
                .into_response();
        }
        Ok(Some((mode_str,))) => mode_str,
        Ok(None) => "machine".to_string(),
        Err(e) => {
            error!("Failed to check encryption mode: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to check encryption status".to_string(),
                )),
            )
                .into_response();
        }
    };

    info!("Upgrading from {} to password-based encryption", mode);

    // Generate salt for password-based encryption
    let salt = match ApiKeyEncryption::generate_salt() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to generate salt: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to generate encryption salt".to_string(),
                )),
            )
                .into_response();
        }
    };

    // SECURITY: Hash password for verification (separate from encryption key)
    let password_hash =
        match ApiKeyEncryption::hash_password_for_verification(&request.password, &salt) {
            Ok(h) => h,
            Err(e) => {
                error!("Failed to hash password: {}", e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        "Failed to process password".to_string(),
                    )),
                )
                    .into_response();
            }
        };

    // Create old and new encryption instances
    let old_encryption = match ApiKeyEncryption::with_machine_key() {
        Ok(enc) => enc,
        Err(e) => {
            error!("Failed to create machine encryption: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to initialize encryption".to_string(),
                )),
            )
                .into_response();
        }
    };

    let new_encryption = match ApiKeyEncryption::with_password(&request.password, &salt) {
        Ok(enc) => enc,
        Err(e) => {
            error!("Failed to create password encryption: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to initialize password encryption".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Re-encrypt all API keys with new password-based encryption
    if let Err(e) = db
        .user_storage
        .rotate_encryption_keys(&current_user.id, &old_encryption, &new_encryption)
        .await
    {
        error!("Failed to rotate encryption keys: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to re-encrypt API keys".to_string(),
            )),
        )
            .into_response();
    }

    // Save encryption settings to database
    let result = sqlx::query(
        r#"
        UPDATE encryption_settings
        SET encryption_mode = 'password',
            password_salt = ?,
            password_hash = ?,
            updated_at = datetime('now')
        WHERE id = 1
        "#,
    )
    .bind(&salt)
    .bind(&password_hash)
    .execute(&db.pool)
    .await;

    if let Err(e) = result {
        error!("Failed to save encryption settings: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to save encryption settings".to_string(),
            )),
        )
            .into_response();
    }

    info!("Successfully upgraded to password-based encryption");

    let response = serde_json::json!({
        "message": "Password-based encryption enabled successfully",
        "encryptionMode": "password"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Failed to enable password-based encryption",
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
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(request): Json<ChangePasswordRequest>,
) -> impl IntoResponse {
    info!("Changing encryption password");

    // SECURITY: Use transaction to prevent race conditions in lockout checking
    let mut tx = match db.pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to process request".to_string(),
                )),
            )
                .into_response();
        }
    };

    // SECURITY: Check account lockout status atomically within transaction
    let (attempt_count, _locked_until) = match check_lockout_status(&mut tx).await {
        Ok(result) => result,
        Err(response) => {
            let _ = tx.rollback().await;
            return response.into_response();
        }
    };

    // Get current encryption settings
    let settings: EncryptionSettingsResult =
        sqlx::query_as("SELECT encryption_mode, password_salt, password_hash FROM encryption_settings WHERE id = 1")
            .fetch_optional(&mut *tx)
            .await;

    let (mode, salt, stored_hash) = match settings {
        Ok(Some(row)) => row,
        Ok(None) => {
            error!("No encryption settings found");
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Encryption settings not found".to_string(),
                )),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to fetch encryption settings: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to fetch encryption settings".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Ensure we're using password-based encryption
    if mode != "password" {
        error!("Not using password-based encryption");
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Not using password-based encryption. Use set-password instead.".to_string(),
            )),
        )
            .into_response();
    }

    let salt = match salt {
        Some(s) => s,
        None => {
            error!("Password salt not found");
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Password configuration is corrupt".to_string(),
                )),
            )
                .into_response();
        }
    };

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => {
            error!("Password hash not found");
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Password configuration is corrupt".to_string(),
                )),
            )
                .into_response();
        }
    };

    // SECURITY: Verify current password using constant-time comparison (via Argon2)
    // Note: This is a slow operation but must be done within the transaction for atomicity
    let password_valid =
        match ApiKeyEncryption::verify_password(&request.current_password, &salt, &stored_hash) {
            Ok(valid) => valid,
            Err(e) => {
                error!("Password verification failed: {}", e);
                let _ = tx.rollback().await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        "Password verification failed".to_string(),
                    )),
                )
                    .into_response();
            }
        };

    // SECURITY: Update attempt counter atomically within transaction
    let locked_until = match update_attempt_counter(&mut tx, attempt_count, password_valid).await {
        Ok(locked) => locked,
        Err(e) => {
            error!("Failed to update attempt counter: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to update security status".to_string(),
                )),
            )
                .into_response();
        }
    };

    if !password_valid {
        // Commit the failed attempt before returning
        if let Err(e) = tx.commit().await {
            error!("Failed to commit transaction: {}", e);
        }

        if locked_until.is_some() {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ApiResponse::<()>::error(
                    "Too many failed password attempts. Account locked.".to_string(),
                )),
            )
                .into_response();
        }

        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error(
                "Current password is incorrect".to_string(),
            )),
        )
            .into_response();
    }

    // SECURITY: Validate new password strength (never log password)
    if let Err(error_message) = validate_password_strength(&request.new_password) {
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(error_message)),
        )
            .into_response();
    }

    // Generate new salt for new password
    let new_salt = match ApiKeyEncryption::generate_salt() {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to generate new salt: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to generate encryption salt".to_string(),
                )),
            )
                .into_response();
        }
    };

    // SECURITY: Hash new password for verification
    let new_password_hash =
        match ApiKeyEncryption::hash_password_for_verification(&request.new_password, &new_salt) {
            Ok(h) => h,
            Err(e) => {
                error!("Failed to hash new password: {}", e);
                let _ = tx.rollback().await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        "Failed to process new password".to_string(),
                    )),
                )
                    .into_response();
            }
        };

    // Create encryption instances
    let old_encryption = match ApiKeyEncryption::with_password(&request.current_password, &salt) {
        Ok(enc) => enc,
        Err(e) => {
            error!("Failed to create old encryption: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to initialize encryption".to_string(),
                )),
            )
                .into_response();
        }
    };

    let new_encryption = match ApiKeyEncryption::with_password(&request.new_password, &new_salt) {
        Ok(enc) => enc,
        Err(e) => {
            error!("Failed to create new encryption: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to initialize new encryption".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Re-encrypt all API keys
    if let Err(e) = db
        .user_storage
        .rotate_encryption_keys(&current_user.id, &old_encryption, &new_encryption)
        .await
    {
        error!("Failed to rotate encryption keys: {}", e);
        let _ = tx.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to re-encrypt API keys".to_string(),
            )),
        )
            .into_response();
    }

    // Save new encryption settings within the transaction
    let result = sqlx::query(
        r#"
        UPDATE encryption_settings
        SET password_salt = ?,
            password_hash = ?,
            updated_at = datetime('now')
        WHERE id = 1
        "#,
    )
    .bind(&new_salt)
    .bind(&new_password_hash)
    .execute(&mut *tx)
    .await;

    if let Err(e) = result {
        error!("Failed to save new encryption settings: {}", e);
        let _ = tx.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to save new encryption settings".to_string(),
            )),
        )
            .into_response();
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to save changes".to_string(),
            )),
        )
            .into_response();
    }

    info!("Successfully changed encryption password");

    let response = serde_json::json!({
        "message": "Password changed successfully",
        "encryptionMode": "password"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Failed to change password",
    )
}

/// Request body for removing password
#[derive(Deserialize)]
pub struct RemovePasswordRequest {
    pub current_password: String,
}

/// Remove password-based encryption (downgrade to machine-based)
pub async fn remove_password(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(request): Json<RemovePasswordRequest>,
) -> impl IntoResponse {
    info!("Removing password-based encryption (downgrading to machine-based)");

    // SECURITY: Use transaction to prevent race conditions in lockout checking
    let mut tx = match db.pool.begin().await {
        Ok(t) => t,
        Err(e) => {
            error!("Failed to start transaction: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to process request".to_string(),
                )),
            )
                .into_response();
        }
    };

    // SECURITY: Check account lockout status atomically within transaction
    let (attempt_count, _locked_until) = match check_lockout_status(&mut tx).await {
        Ok(result) => result,
        Err(response) => {
            let _ = tx.rollback().await;
            return response.into_response();
        }
    };

    // Get current encryption settings
    let settings: EncryptionSettingsResult =
        sqlx::query_as("SELECT encryption_mode, password_salt, password_hash FROM encryption_settings WHERE id = 1")
            .fetch_optional(&mut *tx)
            .await;

    let (mode, salt, stored_hash) = match settings {
        Ok(Some(row)) => row,
        Ok(None) => {
            error!("No encryption settings found");
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Encryption settings not found".to_string(),
                )),
            )
                .into_response();
        }
        Err(e) => {
            error!("Failed to fetch encryption settings: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to fetch encryption settings".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Ensure we're using password-based encryption
    if mode != "password" {
        error!("Not using password-based encryption");
        let _ = tx.rollback().await;
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<()>::error(
                "Not using password-based encryption. Already using machine-based encryption."
                    .to_string(),
            )),
        )
            .into_response();
    }

    let salt = match salt {
        Some(s) => s,
        None => {
            error!("Password salt not found");
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Password configuration is corrupt".to_string(),
                )),
            )
                .into_response();
        }
    };

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => {
            error!("Password hash not found");
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Password configuration is corrupt".to_string(),
                )),
            )
                .into_response();
        }
    };

    // SECURITY: Verify current password before removing protection
    // Note: This is a slow operation but must be done within the transaction for atomicity
    let password_valid =
        match ApiKeyEncryption::verify_password(&request.current_password, &salt, &stored_hash) {
            Ok(valid) => valid,
            Err(e) => {
                error!("Password verification failed: {}", e);
                let _ = tx.rollback().await;
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::<()>::error(
                        "Password verification failed".to_string(),
                    )),
                )
                    .into_response();
            }
        };

    // SECURITY: Update attempt counter atomically within transaction
    let locked_until = match update_attempt_counter(&mut tx, attempt_count, password_valid).await {
        Ok(locked) => locked,
        Err(e) => {
            error!("Failed to update attempt counter: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to update security status".to_string(),
                )),
            )
                .into_response();
        }
    };

    if !password_valid {
        // Commit the failed attempt before returning
        if let Err(e) = tx.commit().await {
            error!("Failed to commit transaction: {}", e);
        }

        if locked_until.is_some() {
            return (
                StatusCode::TOO_MANY_REQUESTS,
                Json(ApiResponse::<()>::error(
                    "Too many failed password attempts. Account locked.".to_string(),
                )),
            )
                .into_response();
        }

        return (
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::<()>::error(
                "Current password is incorrect".to_string(),
            )),
        )
            .into_response();
    }

    // Create encryption instances
    let old_encryption = match ApiKeyEncryption::with_password(&request.current_password, &salt) {
        Ok(enc) => enc,
        Err(e) => {
            error!("Failed to create password encryption: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to initialize encryption".to_string(),
                )),
            )
                .into_response();
        }
    };

    let new_encryption = match ApiKeyEncryption::with_machine_key() {
        Ok(enc) => enc,
        Err(e) => {
            error!("Failed to create machine encryption: {}", e);
            let _ = tx.rollback().await;
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(
                    "Failed to initialize machine encryption".to_string(),
                )),
            )
                .into_response();
        }
    };

    // Re-encrypt all API keys with machine-based encryption
    if let Err(e) = db
        .user_storage
        .rotate_encryption_keys(&current_user.id, &old_encryption, &new_encryption)
        .await
    {
        error!("Failed to rotate encryption keys: {}", e);
        let _ = tx.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to re-encrypt API keys".to_string(),
            )),
        )
            .into_response();
    }

    // Update encryption settings to machine-based within the transaction
    let result = sqlx::query(
        r#"
        UPDATE encryption_settings
        SET encryption_mode = 'machine',
            password_salt = NULL,
            password_hash = NULL,
            updated_at = datetime('now')
        WHERE id = 1
        "#,
    )
    .execute(&mut *tx)
    .await;

    if let Err(e) = result {
        error!("Failed to update encryption settings: {}", e);
        let _ = tx.rollback().await;
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to update encryption settings".to_string(),
            )),
        )
            .into_response();
    }

    // Commit the transaction
    if let Err(e) = tx.commit().await {
        error!("Failed to commit transaction: {}", e);
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<()>::error(
                "Failed to save changes".to_string(),
            )),
        )
            .into_response();
    }

    info!("Successfully downgraded to machine-based encryption");

    let response = serde_json::json!({
        "message": "Password-based encryption removed. Now using machine-based encryption.",
        "encryptionMode": "machine"
    });

    ok_or_internal_error::<serde_json::Value, sqlx::Error>(
        Ok(response),
        "Failed to remove password-based encryption",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    // Password strength validation tests
    #[test]
    fn test_validate_password_strength_valid() {
        assert!(validate_password_strength("Password123!").is_ok());
        assert!(validate_password_strength("Secure#Pass1").is_ok());
        assert!(validate_password_strength("MyP@ssw0rd").is_ok());
    }

    #[test]
    fn test_validate_password_strength_too_short() {
        let result = validate_password_strength("Pass1!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 8 characters"));
    }

    #[test]
    fn test_validate_password_strength_no_uppercase() {
        let result = validate_password_strength("password123!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("uppercase letter"));
    }

    #[test]
    fn test_validate_password_strength_no_lowercase() {
        let result = validate_password_strength("PASSWORD123!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("lowercase letter"));
    }

    #[test]
    fn test_validate_password_strength_no_digit() {
        let result = validate_password_strength("Password!");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("digit"));
    }

    #[test]
    fn test_validate_password_strength_no_special() {
        let result = validate_password_strength("Password123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("special character"));
    }

    #[test]
    fn test_validate_password_strength_comprehensive() {
        // Test all missing requirements
        assert!(validate_password_strength("alllowercase").is_err());
        assert!(validate_password_strength("ALLUPPERCASE").is_err());
        assert!(validate_password_strength("NoDigitsHere!").is_err());
        assert!(validate_password_strength("NoSpecial123").is_err());

        // Valid combinations
        assert!(validate_password_strength("Abcdefg1!").is_ok());
        assert!(validate_password_strength("MySecure#Pass123").is_ok());
    }
}
