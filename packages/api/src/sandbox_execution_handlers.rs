// ABOUTME: HTTP request handlers for sandbox execution operations
// ABOUTME: Handles logs, artifacts, and lifecycle operations for containerized executions

use axum::{
    extract::{Path, Query, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use orkee_sandboxes::{ExecutionOrchestrator, ExecutionStorage, LogEntry};

use super::response::ok_or_internal_error;

/// Shared state for sandbox execution operations
#[derive(Clone)]
pub struct SandboxExecutionState {
    pub orchestrator: Arc<ExecutionOrchestrator>,
    pub storage: Arc<ExecutionStorage>,
}

// ==================== Execution Control ====================

/// Request to stop an execution
#[derive(Deserialize)]
pub struct StopExecutionRequest {
    #[serde(rename = "containerId")]
    pub container_id: String,
}

/// Stop a running execution
///
/// POST /api/sandbox/executions/:execution_id/stop
pub async fn stop_execution(
    State(state): State<SandboxExecutionState>,
    Path(execution_id): Path<String>,
    Json(request): Json<StopExecutionRequest>,
) -> impl IntoResponse {
    info!("Stopping execution: {}", execution_id);

    let result = state
        .orchestrator
        .stop_execution(&execution_id, &request.container_id)
        .await
        .map(|_| "Execution stopped successfully");

    ok_or_internal_error(result, "Failed to stop execution")
}

/// Request to retry an execution
#[derive(Deserialize)]
pub struct RetryExecutionRequest {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "agentId")]
    pub agent_id: Option<String>,
    pub model: Option<String>,
}

/// Retry a failed execution
///
/// POST /api/sandbox/executions/:execution_id/retry
pub async fn retry_execution(
    State(_state): State<SandboxExecutionState>,
    Path(_execution_id): Path<String>,
    Json(_request): Json<RetryExecutionRequest>,
) -> impl IntoResponse {
    // TODO: Implement retry logic by creating a new execution with retry_attempt incremented
    ok_or_internal_error(
        Err::<&str, _>(orkee_sandboxes::SandboxError::Unknown(
            "Retry not yet implemented".to_string(),
        )),
        "Retry functionality coming in Phase 5",
    )
}

// ==================== Log Operations ====================

/// Query parameters for log listing
#[derive(Deserialize)]
pub struct LogQueryParams {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Response for log listing
#[derive(Serialize)]
pub struct LogsResponse {
    pub logs: Vec<LogEntry>,
    pub total: i64,
}

/// Get paginated logs for an execution
///
/// GET /api/sandbox/executions/:execution_id/logs
pub async fn get_execution_logs(
    State(state): State<SandboxExecutionState>,
    Path(execution_id): Path<String>,
    Query(params): Query<LogQueryParams>,
) -> impl IntoResponse {
    info!("Getting logs for execution: {}", execution_id);

    let result = state
        .storage
        .get_logs(&execution_id, params.limit, params.offset)
        .await
        .map(|(logs, total)| LogsResponse { logs, total });

    ok_or_internal_error(result, "Failed to get logs")
}

/// Query parameters for log streaming
#[derive(Deserialize)]
pub struct StreamLogsParams {
    #[serde(rename = "lastSequence")]
    pub last_sequence: Option<i64>,
}

/// Stream logs via Server-Sent Events (SSE)
///
/// GET /api/sandbox/executions/:execution_id/logs/stream
///
/// Note: This is a placeholder for Phase 5 SSE implementation
pub async fn stream_execution_logs(
    State(_state): State<SandboxExecutionState>,
    Path(_execution_id): Path<String>,
    Query(_params): Query<StreamLogsParams>,
) -> impl IntoResponse {
    // TODO: Implement SSE streaming in Phase 5
    ok_or_internal_error(
        Err::<&str, _>(orkee_sandboxes::SandboxError::Unknown(
            "SSE streaming not yet implemented".to_string(),
        )),
        "SSE log streaming coming in Phase 5",
    )
}

/// Query parameters for log search
#[derive(Deserialize)]
pub struct SearchLogsParams {
    #[serde(rename = "logLevel")]
    pub log_level: Option<String>,
    pub source: Option<String>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// Search logs with filters
///
/// GET /api/sandbox/executions/:execution_id/logs/search
pub async fn search_logs(
    State(state): State<SandboxExecutionState>,
    Path(execution_id): Path<String>,
    Query(params): Query<SearchLogsParams>,
) -> impl IntoResponse {
    info!("Searching logs for execution: {}", execution_id);

    let result = state
        .storage
        .search_logs(
            &execution_id,
            params.log_level.as_deref(),
            params.source.as_deref(),
            params.search.as_deref(),
            params.limit,
            params.offset,
        )
        .await
        .map(|(logs, total)| LogsResponse { logs, total });

    ok_or_internal_error(result, "Failed to search logs")
}

// ==================== Artifact Operations ====================

/// List artifacts for an execution
///
/// GET /api/sandbox/executions/:execution_id/artifacts
pub async fn list_artifacts(
    State(state): State<SandboxExecutionState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing artifacts for execution: {}", execution_id);

    let result = state.storage.list_artifacts(&execution_id).await;
    ok_or_internal_error(result, "Failed to list artifacts")
}

/// Get artifact metadata
///
/// GET /api/sandbox/artifacts/:artifact_id
pub async fn get_artifact(
    State(state): State<SandboxExecutionState>,
    Path(artifact_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting artifact: {}", artifact_id);

    let result = state.storage.get_artifact(&artifact_id).await;
    ok_or_internal_error(result, "Failed to get artifact")
}

/// Download artifact file
///
/// GET /api/sandbox/artifacts/:artifact_id/download
pub async fn download_artifact(
    State(state): State<SandboxExecutionState>,
    Path(artifact_id): Path<String>,
) -> impl IntoResponse {
    info!("Downloading artifact: {}", artifact_id);

    // Get artifact metadata
    let artifact = match state.storage.get_artifact(&artifact_id).await {
        Ok(art) => art,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to get artifact: {}", e),
            )
                .into_response();
        }
    };

    // Check if artifact has a stored path
    let stored_path = match artifact.stored_path {
        Some(path) => path,
        None => {
            return (
                StatusCode::NOT_FOUND,
                "Artifact file not found on storage",
            )
                .into_response();
        }
    };

    // Read the file
    let file_bytes = match tokio::fs::read(&stored_path).await {
        Ok(bytes) => bytes,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to read artifact file: {}", e),
            )
                .into_response();
        }
    };

    // Return file with appropriate headers
    let mime_type = artifact
        .mime_type
        .unwrap_or_else(|| "application/octet-stream".to_string());

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, mime_type),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{}\"", artifact.file_name),
            ),
        ],
        file_bytes,
    )
        .into_response()
}

/// Delete an artifact
///
/// DELETE /api/sandbox/artifacts/:artifact_id
pub async fn delete_artifact(
    State(state): State<SandboxExecutionState>,
    Path(artifact_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting artifact: {}", artifact_id);

    // Get artifact metadata first to get the stored path
    let artifact = match state.storage.get_artifact(&artifact_id).await {
        Ok(art) => art,
        Err(e) => {
            return ok_or_internal_error(Err::<&str, _>(e), "Failed to get artifact");
        }
    };

    // Delete the file if it exists
    if let Some(stored_path) = &artifact.stored_path {
        if let Err(e) = tokio::fs::remove_file(stored_path).await {
            tracing::warn!(
                "Failed to delete artifact file {}: {}",
                stored_path,
                e
            );
        }
    }

    // Delete from database
    let result = state
        .storage
        .delete_artifact(&artifact_id)
        .await
        .map(|_| "Artifact deleted successfully");

    ok_or_internal_error(result, "Failed to delete artifact")
}
