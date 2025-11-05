// ABOUTME: HTTP request handlers for Docker container management operations
// ABOUTME: Provides REST API for container lifecycle, monitoring, and cleanup

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

use orkee_sandboxes::{ContainerInfo, ContainerManager};

use super::response::ok_or_internal_error;

/// Shared state for container operations
#[derive(Clone)]
pub struct ContainerState {
    pub container_manager: Arc<ContainerManager>,
}

/// Response for list containers endpoint
#[derive(Serialize)]
pub struct ListContainersResponse {
    pub containers: Vec<ContainerInfoDto>,
}

/// Container info DTO for API responses
#[derive(Serialize)]
pub struct ContainerInfoDto {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: i64,
}

impl From<ContainerInfo> for ContainerInfoDto {
    fn from(info: ContainerInfo) -> Self {
        Self {
            id: info.id,
            name: info.name,
            image: info.image,
            status: info.status,
            state: info.state,
            created: info.created,
        }
    }
}

/// List all Orkee-managed containers
///
/// GET /api/containers
pub async fn list_containers(State(state): State<ContainerState>) -> impl IntoResponse {
    info!("Listing all Orkee-managed containers");

    let result = state
        .container_manager
        .list_containers(None, None)
        .await
        .map(|containers| {
            let dtos: Vec<ContainerInfoDto> = containers.into_iter().map(Into::into).collect();
            ListContainersResponse { containers: dtos }
        });

    ok_or_internal_error(result, "Failed to list containers")
}

/// Get container details by ID
///
/// GET /api/containers/:id
pub async fn get_container(
    State(state): State<ContainerState>,
    Path(container_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting container details: {}", container_id);

    let result = state
        .container_manager
        .get_container_info(&container_id)
        .await
        .map(|info| serde_json::to_value(info).unwrap_or_default());

    ok_or_internal_error(result, "Failed to get container")
}

/// Response for stats endpoint
#[derive(Serialize)]
pub struct ContainerStatsResponse {
    #[serde(rename = "memoryUsedMb")]
    pub memory_used_mb: u64,
    #[serde(rename = "cpuUsagePercent")]
    pub cpu_usage_percent: f64,
}

/// Get container resource stats (CPU, memory)
///
/// GET /api/containers/:id/stats
pub async fn get_container_stats(
    State(state): State<ContainerState>,
    Path(container_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting stats for container: {}", container_id);

    let result = state
        .container_manager
        .get_container_stats(&container_id)
        .await
        .map(|usage| ContainerStatsResponse {
            memory_used_mb: usage.memory_used_mb,
            cpu_usage_percent: usage.cpu_usage_percent,
        });

    ok_or_internal_error(result, "Failed to get container stats")
}

/// Request body for stop container endpoint
#[derive(Deserialize)]
pub struct StopContainerRequest {
    /// Timeout in seconds before force killing
    #[serde(rename = "timeoutSecs")]
    pub timeout_secs: Option<i64>,
}

/// Stop a container gracefully
///
/// POST /api/containers/:id/stop
pub async fn stop_container(
    State(state): State<ContainerState>,
    Path(container_id): Path<String>,
    Json(request): Json<StopContainerRequest>,
) -> impl IntoResponse {
    info!("Stopping container: {}", container_id);

    let result = state
        .container_manager
        .stop_container(&container_id, request.timeout_secs)
        .await
        .map(|_| "Container stopped successfully");

    ok_or_internal_error(result, "Failed to stop container")
}

/// Restart a container
///
/// POST /api/containers/:id/restart
pub async fn restart_container(
    State(state): State<ContainerState>,
    Path(container_id): Path<String>,
) -> impl IntoResponse {
    info!("Restarting container: {}", container_id);

    // Stop the container first
    let stop_result = state
        .container_manager
        .stop_container(&container_id, Some(10))
        .await;

    if let Err(e) = stop_result {
        return ok_or_internal_error(Err::<&str, _>(e), "Failed to restart container");
    }

    // Start the container again
    let result = state
        .container_manager
        .start_container(&container_id)
        .await
        .map(|_| "Container restarted successfully");

    ok_or_internal_error(result, "Failed to restart container")
}

/// Request body for delete container endpoint
#[derive(Deserialize)]
pub struct DeleteContainerRequest {
    /// Force remove even if running
    #[serde(default)]
    pub force: bool,
}

/// Remove a container
///
/// DELETE /api/containers/:id
pub async fn delete_container(
    State(state): State<ContainerState>,
    Path(container_id): Path<String>,
    Json(request): Json<DeleteContainerRequest>,
) -> impl IntoResponse {
    info!(
        "Deleting container: {} (force={})",
        container_id, request.force
    );

    let result = state
        .container_manager
        .remove_container(&container_id, request.force)
        .await
        .map(|_| "Container deleted successfully");

    ok_or_internal_error(result, "Failed to delete container")
}
