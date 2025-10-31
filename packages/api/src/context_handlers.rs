use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info};

use context::types::{
    ContextConfiguration, ContextGenerationRequest, ContextMetadata, FileInfo, GeneratedContext,
    ListFilesResponse,
};
use orkee_projects::DbState;

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
    context_content.push_str(&format!("# Context for Project: {}\n\n", project.name));
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
    let file_count_i32 = file_count as i32;
    let total_tokens_i32 = total_tokens as i32;
    sqlx::query!(
        "INSERT INTO context_snapshots (id, project_id, content, file_count, total_tokens, metadata)
         VALUES (?, ?, ?, ?, ?, ?)",
        snapshot_id,
        project_id,
        context_content,
        file_count_i32,
        total_tokens_i32,
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
        let pattern_id = nanoid::nanoid!(16);
        let _ = sqlx::query!(
            "INSERT INTO context_usage_patterns (id, project_id, file_path, inclusion_count, last_used)
             VALUES (?, ?, ?, 1, datetime('now'))
             ON CONFLICT(project_id, file_path) DO UPDATE SET
                inclusion_count = inclusion_count + 1,
                last_used = datetime('now')",
            pattern_id,
            project_id,
            file_path
        )
        .execute(&db.pool)
        .await;
    }

    Ok::<_, (StatusCode, Json<serde_json::Value>)>(Json(GeneratedContext {
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

    let configs = sqlx::query_as::<_, ContextConfiguration>(
        "SELECT 
            id,
            project_id,
            name,
            description,
            include_patterns,
            exclude_patterns,
            max_tokens,
            created_at,
            updated_at,
            spec_capability_id
        FROM context_configurations
        WHERE project_id = ?
        ORDER BY updated_at DESC",
    )
    .bind(&project_id)
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

    Ok::<_, (StatusCode, Json<serde_json::Value>)>(Json(configs))
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
    let saved_config = sqlx::query_as::<_, ContextConfiguration>(
        "SELECT 
            id,
            project_id,
            name,
            description,
            include_patterns,
            exclude_patterns,
            max_tokens,
            created_at,
            updated_at,
            spec_capability_id
        FROM context_configurations
        WHERE id = ?",
    )
    .bind(&config_id)
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

    Ok::<_, (StatusCode, Json<serde_json::Value>)>(Json(saved_config))
}

/// Request body for generating context from PRD
#[derive(Debug, Deserialize)]
pub struct GeneratePRDContextRequest {
    pub prd_id: String,
}

/// Generate context from a PRD
pub async fn generate_prd_context(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
    Json(request): Json<GeneratePRDContextRequest>,
) -> Result<Json<GeneratedContext>, impl IntoResponse> {
    info!(
        "Generating PRD context for project: {}, PRD: {}",
        project_id, request.prd_id
    );

    // Get PRD from database
    let prd = match orkee_projects::get_prd(&db.pool, &request.prd_id).await {
        Ok(prd) => prd,
        Err(e) => {
            error!("Failed to fetch PRD {}: {}", request.prd_id, e);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("PRD not found: {}", e)
                })),
            ));
        }
    };

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

    // Generate simple context from PRD content
    let context_content = format!(
        "# PRD Context\n\n## Project: {}\n\n## PRD: {}\n\n{}\n",
        project.name, prd.title, prd.content_markdown
    );

    let total_tokens = context_content.len() / 4;

    Ok(Json(GeneratedContext {
        content: context_content,
        file_count: 0,
        total_tokens,
        files_included: vec![],
        truncated: false,
    })) as Result<Json<GeneratedContext>, (StatusCode, Json<serde_json::Value>)>
}

/// Request body for generating context from task
#[derive(Debug, Deserialize)]
pub struct GenerateTaskContextRequest {
    pub task_id: String,
}

/// Generate context from a task
pub async fn generate_task_context(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
    Json(request): Json<GenerateTaskContextRequest>,
) -> Result<Json<GeneratedContext>, impl IntoResponse> {
    info!(
        "Generating task context for project: {}, task: {}",
        project_id, request.task_id
    );

    // Get task from database
    let task = match db.task_storage.get_task(&request.task_id).await {
        Ok(task) => task,
        Err(e) => {
            error!("Failed to fetch task {}: {}", request.task_id, e);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": format!("Task not found: {}", e)
                })),
            ));
        }
    };

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

    // Generate simple context from task details
    let context_content = format!(
        "# Task Context\n\n## Project: {}\n\n## Task: {}\n\nDescription: {}\n\nStatus: {:?}\n",
        project.name,
        task.title,
        task.description.as_deref().unwrap_or("No description"),
        task.status
    );

    let total_tokens = context_content.len() / 4;

    Ok(Json(GeneratedContext {
        content: context_content,
        file_count: 0,
        total_tokens,
        files_included: vec![],
        truncated: false,
    })) as Result<Json<GeneratedContext>, (StatusCode, Json<serde_json::Value>)>
}

// Spec validation has been removed as it was part of the OpenSpec functionality
// which has been deprecated in favor of CCPM (Conversational Mode)

/// Get context history for a project
pub async fn get_context_history(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    info!("Getting context history for project: {}", project_id);

    let snapshots = sqlx::query!(
        r#"
        SELECT 
            id,
            project_id,
            content,
            CAST(file_count AS INTEGER) as "file_count!: i32",
            CAST(total_tokens AS INTEGER) as "total_tokens!: i32",
            metadata,
            created_at
        FROM context_snapshots
        WHERE project_id = ?
        ORDER BY created_at DESC
        LIMIT 50
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

    let history: Vec<serde_json::Value> = snapshots
        .iter()
        .map(|s| {
            serde_json::json!({
                "id": s.id,
                "project_id": s.project_id,
                "file_count": s.file_count,
                "total_tokens": s.total_tokens,
                "created_at": s.created_at,
                "metadata": serde_json::from_str::<serde_json::Value>(&s.metadata).ok()
            })
        })
        .collect();

    Ok::<_, (StatusCode, Json<serde_json::Value>)>(Json(serde_json::json!({
        "snapshots": history
    })))
}

/// Get context usage statistics
pub async fn get_context_stats(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
) -> Result<impl IntoResponse, impl IntoResponse> {
    info!("Getting context stats for project: {}", project_id);

    // Get snapshot count
    let snapshot_count = sqlx::query!(
        "SELECT CAST(COUNT(*) AS INTEGER) as \"count!: i32\" FROM context_snapshots WHERE project_id = ?",
        project_id
    )
    .fetch_one(&db.pool)
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

    // Get most used files
    let most_used = sqlx::query!(
        r#"
        SELECT file_path, CAST(inclusion_count AS INTEGER) as "inclusion_count!: i32", last_used
        FROM context_usage_patterns
        WHERE project_id = ?
        ORDER BY inclusion_count DESC
        LIMIT 10
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

    let most_used_files: Vec<serde_json::Value> = most_used
        .iter()
        .map(|f| {
            serde_json::json!({
                "path": f.file_path,
                "inclusion_count": f.inclusion_count,
                "last_used": f.last_used
            })
        })
        .collect();

    Ok::<_, (StatusCode, Json<serde_json::Value>)>(Json(serde_json::json!({
        "total_snapshots": snapshot_count.count,
        "most_used_files": most_used_files,
    })))
}

/// Request body for restoring context snapshot
#[derive(Debug, Deserialize)]
pub struct RestoreContextRequest {
    pub snapshot_id: String,
}

/// Restore a context snapshot
pub async fn restore_context_snapshot(
    Path(project_id): Path<String>,
    State(db): State<DbState>,
    Json(request): Json<RestoreContextRequest>,
) -> Result<Json<GeneratedContext>, impl IntoResponse> {
    info!(
        "Restoring context snapshot for project: {}, snapshot: {}",
        project_id, request.snapshot_id
    );

    // Get snapshot from database
    let snapshot = sqlx::query!(
        r#"
        SELECT 
            id,
            project_id,
            content,
            CAST(file_count AS INTEGER) as "file_count!: i32",
            CAST(total_tokens AS INTEGER) as "total_tokens!: i32",
            metadata
        FROM context_snapshots
        WHERE id = ? AND project_id = ?
        "#,
        request.snapshot_id,
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
                "error": "Snapshot not found"
            })),
        )
    })?;

    // Parse metadata to get files_included
    let metadata: ContextMetadata =
        serde_json::from_str(&snapshot.metadata).unwrap_or(ContextMetadata {
            files_included: vec![],
            generation_time_ms: 0,
            git_commit: None,
        });

    Ok::<_, (StatusCode, Json<serde_json::Value>)>(Json(GeneratedContext {
        content: snapshot.content,
        file_count: snapshot.file_count as usize,
        total_tokens: snapshot.total_tokens as usize,
        files_included: metadata.files_included,
        truncated: false,
    }))
}
