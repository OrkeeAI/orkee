// ABOUTME: Beam provider stub for serverless GPU infrastructure
// ABOUTME: Placeholder implementation - returns NotSupported for all operations

use super::{
    ContainerConfig, ContainerInfo, ContainerMetrics, ExecResult, OutputStream, Provider,
    ProviderCapabilities, ProviderError, ProviderInfo, ProviderStatus, Result,
};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::warn;

/// Beam provider for serverless GPU workloads
/// TODO: Implement using Beam's REST API
/// Documentation: https://docs.beam.cloud
pub struct BeamProvider {
    api_key: String,
    workspace_id: Option<String>,
    api_endpoint: String,
}

impl BeamProvider {
    /// Create a new Beam provider from database settings
    pub fn new(api_key: String, workspace_id: Option<String>, api_endpoint: Option<String>) -> Result<Self> {
        if api_key.is_empty() {
            return Err(ProviderError::ConfigError(
                "Beam API key is required".to_string(),
            ));
        }

        Ok(Self {
            api_key,
            workspace_id,
            api_endpoint: api_endpoint.unwrap_or_else(|| "https://api.beam.cloud".to_string()),
        })
    }

    fn not_supported(&self) -> ProviderError {
        ProviderError::NotSupported(
            "Beam provider is not yet implemented. Only Local Docker is currently supported. \
             To implement: add beam-sdk dependency and implement the Provider trait methods."
                .to_string(),
        )
    }
}

#[async_trait]
impl Provider for BeamProvider {
    async fn is_available(&self) -> Result<bool> {
        warn!("Beam provider is not yet implemented");
        Ok(false)
    }

    async fn get_info(&self) -> Result<ProviderInfo> {
        Ok(ProviderInfo {
            name: "Beam".to_string(),
            version: "stub".to_string(),
            provider_type: "serverless".to_string(),
            capabilities: ProviderCapabilities {
                gpu_support: true,
                persistent_storage: true,
                network_isolation: false,
                resource_limits: true,
                exec_support: false,
                file_transfer: false,
                metrics: false,
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
