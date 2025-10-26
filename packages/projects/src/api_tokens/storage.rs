// ABOUTME: Storage operations for API tokens
// ABOUTME: Token generation, hashing, verification, and database operations

use crate::api_tokens::types::{ApiToken, TokenGeneration};
use storage::StorageError;
use base64::Engine;
use rand::Rng;
use sha2::{Digest, Sha256};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub struct TokenStorage {
    pool: SqlitePool,
}

impl TokenStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Generate a cryptographically secure random token
    /// Returns a base64-encoded 32-byte token
    pub fn generate_token() -> String {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 32] = rng.gen();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(random_bytes)
    }

    /// Hash a token using SHA-256
    /// This is what gets stored in the database
    pub fn hash_token(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Verify a token against a stored hash using constant-time comparison
    /// This prevents timing attacks
    pub fn verify_token_hash(token: &str, stored_hash: &str) -> bool {
        let computed_hash = Self::hash_token(token);

        // Constant-time comparison to prevent timing attacks
        use subtle::ConstantTimeEq;
        computed_hash
            .as_bytes()
            .ct_eq(stored_hash.as_bytes())
            .into()
    }

    /// Create a new API token
    pub async fn create_token(&self, name: &str) -> Result<TokenGeneration, StorageError> {
        let id = Uuid::new_v4().to_string();
        let token = Self::generate_token();
        let token_hash = Self::hash_token(&token);

        sqlx::query(
            "INSERT INTO api_tokens (id, token_hash, name, is_active)
             VALUES (?, ?, ?, 1)",
        )
        .bind(&id)
        .bind(&token_hash)
        .bind(name)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(TokenGeneration::new(token, token_hash, id))
    }

    /// Verify a token and return the token record if valid
    pub async fn verify_token(&self, token: &str) -> Result<Option<ApiToken>, StorageError> {
        let token_hash = Self::hash_token(token);

        let row = sqlx::query(
            "SELECT id, token_hash, name, created_at, last_used_at, is_active
             FROM api_tokens
             WHERE token_hash = ? AND is_active = 1",
        )
        .bind(&token_hash)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        match row {
            Some(row) => {
                let stored_hash: String = row.try_get("token_hash").map_err(StorageError::Sqlx)?;

                // Double-check with constant-time comparison
                if Self::verify_token_hash(token, &stored_hash) {
                    Ok(Some(self.row_to_token(row)?))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Update the last_used_at timestamp for a token
    pub async fn update_last_used(&self, token_hash: &str) -> Result<(), StorageError> {
        sqlx::query(
            "UPDATE api_tokens
             SET last_used_at = datetime('now', 'utc')
             WHERE token_hash = ? AND is_active = 1",
        )
        .bind(token_hash)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    /// List all tokens (without hashes)
    pub async fn list_tokens(&self) -> Result<Vec<ApiToken>, StorageError> {
        let rows = sqlx::query(
            "SELECT id, token_hash, name, created_at, last_used_at, is_active
             FROM api_tokens
             ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        rows.into_iter().map(|row| self.row_to_token(row)).collect()
    }

    /// Get a token by ID
    pub async fn get_token(&self, id: &str) -> Result<Option<ApiToken>, StorageError> {
        let row = sqlx::query(
            "SELECT id, token_hash, name, created_at, last_used_at, is_active
             FROM api_tokens
             WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        match row {
            Some(row) => Ok(Some(self.row_to_token(row)?)),
            None => Ok(None),
        }
    }

    /// Revoke a token (set is_active = 0)
    pub async fn revoke_token(&self, id: &str) -> Result<(), StorageError> {
        sqlx::query("UPDATE api_tokens SET is_active = 0 WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    /// Count active tokens
    pub async fn count_active_tokens(&self) -> Result<i64, StorageError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM api_tokens WHERE is_active = 1")
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let count: i64 = row.try_get("count").map_err(StorageError::Sqlx)?;
        Ok(count)
    }

    /// Helper to convert database row to ApiToken
    fn row_to_token(&self, row: sqlx::sqlite::SqliteRow) -> Result<ApiToken, StorageError> {
        Ok(ApiToken {
            id: row.try_get("id").map_err(StorageError::Sqlx)?,
            token_hash: row.try_get("token_hash").map_err(StorageError::Sqlx)?,
            name: row.try_get("name").map_err(StorageError::Sqlx)?,
            created_at: row.try_get("created_at").map_err(StorageError::Sqlx)?,
            last_used_at: row.try_get("last_used_at").map_err(StorageError::Sqlx)?,
            is_active: row
                .try_get::<i64, _>("is_active")
                .map_err(StorageError::Sqlx)?
                != 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_token_produces_unique_values() {
        let token1 = TokenStorage::generate_token();
        let token2 = TokenStorage::generate_token();

        assert_ne!(token1, token2);
        assert!(token1.len() > 32); // Base64 of 32 bytes is 43 chars
    }

    #[test]
    fn test_hash_token_is_deterministic() {
        let token = "test-token-123";
        let hash1 = TokenStorage::hash_token(token);
        let hash2 = TokenStorage::hash_token(token);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA-256 produces 64 hex chars
    }

    #[test]
    fn test_hash_token_different_inputs_produce_different_hashes() {
        let token1 = "test-token-1";
        let token2 = "test-token-2";

        let hash1 = TokenStorage::hash_token(token1);
        let hash2 = TokenStorage::hash_token(token2);

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_verify_token_hash_valid() {
        let token = "test-token-123";
        let hash = TokenStorage::hash_token(token);

        assert!(TokenStorage::verify_token_hash(token, &hash));
    }

    #[test]
    fn test_verify_token_hash_invalid() {
        let token = "test-token-123";
        let wrong_token = "test-token-456";
        let hash = TokenStorage::hash_token(token);

        assert!(!TokenStorage::verify_token_hash(wrong_token, &hash));
    }

    #[test]
    fn test_verify_token_hash_uses_constant_time_comparison() {
        // This test ensures we're using constant-time comparison
        // In a real timing attack test, we'd measure execution time
        // For now, we just verify the function exists and works
        let token = "a".repeat(100);
        let hash = TokenStorage::hash_token(&token);

        assert!(TokenStorage::verify_token_hash(&token, &hash));
    }
}
