// ABOUTME: Cloudflare Workers provider stub for edge compute
// ABOUTME: Placeholder implementation - returns NotSupported for all operations

use super::{
    ContainerConfig, ContainerInfo, ContainerMetrics, ExecResult, OutputStream, Provider,
    ProviderCapabilities, ProviderError, ProviderInfo, ProviderStatus, Result,
};
use async_trait::async_trait;
use std::collections::HashMap;
use tracing::warn;

/// Cloudflare Workers provider for edge compute
/// TODO: Implement using Cloudflare Workers API
/// Documentation: https://developers.cloudflare.com/workers
#[allow(dead_code)]
pub struct CloudflareProvider {
    account_id: String,
    api_token: String,
    api_endpoint: String,
}

impl CloudflareProvider {
    /// Create a new Cloudflare provider from database settings
    pub fn new(
        account_id: String,
        api_token: String,
        api_endpoint: Option<String>,
    ) -> Result<Self> {
        if account_id.is_empty() || api_token.is_empty() {
            return Err(ProviderError::ConfigError(
                "Cloudflare account ID and API token are required".to_string(),
            ));
        }

        Ok(Self {
            account_id,
            api_token,
            api_endpoint: api_endpoint
                .unwrap_or_else(|| "https://api.cloudflare.com/client/v4".to_string()),
        })
    }

    fn not_supported(&self) -> ProviderError {
        ProviderError::NotSupported(
            "Cloudflare provider is not yet implemented. Only Local Docker is currently supported. \
             To implement: add Cloudflare Workers API client and implement the Provider trait methods."
                .to_string(),
        )
    }
}

#[async_trait]
impl Provider for CloudflareProvider {
    async fn is_available(&self) -> Result<bool> {
        warn!("Cloudflare provider is not yet implemented");
        Ok(false)
    }

    async fn get_info(&self) -> Result<ProviderInfo> {
        Ok(ProviderInfo {
            name: "Cloudflare Workers".to_string(),
            version: "stub".to_string(),
            provider_type: "edge".to_string(),
            capabilities: ProviderCapabilities {
                gpu_support: false,
                persistent_storage: false,
                network_isolation: true,
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
