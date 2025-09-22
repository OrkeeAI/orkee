//! Cloud sync API endpoints
//!
//! This module provides API endpoints that bridge to the CloudClient for Orkee Cloud functionality.
//! These endpoints are used by the dashboard to interact with Orkee Cloud.

use axum::{
    extract::Path,
    http::StatusCode,
    response::Json,
    Extension,
};
use std::sync::Arc;
use serde::{Deserialize, Serialize};

#[cfg(feature = "cloud")]
use orkee_cloud::{CloudClient, CloudError, CloudProject};

// Mock CloudProject for when cloud feature is disabled
#[cfg(not(feature = "cloud"))]
#[derive(Serialize)]
pub struct CloudProject {
    pub id: String,
    pub name: String,
    pub path: String,
    pub description: Option<String>,
}

// API Response format matching the existing patterns
#[derive(Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Cloud authentication status
#[derive(Serialize)]
pub struct CloudAuthStatus {
    pub authenticated: bool,
    pub user_id: Option<String>,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    pub subscription_tier: Option<String>,
}

// Global sync status for all projects
#[derive(Serialize)]
pub struct GlobalSyncStatus {
    pub total_projects: usize,
    pub synced_projects: usize,
    pub pending_projects: usize,
    pub syncing_projects: usize,
    pub conflict_projects: usize,
    pub last_sync: Option<String>,
    pub is_syncing: bool,
    pub current_sync_progress: f32, // 0.0 to 1.0
}

// Individual project sync status
#[derive(Serialize)]
pub struct ProjectSyncStatus {
    pub project_id: String,
    pub cloud_project_id: Option<String>,
    pub status: String, // "synced", "pending", "syncing", "conflict", "error"
    pub last_sync: Option<String>,
    pub has_conflicts: bool,
    pub sync_progress: Option<f32>,
    pub error_message: Option<String>,
}

// Sync result for operations
#[derive(Serialize)]
pub struct SyncResult {
    pub project_id: String,
    pub success: bool,
    pub message: String,
    pub conflicts_detected: bool,
}

// OAuth initialization response
#[derive(Serialize)]
pub struct OAuthInitResponse {
    pub auth_url: String,
    pub state: String,
    pub expires_at: String,
}

// Request bodies
#[derive(Deserialize)]
pub struct OAuthCallbackRequest {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize)]
pub struct SyncAllRequest {
    pub force: Option<bool>,
    pub exclude_projects: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct SyncProjectRequest {
    pub force: Option<bool>,
}

// Cloud state shared across handlers
#[derive(Clone)]
pub struct CloudState {
    #[cfg(feature = "cloud")]
    pub cloud_client: Arc<tokio::sync::Mutex<Option<CloudClient>>>,
}

impl Default for CloudState {
    fn default() -> Self {
        Self::new()
    }
}

impl CloudState {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "cloud")]
            cloud_client: Arc::new(tokio::sync::Mutex::new(None)),
        }
    }

    #[cfg(feature = "cloud")]
    async fn get_or_create_client(&self) -> Result<CloudClient, CloudError> {
        // For now, always create a new client since CloudClient doesn't support cloning
        // TODO: Implement proper client caching when CloudClient supports it
        let api_url = std::env::var("ORKEE_CLOUD_API_URL")
            .unwrap_or_else(|_| "https://api.orkee.ai".to_string());
        CloudClient::new(api_url).await
    }
}

/// Initialize OAuth flow and return auth URL
pub async fn init_oauth_flow(
    Extension(state): Extension<CloudState>,
) -> Result<Json<ApiResponse<OAuthInitResponse>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        return Ok(Json(ApiResponse::error(
            "Cloud feature not enabled. Build with --features cloud".to_string(),
        )));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(_client) => {
                // Get the API URL from environment or use default
                let api_url = std::env::var("ORKEE_CLOUD_API_URL")
                    .unwrap_or_else(|_| "https://api.orkee.ai".to_string());
                
                // For now, we'll return a placeholder response
                // The actual OAuth flow will be implemented with the full CloudClient integration
                let response = OAuthInitResponse {
                    auth_url: format!("{}/auth/oauth/authorize?client_id=orkee-cli", api_url),
                    state: "placeholder_state".to_string(),
                    expires_at: chrono::Utc::now().to_rfc3339(),
                };
                Ok(Json(ApiResponse::success(response)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to initialize cloud client: {}", e)))),
        }
    }
}

/// Handle OAuth callback
pub async fn handle_oauth_callback(
    Extension(state): Extension<CloudState>,
    Json(_request): Json<OAuthCallbackRequest>,
) -> Result<Json<ApiResponse<CloudAuthStatus>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        return Ok(Json(ApiResponse::error(
            "Cloud feature not enabled. Build with --features cloud".to_string(),
        )));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(mut client) => {
                match client.login().await {
                    Ok(token_info) => {
                        let auth_status = CloudAuthStatus {
                            authenticated: true,
                            user_id: Some(token_info.user_id),
                            user_email: Some(token_info.user_email),
                            user_name: Some(token_info.user_name),
                            subscription_tier: Some("free".to_string()), // TODO: Get from token
                        };
                        Ok(Json(ApiResponse::success(auth_status)))
                    }
                    Err(e) => Ok(Json(ApiResponse::error(format!("OAuth login failed: {}", e)))),
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to initialize cloud client: {}", e)))),
        }
    }
}

/// Get current authentication status
pub async fn get_auth_status(
    Extension(state): Extension<CloudState>,
) -> Result<Json<ApiResponse<CloudAuthStatus>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        let auth_status = CloudAuthStatus {
            authenticated: false,
            user_id: None,
            user_email: None,
            user_name: None,
            subscription_tier: None,
        };
        return Ok(Json(ApiResponse::success(auth_status)));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(client) => {
                let auth_status = if client.is_authenticated() {
                    let user_info = client.user_info();
                    CloudAuthStatus {
                        authenticated: true,
                        user_id: user_info.as_ref().map(|(id, _, _)| id.clone()),
                        user_email: user_info.as_ref().map(|(_, email, _)| email.clone()),
                        user_name: user_info.as_ref().map(|(_, _, name)| name.clone()),
                        subscription_tier: Some("free".to_string()), // TODO: Get actual tier
                    }
                } else {
                    CloudAuthStatus {
                        authenticated: false,
                        user_id: None,
                        user_email: None,
                        user_name: None,
                        subscription_tier: None,
                    }
                };
                Ok(Json(ApiResponse::success(auth_status)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to check auth status: {}", e)))),
        }
    }
}

/// Logout and clear authentication
pub async fn logout(
    Extension(state): Extension<CloudState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        return Ok(Json(ApiResponse::error(
            "Cloud feature not enabled. Build with --features cloud".to_string(),
        )));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(mut client) => {
                match client.logout().await {
                    Ok(_) => {
                        // Clear the cached client
                        *state.cloud_client.lock().await = None;
                        Ok(Json(ApiResponse::success("Successfully logged out".to_string())))
                    }
                    Err(e) => Ok(Json(ApiResponse::error(format!("Logout failed: {}", e)))),
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to initialize cloud client: {}", e)))),
        }
    }
}

/// Get global sync status for all projects
pub async fn get_global_sync_status(
    Extension(state): Extension<CloudState>,
) -> Result<Json<ApiResponse<GlobalSyncStatus>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        let status = GlobalSyncStatus {
            total_projects: 0,
            synced_projects: 0,
            pending_projects: 0,
            syncing_projects: 0,
            conflict_projects: 0,
            last_sync: None,
            is_syncing: false,
            current_sync_progress: 0.0,
        };
        return Ok(Json(ApiResponse::success(status)));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(client) => {
                if !client.is_authenticated() {
                    let status = GlobalSyncStatus {
                        total_projects: 0,
                        synced_projects: 0,
                        pending_projects: 0,
                        syncing_projects: 0,
                        conflict_projects: 0,
                        last_sync: None,
                        is_syncing: false,
                        current_sync_progress: 0.0,
                    };
                    return Ok(Json(ApiResponse::success(status)));
                }

                // TODO: Implement actual sync status checking by comparing local and cloud projects
                // For now, return mock data
                let status = GlobalSyncStatus {
                    total_projects: 5,
                    synced_projects: 3,
                    pending_projects: 1,
                    syncing_projects: 1,
                    conflict_projects: 0,
                    last_sync: Some(chrono::Utc::now().to_rfc3339()),
                    is_syncing: true,
                    current_sync_progress: 0.6,
                };
                Ok(Json(ApiResponse::success(status)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get sync status: {}", e)))),
        }
    }
}

/// List cloud projects
pub async fn list_cloud_projects(
    Extension(state): Extension<CloudState>,
) -> Result<Json<ApiResponse<Vec<CloudProject>>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        return Ok(Json(ApiResponse::success(vec![])));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(client) => {
                if !client.is_authenticated() {
                    return Ok(Json(ApiResponse::error("Not authenticated".to_string())));
                }

                match client.list_projects().await {
                    Ok(projects) => Ok(Json(ApiResponse::success(projects))),
                    Err(e) => Ok(Json(ApiResponse::error(format!("Failed to list cloud projects: {}", e)))),
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to initialize cloud client: {}", e)))),
        }
    }
}

/// Sync all projects to cloud
pub async fn sync_all_projects(
    Extension(_state): Extension<CloudState>,
    Json(request): Json<SyncAllRequest>,
) -> Result<Json<ApiResponse<Vec<SyncResult>>>, StatusCode> {
    tracing::info!("[OSS API] sync_all_projects called with request: {:?}", request);
    
    // Get all local projects using the projects API
    let manager = orkee_projects::ProjectsManager::new()
        .await
        .map_err(|e| {
            tracing::error!("Failed to create project manager: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    let projects = manager.list_projects()
        .await
        .map_err(|e| {
            tracing::error!("Failed to list projects: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    
    tracing::info!("[OSS API] Found {} local projects to sync", projects.len());
    
    let mut results = Vec::new();
    let mut synced_count = 0;
    let mut failed_count = 0;
    
    // Get the cloud API URL - default to local cloud API server on port 8080
    let cloud_api_url = std::env::var("ORKEE_CLOUD_API_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".to_string());
    
    // Get the authentication token
    let auth_token = {
        // TODO: Implement proper authentication when cloud features are fully implemented
        tracing::warn!("[OSS API] Using development JWT token for testing");
        // This is a valid JWT token generated for user test@orkee.ai (ID: 35ea4b35-376e-4e84-9ed3-5399bc84d20f)
        // Generated using the JWT_SECRET from cloud API with 24-hour expiration
        "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIzNWVhNGIzNS0zNzZlLTRlODQtOWVkMy01Mzk5YmM4NGQyMGYiLCJpYXQiOjE3NTc5MDY2MjMsImV4cCI6MTc1Nzk5MzAyMywibmJmIjoxNzU3OTA2NjIzLCJpc3MiOiJvcmtlZS1jbG91ZCIsImF1ZCI6WyJvcmtlZS1jbG91ZC1hcGkiXSwianRpIjoiOGQzOWM2ZjItNDIyMS00MGY0LWJlMmItNzhmMmU2YjFmMTBiIiwidHlwIjoiYWNjZXNzIiwiZW1haWwiOiJ0ZXN0QG9ya2VlLmFpIiwicm9sZXMiOlsidXNlciJdLCJtZXRhZGF0YSI6eyJhdmF0YXJfdXJsIjoiaHR0cHM6Ly9hdmF0YXJzLmdpdGh1YnVzZXJjb250ZW50LmNvbS91LzE0MjM3Nzc_dj00IiwibmFtZSI6IkpvZSBEYW56aWdlciJ9fQ.tgjyHgoD6aay2ZU5yFihHWOJ0l0aslgHnz92wSqQzAs".to_string()
    };
    
    // Iterate through projects and sync them
    tracing::info!("[OSS API] Starting to sync projects");
    for (idx, project) in projects.into_iter().enumerate() {
        tracing::info!("[OSS API] Syncing project {}: {}", idx + 1, project.name);
        // Skip if in exclude list
        if let Some(ref exclude) = request.exclude_projects {
            if exclude.contains(&project.id) {
                continue;
            }
        }
        
        // Create project payload for cloud API (matching CreateProjectRequest structure)
        let project_payload = serde_json::json!({
            "name": project.name,
            "description": project.description,
            "project_root": project.project_root,
            "setup_script": project.setup_script,
            "dev_script": project.dev_script,
            "cleanup_script": project.cleanup_script,
            "tags": project.tags.clone().unwrap_or_default(),
            "status": Some("active"),
            "priority": Some("medium"),
            "rank": project.rank,
            "task_source": project.task_source.as_ref().map(|ts| ts.to_string()),
            "mcp_servers": project.mcp_servers.clone().unwrap_or_default(),
            "git_repository": project.git_repository.clone(),
            "metadata": serde_json::json!({}),
        });
        
        // Send to cloud API with authentication
        tracing::info!("[OSS API] Sending project '{}' to cloud API at {}/api/projects", project.name, cloud_api_url);
        tracing::debug!("[OSS API] Project payload: {:?}", project_payload);
        
        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/api/projects", cloud_api_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&project_payload)
            .send()
            .await;
        
        match response {
            Ok(res) if res.status().is_success() => {
                synced_count += 1;
                tracing::info!("[OSS API] Successfully synced project '{}'", project.name);
                results.push(SyncResult {
                    project_id: project.id.clone(),
                    success: true,
                    message: format!("Successfully synced '{}'", project.name),
                    conflicts_detected: false,
                });
            }
            Ok(res) => {
                failed_count += 1;
                let status = res.status();
                let error_msg = res.text().await.unwrap_or_else(|_| "Unknown error".to_string());
                tracing::error!("[OSS API] Failed to sync project '{}' - Status: {}, Error: {}", project.name, status, error_msg);
                results.push(SyncResult {
                    project_id: project.id.clone(),
                    success: false,
                    message: format!("Failed to sync '{}': {}", project.name, error_msg),
                    conflicts_detected: false,
                });
            }
            Err(e) => {
                failed_count += 1;
                tracing::error!("[OSS API] Network error syncing project '{}': {}", project.name, e);
                results.push(SyncResult {
                    project_id: project.id.clone(),
                    success: false,
                    message: format!("Failed to sync '{}': {}", project.name, e),
                    conflicts_detected: false,
                });
            }
        }
    }
    
    // Log summary
    if failed_count > 0 {
        tracing::warn!("[OSS API] Sync completed with {} failures out of {} projects", failed_count, results.len());
    } else if synced_count > 0 {
        tracing::info!("[OSS API] Successfully synced {} projects", synced_count);
    } else {
        tracing::info!("[OSS API] No projects were synced");
    }
    
    tracing::info!("[OSS API] Returning {} sync results", results.len());
    Ok(Json(ApiResponse::success(results)))
}

/// Sync specific project
pub async fn sync_project(
    Extension(state): Extension<CloudState>,
    Path(project_id): Path<String>,
    Json(_request): Json<SyncProjectRequest>,
) -> Result<Json<ApiResponse<SyncResult>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        return Ok(Json(ApiResponse::error(
            "Cloud feature not enabled. Build with --features cloud".to_string(),
        )));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(client) => {
                if !client.is_authenticated() {
                    return Ok(Json(ApiResponse::error("Not authenticated".to_string())));
                }

                // TODO: Implement actual project sync logic using CloudClient
                let result = SyncResult {
                    project_id: project_id.clone(),
                    success: true,
                    message: "Successfully synced project".to_string(),
                    conflicts_detected: false,
                };
                Ok(Json(ApiResponse::success(result)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to initialize cloud client: {}", e)))),
        }
    }
}

/// Get project sync status
pub async fn get_project_sync_status(
    Extension(state): Extension<CloudState>,
    Path(project_id): Path<String>,
) -> Result<Json<ApiResponse<ProjectSyncStatus>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        let status = ProjectSyncStatus {
            project_id: project_id.clone(),
            cloud_project_id: None,
            status: "not_available".to_string(),
            last_sync: None,
            has_conflicts: false,
            sync_progress: None,
            error_message: Some("Cloud feature not enabled".to_string()),
        };
        return Ok(Json(ApiResponse::success(status)));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(client) => {
                if !client.is_authenticated() {
                    let status = ProjectSyncStatus {
                        project_id: project_id.clone(),
                        cloud_project_id: None,
                        status: "not_authenticated".to_string(),
                        last_sync: None,
                        has_conflicts: false,
                        sync_progress: None,
                        error_message: Some("Not authenticated with cloud".to_string()),
                    };
                    return Ok(Json(ApiResponse::success(status)));
                }

                // TODO: Check actual project sync status
                let status = ProjectSyncStatus {
                    project_id: project_id.clone(),
                    cloud_project_id: Some(format!("cloud-{}", project_id)),
                    status: "synced".to_string(),
                    last_sync: Some(chrono::Utc::now().to_rfc3339()),
                    has_conflicts: false,
                    sync_progress: None,
                    error_message: None,
                };
                Ok(Json(ApiResponse::success(status)))
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get project sync status: {}", e)))),
        }
    }
}

/// Get usage statistics
pub async fn get_usage_stats(
    Extension(state): Extension<CloudState>,
) -> Result<Json<ApiResponse<String>>, StatusCode> {
    #[cfg(not(feature = "cloud"))]
    {
        return Ok(Json(ApiResponse::error(
            "Cloud feature not enabled. Build with --features cloud".to_string(),
        )));
    }

    #[cfg(feature = "cloud")]
    {
        match state.get_or_create_client().await {
            Ok(client) => {
                if !client.is_authenticated() {
                    return Ok(Json(ApiResponse::error("Not authenticated".to_string())));
                }

                match client.get_usage().await {
                    Ok(_usage) => Ok(Json(ApiResponse::success("Usage data retrieved".to_string()))),
                    Err(e) => Ok(Json(ApiResponse::error(format!("Failed to get usage stats: {}", e)))),
                }
            }
            Err(e) => Ok(Json(ApiResponse::error(format!("Failed to initialize cloud client: {}", e)))),
        }
    }
}