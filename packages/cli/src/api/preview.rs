use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use orkee_preview::{
    types::{
        ApiResponse, ServerLogsResponse, ServerStatusInfo, ServerStatusResponse, ServersResponse,
        StartServerRequest, StartServerResponse,
    },
    PreviewManager, ServerInfo,
};
use orkee_projects::manager::ProjectsManager;
use serde::Deserialize;
use std::sync::Arc;
use tracing::{error, info};

/// Shared state for preview endpoints
#[derive(Clone)]
pub struct PreviewState {
    pub preview_manager: Arc<PreviewManager>,
    pub project_manager: Arc<ProjectsManager>,
}

/// Start a development server for a project
#[axum::debug_handler]
pub async fn start_server(
    Path(project_id): Path<String>,
    State(state): State<PreviewState>,
    Json(_request): Json<StartServerRequest>,
) -> Result<Json<ApiResponse<StartServerResponse>>, StatusCode> {
    info!("Starting simple preview server for project: {}", project_id);

    // Get project from projects service
    let project = match state.project_manager.get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            error!("Project not found: {}", project_id);
            return Ok(Json(ApiResponse::error("Project not found")));
        }
        Err(e) => {
            error!("Failed to get project {}: {}", project_id, e);
            return Ok(Json(ApiResponse::error(format!(
                "Project manager error: {}",
                e
            ))));
        }
    };

    let project_root = std::path::PathBuf::from(&project.project_root);

    // Start the simplified server
    match state
        .preview_manager
        .start_server(project_id.clone(), project_root)
        .await
    {
        Ok(server_info) => {
            info!("Successfully started server: {}", server_info.id);

            // Convert ServerInfo to DevServerInstance for compatibility
            let instance = convert_server_info_to_instance(server_info);
            Ok(Json(ApiResponse::success(StartServerResponse { instance })))
        }
        Err(e) => {
            error!("Failed to start server: {}", e);
            Ok(Json(ApiResponse::error(format!(
                "Preview server error: {}",
                e
            ))))
        }
    }
}

/// Convert ServerInfo to DevServerInstance for API compatibility
fn convert_server_info_to_instance(info: ServerInfo) -> orkee_preview::types::DevServerInstance {
    use chrono::Utc;
    use orkee_preview::types::*;

    // Use real framework name or fallback
    let framework_name = info
        .framework_name
        .unwrap_or_else(|| "Development Server".to_string());
    let dev_command = info.actual_command.unwrap_or_else(|| "unknown".to_string());

    // Detect project type based on framework
    let project_type =
        if framework_name.contains("Static") || framework_name.contains("HTTP Server") {
            ProjectType::Static
        } else if framework_name.contains("Next") {
            ProjectType::Nextjs
        } else if framework_name.contains("React") {
            ProjectType::React
        } else if framework_name.contains("Vue") {
            ProjectType::Vue
        } else {
            ProjectType::Unknown
        };

    DevServerInstance {
        id: info.id,
        project_id: info.project_id,
        config: DevServerConfig {
            project_type,
            dev_command,
            port: info.port,
            package_manager: PackageManager::Npm,
            framework: Some(Framework {
                name: framework_name,
                version: None,
            }),
        },
        status: info.status,
        preview_url: info.preview_url,
        started_at: Some(Utc::now()),
        last_activity: Some(Utc::now()),
        error: None,
        pid: info.pid,
    }
}

/// Stop a development server
pub async fn stop_server(
    Path(project_id): Path<String>,
    State(state): State<PreviewState>,
) -> Json<ApiResponse<()>> {
    info!("Stopping server for project: {}", project_id);

    match state.preview_manager.stop_server(&project_id).await {
        Ok(_) => {
            info!("Successfully stopped server for project: {}", project_id);
            Json(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to stop server: {}", e);
            Json(ApiResponse::error(e.to_string()))
        }
    }
}

/// Stop all development servers
pub async fn stop_all_servers(
    State(state): State<PreviewState>,
) -> Json<ApiResponse<()>> {
    info!("Stopping all development servers");

    let servers = state.preview_manager.list_servers().await;
    let mut errors = Vec::new();
    let mut stopped_count = 0;

    for server in servers {
        match state.preview_manager.stop_server(&server.project_id).await {
            Ok(_) => {
                info!("Stopped server for project: {}", server.project_id);
                stopped_count += 1;
            }
            Err(e) => {
                error!("Failed to stop server for project {}: {}", server.project_id, e);
                errors.push(format!("{}: {}", server.project_id, e));
            }
        }
    }

    if errors.is_empty() {
        info!("Successfully stopped {} servers", stopped_count);
        Json(ApiResponse::success(()))
    } else {
        let error_msg = format!(
            "Stopped {} servers, but {} failed: {}",
            stopped_count,
            errors.len(),
            errors.join(", ")
        );
        error!("{}", error_msg);
        Json(ApiResponse::error(error_msg))
    }
}

/// Get server status
pub async fn get_server_status(
    Path(project_id): Path<String>,
    State(state): State<PreviewState>,
) -> Json<ApiResponse<ServerStatusResponse>> {
    let server_info = state.preview_manager.get_server_status(&project_id).await;
    let instance = server_info.map(convert_server_info_to_instance);
    Json(ApiResponse::success(ServerStatusResponse { instance }))
}

/// Query parameters for getting server logs
#[derive(Debug, Deserialize)]
pub struct LogsQuery {
    since: Option<DateTime<Utc>>,
    limit: Option<usize>,
}

/// Get server logs
pub async fn get_server_logs(
    Path(project_id): Path<String>,
    Query(query): Query<LogsQuery>,
    State(state): State<PreviewState>,
) -> Json<ApiResponse<ServerLogsResponse>> {
    let since = query.since;
    let limit = query.limit;

    let logs = state
        .preview_manager
        .get_server_logs(&project_id, since, limit)
        .await;
    Json(ApiResponse::success(ServerLogsResponse { logs }))
}

/// Clear server logs
pub async fn clear_server_logs(
    Path(project_id): Path<String>,
    State(state): State<PreviewState>,
) -> Json<ApiResponse<()>> {
    state.preview_manager.clear_server_logs(&project_id).await;
    Json(ApiResponse::success(()))
}

/// Update server activity timestamp - simplified implementation
pub async fn update_server_activity(
    Path(_project_id): Path<String>,
    State(_state): State<PreviewState>,
) -> Json<ApiResponse<()>> {
    // Simplified: activity tracking not implemented in simple manager
    Json(ApiResponse::success(()))
}

/// Get all active servers (for debugging/monitoring)
pub async fn list_active_servers(
    State(state): State<PreviewState>,
) -> Json<ApiResponse<ServersResponse>> {
    let servers = state.preview_manager.list_servers().await;

    // Fetch all projects in a single batch to get project names
    // This avoids N+1 query problem where each server would require a separate project lookup
    let project_names = match state.project_manager.list_projects().await {
        Ok(projects) => {
            // Build a map of project_id -> project_name for fast lookups
            projects
                .into_iter()
                .map(|p| (p.id.clone(), p.name.clone()))
                .collect::<std::collections::HashMap<String, String>>()
        }
        Err(e) => {
            error!("Failed to fetch projects for server list: {}", e);
            std::collections::HashMap::new()
        }
    };

    // Convert ServerInfo to a format suitable for the tray menu
    let server_list: Vec<ServerStatusInfo> = servers
        .into_iter()
        .map(|info| {
            // Get the project name from the batch-fetched map
            let project_name = project_names.get(&info.project_id).cloned();

            ServerStatusInfo {
                id: info.id.to_string(),
                project_id: info.project_id.clone(),
                project_name,
                port: info.port,
                url: info
                    .preview_url
                    .clone()
                    .unwrap_or_else(|| format!("http://localhost:{}", info.port)),
                status: format!("{:?}", info.status), // Convert enum to string
                framework_name: info.framework_name.clone(),
                started_at: None, // Could add timestamp tracking if needed
            }
        })
        .collect();

    Json(ApiResponse::success(ServersResponse {
        servers: server_list,
    }))
}

/// Health check endpoint for the preview service
pub async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success(
        "Preview service is healthy".to_string(),
    ))
}
