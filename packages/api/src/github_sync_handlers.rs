// ABOUTME: API handlers for GitHub synchronization
// ABOUTME: Endpoints for syncing Epics and Tasks with GitHub Issues

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::ok_or_internal_error;
use ideate::{EpicManager, GitHubConfig, GitHubSyncService, SyncResult};
use orkee_projects::DbState;

/// Request to sync an Epic to GitHub
#[derive(Debug, Deserialize)]
pub struct SyncEpicRequest {
    #[serde(default)]
    pub create_new: bool, // If false, updates existing issue
}

/// Response with sync results
#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub results: Vec<SyncResult>,
}

/// Sync status response
#[derive(Debug, Serialize)]
pub struct SyncStatusResponse {
    pub syncs: Vec<ideate::GitHubSync>,
}

/// Get GitHub configuration from project
async fn get_github_config(
    pool: &sqlx::SqlitePool,
    project_id: &str,
) -> Result<GitHubConfig, String> {
    // Get project GitHub settings
    use sqlx::Row;
    let project = sqlx::query(
        "SELECT github_owner, github_repo, github_token_encrypted,
               github_labels_config, github_default_assignee, github_sync_enabled
        FROM projects WHERE id = ?"
    )
    .bind(project_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| format!("Database error: {}", e))?
    .ok_or_else(|| "Project not found".to_string())?;

    // Extract fields from row
    let github_sync_enabled: Option<bool> = project.get("github_sync_enabled");
    let github_owner: Option<String> = project.get("github_owner");
    let github_repo: Option<String> = project.get("github_repo");
    let github_token_encrypted: Option<String> = project.get("github_token_encrypted");
    let github_labels_config: Option<String> = project.get("github_labels_config");
    let github_default_assignee: Option<String> = project.get("github_default_assignee");

    // Check if GitHub sync is enabled
    if !github_sync_enabled.unwrap_or(false) {
        return Err("GitHub sync not enabled for this project".to_string());
    }

    // Validate required fields
    let owner = github_owner
        .ok_or_else(|| "GitHub owner not configured".to_string())?;
    let repo = github_repo
        .ok_or_else(|| "GitHub repo not configured".to_string())?;

    let encrypted_token = github_token_encrypted
        .ok_or_else(|| "GitHub token not configured".to_string())?;

    // Decrypt token using security package
    let encryption = security::ApiKeyEncryption::new()
        .map_err(|e| format!("Failed to initialize encryption: {}", e))?;
    let token = encryption.decrypt(&encrypted_token)
        .map_err(|e| format!("Failed to decrypt GitHub token: {}", e))?;

    // Parse labels config if present
    let labels_config = github_labels_config
        .as_ref()
        .and_then(|config_str| serde_json::from_str::<std::collections::HashMap<String, String>>(config_str).ok());

    Ok(GitHubConfig {
        owner,
        repo,
        token,
        labels_config,
        default_assignee: github_default_assignee,
    })
}

/// POST /api/github/sync/epic/:epic_id
/// Sync an Epic to GitHub (create or update issue)
pub async fn sync_epic(
    State(db): State<DbState>,
    Path(epic_id): Path<String>,
    Json(request): Json<SyncEpicRequest>,
) -> impl IntoResponse {
    info!("Syncing Epic {} to GitHub", epic_id);

    // Use EpicManager to get the Epic
    let epic_manager = EpicManager::new(db.pool.clone());

    // First, get just the project_id to avoid fetching full Epic if config fails
    use sqlx::Row;
    let project_id_result = sqlx::query("SELECT project_id FROM epics WHERE id = ?")
        .bind(&epic_id)
        .fetch_optional(&db.pool)
        .await;

    let project_id: String = match project_id_result {
        Ok(Some(row)) => row.get("project_id"),
        Ok(None) => return ok_or_internal_error::<SyncResult, String>(Err("Epic not found".to_string()), "Epic not found"),
        Err(e) => {
            return ok_or_internal_error::<SyncResult, String>(
                Err(format!("Database error: {}", e)),
                "Failed to fetch Epic",
            )
        }
    };

    // Get GitHub config first
    let config = match get_github_config(&db.pool, &project_id).await {
        Ok(cfg) => cfg,
        Err(e) => return ok_or_internal_error::<SyncResult, String>(Err(e), "Failed to get GitHub configuration"),
    };

    // Now get the full Epic using EpicManager
    let epic = match epic_manager.get_epic(&project_id, &epic_id).await {
        Ok(Some(e)) => e,
        Ok(None) => return ok_or_internal_error::<SyncResult, String>(Err("Epic not found".to_string()), "Epic not found"),
        Err(e) => {
            return ok_or_internal_error::<SyncResult, String>(
                Err(e.to_string()),
                "Failed to fetch Epic",
            )
        }
    };

    // Create GitHub service
    let service = GitHubSyncService::new();

    // Sync to GitHub
    let result = if request.create_new || epic.github_issue_number.is_none() {
        service.create_epic_issue(&epic, &config, &db.pool).await
    } else {
        service.sync_epic_to_github(&epic, &config, &db.pool).await
    };

    match result {
        Ok(sync_result) => {
            info!(
                "Successfully synced Epic {} to GitHub issue #{}",
                epic_id, sync_result.issue_number
            );
            ok_or_internal_error::<SyncResult, String>(Ok(sync_result), "")
        }
        Err(e) => ok_or_internal_error::<SyncResult, String>(Err(e.to_string()), "Failed to sync Epic to GitHub"),
    }
}

/// POST /api/github/sync/tasks/:epic_id
/// Create GitHub issues for all tasks in an Epic
pub async fn sync_tasks(
    State(db): State<DbState>,
    Path(epic_id): Path<String>,
) -> impl IntoResponse {
    info!("Syncing tasks for Epic {} to GitHub", epic_id);

    // Get project ID for Epic
    use sqlx::Row;
    let epic_result = sqlx::query("SELECT project_id FROM epics WHERE id = ?")
        .bind(&epic_id)
        .fetch_optional(&db.pool)
        .await;

    let project_id: String = match epic_result {
        Ok(Some(row)) => row.get("project_id"),
        Ok(None) => return ok_or_internal_error::<SyncResponse, String>(Err("Epic not found".to_string()), "Epic not found"),
        Err(e) => {
            return ok_or_internal_error::<SyncResponse, String>(
                Err(format!("Database error: {}", e)),
                "Failed to fetch Epic",
            )
        }
    };

    // Get GitHub config
    let config = match get_github_config(&db.pool, &project_id).await {
        Ok(cfg) => cfg,
        Err(e) => return ok_or_internal_error::<SyncResponse, String>(Err(e), "Failed to get GitHub configuration"),
    };

    // Create GitHub service
    let service = GitHubSyncService::new();

    // Sync tasks
    let result = service
        .create_task_issues(&epic_id, &project_id, &config, &db.pool)
        .await;

    match result {
        Ok(results) => {
            info!("Successfully synced {} tasks to GitHub", results.len());
            ok_or_internal_error::<SyncResponse, String>(Ok(SyncResponse { results }), "")
        }
        Err(e) => ok_or_internal_error::<SyncResponse, String>(Err(e.to_string()), "Failed to sync tasks to GitHub"),
    }
}

/// GET /api/github/sync/status/:project_id
/// Get GitHub sync status for a project
pub async fn get_sync_status(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting GitHub sync status for project {}", project_id);

    // Create GitHub service
    let service = GitHubSyncService::new();

    let syncs = match service.get_sync_status(&db.pool, &project_id).await {
        Ok(s) => s,
        Err(e) => return ok_or_internal_error::<SyncStatusResponse, String>(Err(e.to_string()), "Failed to get sync status"),
    };

    ok_or_internal_error::<SyncStatusResponse, String>(Ok(SyncStatusResponse { syncs }), "")
}
