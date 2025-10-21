// ABOUTME: Tag storage layer using SQLite
// ABOUTME: Handles CRUD operations for tags with archive support

use chrono::Utc;
use sqlx::{Row, SqlitePool};
use tracing::debug;

use super::types::{Tag, TagCreateInput, TagUpdateInput};
use crate::storage::StorageError;

pub struct TagStorage {
    pool: SqlitePool,
}

impl TagStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// List all tags, optionally including archived tags
    pub async fn list_tags(&self, include_archived: bool) -> Result<Vec<Tag>, StorageError> {
        let (tags, _) = self.list_tags_paginated(include_archived, None, None).await?;
        Ok(tags)
    }

    /// List all tags with pagination
    pub async fn list_tags_paginated(
        &self,
        include_archived: bool,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<(Vec<Tag>, i64), StorageError> {
        debug!("Fetching tags (include_archived: {}, limit: {:?}, offset: {:?})", include_archived, limit, offset);

        // Get total count
        let count_query = if include_archived {
            "SELECT COUNT(*) FROM tags"
        } else {
            "SELECT COUNT(*) FROM tags WHERE archived_at IS NULL"
        };

        let count: i64 = sqlx::query_scalar(count_query)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        // Build query with optional pagination
        let mut query = if include_archived {
            String::from("SELECT * FROM tags ORDER BY name")
        } else {
            String::from("SELECT * FROM tags WHERE archived_at IS NULL ORDER BY name")
        };

        if let Some(lim) = limit {
            query.push_str(&format!(" LIMIT {}", lim));
        }
        if let Some(off) = offset {
            query.push_str(&format!(" OFFSET {}", off));
        }

        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let tags = rows
            .iter()
            .map(|row| self.row_to_tag(row))
            .collect::<Result<Vec<_>, _>>()?;

        Ok((tags, count))
    }

    /// Get a single tag by ID
    pub async fn get_tag(&self, tag_id: &str) -> Result<Tag, StorageError> {
        debug!("Fetching tag: {}", tag_id);

        let row = sqlx::query("SELECT * FROM tags WHERE id = ?")
            .bind(tag_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.row_to_tag(&row)
    }

    /// Get a tag by name
    pub async fn get_tag_by_name(&self, name: &str) -> Result<Option<Tag>, StorageError> {
        debug!("Fetching tag by name: {}", name);

        let row = sqlx::query("SELECT * FROM tags WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        match row {
            Some(r) => Ok(Some(self.row_to_tag(&r)?)),
            None => Ok(None),
        }
    }

    /// Create a new tag
    pub async fn create_tag(&self, input: TagCreateInput) -> Result<Tag, StorageError> {
        let tag_id = format!("tag-{}", nanoid::nanoid!());
        let now = Utc::now();

        debug!("Creating tag: {} (name: {})", tag_id, input.name);

        sqlx::query(
            r#"
            INSERT INTO tags (id, name, color, description, created_at)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&tag_id)
        .bind(&input.name)
        .bind(&input.color)
        .bind(&input.description)
        .bind(now)
        .execute(&self.pool)
        .await
        .map_err(StorageError::Sqlx)?;

        self.get_tag(&tag_id).await
    }

    /// Update a tag
    pub async fn update_tag(
        &self,
        tag_id: &str,
        input: TagUpdateInput,
    ) -> Result<Tag, StorageError> {
        debug!("Updating tag: {}", tag_id);

        // Build update query dynamically based on provided fields
        let mut query_parts = Vec::new();

        if input.name.is_some() {
            query_parts.push("name = ?");
        }
        if input.color.is_some() {
            query_parts.push("color = ?");
        }
        if input.description.is_some() {
            query_parts.push("description = ?");
        }
        if input.archived_at.is_some() {
            query_parts.push("archived_at = ?");
        }

        if query_parts.is_empty() {
            return self.get_tag(tag_id).await;
        }

        let query_str = format!("UPDATE tags SET {} WHERE id = ?", query_parts.join(", "));
        let mut query = sqlx::query(&query_str);

        // Bind parameters in the same order
        if let Some(name) = input.name {
            query = query.bind(name);
        }
        if let Some(color) = input.color {
            query = query.bind(color);
        }
        if let Some(description) = input.description {
            query = query.bind(description);
        }
        if let Some(archived_at) = input.archived_at {
            query = query.bind(archived_at);
        }

        query = query.bind(tag_id);

        query
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.get_tag(tag_id).await
    }

    /// Archive a tag (soft delete)
    pub async fn archive_tag(&self, tag_id: &str) -> Result<Tag, StorageError> {
        debug!("Archiving tag: {}", tag_id);

        let now = Utc::now();

        sqlx::query("UPDATE tags SET archived_at = ? WHERE id = ?")
            .bind(now)
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.get_tag(tag_id).await
    }

    /// Unarchive a tag
    pub async fn unarchive_tag(&self, tag_id: &str) -> Result<Tag, StorageError> {
        debug!("Unarchiving tag: {}", tag_id);

        sqlx::query("UPDATE tags SET archived_at = NULL WHERE id = ?")
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        self.get_tag(tag_id).await
    }

    /// Delete a tag permanently (only if no tasks are using it)
    pub async fn delete_tag(&self, tag_id: &str) -> Result<(), StorageError> {
        debug!("Deleting tag: {}", tag_id);

        // Check if any tasks are using this tag
        let row = sqlx::query("SELECT COUNT(*) as count FROM tasks WHERE tag_id = ?")
            .bind(tag_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        let count: i64 = row.try_get("count").map_err(StorageError::Sqlx)?;

        if count > 0 {
            return Err(StorageError::Database(format!(
                "Cannot delete tag '{}': {} tasks are using it. Archive it instead.",
                tag_id, count
            )));
        }

        sqlx::query("DELETE FROM tags WHERE id = ?")
            .bind(tag_id)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Sqlx)?;

        Ok(())
    }

    /// Convert a database row to a Tag
    fn row_to_tag(&self, row: &sqlx::sqlite::SqliteRow) -> Result<Tag, StorageError> {
        Ok(Tag {
            id: row.try_get("id").map_err(StorageError::Sqlx)?,
            name: row.try_get("name").map_err(StorageError::Sqlx)?,
            color: row.try_get("color").map_err(StorageError::Sqlx)?,
            description: row.try_get("description").map_err(StorageError::Sqlx)?,
            created_at: row.try_get("created_at").map_err(StorageError::Sqlx)?,
            archived_at: row.try_get("archived_at").map_err(StorageError::Sqlx)?,
        })
    }
}
