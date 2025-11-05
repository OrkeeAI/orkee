// ABOUTME: Daytona provider stub for development environment management
// ABOUTME: Placeholder implementation - returns NotSupported for all operations

use super::{
    ContainerConfig, ContainerInfo, ContainerMetrics, ExecResult, OutputStream, Provider,
    ProviderCapabilities, ProviderError, ProviderInfo, ProviderStatus, Result,
};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::warn;

/// Daytona provider for development workspaces
/// TODO: Implement using Daytona's REST API
/// Documentation: https://www.daytona.io/docs
pub struct DaytonaProvider {
    api_key: String,
    workspace_url: String,
}

impl DaytonaProvider {
    /// Create a new Daytona provider from database settings
    pub fn new(api_key: String, workspace_url: String) -> Result<Self> {
        if api_key.is_empty() || workspace_url.is_empty() {
            return Err(ProviderError::ConfigError(
                "Daytona API key and workspace URL are required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            workspace_url,
        })
    }

    fn not_supported(&self) -> ProviderError {
        ProviderError::NotSupported(
            "Daytona provider is not yet implemented. Only Local Docker is currently supported. \
             To implement: add Daytona REST API client and implement the Provider trait methods."
                .to_string(),
        )
    }
}

#[async_trait]
impl Provider for DaytonaProvider {
    async fn is_available(&self) -> Result<bool> {
        warn!("Daytona provider is not yet implemented");
        Ok(false)
    }

    async fn get_info(&self) -> Result<ProviderInfo> {
        Ok(ProviderInfo {
            name: "Daytona".to_string(),
            version: "stub".to_string(),
            provider_type: "workspace".to_string(),
            capabilities: ProviderCapabilities {
                gpu_support: false,
                persistent_storage: true,
                network_isolation: true,
                resource_limits: true,
                exec_support: true,
                file_transfer: true,
                metrics: true,
            },
            status: ProviderStatus::NotAvailable("Not yet implemented".to_string()),
        })
    }

    async fn create_container(&self, _config: &ContainerConfig) -> Result<String> {
        Err(self.not_supported())
    }

    async fn start_container(&self, _container_id: &str) -> Result<()> {
        Err(self.not_supported())
    }

    async fn stop_container(&self, _container_id: &str, _timeout_secs: u64) -> Result<()> {
        Err(self.not_supported())
    }

    async fn remove_container(&self, _container_id: &str, _force: bool) -> Result<()> {
        Err(self.not_supported())
    }

    async fn get_container_info(&self, _container_id: &str) -> Result<ContainerInfo> {
        Err(self.not_supported())
    }

    async fn list_containers(&self, _include_stopped: bool) -> Result<Vec<ContainerInfo>> {
        Err(self.not_supported())
    }

    async fn exec_command(
        &self,
        _container_id: &str,
        _command: Vec<String>,
        _env_vars: Option<HashMap<String, String>>,
    ) -> Result<ExecResult> {
        Err(self.not_supported())
    }

    async fn stream_logs(
        &self,
        _container_id: &str,
        _follow: bool,
        _since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OutputStream> {
        Err(self.not_supported())
    }

    async fn copy_to_container(
        &self,
        _container_id: &str,
        _source_path: &str,
        _dest_path: &str,
    ) -> Result<()> {
        Err(self.not_supported())
    }

    async fn copy_from_container(
        &self,
        _container_id: &str,
        _source_path: &str,
        _dest_path: &str,
    ) -> Result<()> {
        Err(self.not_supported())
    }

    async fn get_metrics(&self, _container_id: &str) -> Result<ContainerMetrics> {
        Err(self.not_supported())
    }

    async fn pull_image(&self, _image: &str, _force: bool) -> Result<()> {
        Err(self.not_supported())
    }

    async fn image_exists(&self, _image: &str) -> Result<bool> {
        Err(self.not_supported())
    }
}
