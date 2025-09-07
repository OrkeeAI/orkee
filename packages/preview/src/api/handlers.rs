use crate::{
    manager::DevServerManager,
    types::{
        ApiResponse, ServerLogsResponse, ServerStatusResponse,
        StartServerRequest, StartServerResponse,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::{path::PathBuf, sync::Arc};
use tracing::{error, info};

/// Shared application state containing the dev server manager
pub type AppState = Arc<DevServerManager>;

/// Start a development server for a project
pub async fn start_server(
    Path(project_id): Path<String>,
    State(manager): State<AppState>,
    Json(request): Json<StartServerRequest>,
) -> Result<Json<ApiResponse<StartServerResponse>>, StatusCode> {
    info!("Starting server for project: {}", project_id);

    // In a real implementation, we'd get the project root from the projects service
    // For now, we'll need to pass it through the request or look it up
    let project_root = get_project_root(&project_id).await.map_err(|_| {
        error!("Failed to get project root for: {}", project_id);
        StatusCode::NOT_FOUND
    })?;

    match manager
        .start_dev_server(project_id, project_root, request.custom_port)
        .await
    {
        Ok(instance) => {
            info!("Successfully started server: {}", instance.id);
            Ok(Json(ApiResponse::success(StartServerResponse {
                instance,
            })))
        }
        Err(e) => {
            error!("Failed to start server: {}", e);
            Ok(Json(ApiResponse::error(e)))
        }
    }
}

/// Stop a development server
pub async fn stop_server(
    Path(project_id): Path<String>,
    State(manager): State<AppState>,
) -> Json<ApiResponse<()>> {
    info!("Stopping server for project: {}", project_id);

    match manager.stop_dev_server(&project_id).await {
        Ok(_) => {
            info!("Successfully stopped server for project: {}", project_id);
            Json(ApiResponse::success(()))
        }
        Err(e) => {
            error!("Failed to stop server: {}", e);
            Json(ApiResponse::error(e))
        }
    }
}

/// Get server status
pub async fn get_server_status(
    Path(project_id): Path<String>,
    State(manager): State<AppState>,
) -> Json<ApiResponse<ServerStatusResponse>> {
    let instance = manager.get_server_status(&project_id).await;

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
    State(manager): State<AppState>,
) -> Json<ApiResponse<ServerLogsResponse>> {
    let logs = manager
        .get_server_logs(&project_id, query.since, query.limit)
        .await;

    Json(ApiResponse::success(ServerLogsResponse { logs }))
}

/// Clear server logs
pub async fn clear_server_logs(
    Path(project_id): Path<String>,
    State(manager): State<AppState>,
) -> Json<ApiResponse<()>> {
    manager.clear_logs(&project_id).await;
    info!("Cleared logs for project: {}", project_id);
    Json(ApiResponse::success(()))
}

/// Update server activity timestamp
pub async fn update_server_activity(
    Path(project_id): Path<String>,
    State(manager): State<AppState>,
) -> Json<ApiResponse<()>> {
    manager.update_activity(&project_id).await;
    Json(ApiResponse::success(()))
}

/// Get all active servers (for debugging/monitoring)
pub async fn list_active_servers(
    State(manager): State<AppState>,
) -> Json<ApiResponse<Vec<String>>> {
    // This is a simple implementation - in practice, you might want to return full server details
    let active_servers = manager.active_servers.read().await;
    let project_ids: Vec<String> = active_servers.keys().cloned().collect();

    Json(ApiResponse::success(project_ids))
}

/// Health check endpoint for the preview service
pub async fn health_check() -> Json<ApiResponse<String>> {
    Json(ApiResponse::success("Preview service is healthy".to_string()))
}

/// Helper function to get project root path
/// In a real implementation, this would integrate with the projects service
async fn get_project_root(project_id: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // This is a placeholder implementation
    // In the real application, you would:
    // 1. Query the projects service/database to get project details
    // 2. Extract the project_root path from the project data
    // 3. Validate that the path exists and is accessible

    // For now, we'll create a mock implementation
    // You should replace this with actual project lookup logic
    
    // Mock implementation - replace with real project service integration
    let projects_service = get_projects_service().await?;
    let project = projects_service.get_project(project_id).await?;
    Ok(PathBuf::from(project.project_root))
}

/// Mock projects service - replace with actual implementation
struct Project {
    project_root: String,
}

async fn get_projects_service() -> Result<ProjectsService, Box<dyn std::error::Error>> {
    Ok(ProjectsService)
}

struct ProjectsService;

impl ProjectsService {
    async fn get_project(&self, project_id: &str) -> Result<Project, Box<dyn std::error::Error>> {
        // This is where you would integrate with the actual projects service
        // For example, you might make an HTTP request to the projects API
        // or query a shared database
        
        // Mock implementation - replace with real logic
        match project_id {
            _ => {
                // In practice, you'd look up the project in your database/service
                // For now, return an error to indicate the integration is needed
                Err("Project service integration not implemented".into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::DevServerManager;
    use axum::{
        body::Body,
        http::{Method, Request, StatusCode},
        routing::{get, post},
        Router,
    };
    use tower::ServiceExt;

    fn create_test_app() -> Router {
        let manager = Arc::new(DevServerManager::new().unwrap());
        
        Router::new()
            .route("/health", get(health_check))
            .route("/servers", get(list_active_servers))
            .route("/servers/:project_id/status", get(get_server_status))
            .route("/servers/:project_id/start", post(start_server))
            .route("/servers/:project_id/stop", post(stop_server))
            .route("/servers/:project_id/logs", get(get_server_logs))
            .route("/servers/:project_id/logs/clear", post(clear_server_logs))
            .route("/servers/:project_id/activity", post(update_server_activity))
            .with_state(manager)
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = create_test_app();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/health")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_active_servers() {
        let app = create_test_app();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/servers")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_get_server_status_not_found() {
        let app = create_test_app();

        let request = Request::builder()
            .method(Method::GET)
            .uri("/servers/nonexistent/status")
            .body(Body::empty())
            .unwrap();

        let response = app.oneshot(request).await.unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        // The endpoint should return OK with instance: None for non-existent servers
    }
}