use chrono::{DateTime, Utc};
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

use crate::subscription::{CloudSubscription, CloudTier};

/// Cloud operation errors
#[derive(Error, Debug)]
pub enum CloudError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Configuration(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Limit exceeded: {0}")]
    LimitExceeded(String),
    #[error("Sync conflict: {0}")]
    SyncConflict(String),
    #[error("Subscription required: {0}")]
    SubscriptionRequired(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("Encryption error: {0}")]
    Encryption(String),
}

pub type CloudResult<T> = Result<T, CloudError>;

/// Cloud project representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProject {
    pub id: String,
    pub name: String,
    pub project_root: String,
    pub description: Option<String>,
    pub status: String,
    pub priority: String,
    pub local_version: i32,
    pub cloud_version: i32,
    pub sync_status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Main cloud client for Orkee Cloud operations
#[derive(Clone)]
pub struct CloudClient {
    http_client: Client,
    project_url: String,
    anon_key: String,
    access_token: Option<String>,
    subscription: CloudSubscription,
}

impl CloudClient {
    /// Create a new cloud client (internal - Supabase implementation)
    pub(crate) async fn new(
        project_url: String,
        anon_key: String,
    ) -> CloudResult<Self> {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| CloudError::Network(e.to_string()))?;

        // Start with free tier by default
        let subscription = CloudSubscription {
            tier: CloudTier::Free,
            project_limit: 2,
            storage_limit_mb: 100,
            auto_sync_enabled: false,
            realtime_enabled: false,
            collaboration_enabled: false,
        };

        Ok(Self {
            http_client,
            project_url,
            anon_key,
            access_token: None,
            subscription,
        })
    }

    /// Set the access token after authentication
    pub fn set_access_token(&mut self, token: String) {
        self.access_token = Some(token);
    }

    /// Update subscription information from JWT claims
    pub fn update_subscription(&mut self, subscription: CloudSubscription) {
        self.subscription = subscription;
    }

    /// Get current subscription
    pub fn subscription(&self) -> &CloudSubscription {
        &self.subscription
    }

    /// Get authorization header value
    fn get_auth_header(&self) -> String {
        if let Some(token) = &self.access_token {
            format!("Bearer {}", token)
        } else {
            format!("Bearer {}", self.anon_key)
        }
    }

    /// Check if cloud is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.access_token.is_some()
    }

    /// Test connection to cloud
    pub async fn test_connection(&self) -> CloudResult<bool> {
        let url = format!("{}/rest/v1/", self.project_url);
        
        let response = self.http_client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", self.get_auth_header())
            .send()
            .await
            .map_err(|e| CloudError::Network(e.to_string()))?;

        Ok(response.status().is_success())
    }

    /// Sync a project to the cloud
    pub async fn sync_project(&self, project: &CloudProject) -> CloudResult<CloudProject> {
        // Check project limit for free tier
        if self.subscription.tier == CloudTier::Free {
            let count = self.count_projects().await?;
            if count >= self.subscription.project_limit as usize {
                return Err(CloudError::LimitExceeded(
                    format!("Free tier limited to {} projects. Upgrade to sync more.", self.subscription.project_limit)
                ));
            }
        }

        let url = format!("{}/rest/v1/projects", self.project_url);
        
        let response = self.http_client
            .post(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .json(project)
            .send()
            .await
            .map_err(|e| CloudError::Network(e.to_string()))?;

        match response.status() {
            StatusCode::CREATED | StatusCode::OK => {
                response.json::<CloudProject>().await
                    .map_err(|e| CloudError::InvalidResponse(e.to_string()))
            }
            StatusCode::CONFLICT => {
                Err(CloudError::SyncConflict("Project already exists in cloud".to_string()))
            }
            StatusCode::UNAUTHORIZED => {
                Err(CloudError::Authentication("Invalid or expired token".to_string()))
            }
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| status.to_string());
                Err(CloudError::Http(error_text))
            }
        }
    }

    /// Get a project from the cloud
    pub async fn get_project(&self, project_id: &str) -> CloudResult<CloudProject> {
        let url = format!("{}/rest/v1/projects", self.project_url);
        
        let response = self.http_client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", self.get_auth_header())
            .query(&[("id", format!("eq.{}", project_id))])
            .send()
            .await
            .map_err(|e| CloudError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CloudError::Http(response.status().to_string()));
        }

        let projects: Vec<CloudProject> = response.json().await
            .map_err(|e| CloudError::InvalidResponse(e.to_string()))?;

        projects.into_iter().next()
            .ok_or_else(|| CloudError::NotFound(format!("Project {} not found", project_id)))
    }

    /// List all projects in the cloud
    pub async fn list_projects(&self) -> CloudResult<Vec<CloudProject>> {
        let url = format!("{}/rest/v1/projects", self.project_url);
        
        let response = self.http_client
            .get(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", self.get_auth_header())
            .query(&[("order", "updated_at.desc")])
            .send()
            .await
            .map_err(|e| CloudError::Network(e.to_string()))?;

        if !response.status().is_success() {
            return Err(CloudError::Http(response.status().to_string()));
        }

        response.json::<Vec<CloudProject>>().await
            .map_err(|e| CloudError::InvalidResponse(e.to_string()))
    }

    /// Update a project in the cloud
    pub async fn update_project(&self, project: &CloudProject) -> CloudResult<CloudProject> {
        let url = format!("{}/rest/v1/projects", self.project_url);
        
        let response = self.http_client
            .patch(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", self.get_auth_header())
            .header("Content-Type", "application/json")
            .header("Prefer", "return=representation")
            .query(&[("id", format!("eq.{}", project.id))])
            .json(project)
            .send()
            .await
            .map_err(|e| CloudError::Network(e.to_string()))?;

        match response.status() {
            StatusCode::OK => {
                let projects: Vec<CloudProject> = response.json().await
                    .map_err(|e| CloudError::InvalidResponse(e.to_string()))?;
                projects.into_iter().next()
                    .ok_or_else(|| CloudError::InvalidResponse("No project returned".to_string()))
            }
            StatusCode::UNAUTHORIZED => {
                Err(CloudError::Authentication("Invalid or expired token".to_string()))
            }
            status => {
                let error_text = response.text().await.unwrap_or_else(|_| status.to_string());
                Err(CloudError::Http(error_text))
            }
        }
    }

    /// Delete a project from the cloud
    pub async fn delete_project(&self, project_id: &str) -> CloudResult<()> {
        let url = format!("{}/rest/v1/projects", self.project_url);
        
        let response = self.http_client
            .delete(&url)
            .header("apikey", &self.anon_key)
            .header("Authorization", self.get_auth_header())
            .query(&[("id", format!("eq.{}", project_id))])
            .send()
            .await
            .map_err(|e| CloudError::Network(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CloudError::Http(response.status().to_string()))
        }
    }

    /// Count total projects for the current user
    async fn count_projects(&self) -> CloudResult<usize> {
        let projects = self.list_projects().await?;
        Ok(projects.len())
    }

    /// Get storage usage information
    pub async fn get_storage_usage(&self) -> CloudResult<StorageUsage> {
        // This would query usage_metrics table in Supabase
        // For now, return a placeholder
        Ok(StorageUsage {
            used_mb: 0,
            limit_mb: self.subscription.storage_limit_mb,
            project_count: self.count_projects().await?,
            project_limit: self.subscription.project_limit,
        })
    }
}

/// Storage usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageUsage {
    pub used_mb: i32,
    pub limit_mb: i32,
    pub project_count: usize,
    pub project_limit: i32,
}