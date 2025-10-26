// ABOUTME: Storage operations for system settings
// ABOUTME: Database CRUD for runtime configuration

use crate::settings::types::{BulkSettingUpdate, SettingUpdate, SystemSetting};
use storage::StorageError;
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
        // Get the setting first to check is_env_only and data_type
        let setting = self.get(key).await?;

        // Enforce is_env_only restriction
        if setting.is_env_only {
            return Err(StorageError::EnvOnly(key.to_string()));
        }

        // Validate the new value
        crate::settings::validation::validate_setting_value(
            key,
            &update.value,
            &setting.data_type,
        )?;

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
        // Pre-validate all settings before starting transaction
        let mut env_only_keys = Vec::new();
        let mut validation_errors = Vec::new();

        for item in &updates.settings {
            // Get the setting to check is_env_only and data_type
            let setting = self.get(&item.key).await?;

            // Check is_env_only restriction
            if setting.is_env_only {
                env_only_keys.push(item.key.clone());
            }

            // Validate the value
            if let Err(e) = crate::settings::validation::validate_setting_value(
                &item.key,
                &item.value,
                &setting.data_type,
            ) {
                validation_errors.push(format!("{}: {}", item.key, e));
            }
        }

        // If any settings are env-only, reject the entire batch
        if !env_only_keys.is_empty() {
            return Err(StorageError::Validation(format!(
                "The following settings are environment-only and cannot be modified via API: {}",
                env_only_keys.join(", ")
            )));
        }

        // If any validation errors, reject the entire batch
        if !validation_errors.is_empty() {
            return Err(StorageError::Validation(format!(
                "Validation failed for {} setting(s): {}",
                validation_errors.len(),
                validation_errors.join("; ")
            )));
        }

        // All validations passed, proceed with transaction
        let mut tx = self.pool.begin().await.map_err(StorageError::Sqlx)?;

        // Execute all updates in transaction, with explicit rollback logging on error
        let update_result: Result<(), StorageError> = async {
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
            Ok(())
        }
        .await;

        match update_result {
            Ok(_) => {
                tx.commit().await.map_err(StorageError::Sqlx)?;
            }
            Err(e) => {
                tracing::error!(
                    "Transaction failed during bulk_update, rolling back {} settings: {}",
                    updates.settings.len(),
                    e
                );
                // Transaction will auto-rollback on drop, but we explicitly log it
                return Err(e);
            }
        }

        // Fetch all updated settings in a single query
        let keys: Vec<String> = updates.settings.iter().map(|s| s.key.clone()).collect();
        if keys.is_empty() {
            return Ok(Vec::new());
        }

        // Build query with IN clause
        // SQL Injection Safety: The dynamic SQL construction here is safe because:
        // 1. `placeholders` contains only "?" characters (count matches keys.len())
        // 2. No user input is interpolated into the query string via format!()
        // 3. All actual key values are bound as parameters via query.bind() below
        // Example: keys = ["a", "b"] -> "... IN (?,?)" then bind("a"), bind("b")
        let placeholders = keys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query_str = format!(
            "SELECT * FROM system_settings WHERE key IN ({}) ORDER BY category, key",
            placeholders
        );

        let mut query = sqlx::query(&query_str);
        for key in &keys {
            query = query.bind(key);
        }

        let rows = query
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let results = rows
            .into_iter()
            .map(|row| self.row_to_setting(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(results)
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
