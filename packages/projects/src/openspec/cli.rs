// ABOUTME: CLI wrapper functions for OpenSpec operations
// ABOUTME: Provides convenient functions that handle database connection internally

use super::archive::{archive_change, ArchiveResult};
use super::db::{
    get_deltas_by_change, get_spec_change, get_spec_changes_by_project, DbResult,
};
use super::markdown_validator::OpenSpecMarkdownValidator;
use super::materializer::{OpenSpecMaterializer, ImportReport};
use super::sync::MergeStrategy;
use super::types::{SpecChange, SpecDelta};
use crate::constants::orkee_dir;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::path::Path;

/// Get the database pool
async fn get_pool() -> DbResult<Pool<Sqlite>> {
    let db_path = orkee_dir().join("orkee.db");
    let database_url = format!("sqlite:{}", db_path.display());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .map_err(|e| super::db::DbError::InvalidInput(format!("Failed to connect to database: {}", e)))?;

    Ok(pool)
}

/// List all changes for a project
pub async fn list_changes(project_id: Option<&str>) -> DbResult<Vec<SpecChange>> {
    let pool = get_pool().await?;

    if let Some(pid) = project_id {
        get_spec_changes_by_project(&pool, pid).await
    } else {
        // Get all changes across all projects
        let changes: Vec<SpecChange> = sqlx::query_as(
            "SELECT * FROM spec_changes WHERE deleted_at IS NULL ORDER BY created_at DESC"
        )
        .fetch_all(&pool)
        .await?;
        Ok(changes)
    }
}

/// Show a specific change with its deltas
pub async fn show_change(change_id: &str) -> DbResult<(SpecChange, Vec<SpecDelta>)> {
    let pool = get_pool().await?;

    let change = get_spec_change(&pool, change_id).await?;
    let deltas = get_deltas_by_change(&pool, change_id).await?;

    Ok((change, deltas))
}

/// Result of validation
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

/// Validate a change
pub async fn validate_change_cli(change_id: &str, strict: bool) -> DbResult<ValidationResult> {
    let pool = get_pool().await?;

    let deltas = get_deltas_by_change(&pool, change_id).await?;
    let validator = OpenSpecMarkdownValidator::new(strict);

    let mut all_errors = Vec::new();

    for delta in deltas {
        let errors = validator.validate_delta_markdown(&delta.delta_markdown);
        all_errors.extend(errors.into_iter().map(|e| e.message));
    }

    Ok(ValidationResult {
        is_valid: all_errors.is_empty(),
        errors: all_errors,
    })
}

/// Archive a change and apply its deltas
pub async fn archive_change_cli(change_id: &str, apply_specs: bool) -> ArchiveResult<()> {
    let pool = get_pool().await?;
    archive_change(&pool, change_id, apply_specs).await
}

/// Export OpenSpec structure to filesystem
pub async fn export_specs(project_id: &str, path: &Path) -> Result<(), String> {
    let pool = get_pool().await.map_err(|e| e.to_string())?;

    let materializer = OpenSpecMaterializer::new(pool);
    materializer
        .materialize_to_path(project_id, path)
        .await
        .map_err(|e| e.to_string())
}

/// Import OpenSpec structure from filesystem
pub async fn import_specs(
    project_id: &str,
    path: &Path,
    force: bool,
) -> Result<ImportReport, String> {
    let pool = get_pool().await.map_err(|e| e.to_string())?;

    let materializer = OpenSpecMaterializer::new(pool);

    // Use PreferRemote strategy if force is true, otherwise PreferLocal
    let strategy = if force {
        MergeStrategy::PreferRemote
    } else {
        MergeStrategy::PreferLocal
    };

    materializer
        .import_from_path(project_id, path, strategy)
        .await
        .map_err(|e| e.to_string())
}
