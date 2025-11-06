// ABOUTME: Sandbox lifecycle manager orchestrating storage and provider operations
// ABOUTME: Manages complete sandbox lifecycle from creation to termination with database settings

use crate::providers::{
    ContainerConfig, ContainerInfo, PortMapping, Provider, ProviderError, VolumeMount,
};
use crate::settings::SettingsManager;
use crate::storage::{
    EnvVar, ExecutionStatus, Sandbox, SandboxExecution, SandboxStatus, SandboxStorage,
    StorageError, Volume,
};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("Provider error: {0}")]
    Provider(#[from] ProviderError),

    #[error("Sandbox not found: {0}")]
    NotFound(String),

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Settings error: {0}")]
    SettingsError(String),
}

pub type Result<T> = std::result::Result<T, ManagerError>;

/// Request to create a new sandbox
#[derive(Debug, Clone)]
pub struct CreateSandboxRequest {
    pub name: String,
    pub provider: String,
    pub agent_id: String,
    pub user_id: String,
    pub project_id: Option<String>,
    pub image: Option<String>,
    pub cpu_cores: Option<f32>,
    pub memory_mb: Option<u32>,
    pub storage_gb: Option<u32>,
    pub gpu_enabled: bool,
    pub gpu_model: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    pub ports: Vec<PortMapping>,
    pub ssh_enabled: bool,
    pub config: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Sandbox lifecycle manager
pub struct SandboxManager {
    storage: Arc<SandboxStorage>,
    settings: Arc<RwLock<SettingsManager>>,
    providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
}

impl SandboxManager {
    pub fn new(storage: Arc<SandboxStorage>, settings: Arc<RwLock<SettingsManager>>) -> Self {
        Self {
            storage,
            settings,
            providers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a provider implementation
    pub async fn register_provider(&self, name: String, provider: Arc<dyn Provider>) {
        let mut providers = self.providers.write().await;
        providers.insert(name, provider);
    }

    /// Get a registered provider
    pub async fn get_provider(&self, name: &str) -> Result<Arc<dyn Provider>> {
        let providers = self.providers.read().await;
        providers
            .get(name)
            .cloned()
            .ok_or_else(|| ManagerError::ConfigError(format!("Provider not found: {}", name)))
    }

    /// Create a new sandbox
    pub async fn create_sandbox(&self, request: CreateSandboxRequest) -> Result<Sandbox> {
        // Load settings
        let settings_guard = self.settings.read().await;
        let sandbox_settings = settings_guard
            .get_sandbox_settings()
            .await
            .map_err(|e| ManagerError::SettingsError(e.to_string()))?;

        // Check if sandboxes are enabled
        if !sandbox_settings.enabled {
            return Err(ManagerError::ConfigError(
                "Sandboxes are disabled in settings".to_string(),
            ));
        }

        // Get provider settings
        let provider_settings = settings_guard
            .get_provider_settings(&request.provider)
            .await
            .map_err(|e| ManagerError::SettingsError(e.to_string()))?;

        if !provider_settings.enabled {
            return Err(ManagerError::ConfigError(format!(
                "Provider {} is not enabled",
                request.provider
            )));
        }

        // Validate resource limits
        let cpu_cores: f32 = request.cpu_cores.unwrap_or_else(|| {
            provider_settings
                .default_cpu_cores
                .map(|v| v as f32)
                .unwrap_or(sandbox_settings.max_cpu_cores_per_sandbox as f32)
        });
        let memory_mb: u32 = request.memory_mb.unwrap_or_else(|| {
            provider_settings
                .default_memory_mb
                .map(|v| v as u32)
                .unwrap_or((sandbox_settings.max_memory_gb_per_sandbox * 1024) as u32)
        });
        let storage_gb: u32 = request.storage_gb.unwrap_or_else(|| {
            provider_settings
                .default_disk_gb
                .map(|v| v as u32)
                .unwrap_or(sandbox_settings.max_disk_gb_per_sandbox as u32)
        });

        // Check resource limits
        if cpu_cores as i64 > sandbox_settings.max_cpu_cores_per_sandbox {
            return Err(ManagerError::ResourceLimitExceeded(format!(
                "CPU cores {} exceeds limit of {}",
                cpu_cores, sandbox_settings.max_cpu_cores_per_sandbox
            )));
        }

        if (memory_mb / 1024) as i64 > sandbox_settings.max_memory_gb_per_sandbox {
            return Err(ManagerError::ResourceLimitExceeded(format!(
                "Memory {}MB exceeds limit of {}GB",
                memory_mb, sandbox_settings.max_memory_gb_per_sandbox
            )));
        }

        if storage_gb as i64 > sandbox_settings.max_disk_gb_per_sandbox {
            return Err(ManagerError::ResourceLimitExceeded(format!(
                "Storage {}GB exceeds limit of {}GB",
                storage_gb, sandbox_settings.max_disk_gb_per_sandbox
            )));
        }

        drop(settings_guard);

        // Create sandbox record
        let sandbox = Sandbox {
            id: String::new(), // Will be generated by storage
            name: request.name.clone(),
            provider: request.provider.clone(),
            agent_id: request.agent_id.clone(),
            status: SandboxStatus::Creating,
            container_id: None,
            port: None,
            cpu_cores,
            memory_mb,
            storage_gb,
            gpu_enabled: request.gpu_enabled,
            gpu_model: request.gpu_model.clone(),
            public_url: None,
            ssh_enabled: request.ssh_enabled,
            ssh_key: None,
            created_at: Utc::now(),
            started_at: None,
            stopped_at: None,
            terminated_at: None,
            error_message: None,
            cost_estimate: None,
            project_id: request.project_id.clone(),
            user_id: request.user_id.clone(),
            config: request.config.clone(),
            metadata: request.metadata.clone(),
        };

        // Save sandbox to database
        let sandbox = self.storage.create_sandbox(sandbox).await?;

        // Get provider
        let provider = self.get_provider(&request.provider).await?;

        // Get default image with fallback chain:
        // 1. Request image (explicit override)
        // 2. Provider default image (provider-specific)
        // 3. Global default image (sandbox settings)
        // 4. Hardcoded fallback
        let image = request
            .image
            .or(provider_settings.default_image.clone())
            .unwrap_or_else(|| sandbox_settings.default_image.clone());

        // Create container configuration
        let mut labels = HashMap::new();
        labels.insert("orkee.sandbox.id".to_string(), sandbox.id.clone());
        labels.insert("orkee.sandbox.agent".to_string(), request.agent_id.clone());

        let config = ContainerConfig {
            image,
            name: request.name.clone(),
            env_vars: request.env_vars.clone(),
            volumes: request.volumes.clone(),
            ports: request.ports.clone(),
            cpu_cores,
            memory_mb: memory_mb as u64,
            storage_gb: storage_gb as u64,
            command: None,
            working_dir: Some("/workspace".to_string()),
            labels,
        };

        // Create container with provider
        match provider.create_container(&config).await {
            Ok(container_id) => {
                // Update sandbox with container ID and status
                let mut updated_sandbox = sandbox.clone();
                updated_sandbox.container_id = Some(container_id.clone());
                updated_sandbox.status = SandboxStatus::Running;
                updated_sandbox.started_at = Some(Utc::now());

                // Atomically update container ID and status in database
                // This ensures both fields are updated together, preventing inconsistent state
                self.storage
                    .update_sandbox_with_container(
                        &sandbox.id,
                        &container_id,
                        SandboxStatus::Running,
                        None,
                    )
                    .await?;

                // Store environment variables
                for (name, value) in request.env_vars.iter() {
                    let env_var = EnvVar {
                        id: None,
                        sandbox_id: sandbox.id.clone(),
                        name: name.clone(),
                        value: value.clone(),
                        is_secret: false,
                        created_at: Utc::now(),
                    };
                    self.storage.add_env_var(env_var).await?;
                }

                // Store volumes
                for volume in request.volumes.iter() {
                    let vol = Volume {
                        id: None,
                        sandbox_id: sandbox.id.clone(),
                        host_path: volume.host_path.clone(),
                        container_path: volume.container_path.clone(),
                        read_only: volume.readonly,
                        created_at: Utc::now(),
                    };
                    self.storage.add_volume(vol).await?;
                }

                Ok(updated_sandbox)
            }
            Err(e) => {
                // Update sandbox status to error
                self.storage
                    .update_sandbox_status(&sandbox.id, SandboxStatus::Error, Some(e.to_string()))
                    .await?;

                Err(ManagerError::Provider(e))
            }
        }
    }

    /// Start a stopped sandbox
    pub async fn start_sandbox(&self, sandbox_id: &str) -> Result<()> {
        let sandbox = self.storage.get_sandbox(sandbox_id).await?;

        if sandbox.status != SandboxStatus::Stopped {
            return Err(ManagerError::InvalidStateTransition(format!(
                "Cannot start sandbox in state {:?}",
                sandbox.status
            )));
        }

        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ManagerError::ConfigError("No container ID".to_string()))?;

        let provider = self.get_provider(&sandbox.provider).await?;

        // Update status to starting
        self.storage
            .update_sandbox_status(sandbox_id, SandboxStatus::Starting, None)
            .await?;

        // Start container
        match provider.start_container(container_id).await {
            Ok(()) => {
                self.storage
                    .update_sandbox_status(sandbox_id, SandboxStatus::Running, None)
                    .await?;
                Ok(())
            }
            Err(e) => {
                self.storage
                    .update_sandbox_status(sandbox_id, SandboxStatus::Error, Some(e.to_string()))
                    .await?;
                Err(ManagerError::Provider(e))
            }
        }
    }

    /// Stop a running sandbox
    pub async fn stop_sandbox(&self, sandbox_id: &str, timeout_secs: u64) -> Result<()> {
        let sandbox = self.storage.get_sandbox(sandbox_id).await?;

        if sandbox.status != SandboxStatus::Running {
            return Err(ManagerError::InvalidStateTransition(format!(
                "Cannot stop sandbox in state {:?}",
                sandbox.status
            )));
        }

        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ManagerError::ConfigError("No container ID".to_string()))?;

        let provider = self.get_provider(&sandbox.provider).await?;

        // Update status to stopping
        self.storage
            .update_sandbox_status(sandbox_id, SandboxStatus::Stopping, None)
            .await?;

        // Stop container
        match provider.stop_container(container_id, timeout_secs).await {
            Ok(()) => {
                self.storage
                    .update_sandbox_status(sandbox_id, SandboxStatus::Stopped, None)
                    .await?;
                Ok(())
            }
            Err(e) => {
                self.storage
                    .update_sandbox_status(sandbox_id, SandboxStatus::Error, Some(e.to_string()))
                    .await?;
                Err(ManagerError::Provider(e))
            }
        }
    }

    /// Remove/terminate a sandbox
    pub async fn remove_sandbox(&self, sandbox_id: &str, force: bool) -> Result<()> {
        let sandbox = self.storage.get_sandbox(sandbox_id).await?;

        if let Some(container_id) = &sandbox.container_id {
            let provider = self.get_provider(&sandbox.provider).await?;

            // Stop if running and not forcing
            if sandbox.status == SandboxStatus::Running && !force {
                self.stop_sandbox(sandbox_id, 10).await?;
            }

            // Remove container
            provider.remove_container(container_id, force).await?;
        }

        // Update status to terminated
        self.storage
            .update_sandbox_status(sandbox_id, SandboxStatus::Terminated, None)
            .await?;

        // Delete from database if settings allow
        let settings_guard = self.settings.read().await;
        let sandbox_settings = settings_guard
            .get_sandbox_settings()
            .await
            .map_err(|e| ManagerError::SettingsError(e.to_string()))?;

        if !sandbox_settings.preserve_stopped_sandboxes {
            self.storage.delete_sandbox(sandbox_id).await?;
        }

        Ok(())
    }

    /// Get sandbox status
    pub async fn get_sandbox(&self, sandbox_id: &str) -> Result<Sandbox> {
        Ok(self.storage.get_sandbox(sandbox_id).await?)
    }

    /// List all sandboxes
    pub async fn list_sandboxes(
        &self,
        user_id: Option<&str>,
        status: Option<SandboxStatus>,
    ) -> Result<Vec<Sandbox>> {
        Ok(self.storage.list_sandboxes(user_id, None, status).await?)
    }

    /// Get container info from provider
    pub async fn get_container_info(&self, sandbox_id: &str) -> Result<ContainerInfo> {
        let sandbox = self.storage.get_sandbox(sandbox_id).await?;

        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ManagerError::ConfigError("No container ID".to_string()))?;

        let provider = self.get_provider(&sandbox.provider).await?;
        Ok(provider.get_container_info(container_id).await?)
    }

    /// Create an execution record
    pub async fn create_execution(
        &self,
        sandbox_id: &str,
        command: String,
        working_directory: Option<String>,
        created_by: Option<String>,
        agent_execution_id: Option<String>,
    ) -> Result<SandboxExecution> {
        let execution = SandboxExecution {
            id: String::new(), // Will be generated by storage
            sandbox_id: sandbox_id.to_string(),
            command,
            working_directory: working_directory.unwrap_or_else(|| "/workspace".to_string()),
            status: ExecutionStatus::Queued,
            started_at: None,
            completed_at: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            cpu_time_seconds: None,
            memory_peak_mb: None,
            created_at: Utc::now(),
            created_by,
            agent_execution_id,
        };

        Ok(self.storage.create_execution(execution).await?)
    }

    /// Update execution status
    pub async fn update_execution_status(
        &self,
        execution_id: &str,
        status: ExecutionStatus,
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> Result<()> {
        Ok(self
            .storage
            .update_execution_status(execution_id, status, exit_code, stdout, stderr)
            .await?)
    }

    /// Get execution by ID
    pub async fn get_execution(&self, execution_id: &str) -> Result<SandboxExecution> {
        Ok(self.storage.get_execution(execution_id).await?)
    }

    /// List executions for a sandbox
    pub async fn list_executions(&self, sandbox_id: &str) -> Result<Vec<SandboxExecution>> {
        Ok(self.storage.list_executions(sandbox_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{ContainerStatus, PortMapping, ProviderInfo, VolumeMount};
    use async_trait::async_trait;
    use std::collections::HashMap;

    // Mock provider for testing
    struct MockProvider;

    #[async_trait]
    impl Provider for MockProvider {
        async fn is_available(&self) -> std::result::Result<bool, ProviderError> {
            Ok(true)
        }

        async fn get_info(&self) -> std::result::Result<ProviderInfo, ProviderError> {
            Ok(ProviderInfo {
                name: "Mock".to_string(),
                version: "1.0.0".to_string(),
                provider_type: "mock".to_string(),
                capabilities: crate::providers::ProviderCapabilities {
                    gpu_support: false,
                    persistent_storage: true,
                    network_isolation: true,
                    resource_limits: true,
                    exec_support: true,
                    file_transfer: true,
                    metrics: true,
                },
                status: crate::providers::ProviderStatus::Ready,
            })
        }

        async fn create_container(
            &self,
            _config: &ContainerConfig,
        ) -> std::result::Result<String, ProviderError> {
            Ok("mock-container-id".to_string())
        }

        async fn start_container(
            &self,
            _container_id: &str,
        ) -> std::result::Result<(), ProviderError> {
            Ok(())
        }

        async fn stop_container(
            &self,
            _container_id: &str,
            _timeout_secs: u64,
        ) -> std::result::Result<(), ProviderError> {
            Ok(())
        }

        async fn remove_container(
            &self,
            _container_id: &str,
            _force: bool,
        ) -> std::result::Result<(), ProviderError> {
            Ok(())
        }

        async fn get_container_info(
            &self,
            container_id: &str,
        ) -> std::result::Result<ContainerInfo, ProviderError> {
            Ok(ContainerInfo {
                id: container_id.to_string(),
                name: "mock-container".to_string(),
                status: ContainerStatus::Running,
                ip_address: Some("127.0.0.1".to_string()),
                ports: HashMap::new(),
                created_at: Utc::now(),
                started_at: Some(Utc::now()),
                metrics: None,
            })
        }

        async fn list_containers(
            &self,
            _include_stopped: bool,
        ) -> std::result::Result<Vec<ContainerInfo>, ProviderError> {
            Ok(vec![])
        }

        async fn exec_command(
            &self,
            _container_id: &str,
            _command: Vec<String>,
            _env_vars: Option<HashMap<String, String>>,
        ) -> std::result::Result<crate::providers::ExecResult, ProviderError> {
            Ok(crate::providers::ExecResult {
                exit_code: 0,
                stdout: vec![],
                stderr: vec![],
            })
        }

        async fn stream_logs(
            &self,
            _container_id: &str,
            _follow: bool,
            _since: Option<chrono::DateTime<chrono::Utc>>,
        ) -> std::result::Result<crate::providers::OutputStream, ProviderError> {
            let (_tx, rx) = tokio::sync::mpsc::unbounded_channel();
            Ok(crate::providers::OutputStream { receiver: rx })
        }

        async fn copy_to_container(
            &self,
            _container_id: &str,
            _src_path: &str,
            _dst_path: &str,
        ) -> std::result::Result<(), ProviderError> {
            Ok(())
        }

        async fn copy_from_container(
            &self,
            _container_id: &str,
            _src_path: &str,
            _dst_path: &str,
        ) -> std::result::Result<(), ProviderError> {
            Ok(())
        }

        async fn get_metrics(
            &self,
            _container_id: &str,
        ) -> std::result::Result<crate::providers::ContainerMetrics, ProviderError> {
            Ok(crate::providers::ContainerMetrics {
                cpu_usage_percent: 10.0,
                memory_usage_mb: 128,
                memory_limit_mb: 2048,
                network_rx_bytes: 1024,
                network_tx_bytes: 2048,
            })
        }

        async fn pull_image(
            &self,
            _image: &str,
            _force: bool,
        ) -> std::result::Result<(), ProviderError> {
            Ok(())
        }

        async fn image_exists(&self, _image: &str) -> std::result::Result<bool, ProviderError> {
            Ok(true)
        }
    }

    // Note: Full integration tests would require setting up test database
    // These are unit tests for the manager logic
}
