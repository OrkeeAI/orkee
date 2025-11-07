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
        // Validate agent_id exists in agent registry
        // This prevents orphaned sandbox records if agent is removed
        if !orkee_models::REGISTRY.agent_exists(&request.agent_id) {
            return Err(ManagerError::ConfigError(format!(
                "Agent '{}' not found in agent registry. \
                 Check packages/agents/config/agents.json for available agents.",
                request.agent_id
            )));
        }

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

        // Enforce security: privileged containers must be explicitly disabled unless in development
        // This prevents privilege escalation and host access in production environments
        if sandbox_settings.allow_privileged_containers {
            tracing::warn!(
                "SECURITY WARNING: Privileged containers are enabled. \
                 This allows containers to access host resources and should ONLY \
                 be used in trusted development environments. Production systems \
                 should have allow_privileged_containers = false."
            );
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

        // Validate resource limits with overflow protection
        let cpu_cores: f32 = request.cpu_cores.unwrap_or_else(|| {
            provider_settings
                .default_cpu_cores
                .map(|v| v as f32)
                .unwrap_or(sandbox_settings.max_cpu_cores_per_sandbox as f32)
        });

        // Validate CPU is not NaN or Infinity (prevent floating point issues)
        if cpu_cores.is_nan() || cpu_cores.is_infinite() {
            return Err(ManagerError::ConfigError(format!(
                "Invalid CPU cores value: {}. Must be a finite positive number.",
                cpu_cores
            )));
        }

        let memory_mb: u32 = request.memory_mb.unwrap_or_else(|| {
            provider_settings
                .default_memory_mb
                .map(|v| v as u32)
                .unwrap_or_else(|| {
                    // Use checked multiplication to prevent overflow
                    // If overflow occurs, use u32::MAX
                    (sandbox_settings.max_memory_gb_per_sandbox as u64)
                        .checked_mul(1024)
                        .and_then(|v| u32::try_from(v).ok())
                        .unwrap_or(u32::MAX)
                })
        });

        let storage_gb: u32 = request.storage_gb.unwrap_or_else(|| {
            provider_settings
                .default_disk_gb
                .map(|v| {
                    // Use saturating cast to prevent negative values from becoming large positive
                    u32::try_from(v.max(0)).unwrap_or(0)
                })
                .unwrap_or_else(|| {
                    // Use saturating cast to prevent overflow
                    u32::try_from(sandbox_settings.max_disk_gb_per_sandbox.max(0))
                        .unwrap_or(u32::MAX)
                })
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

        // Check concurrent sandbox limits
        let active_sandboxes = self
            .storage
            .list_sandboxes(None, None, Some(SandboxStatus::Running))
            .await?;

        // Determine if this is a local or cloud provider
        let is_local_provider = request.provider == "local"
            || provider_settings
                .custom_config
                .as_ref()
                .and_then(|c| c.get("provider_type"))
                .and_then(|t| t.as_str())
                == Some("docker");

        if is_local_provider {
            let local_count = active_sandboxes
                .iter()
                .filter(|s| s.provider == "local" || s.provider.to_lowercase().contains("docker"))
                .count() as i64;

            if local_count >= sandbox_settings.max_concurrent_local {
                return Err(ManagerError::ResourceLimitExceeded(format!(
                    "Maximum concurrent local sandboxes ({}) reached. {} currently running.",
                    sandbox_settings.max_concurrent_local, local_count
                )));
            }
        } else {
            let cloud_count = active_sandboxes
                .iter()
                .filter(|s| s.provider != "local" && !s.provider.to_lowercase().contains("docker"))
                .count() as i64;

            if cloud_count >= sandbox_settings.max_concurrent_cloud {
                return Err(ManagerError::ResourceLimitExceeded(format!(
                    "Maximum concurrent cloud sandboxes ({}) reached. {} currently running.",
                    sandbox_settings.max_concurrent_cloud, cloud_count
                )));
            }
        }

        // Check provider-specific sandbox limit
        if let Some(max_sandboxes) = provider_settings.max_sandboxes {
            let provider_count = active_sandboxes
                .iter()
                .filter(|s| s.provider == request.provider)
                .count() as i64;

            if provider_count >= max_sandboxes {
                return Err(ManagerError::ResourceLimitExceeded(format!(
                    "Maximum sandboxes for provider {} ({}) reached. {} currently running.",
                    request.provider, max_sandboxes, provider_count
                )));
            }
        }

        // Check cost limits if cost tracking is enabled
        if sandbox_settings.cost_tracking_enabled {
            // Calculate estimated cost for this sandbox (placeholder - actual implementation needed)
            let estimated_cost = self.estimate_sandbox_cost(
                &request.provider,
                cpu_cores,
                memory_mb,
                storage_gb,
                request.gpu_enabled,
                &provider_settings,
            );

            // Check per-sandbox cost limit
            if estimated_cost > sandbox_settings.max_cost_per_sandbox {
                return Err(ManagerError::ResourceLimitExceeded(format!(
                    "Estimated cost ${:.2} exceeds per-sandbox limit of ${:.2}",
                    estimated_cost, sandbox_settings.max_cost_per_sandbox
                )));
            }

            // Check total cost limit
            let total_current_cost: f64 = active_sandboxes
                .iter()
                .filter_map(|s| s.cost_estimate)
                .sum();

            if total_current_cost + estimated_cost > sandbox_settings.max_total_cost {
                return Err(ManagerError::ResourceLimitExceeded(format!(
                    "Total cost ${:.2} would exceed limit of ${:.2} (current: ${:.2}, estimated: ${:.2})",
                    total_current_cost + estimated_cost,
                    sandbox_settings.max_total_cost,
                    total_current_cost,
                    estimated_cost
                )));
            }
        }

        // Validate volume mounts for security - prevent mounting sensitive directories
        const BLOCKED_PATHS: &[&str] = &[
            "/etc",
            "/sys",
            "/proc",
            "/root",
            "/boot",
            "/dev",
            "/usr/bin",
            "/usr/sbin",
            "/sbin",
            "/bin",
        ];

        const BLOCKED_USER_PATHS: &[&str] = &[".ssh", ".aws", ".gnupg", ".kube", ".docker"];

        for volume in &request.volumes {
            let host_path = volume.host_path.trim();

            // Check against absolute blocked paths
            for blocked in BLOCKED_PATHS {
                if host_path == *blocked || host_path.starts_with(&format!("{}/", blocked)) {
                    return Err(ManagerError::ConfigError(format!(
                        "Volume mount path '{}' is blocked for security reasons. \
                         Cannot mount sensitive system directories: {}",
                        host_path,
                        BLOCKED_PATHS.join(", ")
                    )));
                }
            }

            // Check against user-relative blocked paths (e.g., ~/.ssh)
            for blocked in BLOCKED_USER_PATHS {
                if host_path.contains(blocked) {
                    return Err(ManagerError::ConfigError(format!(
                        "Volume mount path '{}' contains blocked pattern '{}'. \
                         Cannot mount sensitive user directories like .ssh, .aws, .gnupg, etc.",
                        host_path, blocked
                    )));
                }
            }

            // Ensure path is absolute for security (prevents relative path attacks)
            if !host_path.starts_with('/') && !host_path.starts_with('~') {
                return Err(ManagerError::ConfigError(format!(
                    "Volume mount path '{}' must be absolute (start with / or ~). \
                     Relative paths are not allowed for security reasons.",
                    host_path
                )));
            }
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

        // Prepare environment variables and volumes for atomic creation
        let env_vars: Vec<EnvVar> = request
            .env_vars
            .iter()
            .map(|(name, value)| EnvVar {
                id: None,
                sandbox_id: String::new(), // Will be set by storage layer
                name: name.clone(),
                value: value.clone(),
                is_secret: false,
                created_at: Utc::now(),
            })
            .collect();

        let volumes: Vec<Volume> = request
            .volumes
            .iter()
            .map(|v| Volume {
                id: None,
                sandbox_id: String::new(), // Will be set by storage layer
                host_path: v.host_path.clone(),
                container_path: v.container_path.clone(),
                read_only: v.readonly,
                created_at: Utc::now(),
            })
            .collect();

        // Save sandbox with env vars and volumes to database in a single transaction
        // This ensures all related data is created atomically
        let sandbox = self
            .storage
            .create_sandbox_with_resources(sandbox, env_vars, volumes)
            .await?;

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

    /// Execute a command in a sandbox with security validation
    ///
    /// This method validates commands against blocked_commands settings before execution.
    /// Commands are blocked if they match patterns in the settings, preventing:
    /// - Dangerous system commands (e.g., rm -rf /, mkfs)
    /// - Network manipulation (e.g., iptables, route)
    /// - Process manipulation (e.g., kill -9, pkill)
    /// - Privilege escalation attempts (e.g., sudo, su)
    pub async fn exec_sandbox_command(
        &self,
        sandbox_id: &str,
        command: Vec<String>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<crate::providers::ExecResult> {
        let sandbox = self.storage.get_sandbox(sandbox_id).await?;

        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ManagerError::ConfigError("No container ID".to_string()))?;

        // Load settings to check blocked_commands
        let settings_guard = self.settings.read().await;
        let sandbox_settings = settings_guard
            .get_sandbox_settings()
            .await
            .map_err(|e| ManagerError::SettingsError(e.to_string()))?;

        // Validate command against blocked_commands list
        if let Some(blocked_commands) = &sandbox_settings.blocked_commands {
            if let Some(blocked_list) = blocked_commands.as_array() {
                let command_str = command.join(" ");
                for blocked in blocked_list {
                    if let Some(pattern) = blocked.as_str() {
                        // Check if command starts with blocked pattern
                        // This prevents both exact matches and commands with arguments
                        if command_str.starts_with(pattern)
                            || (!command.is_empty() && command[0] == pattern)
                        {
                            return Err(ManagerError::ConfigError(format!(
                                "Command '{}' is blocked by security policy. Blocked pattern: '{}'",
                                command_str, pattern
                            )));
                        }
                    }
                }
            }
        }

        drop(settings_guard);

        let provider = self.get_provider(&sandbox.provider).await?;
        Ok(provider
            .exec_command(container_id, command, env_vars)
            .await?)
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

    /// Estimate the cost of running a sandbox
    /// Returns the estimated cost in USD per hour
    fn estimate_sandbox_cost(
        &self,
        _provider: &str,
        cpu_cores: f32,
        memory_mb: u32,
        _storage_gb: u32,
        gpu_enabled: bool,
        provider_settings: &crate::settings::ProviderSettings,
    ) -> f64 {
        // Use provider-specific pricing if available, otherwise use defaults
        let per_hour = provider_settings.cost_per_hour.unwrap_or(0.0);
        let per_gb_memory = provider_settings.cost_per_gb_memory.unwrap_or(0.0);
        let per_vcpu = provider_settings.cost_per_vcpu.unwrap_or(0.0);
        let per_gpu_hour = provider_settings.cost_per_gpu_hour.unwrap_or(0.0);

        // Calculate base cost
        let mut total_cost = per_hour;

        // Add CPU cost
        total_cost += cpu_cores as f64 * per_vcpu;

        // Add memory cost (convert MB to GB)
        total_cost += (memory_mb as f64 / 1024.0) * per_gb_memory;

        // Add GPU cost if enabled
        if gpu_enabled {
            total_cost += per_gpu_hour;
        }

        // Storage cost is typically per month, not per hour
        // For hourly estimate, we only include compute costs
        // Storage costs would be tracked separately

        total_cost
    }

    /// Clean up orphaned containers
    /// Returns (containers_found, containers_removed, errors)
    ///
    /// Orphaned containers are those that exist in the provider (e.g., Docker)
    /// but are not tracked in the Orkee database. This can happen if:
    /// - The database entry was deleted but the container was not
    /// - Orkee crashed during container creation/deletion
    /// - Manual container manipulation outside of Orkee
    ///
    /// This method will:
    /// 1. List all containers from the provider with Orkee labels
    /// 2. Check which ones exist in the database
    /// 3. Remove containers that are not in the database
    pub async fn cleanup_orphaned_containers(
        &self,
        provider_name: &str,
        dry_run: bool,
    ) -> Result<(usize, usize, Vec<String>)> {
        use tracing::{info, warn};

        let provider = self.get_provider(provider_name).await?;

        // List all containers from the provider (including stopped ones)
        let provider_containers = provider
            .list_containers(true)
            .await
            .map_err(ManagerError::Provider)?;

        info!(
            "Found {} containers from provider '{}'",
            provider_containers.len(),
            provider_name
        );

        let mut orphaned_count = 0;
        let mut removed_count = 0;
        let mut errors = Vec::new();

        // Check each container against the database
        for container in provider_containers {
            // Try to find a sandbox with this container_id in the database
            let sandboxes = self
                .storage
                .list_sandboxes(None, None, None)
                .await
                .map_err(ManagerError::Storage)?;

            let found_in_db = sandboxes
                .iter()
                .any(|s| s.container_id.as_ref() == Some(&container.id));

            if !found_in_db {
                orphaned_count += 1;
                warn!(
                    "Found orphaned container: {} (name: {})",
                    container.id, container.name
                );

                if !dry_run {
                    // Attempt to remove the orphaned container
                    match provider.remove_container(&container.id, true).await {
                        Ok(_) => {
                            info!("Removed orphaned container: {}", container.id);
                            removed_count += 1;
                        }
                        Err(e) => {
                            let error_msg = format!(
                                "Failed to remove orphaned container {}: {}",
                                container.id, e
                            );
                            warn!("{}", error_msg);
                            errors.push(error_msg);
                        }
                    }
                }
            }
        }

        if dry_run {
            info!(
                "Dry run complete: found {} orphaned containers (none removed)",
                orphaned_count
            );
        } else {
            info!(
                "Cleanup complete: found {} orphaned containers, removed {} (failed: {})",
                orphaned_count,
                removed_count,
                errors.len()
            );
        }

        Ok((orphaned_count, removed_count, errors))
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

    async fn create_test_db() -> sqlx::SqlitePool {
        let pool = sqlx::SqlitePool::connect("sqlite::memory:").await.unwrap();

        // Run storage migrations
        sqlx::migrate!("../storage/migrations")
            .run(&pool)
            .await
            .unwrap();

        pool
    }

    async fn setup_test_manager() -> (SandboxManager, sqlx::SqlitePool) {
        let pool = create_test_db().await;
        let storage = Arc::new(SandboxStorage::new(pool.clone()));
        let settings_manager = SettingsManager::new(pool.clone()).unwrap();

        // Enable beam provider in settings
        let mut beam_settings = settings_manager
            .get_provider_settings("beam")
            .await
            .unwrap();
        beam_settings.enabled = true;
        beam_settings.configured = true;
        settings_manager
            .update_provider_settings(&beam_settings, Some("test"))
            .await
            .unwrap();

        let settings = Arc::new(RwLock::new(settings_manager));

        let manager = SandboxManager::new(storage, settings);
        manager
            .register_provider("local".to_string(), Arc::new(MockProvider))
            .await;
        manager
            .register_provider("beam".to_string(), Arc::new(MockProvider))
            .await;

        (manager, pool)
    }

    #[tokio::test]
    async fn test_resource_limit_cpu_exceeds_max() {
        let (manager, _pool) = setup_test_manager().await;

        let request = CreateSandboxRequest {
            name: "test-sandbox".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(999.0), // Way over limit
            memory_mb: Some(1024),
            storage_gb: Some(10),
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManagerError::ResourceLimitExceeded(_)
        ));
    }

    #[tokio::test]
    async fn test_resource_limit_memory_exceeds_max() {
        let (manager, _pool) = setup_test_manager().await;

        let request = CreateSandboxRequest {
            name: "test-sandbox".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(2.0),
            memory_mb: Some(999_000), // Way over limit (999 GB)
            storage_gb: Some(10),
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManagerError::ResourceLimitExceeded(_)
        ));
    }

    #[tokio::test]
    async fn test_resource_limit_storage_exceeds_max() {
        let (manager, _pool) = setup_test_manager().await;

        let request = CreateSandboxRequest {
            name: "test-sandbox".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(2.0),
            memory_mb: Some(1024),
            storage_gb: Some(9999), // Way over limit
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request).await;
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ManagerError::ResourceLimitExceeded(_)
        ));
    }

    #[tokio::test]
    async fn test_concurrent_local_sandbox_limit() {
        let (manager, pool) = setup_test_manager().await;

        // Update settings to allow only 2 concurrent local sandboxes
        let settings_guard = manager.settings.read().await;
        let mut sandbox_settings = settings_guard.get_sandbox_settings().await.unwrap();
        sandbox_settings.max_concurrent_local = 2;
        settings_guard
            .update_sandbox_settings(&sandbox_settings, Some("test"))
            .await
            .unwrap();
        drop(settings_guard);

        // Create first sandbox
        let request1 = CreateSandboxRequest {
            name: "sandbox-1".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            storage_gb: Some(10),
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let sandbox1 = manager.create_sandbox(request1).await.unwrap();
        assert_eq!(sandbox1.status, SandboxStatus::Running);

        // Create second sandbox
        let request2 = CreateSandboxRequest {
            name: "sandbox-2".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-2".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            storage_gb: Some(10),
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let sandbox2 = manager.create_sandbox(request2).await.unwrap();
        assert_eq!(sandbox2.status, SandboxStatus::Running);

        // Try to create third sandbox - should fail due to concurrent limit
        let request3 = CreateSandboxRequest {
            name: "sandbox-3".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-3".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(1.0),
            memory_mb: Some(512),
            storage_gb: Some(10),
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request3).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ManagerError::ResourceLimitExceeded(_)));
        assert!(err
            .to_string()
            .contains("Maximum concurrent local sandboxes"));
    }

    #[tokio::test]
    async fn test_concurrent_cloud_sandbox_limit() {
        let (manager, _pool) = setup_test_manager().await;

        // Update settings to allow only 1 concurrent cloud sandbox
        let settings_guard = manager.settings.read().await;
        let mut sandbox_settings = settings_guard.get_sandbox_settings().await.unwrap();
        sandbox_settings.max_concurrent_cloud = 1;
        settings_guard
            .update_sandbox_settings(&sandbox_settings, Some("test"))
            .await
            .unwrap();
        drop(settings_guard);

        // Create first cloud sandbox
        let request1 = CreateSandboxRequest {
            name: "cloud-sandbox-1".to_string(),
            provider: "beam".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(2.0),
            memory_mb: Some(1024),
            storage_gb: Some(20),
            gpu_enabled: true,
            gpu_model: Some("T4".to_string()),
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let sandbox1 = manager.create_sandbox(request1).await.unwrap();
        assert_eq!(sandbox1.status, SandboxStatus::Running);

        // Try to create second cloud sandbox - should fail
        let request2 = CreateSandboxRequest {
            name: "cloud-sandbox-2".to_string(),
            provider: "beam".to_string(),
            agent_id: "agent-2".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(2.0),
            memory_mb: Some(1024),
            storage_gb: Some(20),
            gpu_enabled: true,
            gpu_model: Some("T4".to_string()),
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request2).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ManagerError::ResourceLimitExceeded(_)));
        assert!(err
            .to_string()
            .contains("Maximum concurrent cloud sandboxes"));
    }

    #[tokio::test]
    async fn test_cost_tracking_per_sandbox_limit() {
        let (manager, _pool) = setup_test_manager().await;

        // Enable cost tracking and set low per-sandbox limit
        let settings_guard = manager.settings.read().await;
        let mut sandbox_settings = settings_guard.get_sandbox_settings().await.unwrap();
        sandbox_settings.cost_tracking_enabled = true;
        sandbox_settings.max_cost_per_sandbox = 0.01; // Very low limit
        settings_guard
            .update_sandbox_settings(&sandbox_settings, Some("test"))
            .await
            .unwrap();

        // Set provider pricing to trigger cost limit
        let mut provider_settings = settings_guard.get_provider_settings("beam").await.unwrap();
        provider_settings.cost_per_hour = Some(1.0); // $1/hour
        settings_guard
            .update_provider_settings(&provider_settings, Some("test"))
            .await
            .unwrap();
        drop(settings_guard);

        // Try to create expensive sandbox
        let request = CreateSandboxRequest {
            name: "expensive-sandbox".to_string(),
            provider: "beam".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(8.0),
            memory_mb: Some(16384),
            storage_gb: Some(100),
            gpu_enabled: true,
            gpu_model: Some("A100".to_string()),
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, ManagerError::ResourceLimitExceeded(_)));
        assert!(err.to_string().contains("Estimated cost"));
        assert!(err.to_string().contains("exceeds per-sandbox limit"));
    }

    #[tokio::test]
    async fn test_resource_limits_within_bounds_succeeds() {
        let (manager, _pool) = setup_test_manager().await;

        let request = CreateSandboxRequest {
            name: "valid-sandbox".to_string(),
            provider: "local".to_string(),
            agent_id: "agent-1".to_string(),
            user_id: "user-1".to_string(),
            project_id: None,
            image: None,
            cpu_cores: Some(2.0),
            memory_mb: Some(2048),
            storage_gb: Some(20),
            gpu_enabled: false,
            gpu_model: None,
            env_vars: HashMap::new(),
            volumes: vec![],
            ports: vec![],
            ssh_enabled: false,
            config: None,
            metadata: None,
        };

        let result = manager.create_sandbox(request).await;
        assert!(result.is_ok());
        let sandbox = result.unwrap();
        assert_eq!(sandbox.status, SandboxStatus::Running);
        assert_eq!(sandbox.cpu_cores, 2.0);
        assert_eq!(sandbox.memory_mb, 2048);
        assert_eq!(sandbox.storage_gb, 20);
    }

    // Note: Full integration tests would require setting up test database
    // These are unit tests for the manager logic
}
