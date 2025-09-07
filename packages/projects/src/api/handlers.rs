use crate::manager::{
    create_project as manager_create_project,
    delete_project as manager_delete_project,
    get_all_projects, 
    get_project as manager_get_project,
    get_project_by_name as manager_get_project_by_name,
    get_project_by_path as manager_get_project_by_path,
    update_project as manager_update_project,
    ManagerError,
};
use crate::types::{ProjectCreateInput, ProjectUpdateInput};
use axum::{
    extract::{Path, Json},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

/// Standard API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Request body for getting project by path
#[derive(Deserialize)]
pub struct GetProjectByPathRequest {
    #[serde(rename = "projectRoot")]
    project_root: String,
}

/// Request body for checking taskmaster folder
#[derive(Deserialize)]
pub struct CheckTaskmasterRequest {
    #[serde(rename = "projectRoot")]
    project_root: String,
}

/// Response for taskmaster check
#[derive(Serialize)]
pub struct CheckTaskmasterResponse {
    #[serde(rename = "hasTaskmaster")]
    has_taskmaster: bool,
    #[serde(rename = "taskSource")]
    task_source: String,
}

/// Convert manager errors to HTTP responses
impl IntoResponse for ManagerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ManagerError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ManagerError::DuplicateName(_) | ManagerError::DuplicatePath(_) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            ManagerError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ManagerError::Storage(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Storage error".to_string()),
        };
        
        let response = ApiResponse::<()>::error(message);
        (status, ResponseJson(response)).into_response()
    }
}

/// List all projects
pub async fn list_projects() -> impl IntoResponse {
    info!("Listing all projects");
    
    match get_all_projects().await {
        Ok(projects) => {
            info!("Retrieved {} projects", projects.len());
            (StatusCode::OK, ResponseJson(ApiResponse::success(projects))).into_response()
        }
        Err(e) => {
            error!("Failed to list projects: {}", e);
            e.into_response()
        }
    }
}

/// Get a specific project by ID
pub async fn get_project(Path(id): Path<String>) -> impl IntoResponse {
    info!("Getting project with ID: {}", id);
    
    match manager_get_project(&id).await {
        Ok(Some(project)) => {
            info!("Found project: {}", project.name);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Ok(None) => {
            info!("Project not found: {}", id);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            ).into_response()
        }
        Err(e) => {
            error!("Failed to get project {}: {}", id, e);
            e.into_response()
        }
    }
}

/// Get a project by name
pub async fn get_project_by_name(Path(name): Path<String>) -> impl IntoResponse {
    info!("Getting project with name: {}", name);
    
    match manager_get_project_by_name(&name).await {
        Ok(Some(project)) => {
            info!("Found project by name: {}", project.name);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Ok(None) => {
            info!("Project not found by name: {}", name);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            ).into_response()
        }
        Err(e) => {
            error!("Failed to get project by name {}: {}", name, e);
            e.into_response()
        }
    }
}

/// Get a project by path
pub async fn get_project_by_path(Json(request): Json<GetProjectByPathRequest>) -> impl IntoResponse {
    info!("Getting project with path: {}", request.project_root);
    
    match manager_get_project_by_path(&request.project_root).await {
        Ok(Some(project)) => {
            info!("Found project by path: {}", project.name);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Ok(None) => {
            info!("Project not found by path: {}", request.project_root);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            ).into_response()
        }
        Err(e) => {
            error!("Failed to get project by path {}: {}", request.project_root, e);
            e.into_response()
        }
    }
}

/// Create a new project
pub async fn create_project(Json(input): Json<ProjectCreateInput>) -> impl IntoResponse {
    info!("Creating project: {}", input.name);
    
    match manager_create_project(input).await {
        Ok(project) => {
            info!("Created project: {} (ID: {})", project.name, project.id);
            (StatusCode::CREATED, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            e.into_response()
        }
    }
}

/// Update an existing project
pub async fn update_project(
    Path(id): Path<String>,
    Json(updates): Json<ProjectUpdateInput>,
) -> impl IntoResponse {
    info!("Updating project: {}", id);
    
    match manager_update_project(&id, updates).await {
        Ok(project) => {
            info!("Updated project: {} (ID: {})", project.name, project.id);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Err(e) => {
            error!("Failed to update project {}: {}", id, e);
            e.into_response()
        }
    }
}

/// Delete a project
pub async fn delete_project(Path(id): Path<String>) -> impl IntoResponse {
    info!("Deleting project: {}", id);
    
    match manager_delete_project(&id).await {
        Ok(true) => {
            info!("Deleted project: {}", id);
            (
                StatusCode::OK,
                ResponseJson(ApiResponse::success("Project deleted successfully")),
            ).into_response()
        }
        Ok(false) => {
            info!("Project not found for deletion: {}", id);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            ).into_response()
        }
        Err(e) => {
            error!("Failed to delete project {}: {}", id, e);
            e.into_response()
        }
    }
}

/// Check if project has .taskmaster folder
pub async fn check_taskmaster(Json(request): Json<CheckTaskmasterRequest>) -> impl IntoResponse {
    info!("Checking taskmaster folder for: {}", request.project_root);
    
    let taskmaster_path = std::path::Path::new(&request.project_root).join(".taskmaster");
    let has_taskmaster = taskmaster_path.exists() && taskmaster_path.is_dir();
    
    let response = CheckTaskmasterResponse {
        has_taskmaster,
        task_source: if has_taskmaster { "taskmaster".to_string() } else { "manual".to_string() },
    };
    
    info!("Taskmaster check result for {}: has_taskmaster={}, task_source={}", 
          request.project_root, response.has_taskmaster, response.task_source);
    
    (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProjectStatus;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;
    use crate::test_utils::test_helpers::with_temp_home;

    #[tokio::test]
    async fn test_create_and_get_project_api() {
        with_temp_home(|| async {
            let app = crate::api::create_projects_router();
            
            // Create a project
            let create_request = ProjectCreateInput {
                name: "API Test Project".to_string(),
                project_root: "/tmp/api-test".to_string(),
                setup_script: None,
                dev_script: None,
                cleanup_script: None,
                tags: Some(vec!["api".to_string(), "test".to_string()]),
                description: Some("Test project for API".to_string()),
                status: Some(ProjectStatus::Active),
                rank: None,
                priority: None,
                task_source: None,
                manual_tasks: None,
                mcp_servers: None,
            };
            
            let request = Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap();
                
            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);
            
            // List projects
            let request = Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap();
                
            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        }).await;
    }
}