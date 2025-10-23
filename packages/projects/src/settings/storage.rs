// ABOUTME: Storage operations for system settings
// ABOUTME: Database CRUD for runtime configuration

use crate::settings::types::{BulkSettingUpdate, SettingUpdate, SystemSetting};
use crate::storage::StorageError;
use sqlx::{Row, SqlitePool};

pub struct SettingsStorage {
    pool: SqlitePool,
}

impl SettingsStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Get all settings
    pub async fn get_all(&self) -> Result<Vec<SystemSetting>, StorageError> {
        let rows = sqlx::query("SELECT * FROM system_settings ORDER BY category, key")
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let settings = rows
            .into_iter()
            .map(|row| self.row_to_setting(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(settings)
    }

    /// Get settings by category
    pub async fn get_by_category(
        &self,
        category: &str,
    ) -> Result<Vec<SystemSetting>, StorageError> {
        let rows = sqlx::query("SELECT * FROM system_settings WHERE category = ? ORDER BY key")
            .bind(category)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let settings = rows
            .into_iter()
            .map(|row| self.row_to_setting(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(settings)
    }

    /// Get a single setting by key
    pub async fn get(&self, key: &str) -> Result<SystemSetting, StorageError> {
        let row = sqlx::query("SELECT * FROM system_settings WHERE key = ?")
            .bind(key)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_setting(row)
    }

    /// Update a single setting
    pub async fn update(
        &self,
        key: &str,
        update: SettingUpdate,
        updated_by: &str,
    ) -> Result<SystemSetting, StorageError> {
        sqlx::query(
            "UPDATE system_settings 
             SET value = ?, updated_at = datetime('now', 'utc'), updated_by = ? 
             WHERE key = ?",
        )
        .bind(&update.value)
        .bind(updated_by)
        .bind(key)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        self.get(key).await
    }

    /// Update multiple settings
    pub async fn bulk_update(
        &self,
        updates: BulkSettingUpdate,
        updated_by: &str,
    ) -> Result<Vec<SystemSetting>, StorageError> {
        let mut tx = self.pool.begin().await.map_err(StorageError::Sqlx)?;

        for item in &updates.settings {
            sqlx::query(
                "UPDATE system_settings 
                 SET value = ?, updated_at = datetime('now', 'utc'), updated_by = ? 
                 WHERE key = ?",
            )
            .bind(&item.value)
            .bind(updated_by)
            .bind(&item.key)
            .execute(&mut *tx)
            .await
            .map_err(StorageError::Sqlx)?;
        }

        tx.commit().await.map_err(StorageError::Sqlx)?;

        // Return all updated settings
        let keys: Vec<String> = updates.settings.iter().map(|s| s.key.clone()).collect();
        let mut results = Vec::new();
        for key in keys {
            results.push(self.get(&key).await?);
        }

        Ok(results)
    }

    /// Reset settings in a category to defaults
    pub async fn reset_category(&self, category: &str) -> Result<Vec<SystemSetting>, StorageError> {
        // This would require storing default values separately
        // For now, we'll just return current settings
        // TODO: Implement proper reset logic with default values
        self.get_by_category(category).await
    }

    /// Helper to convert row to SystemSetting
    fn row_to_setting(&self, row: sqlx::sqlite::SqliteRow) -> Result<SystemSetting, StorageError> {
        Ok(SystemSetting {
            key: row.try_get("key").map_err(StorageError::Sqlx)?,
            value: row.try_get("value").map_err(StorageError::Sqlx)?,
            category: row.try_get("category").map_err(StorageError::Sqlx)?,
            description: row.try_get("description").map_err(StorageError::Sqlx)?,
            data_type: row.try_get("data_type").map_err(StorageError::Sqlx)?,
            is_secret: row
                .try_get::<i64, _>("is_secret")
                .map_err(StorageError::Sqlx)?
                != 0,
            requires_restart: row
                .try_get::<i64, _>("requires_restart")
                .map_err(StorageError::Sqlx)?
                != 0,
            is_env_only: row
                .try_get::<i64, _>("is_env_only")
                .map_err(StorageError::Sqlx)?
                != 0,
            updated_at: row.try_get("updated_at").map_err(StorageError::Sqlx)?,
            updated_by: row.try_get("updated_by").map_err(StorageError::Sqlx)?,
        })
    }
}
