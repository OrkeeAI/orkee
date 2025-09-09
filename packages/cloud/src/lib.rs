//! Orkee Cloud - Cloud functionality for project synchronization
//! 
//! This package provides cloud sync capabilities for Orkee projects,
//! including authentication, project synchronization, and subscription management.

pub mod auth;
pub mod client;
pub mod config;
pub mod encryption;
pub mod subscription;
pub mod sync;
pub mod types;

// Public exports - Clean cloud API without implementation details
pub use auth::{CloudAuth, CloudAuthToken, AuthError, AuthResult, UserInfo};
pub use client::{CloudClient, CloudError, CloudResult, CloudProject, StorageUsage};
pub use config::{CloudConfig, CloudMode};
pub use subscription::{
    CloudSubscription, CloudTier, CloudFeature, 
    SubscriptionStatus
};
pub use sync::{CloudSync, SyncEngine, SyncResult, SyncStatus, SyncDirection};

use std::sync::Arc;
use tokio::sync::RwLock;

/// Main entry point for Orkee Cloud functionality
pub struct Cloud {
    config: Arc<RwLock<CloudConfig>>,
    client: Arc<RwLock<Option<CloudClient>>>,
    auth: Arc<CloudAuth>,
    sync_engine: Arc<RwLock<Option<SyncEngine>>>,
}

impl Cloud {
    /// Create a new Cloud instance
    pub async fn new() -> CloudResult<Self> {
        let config = CloudConfig::load().await?;
        
        let auth = Arc::new(CloudAuth::new(
            config.project_url.clone(),
            config.anon_key.clone(),
        ));

        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            client: Arc::new(RwLock::new(None)),
            auth,
            sync_engine: Arc::new(RwLock::new(None)),
        })
    }

    /// Enable cloud functionality
    pub async fn enable(&self) -> CloudResult<()> {
        // Authenticate user
        let token = self.auth.login().await
            .map_err(|e| CloudError::Authentication(e.to_string()))?;

        // Create cloud client
        let config = self.config.read().await;
        let mut client = CloudClient::new(
            config.project_url.clone(),
            config.anon_key.clone(),
        ).await?;

        client.set_access_token(token.access_token.clone());

        // Initialize sync engine
        let sync_engine = SyncEngine::new(client.clone());

        // Update state
        *self.client.write().await = Some(client);
        *self.sync_engine.write().await = Some(sync_engine);

        // Update config
        let mut config = self.config.write().await;
        config.mode = CloudMode::Enabled;
        config.save().await?;

        println!("✅ Orkee Cloud enabled!");
        Ok(())
    }

    /// Disable cloud functionality
    pub async fn disable(&self) -> CloudResult<()> {
        // Clear authentication
        self.auth.logout().await
            .map_err(|e| CloudError::Authentication(e.to_string()))?;

        // Clear client and sync engine
        *self.client.write().await = None;
        *self.sync_engine.write().await = None;

        // Update config
        let mut config = self.config.write().await;
        config.mode = CloudMode::Disabled;
        config.save().await?;

        println!("✅ Orkee Cloud disabled");
        Ok(())
    }

    /// Check if cloud is enabled
    pub async fn is_enabled(&self) -> bool {
        let config = self.config.read().await;
        config.mode == CloudMode::Enabled && self.client.read().await.is_some()
    }

    /// Get cloud status
    pub async fn status(&self) -> CloudResult<CloudStatus> {
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| CloudError::Configuration("Cloud not enabled".to_string()))?;

        let is_authenticated = client.is_authenticated();
        let subscription = client.subscription().clone();
        let usage = if is_authenticated {
            Some(client.get_storage_usage().await?)
        } else {
            None
        };

        Ok(CloudStatus {
            enabled: true,
            authenticated: is_authenticated,
            subscription,
            usage,
        })
    }

    /// Sync projects to cloud
    pub async fn sync(&self) -> CloudResult<SyncResult> {
        let sync_guard = self.sync_engine.read().await;
        let sync_engine = sync_guard.as_ref()
            .ok_or_else(|| CloudError::Configuration("Cloud not enabled".to_string()))?;

        sync_engine.sync_all().await
    }

    /// Sync a specific project
    pub async fn sync_project(&self, project_id: &str) -> CloudResult<SyncResult> {
        let sync_guard = self.sync_engine.read().await;
        let sync_engine = sync_guard.as_ref()
            .ok_or_else(|| CloudError::Configuration("Cloud not enabled".to_string()))?;

        sync_engine.sync_project(project_id).await
    }

    /// Restore projects from cloud
    pub async fn restore(&self) -> CloudResult<Vec<CloudProject>> {
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| CloudError::Configuration("Cloud not enabled".to_string()))?;

        client.list_projects().await
    }

    /// List cloud projects
    pub async fn list(&self) -> CloudResult<Vec<CloudProject>> {
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| CloudError::Configuration("Cloud not enabled".to_string()))?;

        client.list_projects().await
    }

    /// Get authentication manager
    pub fn auth(&self) -> &CloudAuth {
        &self.auth
    }

    /// Get current subscription
    pub async fn subscription(&self) -> CloudResult<CloudSubscription> {
        let client_guard = self.client.read().await;
        let client = client_guard.as_ref()
            .ok_or_else(|| CloudError::Configuration("Cloud not enabled".to_string()))?;

        Ok(client.subscription().clone())
    }
}

/// Cloud status information
#[derive(Debug, Clone)]
pub struct CloudStatus {
    pub enabled: bool,
    pub authenticated: bool,
    pub subscription: CloudSubscription,
    pub usage: Option<StorageUsage>,
}

impl CloudStatus {
    /// Get a formatted status string
    pub fn format(&self) -> String {
        if !self.enabled {
            return "Cloud: Disabled".to_string();
        }

        if !self.authenticated {
            return "Cloud: Not authenticated".to_string();
        }

        let tier = &self.subscription.tier;
        let usage = if let Some(usage) = &self.usage {
            format!(
                "Projects: {}/{}, Storage: {}MB/{}MB",
                usage.project_count,
                if self.subscription.project_limit < 0 { "∞".to_string() } else { self.subscription.project_limit.to_string() },
                usage.used_mb,
                if self.subscription.storage_limit_mb < 0 { "∞".to_string() } else { self.subscription.storage_limit_mb.to_string() }
            )
        } else {
            "Usage data unavailable".to_string()
        };

        format!("Cloud: {} tier - {}", tier, usage)
    }
}

/// Initialize cloud functionality
pub async fn init() -> CloudResult<Cloud> {
    Cloud::new().await
}

/// Quick check if cloud is available
pub async fn is_available() -> bool {
    CloudConfig::exists().await
}

/// Version of the cloud sync protocol
pub const CLOUD_SYNC_VERSION: u32 = 1;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cloud_initialization() {
        let cloud = Cloud::new().await;
        assert!(cloud.is_ok());
    }

    #[tokio::test]
    async fn test_cloud_disabled_by_default() {
        if let Ok(cloud) = Cloud::new().await {
            assert!(!cloud.is_enabled().await);
        }
    }
}