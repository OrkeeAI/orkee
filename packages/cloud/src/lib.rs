//! Orkee Cloud - Client package for Orkee Cloud integration
//!
//! This package provides a client interface for connecting to Orkee Cloud API.
//! Features:
//! - OAuth authentication with browser-based flow
//! - Project synchronization and backup
//! - Token management and automatic refresh
//! - Simple, direct API integration

pub mod api;
pub mod auth;
pub mod client;
pub mod config;
pub mod encryption;
pub mod error;
pub mod types;

// Re-export main types
pub use api::{
    ApiError, AuthResponse, CloudProject, ConflictReport, ConflictResolution, ConflictStrategy,
    FieldConflict, FieldResolution, GitRepositoryInfo, ProjectDiff, Usage, User,
};
pub use auth::{AuthManager, CallbackServer, TokenInfo};
pub use client::HttpClient;
pub use error::{CloudError, CloudResult};
pub use types::*;

use api::{ListProjectsResponse, RestoreResponse};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};

/// Main cloud client for interacting with Orkee Cloud
pub struct CloudClient {
    http_client: HttpClient,
    auth_manager: AuthManager,
    api_base_url: String,
}

impl CloudClient {
    /// Create a new cloud client
    pub async fn new(api_base_url: String) -> CloudResult<Self> {
        let mut auth_manager = AuthManager::new()?;
        auth_manager.init().await?;
        let http_client = HttpClient::new(api_base_url.clone(), auth_manager.clone())?;

        Ok(Self {
            http_client,
            auth_manager,
            api_base_url,
        })
    }

    /// Check if user is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.auth_manager.is_authenticated()
    }

    /// Get current user information
    pub fn user_info(&self) -> Option<(String, String, String)> {
        self.auth_manager
            .user_info()
            .map(|(id, email, name)| (id.to_string(), email.to_string(), name.to_string()))
    }

    /// Perform OAuth login flow
    pub async fn login(&mut self) -> CloudResult<TokenInfo> {
        println!("🚀 Starting Orkee Cloud authentication...");

        // Start OAuth flow
        let _state = self
            .auth_manager
            .start_oauth_flow(&self.api_base_url)
            .await?;

        // Start callback server
        let callback_server = CallbackServer::new();
        let auth_code = callback_server.wait_for_callback().await?;

        println!("✅ Authentication code received!");

        // Exchange code for token
        let http_client = reqwest::Client::new();
        let token_info = self
            .auth_manager
            .exchange_code(auth_code, &http_client, &self.api_base_url)
            .await?;

        println!("🎉 Successfully authenticated as {}", token_info.user_name);
        Ok(token_info)
    }

    /// Logout and clear stored token
    pub async fn logout(&mut self) -> CloudResult<()> {
        self.auth_manager.logout().await?;
        println!("👋 Logged out from Orkee Cloud");
        Ok(())
    }

    /// List all projects in the cloud
    pub async fn list_projects(&self) -> CloudResult<Vec<CloudProject>> {
        let response: ListProjectsResponse = self.http_client.get("/api/projects").await?;
        Ok(response.projects)
    }

    /// Sync a project to the cloud
    pub async fn sync_project(
        &self,
        cloud_project: CloudProject,
        _project_data: serde_json::Value,
    ) -> CloudResult<String> {
        // Use the sync endpoint directly with full project data
        #[derive(serde::Serialize)]
        struct SyncRequest {
            id: Option<String>,
            name: String,
            description: Option<String>,
            project_root: Option<String>,
            setup_script: Option<String>,
            dev_script: Option<String>,
            cleanup_script: Option<String>,
            tags: Vec<String>,
            status: String,
            priority: String,
            rank: Option<u32>,
            task_source: Option<String>,
            mcp_servers: std::collections::HashMap<String, bool>,
            git_repository: Option<api::GitRepositoryInfo>,
            manual_tasks: Option<Vec<serde_json::Value>>,
        }

        let request = SyncRequest {
            id: Some(cloud_project.id.clone()),
            name: cloud_project.name,
            description: cloud_project.description,
            project_root: Some(cloud_project.path),
            setup_script: cloud_project.setup_script,
            dev_script: cloud_project.dev_script,
            cleanup_script: cloud_project.cleanup_script,
            tags: cloud_project.tags,
            status: cloud_project.status,
            priority: cloud_project.priority,
            rank: cloud_project.rank,
            task_source: cloud_project.task_source,
            mcp_servers: cloud_project.mcp_servers,
            git_repository: cloud_project.git_repository,
            manual_tasks: cloud_project.manual_tasks,
        };

        #[derive(serde::Deserialize)]
        struct SyncResponse {
            project_id: String,
            #[allow(dead_code)]
            status: String,
            #[allow(dead_code)]
            synced_at: chrono::DateTime<chrono::Utc>,
            #[allow(dead_code)]
            tasks_synced: usize,
        }

        let response: SyncResponse = self
            .http_client
            .post("/api/projects/sync", &request)
            .await?;

        Ok(response.project_id)
    }

    /// Check for sync conflicts
    pub async fn check_conflicts(&self, project_id: &str) -> CloudResult<ConflictReport> {
        let response: ConflictReport = self
            .http_client
            .get(&format!("/api/projects/{}/conflicts", project_id))
            .await?;
        Ok(response)
    }

    /// Resolve sync conflicts
    pub async fn resolve_conflicts(
        &self,
        project_id: &str,
        resolution: ConflictResolution,
    ) -> CloudResult<()> {
        self.http_client
            .post::<_, ()>(
                &format!("/api/projects/{}/resolve", project_id),
                &resolution,
            )
            .await?;
        Ok(())
    }

    /// Incremental sync for changes only
    pub async fn sync_incremental(&self, project_id: &str, diff: ProjectDiff) -> CloudResult<()> {
        self.http_client
            .patch::<_, ()>(&format!("/api/projects/{}/delta", project_id), &diff)
            .await?;
        Ok(())
    }

    /// Get full project with all fields
    pub async fn get_full_project(&self, project_id: &str) -> CloudResult<CloudProject> {
        let response: CloudProject = self
            .http_client
            .get(&format!("/api/projects/{}/full", project_id))
            .await?;
        Ok(response)
    }

    /// Restore a project from the cloud
    pub async fn restore_project(&self, project_id: &str) -> CloudResult<serde_json::Value> {
        let path = format!("/api/projects/{}", project_id);
        let response: RestoreResponse = self.http_client.get(&path).await?;

        // Decode project data
        let project_bytes = BASE64
            .decode(&response.snapshot_data)
            .map_err(|e| CloudError::api(format!("Invalid snapshot data: {}", e)))?;

        let project_json = String::from_utf8(project_bytes)
            .map_err(|e| CloudError::api(format!("Invalid UTF-8 in snapshot: {}", e)))?;

        let project_data: serde_json::Value = serde_json::from_str(&project_json)?;

        println!(
            "📥 Project '{}' restored successfully",
            response.project.name
        );
        Ok(project_data)
    }

    /// Get usage statistics
    pub async fn get_usage(&self) -> CloudResult<Usage> {
        self.http_client.get("/api/usage").await
    }

    /// Get cloud sync status
    pub async fn get_status(&self) -> CloudResult<CloudStatus> {
        if !self.is_authenticated() {
            return Ok(CloudStatus {
                authenticated: false,
                user_email: None,
                user_name: None,
                projects_count: 0,
                last_sync: None,
                subscription_tier: None,
            });
        }

        let usage = self.get_usage().await?;
        let (_user_id, user_email, user_name) = self.user_info().unwrap();

        Ok(CloudStatus {
            authenticated: true,
            user_email: Some(user_email),
            user_name: Some(user_name),
            projects_count: usage.projects_count,
            last_sync: None, // TODO: Track last sync time
            subscription_tier: Some(usage.subscription_tier),
        })
    }
}

/// Cloud sync status information
#[derive(Debug)]
pub struct CloudStatus {
    pub authenticated: bool,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    pub projects_count: usize,
    pub last_sync: Option<chrono::DateTime<chrono::Utc>>,
    pub subscription_tier: Option<String>,
}

/// Legacy compatibility functions (from old CloudConfigBuilder)
pub struct CloudConfigBuilder {
    api_url: Option<String>,
    token: Option<String>,
}

impl Default for CloudConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CloudConfigBuilder {
    pub fn new() -> Self {
        Self {
            api_url: None,
            token: None,
        }
    }

    pub fn api_url(mut self, url: String) -> Self {
        self.api_url = Some(url);
        self
    }

    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }

    pub async fn build(self) -> CloudResult<CloudClient> {
        let api_url = self
            .api_url
            .unwrap_or_else(|| "https://api.orkee.ai".to_string());
        let client = CloudClient::new(api_url).await?;

        // If token is provided, try to use it (for environment variable usage)
        if let Some(token) = self.token {
            if !token.is_empty() {
                // For now, just validate that a token is provided
                // Full token validation will happen on first API call
                tracing::debug!("Token provided via environment variable");
            }
        }

        Ok(client)
    }
}

/// Legacy compatibility - create and initialize cloud client
pub async fn init() -> CloudResult<CloudClient> {
    // Try to get config from environment
    let api_url =
        std::env::var("ORKEE_CLOUD_API_URL").unwrap_or_else(|_| "https://api.orkee.ai".to_string());

    let token = std::env::var("ORKEE_CLOUD_TOKEN").unwrap_or_default();

    CloudConfigBuilder::new()
        .api_url(api_url)
        .token(token)
        .build()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cloud_client_creation() {
        let result = CloudClient::new("https://api.test.com".to_string()).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_config_builder() {
        let result = CloudConfigBuilder::new()
            .api_url("https://api.test.com".to_string())
            .token("test_token".to_string())
            .build()
            .await;
        assert!(result.is_ok());
    }
}
