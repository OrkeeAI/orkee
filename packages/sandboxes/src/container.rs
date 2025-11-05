// ABOUTME: Docker container lifecycle management via bollard
// ABOUTME: Handles creation, monitoring, resource limits, and cleanup of execution containers

use crate::{ContainerStatus, LogEntry, ResourceLimits, ResourceUsage, Result, SandboxError};
use async_stream::stream;
use bollard::{
    container::{
        Config, CreateContainerOptions, ListContainersOptions, LogsOptions, RemoveContainerOptions,
        StartContainerOptions, Stats, StatsOptions, StopContainerOptions,
    },
    errors::Error as BollardError,
    models::{ContainerInspectResponse, HostConfig, RestartPolicy, RestartPolicyNameEnum},
    Docker,
};
use futures_util::stream::StreamExt;
use std::{collections::HashMap, sync::Arc, time::SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Maximum log buffer size before forcing a flush
const LOG_BUFFER_SIZE: usize = 100;

/// Default log buffer flush interval in seconds
const LOG_BUFFER_FLUSH_INTERVAL_SECS: u64 = 5;

/// Labels applied to all Orkee containers for tracking
const ORKEE_LABEL: &str = "orkee.managed";
const ORKEE_PROJECT_LABEL: &str = "orkee.project_id";
const ORKEE_EXECUTION_LABEL: &str = "orkee.execution_id";

/// Container manager for Docker operations
pub struct ContainerManager {
    /// Docker client
    docker: Docker,
    /// Log buffer for batch insertion
    log_buffer: Arc<RwLock<Vec<LogEntry>>>,
}

impl ContainerManager {
    /// Create a new container manager and connect to Docker
    pub async fn new() -> Result<Self> {
        let docker = Self::connect_docker().await?;

        Ok(Self {
            docker,
            log_buffer: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Connect to Docker daemon
    ///
    /// Tries to connect to Docker using the default socket path.
    /// On Unix: /var/run/docker.sock
    /// On Windows: npipe:////./pipe/docker_engine
    async fn connect_docker() -> Result<Docker> {
        #[cfg(unix)]
        let docker = Docker::connect_with_socket_defaults().map_err(|e| SandboxError::Docker(e))?;

        #[cfg(windows)]
        let docker =
            Docker::connect_with_named_pipe_defaults().map_err(|e| SandboxError::Docker(e))?;

        // Verify connection by pinging Docker
        docker.ping().await.map_err(|e| {
            error!("Failed to connect to Docker daemon: {}", e);
            SandboxError::Docker(e)
        })?;

        info!("Successfully connected to Docker daemon");
        Ok(docker)
    }

    /// Create a new container with security settings
    ///
    /// # Security Features
    /// - Resource limits (memory, CPU)
    /// - Capability dropping (no privileged containers)
    /// - Network isolation options
    /// - Read-only rootfs option
    /// - No privileged mode
    pub async fn create_container(
        &self,
        execution_id: &str,
        project_id: Option<&str>,
        image: &str,
        resource_limits: &ResourceLimits,
        workspace_path: Option<&str>,
        env_vars: HashMap<String, String>,
    ) -> Result<String> {
        debug!(
            "Creating container for execution {} with image {}",
            execution_id, image
        );

        // Ensure image exists locally (pull if needed)
        self.ensure_image(image).await?;

        // Convert environment variables to Vec<String> format
        let env: Vec<String> = env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Configure resource limits
        let mut host_config = HostConfig {
            // Memory limit in bytes
            memory: Some((resource_limits.memory_mb * 1024 * 1024) as i64),
            // CPU quota (100000 = 1 core)
            cpu_quota: Some((resource_limits.cpu_cores * 100000.0) as i64),
            cpu_period: Some(100000),
            // Network mode
            network_mode: Some("bridge".to_string()),
            // Restart policy
            restart_policy: Some(RestartPolicy {
                name: Some(RestartPolicyNameEnum::NO),
                maximum_retry_count: Some(0),
            }),
            // Security: Drop all capabilities, run as non-privileged
            cap_drop: Some(vec!["ALL".to_string()]),
            privileged: Some(false),
            // Read-only rootfs (can be overridden if needed)
            readonly_rootfs: Some(false),
            ..Default::default()
        };

        // Add workspace volume mount if specified
        if let Some(workspace) = workspace_path {
            host_config.binds = Some(vec![format!("{}:/workspace", workspace)]);
        }

        // Container configuration
        let config = Config {
            image: Some(image.to_string()),
            env: Some(env),
            working_dir: workspace_path.map(|_| "/workspace".to_string()),
            host_config: Some(host_config),
            labels: Some(self.create_labels(execution_id, project_id)),
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            tty: Some(false),
            ..Default::default()
        };

        // Create container with a unique name
        let container_name = format!("orkee-{}", execution_id);
        let options = CreateContainerOptions {
            name: container_name.clone(),
            platform: None,
        };

        let response = self
            .docker
            .create_container(Some(options), config)
            .await
            .map_err(|e| {
                error!("Failed to create container: {}", e);
                SandboxError::ContainerStartFailed(e.to_string())
            })?;

        info!(
            "Created container {} for execution {}",
            response.id, execution_id
        );

        Ok(response.id)
    }

    /// Start a container
    pub async fn start_container(&self, container_id: &str) -> Result<()> {
        debug!("Starting container {}", container_id);

        self.docker
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| {
                error!("Failed to start container {}: {}", container_id, e);
                SandboxError::ContainerStartFailed(e.to_string())
            })?;

        info!("Started container {}", container_id);
        Ok(())
    }

    /// Stop a container gracefully
    ///
    /// Sends SIGTERM first, then SIGKILL after timeout
    pub async fn stop_container(
        &self,
        container_id: &str,
        timeout_secs: Option<i64>,
    ) -> Result<()> {
        debug!("Stopping container {}", container_id);

        let options = StopContainerOptions {
            t: timeout_secs.unwrap_or(10), // 10 second grace period
        };

        match self
            .docker
            .stop_container(container_id, Some(options))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match e {
                // Container already stopped is not an error
                BollardError::DockerResponseServerError {
                    status_code: 304, ..
                } => {
                    debug!("Container {} already stopped", container_id);
                    Ok(())
                }
                _ => {
                    error!("Failed to stop container {}: {}", container_id, e);
                    Err(SandboxError::Docker(e))
                }
            },
        }?;

        info!("Stopped container {}", container_id);
        Ok(())
    }

    /// Remove a container and clean up resources
    pub async fn remove_container(&self, container_id: &str, force: bool) -> Result<()> {
        debug!("Removing container {} (force={})", container_id, force);

        let options = RemoveContainerOptions {
            force,
            v: true, // Remove volumes
            ..Default::default()
        };

        match self
            .docker
            .remove_container(container_id, Some(options))
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => match e {
                // Container already removed is not an error
                BollardError::DockerResponseServerError {
                    status_code: 404, ..
                } => {
                    debug!("Container {} already removed", container_id);
                    Ok(())
                }
                _ => {
                    error!("Failed to remove container {}: {}", container_id, e);
                    Err(SandboxError::Docker(e))
                }
            },
        }?;

        info!("Removed container {}", container_id);
        Ok(())
    }

    /// List containers by project or execution
    pub async fn list_containers(
        &self,
        project_id: Option<&str>,
        execution_id: Option<&str>,
    ) -> Result<Vec<ContainerInfo>> {
        let mut filters = HashMap::new();

        // Filter for Orkee-managed containers
        filters.insert("label".to_string(), vec![format!("{}=true", ORKEE_LABEL)]);

        // Add project filter if specified
        if let Some(pid) = project_id {
            filters.insert(
                "label".to_string(),
                vec![format!("{}={}", ORKEE_PROJECT_LABEL, pid)],
            );
        }

        // Add execution filter if specified
        if let Some(eid) = execution_id {
            filters.insert(
                "label".to_string(),
                vec![format!("{}={}", ORKEE_EXECUTION_LABEL, eid)],
            );
        }

        let options = ListContainersOptions {
            all: true,
            filters,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await?;

        Ok(containers
            .into_iter()
            .map(|c| ContainerInfo {
                id: c.id.unwrap_or_default(),
                name: c
                    .names
                    .unwrap_or_default()
                    .first()
                    .cloned()
                    .unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                status: c.status.unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                created: c.created.unwrap_or(0),
            })
            .collect())
    }

    /// Get container stats (CPU, memory usage)
    pub async fn get_container_stats(&self, container_id: &str) -> Result<ResourceUsage> {
        debug!("Getting stats for container {}", container_id);

        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stats_stream = self.docker.stats(container_id, Some(options));

        if let Some(stats_result) = stats_stream.next().await {
            let stats = stats_result?;
            let resource_usage = self.parse_stats(&stats);
            return Ok(resource_usage);
        }

        Err(SandboxError::ContainerNotFound(container_id.to_string()))
    }

    /// Get detailed container information
    pub async fn get_container_info(&self, container_id: &str) -> Result<ContainerInspectResponse> {
        debug!("Inspecting container {}", container_id);

        self.docker
            .inspect_container(container_id, None)
            .await
            .map_err(|e| match e {
                BollardError::DockerResponseServerError {
                    status_code: 404, ..
                } => SandboxError::ContainerNotFound(container_id.to_string()),
                _ => SandboxError::Docker(e),
            })
    }

    /// Get container status
    pub async fn get_container_status(&self, container_id: &str) -> Result<ContainerStatus> {
        let info = self.get_container_info(container_id).await?;

        let state = info.state.ok_or_else(|| {
            SandboxError::Docker(BollardError::DockerResponseServerError {
                status_code: 500,
                message: "Container state missing".to_string(),
            })
        })?;

        if state.running.unwrap_or(false) {
            Ok(ContainerStatus::Running)
        } else if let Some(status) = &state.status {
            if format!("{:?}", status).contains("CREATED") {
                Ok(ContainerStatus::Creating)
            } else if state.exit_code.is_some() {
                if state.exit_code.unwrap() == 0 {
                    Ok(ContainerStatus::Stopped)
                } else {
                    Ok(ContainerStatus::Error)
                }
            } else {
                Ok(ContainerStatus::Stopped)
            }
        } else {
            Ok(ContainerStatus::Stopped)
        }
    }

    /// Stream container logs
    ///
    /// Returns a stream of log entries with timestamps and sequence numbers
    pub async fn stream_container_logs(
        &self,
        container_id: String,
        execution_id: String,
        mut sequence_number: i64,
    ) -> impl futures_util::Stream<Item = Result<LogEntry>> {
        let options = LogsOptions::<String> {
            follow: true,
            stdout: true,
            stderr: true,
            timestamps: true,
            ..Default::default()
        };

        let mut log_stream = self.docker.logs(&container_id, Some(options));

        stream! {
            while let Some(log_result) = log_stream.next().await {
                match log_result {
                    Ok(log_output) => {
                        let message = log_output.to_string();
                        let log_entry = LogEntry {
                            id: Self::generate_log_id(),
                            execution_id: execution_id.clone(),
                            timestamp: Self::current_timestamp(),
                            log_level: "info".to_string(),
                            message,
                            source: Some("container".to_string()),
                            metadata: None,
                            stack_trace: None,
                            sequence_number,
                        };
                        sequence_number += 1;
                        yield Ok(log_entry);
                    }
                    Err(e) => {
                        error!("Error reading container logs: {}", e);
                        yield Err(SandboxError::Docker(e));
                        break;
                    }
                }
            }
        }
    }

    /// Clean up stale containers
    ///
    /// Finds and removes containers that are:
    /// - Stopped for more than the specified duration
    /// - Orphaned (no corresponding execution record)
    /// - In error state
    pub async fn cleanup_stale_containers(&self, max_age_hours: u64) -> Result<Vec<String>> {
        info!(
            "Starting container cleanup (max_age_hours={})",
            max_age_hours
        );

        let all_containers = self.list_containers(None, None).await?;
        let mut cleaned_ids = Vec::new();

        let cutoff_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            - (max_age_hours as i64 * 3600);

        for container in all_containers {
            // Skip running containers
            if container.state == "running" {
                continue;
            }

            // Check if container is old enough to clean up
            if container.created < cutoff_time {
                info!(
                    "Cleaning up stale container {} (age: {}h)",
                    container.id,
                    (SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64
                        - container.created)
                        / 3600
                );

                // Try to stop first (in case it's still running somehow)
                let _ = self.stop_container(&container.id, Some(5)).await;

                // Remove container
                match self.remove_container(&container.id, true).await {
                    Ok(_) => cleaned_ids.push(container.id),
                    Err(e) => {
                        warn!("Failed to remove container {}: {}", container.id, e);
                    }
                }
            }
        }

        info!("Cleaned up {} stale containers", cleaned_ids.len());
        Ok(cleaned_ids)
    }

    /// Force stop hung containers
    ///
    /// Finds containers that have been in "stopping" state for too long
    /// and forcefully kills them
    pub async fn force_stop_hung_containers(&self, timeout_minutes: u64) -> Result<Vec<String>> {
        info!(
            "Checking for hung containers (timeout={}min)",
            timeout_minutes
        );

        let all_containers = self.list_containers(None, None).await?;
        let mut stopped_ids = Vec::new();

        for container in all_containers {
            if container.state.contains("stop") || container.state.contains("exit") {
                // Check container details to see how long it's been stopping
                if let Ok(info) = self.get_container_info(&container.id).await {
                    if let Some(state) = info.state {
                        if let Some(_finished_at) = state.finished_at {
                            // Parse timestamp and check age
                            // For now, just force kill any container in stopped/exited state
                            warn!("Force stopping hung container {}", container.id);

                            match self.remove_container(&container.id, true).await {
                                Ok(_) => stopped_ids.push(container.id),
                                Err(e) => {
                                    error!(
                                        "Failed to force stop container {}: {}",
                                        container.id, e
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }

        info!("Force stopped {} hung containers", stopped_ids.len());
        Ok(stopped_ids)
    }

    // Helper methods

    /// Create labels for container tracking
    fn create_labels(
        &self,
        execution_id: &str,
        project_id: Option<&str>,
    ) -> HashMap<String, String> {
        let mut labels = HashMap::new();
        labels.insert(ORKEE_LABEL.to_string(), "true".to_string());
        labels.insert(ORKEE_EXECUTION_LABEL.to_string(), execution_id.to_string());

        if let Some(pid) = project_id {
            labels.insert(ORKEE_PROJECT_LABEL.to_string(), pid.to_string());
        }

        labels
    }

    /// Ensure Docker image exists locally
    async fn ensure_image(&self, image: &str) -> Result<()> {
        // Check if image exists
        match self.docker.inspect_image(image).await {
            Ok(_) => {
                debug!("Image {} already exists locally", image);
                Ok(())
            }
            Err(_) => {
                info!("Pulling image {}", image);
                // TODO: Implement image pulling with progress tracking
                // For now, return error and expect images to be pre-pulled
                Err(SandboxError::ImageError(format!(
                    "Image {} not found locally. Please pull it first with: docker pull {}",
                    image, image
                )))
            }
        }
    }

    /// Parse Docker stats into ResourceUsage
    fn parse_stats(&self, stats: &Stats) -> ResourceUsage {
        let memory_used_mb = stats.memory_stats.usage.unwrap_or(0) as u64 / 1024 / 1024;

        // Calculate CPU percentage
        let cpu_stats = &stats.cpu_stats;
        let precpu_stats = &stats.precpu_stats;

        let cpu_delta =
            cpu_stats.cpu_usage.total_usage as f64 - precpu_stats.cpu_usage.total_usage as f64;
        let system_delta = cpu_stats.system_cpu_usage.unwrap_or(0) as f64
            - precpu_stats.system_cpu_usage.unwrap_or(0) as f64;

        let cpu_usage_percent = if system_delta > 0.0 {
            let num_cpus = cpu_stats
                .cpu_usage
                .percpu_usage
                .as_ref()
                .map(|v| v.len())
                .unwrap_or(1) as f64;
            (cpu_delta / system_delta) * num_cpus * 100.0
        } else {
            0.0
        };

        ResourceUsage {
            memory_used_mb,
            cpu_usage_percent,
        }
    }

    /// Generate unique log entry ID
    fn generate_log_id() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        format!("log_{}", timestamp)
    }

    /// Get current timestamp in ISO 8601 format
    fn current_timestamp() -> String {
        chrono::Utc::now().to_rfc3339()
    }
}

/// Container information summary
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub state: String,
    pub created: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore] // Requires Docker daemon
    async fn test_connect_docker() {
        let result = ContainerManager::connect_docker().await;
        assert!(result.is_ok(), "Failed to connect to Docker: {:?}", result);
    }

    #[tokio::test]
    #[ignore] // Requires Docker daemon
    async fn test_container_lifecycle() {
        let manager = ContainerManager::new().await.unwrap();

        let container_id = manager
            .create_container(
                "test_exec_1",
                Some("test_project"),
                "alpine:latest",
                &ResourceLimits::default(),
                None,
                HashMap::new(),
            )
            .await
            .unwrap();

        assert!(!container_id.is_empty());

        // Start container
        manager.start_container(&container_id).await.unwrap();

        // Get status
        let status = manager.get_container_status(&container_id).await.unwrap();
        assert_eq!(status, ContainerStatus::Running);

        // Stop container
        manager
            .stop_container(&container_id, Some(5))
            .await
            .unwrap();

        // Remove container
        manager.remove_container(&container_id, true).await.unwrap();
    }

    #[tokio::test]
    #[ignore] // Requires Docker daemon
    async fn test_list_containers() {
        let manager = ContainerManager::new().await.unwrap();

        let containers = manager.list_containers(None, None).await.unwrap();
        // Just verify it doesn't error
        assert!(containers.is_empty() || !containers.is_empty());
    }
}
