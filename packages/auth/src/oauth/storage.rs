// ABOUTME: Database storage layer for OAuth tokens and provider configurations
// ABOUTME: Handles encrypted storage and retrieval of OAuth credentials using SQLx

use orkee_security::ApiKeyEncryption;
use sqlx::{Row, SqlitePool};
use tracing::{debug, error};

use crate::{
    error::{AuthError, AuthResult},
    oauth::types::{OAuthProvider, OAuthToken},
};

/// OAuth storage manager for database operations
pub struct OAuthStorage {
    pool: SqlitePool,
    encryption: ApiKeyEncryption,
}

impl OAuthStorage {
    /// Create new OAuth storage with database pool
    pub fn new(pool: SqlitePool) -> AuthResult<Self> {
        // Initialize encryption with machine-based key (default)
        // Users can upgrade to password-based via `orkee security set-password`
        let encryption = ApiKeyEncryption::new()
            .map_err(|e| AuthError::Storage(format!("Failed to initialize encryption: {}", e)))?;

        Ok(Self { pool, encryption })
    }

    /// Store OAuth token (encrypted)
    pub async fn store_token(&self, token: &OAuthToken) -> AuthResult<()> {
        debug!("Storing OAuth token for provider: {}", token.provider);

        // Encrypt access token and refresh token
        let encrypted_access_token = self.encryption.encrypt(&token.access_token).map_err(|e| {
            error!("Failed to encrypt access token: {}", e);
            AuthError::Storage(format!("Token encryption failed: {}", e))
        })?;

        let encrypted_refresh_token = match &token.refresh_token {
            Some(rt) => Some(self.encryption.encrypt(rt).map_err(|e| {
                error!("Failed to encrypt refresh token: {}", e);
                AuthError::Storage(format!("Token encryption failed: {}", e))
            })?),
            None => None,
        };

        sqlx::query(
            r#"
            INSERT INTO oauth_tokens (
                id, user_id, provider, access_token, refresh_token,
                expires_at, token_type, scope, subscription_type, account_email,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, unixepoch(), unixepoch())
            ON CONFLICT(user_id, provider) DO UPDATE SET
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                expires_at = excluded.expires_at,
                token_type = excluded.token_type,
                scope = excluded.scope,
                subscription_type = excluded.subscription_type,
                account_email = excluded.account_email,
                updated_at = unixepoch()
            "#,
        )
        .bind(&token.id)
        .bind(&token.user_id)
        .bind(&token.provider)
        .bind(&encrypted_access_token)
        .bind(&encrypted_refresh_token)
        .bind(token.expires_at)
        .bind(&token.token_type)
        .bind(&token.scope)
        .bind(&token.subscription_type)
        .bind(&token.account_email)
        .execute(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to store OAuth token: {}", e);
            AuthError::Storage(format!("Failed to store token: {}", e))
        })?;

        debug!("Successfully stored encrypted OAuth token");
        Ok(())
    }

    /// Get OAuth token for user and provider
    pub async fn get_token(
        &self,
        user_id: &str,
        provider: OAuthProvider,
    ) -> AuthResult<Option<OAuthToken>> {
        debug!(
            "Fetching OAuth token for user {} provider {}",
            user_id, provider
        );

        let row = sqlx::query(
            r#"
            SELECT id, user_id, provider, access_token, refresh_token,
                   expires_at, token_type, scope, subscription_type, account_email
            FROM oauth_tokens
            WHERE user_id = ? AND provider = ?
            "#,
        )
        .bind(user_id)
        .bind(provider.to_string())
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => {
                // Decrypt access token
                let encrypted_access_token: String = row.try_get("access_token")?;
                let access_token =
                    self.encryption
                        .decrypt(&encrypted_access_token)
                        .map_err(|e| {
                            error!("Failed to decrypt access token: {}", e);
                            AuthError::Storage(format!("Token decryption failed: {}", e))
                        })?;

                // Decrypt refresh token if present
                let encrypted_refresh_token: Option<String> = row.try_get("refresh_token")?;
                let refresh_token = match encrypted_refresh_token {
                    Some(encrypted) => Some(self.encryption.decrypt(&encrypted).map_err(|e| {
                        error!("Failed to decrypt refresh token: {}", e);
                        AuthError::Storage(format!("Token decryption failed: {}", e))
                    })?),
                    None => None,
                };

                let token = OAuthToken {
                    id: row.try_get("id")?,
                    user_id: row.try_get("user_id")?,
                    provider: row.try_get("provider")?,
                    access_token,
                    refresh_token,
                    expires_at: row.try_get("expires_at")?,
                    token_type: row.try_get("token_type")?,
                    scope: row.try_get("scope")?,
                    subscription_type: row.try_get("subscription_type")?,
                    account_email: row.try_get("account_email")?,
                };
                debug!("Found and decrypted OAuth token");
                Ok(Some(token))
            }
            None => {
                debug!("No OAuth token found");
                Ok(None)
            }
        }
    }

    /// Delete OAuth token
    pub async fn delete_token(&self, user_id: &str, provider: OAuthProvider) -> AuthResult<()> {
        debug!(
            "Deleting OAuth token for user {} provider {}",
            user_id, provider
        );

        sqlx::query(
            r#"
            DELETE FROM oauth_tokens
            WHERE user_id = ? AND provider = ?
            "#,
        )
        .bind(user_id)
        .bind(provider.to_string())
        .execute(&self.pool)
        .await?;

        debug!("Deleted OAuth token");
        Ok(())
    }
}
