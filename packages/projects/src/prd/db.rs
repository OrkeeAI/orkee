// ABOUTME: Database operations for PRD (Product Requirements Document) entities
// ABOUTME: Provides CRUD operations for PRDs with soft delete support

use super::types::*;
use chrono::Utc;
use sqlx::{Pool, Sqlite};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type DbResult<T> = Result<T, DbError>;

// ============================================================================
// Content Size Limits
// ============================================================================

/// Maximum size for markdown content fields (1MB)
const MAX_MARKDOWN_SIZE: usize = 1024 * 1024;

// ============================================================================
// Pagination Limits
// ============================================================================

/// Maximum allowed value for LIMIT parameter to prevent DoS attacks
const MAX_PAGINATION_LIMIT: i64 = 10000;

/// Maximum allowed value for OFFSET parameter to prevent DoS attacks
const MAX_PAGINATION_OFFSET: i64 = 1000000;

/// Validate markdown content size
fn validate_content_size(content: &str, field_name: &str) -> DbResult<()> {
    if content.len() > MAX_MARKDOWN_SIZE {
        return Err(DbError::InvalidInput(format!(
            "{} exceeds maximum size of {} bytes (got {} bytes). Consider splitting into multiple documents.",
            field_name,
            MAX_MARKDOWN_SIZE,
            content.len()
        )));
    }
    Ok(())
}

/// Validate pagination parameters to prevent DoS attacks
fn validate_pagination(limit: Option<i64>, offset: Option<i64>) -> DbResult<()> {
    if let Some(lim) = limit {
        if lim < 0 {
            return Err(DbError::InvalidInput(
                "LIMIT must be non-negative".to_string(),
            ));
        }
        if lim > MAX_PAGINATION_LIMIT {
            return Err(DbError::InvalidInput(format!(
                "LIMIT exceeds maximum allowed value of {} (got {})",
                MAX_PAGINATION_LIMIT, lim
            )));
        }
    }
    if let Some(off) = offset {
        if off < 0 {
            return Err(DbError::InvalidInput(
                "OFFSET must be non-negative".to_string(),
            ));
        }
        if off > MAX_PAGINATION_OFFSET {
            return Err(DbError::InvalidInput(format!(
                "OFFSET exceeds maximum allowed value of {} (got {})",
                MAX_PAGINATION_OFFSET, off
            )));
        }
    }
    Ok(())
}

// ============================================================================
// PRD Operations
// ============================================================================

/// Create a new PRD
pub async fn create_prd(
    pool: &Pool<Sqlite>,
    project_id: &str,
    title: &str,
    content_markdown: &str,
    status: PRDStatus,
    source: PRDSource,
    created_by: Option<&str>,
) -> DbResult<PRD> {
    // Validate content size
    validate_content_size(content_markdown, "PRD content")?;

    let id = orkee_core::generate_project_id();
    let now = Utc::now();

    let prd = sqlx::query_as::<_, PRD>(
        r#"
        INSERT INTO prds (id, project_id, title, content_markdown, version, status, source, created_at, updated_at, created_by)
        VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(project_id)
    .bind(title)
    .bind(content_markdown)
    .bind(&status)
    .bind(&source)
    .bind(now)
    .bind(now)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(prd)
}

/// Get a PRD by ID
pub async fn get_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<PRD> {
    sqlx::query_as::<_, PRD>("SELECT * FROM prds WHERE id = ? AND deleted_at IS NULL")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("PRD not found: {}", id)))
}

/// Get all PRDs for a project
pub async fn get_prds_by_project(pool: &Pool<Sqlite>, project_id: &str) -> DbResult<Vec<PRD>> {
    let (prds, _) = get_prds_by_project_paginated(pool, project_id, None, None).await?;
    Ok(prds)
}

/// Get all PRDs for a project with pagination
pub async fn get_prds_by_project_paginated(
    pool: &Pool<Sqlite>,
    project_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<PRD>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prds WHERE project_id = ? AND deleted_at IS NULL")
            .bind(project_id)
            .fetch_one(pool)
            .await?;

    // Build query with optional pagination
    let mut query_str = String::from(
        "SELECT * FROM prds WHERE project_id = ? AND deleted_at IS NULL ORDER BY created_at DESC",
    );

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let prds = sqlx::query_as::<_, PRD>(&query_str)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

    Ok((prds, count))
}

/// Update a PRD
pub async fn update_prd(
    pool: &Pool<Sqlite>,
    id: &str,
    title: Option<&str>,
    content_markdown: Option<&str>,
    status: Option<PRDStatus>,
) -> DbResult<PRD> {
    // Validate new content size if provided
    if let Some(content) = content_markdown {
        validate_content_size(content, "PRD content")?;
    }

    // Get current PRD
    let current = get_prd(pool, id).await?;

    let new_title = title.unwrap_or(&current.title);
    let new_content = content_markdown.unwrap_or(&current.content_markdown);
    let new_status = status.unwrap_or(current.status);
    let now = Utc::now();

    let prd = sqlx::query_as::<_, PRD>(
        r#"
        UPDATE prds
        SET title = ?, content_markdown = ?, status = ?, updated_at = ?, version = version + 1
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(new_title)
    .bind(new_content)
    .bind(&new_status)
    .bind(now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(prd)
}

/// Soft delete a PRD
pub async fn delete_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let now = Utc::now();
    let result = sqlx::query("UPDATE prds SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("PRD not found: {}", id)));
    }

    Ok(())
}

/// Hard delete a PRD (for testing/admin purposes)
pub async fn hard_delete_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM prds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("PRD not found: {}", id)));
    }

    Ok(())
}

/// Restore a soft-deleted PRD
pub async fn restore_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<PRD> {
    let prd = sqlx::query_as::<_, PRD>(
        "UPDATE prds SET deleted_at = NULL WHERE id = ? AND deleted_at IS NOT NULL RETURNING *",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DbError::NotFound(format!("Deleted PRD not found: {}", id)))?;

    Ok(prd)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("../storage/migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }

    async fn create_test_project(pool: &Pool<Sqlite>, project_id: &str) {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, project_root, created_at, updated_at)
            VALUES (?, 'Test Project', '/test/path', datetime('now'), datetime('now'))
            "#,
        )
        .bind(project_id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_get_prd() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        let prd = create_prd(
            &pool,
            "test-project",
            "Test PRD",
            "# Test Content",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        assert_eq!(prd.title, "Test PRD");
        assert_eq!(prd.status, PRDStatus::Draft);

        let fetched = get_prd(&pool, &prd.id).await.unwrap();
        assert_eq!(fetched.id, prd.id);
    }

    #[tokio::test]
    async fn test_pagination_limit_validation() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Test LIMIT exceeds maximum
        let result = get_prds_by_project_paginated(&pool, "test-project", Some(20000), None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test OFFSET exceeds maximum
        let result =
            get_prds_by_project_paginated(&pool, "test-project", None, Some(2000000)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test negative LIMIT
        let result = get_prds_by_project_paginated(&pool, "test-project", Some(-1), None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test negative OFFSET
        let result = get_prds_by_project_paginated(&pool, "test-project", None, Some(-1)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test valid pagination should work
        let result = get_prds_by_project_paginated(&pool, "test-project", Some(10), Some(0)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_soft_delete_count_consistency() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create multiple PRDs
        let prd1 = create_prd(
            &pool,
            "test-project",
            "PRD 1",
            "Content 1",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        let prd2 = create_prd(
            &pool,
            "test-project",
            "PRD 2",
            "Content 2",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        // Soft delete one PRD
        delete_prd(&pool, &prd2.id).await.unwrap();

        // Get PRDs with pagination
        let (prds, count) = get_prds_by_project_paginated(&pool, "test-project", None, None)
            .await
            .unwrap();

        // Count should match the actual number of returned PRDs (both should exclude deleted)
        assert_eq!(
            prds.len() as i64,
            count,
            "Count mismatch: pagination count should match actual results (excluding deleted)"
        );
        assert_eq!(prds.len(), 1, "Should only return non-deleted PRDs");
        assert_eq!(prds[0].id, prd1.id, "Should return the non-deleted PRD");
    }
}
