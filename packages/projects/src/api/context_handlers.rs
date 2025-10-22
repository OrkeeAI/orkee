use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use crate::{
    context::types::{
        ContextConfiguration, ContextGenerationRequest, ContextMetadata, ContextSnapshot,
        FileInfo, GeneratedContext, ListFilesResponse,
    },
    db::DbState,
    manager::ManagerError,
};

/// Query parameters for listing files
#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    #[serde(default)]
    pub max_depth: Option<usize>,
}

/// Generate context for a project
pub async fn generate_context(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
    Json(request): Json<ContextGenerationRequest>,
) -> Result<Json<GeneratedContext>, impl IntoResponse> {
    info!("Generating context for project: {}", project_id);

    // 1. Get project from database
    let project = sqlx::query!(
        "SELECT id, name, project_root FROM projects WHERE id = ?",
        project_id
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database error"
            })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Project not found"
            })),
        )
    })?;

    let project_path = PathBuf::from(&project.project_root);

    // 2. Read files from project directory
    let mut context_content = String::new();
    let mut file_count = 0;
    let mut files_included = Vec::new();

    // Add header
    context_content.push_str(&format!(
        "# Context for Project: {}\n\n",
        project.name
    ));
    context_content.push_str(&format!("Project Root: {}\n\n", project.project_root));
    context_content.push_str("---\n\n");

    // 3. Process each included file/pattern
    for pattern in &request.include_patterns {
        let file_path = project_path.join(pattern);
        
        if file_path.exists() && file_path.is_file() {
            match fs::read_to_string(&file_path) {
                Ok(content) => {
                    // Add file separator
                    context_content.push_str(&format!("\n## File: {}\n\n", pattern));
                    context_content.push_str("```\n");
                    context_content.push_str(&content);
                    context_content.push_str("\n```\n\n");
                    
                    file_count += 1;
                    files_included.push(pattern.clone());
                }
                Err(e) => {
                    error!("Failed to read file {}: {}", pattern, e);
                }
            }
        }
    }

    // 4. Calculate tokens (rough approximation: 1 token ≈ 4 characters)
    let total_tokens = context_content.len() / 4;

    // 5. Check if truncated
    let max_tokens = request.max_tokens.unwrap_or(100000) as usize;
    let truncated = total_tokens > max_tokens;
    
    if truncated {
        // Truncate content to max tokens
        let max_chars = max_tokens * 4;
        context_content.truncate(max_chars);
        context_content.push_str("\n\n[... Content truncated due to token limit ...]");
    }

    // 6. Save snapshot to database
    let metadata = ContextMetadata {
        files_included: files_included.clone(),
        generation_time_ms: 0, // TODO: Track actual generation time
        git_commit: None,      // TODO: Get current git commit
    };
    
    let metadata_json = serde_json::to_string(&metadata).unwrap_or_default();
    
    let snapshot_id = nanoid::nanoid!(16);
    sqlx::query!(
        "INSERT INTO context_snapshots (id, project_id, content, file_count, total_tokens, metadata)
         VALUES (?, ?, ?, ?, ?, ?)",
        snapshot_id,
        project_id,
        context_content,
        file_count as i32,
        total_tokens as i32,
        metadata_json
    )
    .execute(&db.pool)
    .await
    .map_err(|e| {
        error!("Failed to save snapshot: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to save snapshot"
            })),
        )
    })?;

    // 7. Update usage patterns
    for file_path in &files_included {
        let _ = sqlx::query!(
            "INSERT INTO context_usage_patterns (id, project_id, file_path, inclusion_count, last_used)
             VALUES (?, ?, ?, 1, datetime('now'))
             ON CONFLICT(project_id, file_path) DO UPDATE SET
                inclusion_count = inclusion_count + 1,
                last_used = datetime('now')",
            nanoid::nanoid!(16),
            project_id,
            file_path
        )
        .execute(&db.pool)
        .await;
    }

    Ok(Json(GeneratedContext {
        content: context_content,
        file_count,
        total_tokens,
        files_included,
        truncated,
    }))
}

/// List all files in a project directory
pub async fn list_project_files(
    Path(project_id): Path<String>,
    Query(query): Query<ListFilesQuery>,
    State(db): State<DbState>,
) -> Result<Json<ListFilesResponse>, impl IntoResponse> {
    info!("Listing files for project: {}", project_id);

    // Get project from database
    let project = sqlx::query!(
        "SELECT id, name, project_root FROM projects WHERE id = ?",
        project_id
    )
    .fetch_optional(&db.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database error"
            })),
        )
    })?
    .ok_or_else(|| {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Project not found"
            })),
        )
    })?;

    let project_path = PathBuf::from(&project.project_root);
    
    if !project_path.exists() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "Project directory not found"
            })),
        ));
    }

    // Walk directory and collect files
    let mut files = Vec::new();
    let max_depth = query.max_depth.unwrap_or(10);
    
    if let Err(e) = walk_directory(&project_path, &project_path, &mut files, 0, max_depth) {
        error!("Failed to walk directory: {}", e);
        return Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to read directory"
            })),
        ));
    }

    Ok(Json(ListFilesResponse {
        total_count: files.len(),
        files,
    }))
}

/// Recursively walk directory and collect file information
fn walk_directory(
    base_path: &PathBuf,
    current_path: &PathBuf,
    files: &mut Vec<FileInfo>,
    depth: usize,
    max_depth: usize,
) -> Result<(), std::io::Error> {
    if depth > max_depth {
        return Ok(());
    }

    // Skip common directories that should be excluded
    let skip_dirs = [
        "node_modules",
        ".git",
        "target",
        "dist",
        "build",
        ".next",
        ".turbo",
        "coverage",
    ];

    for entry in fs::read_dir(current_path)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        // Get relative path from base
        let relative_path = path
            .strip_prefix(base_path)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();

        // Skip hidden files and directories
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            if name_str.starts_with('.') {
                continue;
            }
            
            // Skip common excluded directories
            if metadata.is_dir() && skip_dirs.contains(&name_str.as_ref()) {
                continue;
            }
        }

        let is_directory = metadata.is_dir();
        let size = metadata.len();
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        files.push(FileInfo {
            path: relative_path,
            size,
            extension,
            is_directory,
        });

        // Recurse into directories
        if is_directory {
            walk_directory(base_path, &path, files, depth + 1, max_depth)?;
        }
    }

    Ok(())
}

/// List saved context configurations for a project
pub async fn list_configurations(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
) -> Result<Json<Vec<ContextConfiguration>>, impl IntoResponse> {
    info!("Listing configurations for project: {}", project_id);

    let configs = sqlx::query_as!(
        ContextConfiguration,
        r#"
        SELECT 
            id,
            project_id,
            name,
            description,
            include_patterns as "include_patterns: Vec<String>",
            exclude_patterns as "exclude_patterns: Vec<String>",
            max_tokens,
            created_at as "created_at: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
        FROM context_configurations
        WHERE project_id = ?
        ORDER BY updated_at DESC
        "#,
        project_id
    )
    .fetch_all(&db.pool)
    .await
    .map_err(|e| {
        error!("Database error: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Database error"
            })),
        )
    })?;

    Ok(Json(configs))
}

/// Save a context configuration
pub async fn save_configuration(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
    Json(config): Json<ContextConfiguration>,
) -> Result<Json<ContextConfiguration>, impl IntoResponse> {
    info!("Saving configuration for project: {}", project_id);

    let config_id = nanoid::nanoid!(16);
    let include_patterns_json = serde_json::to_string(&config.include_patterns).unwrap_or_default();
    let exclude_patterns_json = serde_json::to_string(&config.exclude_patterns).unwrap_or_default();

    sqlx::query!(
        r#"
        INSERT INTO context_configurations 
        (id, project_id, name, description, include_patterns, exclude_patterns, max_tokens)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        "#,
        config_id,
        project_id,
        config.name,
        config.description,
        include_patterns_json,
        exclude_patterns_json,
        config.max_tokens
    )
    .execute(&db.pool)
    .await
    .map_err(|e| {
        error!("Failed to save configuration: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to save configuration"
            })),
        )
    })?;

    // Fetch the created configuration
    let saved_config = sqlx::query_as!(
        ContextConfiguration,
        r#"
        SELECT 
            id,
            project_id,
            name,
            description,
            include_patterns as "include_patterns: Vec<String>",
            exclude_patterns as "exclude_patterns: Vec<String>",
            max_tokens,
            created_at as "created_at: chrono::DateTime<chrono::Utc>",
            updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
        FROM context_configurations
        WHERE id = ?
        "#,
        config_id
    )
    .fetch_one(&db.pool)
    .await
    .map_err(|e| {
        error!("Failed to fetch saved configuration: {}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to fetch configuration"
            })),
        )
    })?;

    Ok(Json(saved_config))
}
