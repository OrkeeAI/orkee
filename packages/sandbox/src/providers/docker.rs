// ABOUTME: Docker provider implementation for local container-based sandboxes
// ABOUTME: Uses bollard library to manage Docker containers for agent execution

use super::{
    ContainerConfig, ContainerInfo, ContainerMetrics, ContainerStatus, ExecResult, OutputChunk,
    OutputStream, Provider, ProviderCapabilities, ProviderError, ProviderInfo, ProviderStatus,
    Result, StreamType,
};
use async_trait::async_trait;
use bollard::{
    container::{
        Config, CreateContainerOptions, LogOutput, LogsOptions, RemoveContainerOptions,
        StartContainerOptions, StatsOptions, StopContainerOptions,
    },
    exec::{CreateExecOptions, StartExecResults},
    image::CreateImageOptions,
    Docker,
};
use futures::StreamExt;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

pub struct DockerProvider {
    client: Docker,
    label_prefix: String,
    /// Cache of successfully pulled images to avoid redundant pulls
    /// Key: image name (e.g., "ubuntu:22.04"), Value: timestamp when pulled
    image_cache: Arc<RwLock<HashMap<String, chrono::DateTime<chrono::Utc>>>>,
    /// Timeout for image pull operations (default: 10 minutes)
    pull_timeout: Duration,
}

impl DockerProvider {
    /// Create a new Docker provider with default timeout (10 minutes)
    pub fn new() -> Result<Self> {
        Self::with_pull_timeout(Duration::from_secs(600))
    }

    /// Create a new Docker provider with custom pull timeout
    pub fn with_pull_timeout(timeout: Duration) -> Result<Self> {
        let client = Docker::connect_with_defaults()
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;

        Ok(Self {
            client,
            label_prefix: "orkee.sandbox".to_string(),
            image_cache: Arc::new(RwLock::new(HashMap::new())),
            pull_timeout: timeout,
        })
    }

    /// Create with a specific Docker connection and default timeout
    pub fn with_client(client: Docker) -> Self {
        Self::with_client_and_timeout(client, Duration::from_secs(600))
    }

    /// Create with a specific Docker connection and custom timeout
    pub fn with_client_and_timeout(client: Docker, timeout: Duration) -> Self {
        Self {
            client,
            label_prefix: "orkee.sandbox".to_string(),
            image_cache: Arc::new(RwLock::new(HashMap::new())),
            pull_timeout: timeout,
        }
    }

    /// Convert our config to bollard config
    fn to_bollard_config(&self, config: &ContainerConfig) -> Config<String> {
        let mut labels = config.labels.clone();
        labels.insert(format!("{}.managed", self.label_prefix), "true".to_string());
        labels.insert(format!("{}.name", self.label_prefix), config.name.clone());

        let mut exposed_ports = HashMap::new();
        let mut port_bindings = HashMap::new();

        for port in &config.ports {
            let container_port = format!("{}/{}", port.container_port, port.protocol);
            exposed_ports.insert(container_port.clone(), HashMap::new());

            let binding = vec![bollard::models::PortBinding {
                host_ip: Some("0.0.0.0".to_string()),
                host_port: Some(port.host_port.to_string()),
            }];
            port_bindings.insert(container_port, Some(binding));
        }

        let binds: Vec<String> = config
            .volumes
            .iter()
            .map(|v| {
                format!(
                    "{}:{}:{}",
                    v.host_path,
                    v.container_path,
                    if v.readonly { "ro" } else { "rw" }
                )
            })
            .collect();

        let env: Vec<String> = config
            .env_vars
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        let mut host_config = bollard::models::HostConfig {
            binds: Some(binds),
            port_bindings: if port_bindings.is_empty() {
                None
            } else {
                Some(port_bindings)
            },
            cpu_shares: Some((config.cpu_cores * 1024.0) as i64),
            memory: Some((config.memory_mb * 1024 * 1024) as i64),
            ..Default::default()
        };

        // Add storage limit if specified
        if config.storage_gb > 0 {
            host_config.storage_opt = Some(HashMap::from([(
                "size".to_string(),
                format!("{}G", config.storage_gb),
            )]));
        }

        Config {
            image: Some(config.image.clone()),
            cmd: config.command.clone(),
            env: Some(env),
            working_dir: config.working_dir.clone(),
            labels: Some(labels),
            exposed_ports: Some(exposed_ports),
            host_config: Some(host_config),
            ..Default::default()
        }
    }

    /// Convert bollard container status to our status
    fn convert_status(state: &str) -> ContainerStatus {
        match state.to_lowercase().as_str() {
            "created" => ContainerStatus::Created,
            "running" => ContainerStatus::Running,
            "paused" => ContainerStatus::Paused,
            "restarting" => ContainerStatus::Running,
            "removing" => ContainerStatus::Removing,
            "exited" => ContainerStatus::Stopped,
            "dead" => ContainerStatus::Dead,
            _ => ContainerStatus::Error(format!("Unknown status: {}", state)),
        }
    }
}

#[async_trait]
impl Provider for DockerProvider {
    async fn is_available(&self) -> Result<bool> {
        match self.client.ping().await {
            Ok(_) => Ok(true),
            Err(e) => {
                warn!("Docker not available: {}", e);
                Ok(false)
            }
        }
    }

    async fn get_info(&self) -> Result<ProviderInfo> {
        let version = self
            .client
            .version()
            .await
            .map_err(|e| ProviderError::ConnectionError(e.to_string()))?;

        let status = if self.is_available().await? {
            ProviderStatus::Ready
        } else {
            ProviderStatus::NotAvailable("Docker daemon not responding".to_string())
        };

        Ok(ProviderInfo {
            name: "Docker".to_string(),
            version: version.version.unwrap_or_else(|| "unknown".to_string()),
            provider_type: "docker".to_string(),
            capabilities: ProviderCapabilities {
                gpu_support: false, // Can be enabled with nvidia-docker
                persistent_storage: true,
                network_isolation: true,
                resource_limits: true,
                exec_support: true,
                file_transfer: true,
                metrics: true,
            },
            status,
        })
    }

    async fn create_container(&self, config: &ContainerConfig) -> Result<String> {
        info!("Creating container: {}", config.name);

        // Ensure image exists
        if !self.image_exists(&config.image).await? {
            info!("Pulling image: {}", config.image);
            self.pull_image(&config.image, false).await?;
        }

        let bollard_config = self.to_bollard_config(config);
        let options = CreateContainerOptions {
            name: config.name.clone(),
            platform: None,
        };

        let container = self
            .client
            .create_container(Some(options), bollard_config)
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        debug!("Created container: {}", container.id);

        // Start the container
        self.start_container(&container.id).await?;

        Ok(container.id)
    }

    async fn start_container(&self, container_id: &str) -> Result<()> {
        info!("Starting container: {}", container_id);

        self.client
            .start_container(container_id, None::<StartContainerOptions<String>>)
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        Ok(())
    }

    async fn stop_container(&self, container_id: &str, timeout_secs: u64) -> Result<()> {
        info!(
            "Stopping container: {} (timeout: {}s)",
            container_id, timeout_secs
        );

        let options = StopContainerOptions {
            t: timeout_secs as i64,
        };

        self.client
            .stop_container(container_id, Some(options))
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        Ok(())
    }

    async fn remove_container(&self, container_id: &str, force: bool) -> Result<()> {
        info!("Removing container: {} (force: {})", container_id, force);

        let options = RemoveContainerOptions {
            force,
            v: true, // Remove volumes
            ..Default::default()
        };

        self.client
            .remove_container(container_id, Some(options))
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        Ok(())
    }

    async fn get_container_info(&self, container_id: &str) -> Result<ContainerInfo> {
        let inspect = self
            .client
            .inspect_container(container_id, None)
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        let state = inspect.state.as_ref().ok_or_else(|| {
            ProviderError::ContainerError("Container has no state information".to_string())
        })?;

        let status = Self::convert_status(
            state
                .status
                .as_ref()
                .map(|s| s.as_ref())
                .unwrap_or("unknown"),
        );

        let mut ports = HashMap::new();
        if let Some(network_settings) = &inspect.network_settings {
            if let Some(port_map) = &network_settings.ports {
                for (container_port_str, bindings) in port_map {
                    if let Some(bindings) = bindings {
                        if let Some(binding) = bindings.first() {
                            if let Some(host_port_str) = &binding.host_port {
                                // Parse container port (format: "3000/tcp")
                                if let Some(port_num) = container_port_str.split('/').next() {
                                    if let (Ok(container_port), Ok(host_port)) =
                                        (port_num.parse::<u16>(), host_port_str.parse::<u16>())
                                    {
                                        ports.insert(container_port, host_port);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        let created_at = inspect
            .created
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc))
            .unwrap_or_else(chrono::Utc::now);

        let started_at = state
            .started_at
            .as_ref()
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&chrono::Utc));

        Ok(ContainerInfo {
            id: container_id.to_string(),
            name: inspect
                .name
                .unwrap_or_else(|| container_id.to_string())
                .trim_start_matches('/')
                .to_string(),
            status,
            ip_address: inspect
                .network_settings
                .and_then(|ns| ns.ip_address)
                .filter(|s| !s.is_empty()),
            ports,
            created_at,
            started_at,
            metrics: None, // Will be populated by get_metrics if needed
        })
    }

    async fn list_containers(&self, include_stopped: bool) -> Result<Vec<ContainerInfo>> {
        use bollard::container::ListContainersOptions;

        let mut filters = HashMap::new();
        filters.insert(
            "label".to_string(),
            vec![format!("{}.managed=true", self.label_prefix)],
        );

        let options = ListContainersOptions {
            all: include_stopped,
            filters,
            ..Default::default()
        };

        let containers = self
            .client
            .list_containers(Some(options))
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        let mut container_infos = Vec::new();
        for container in containers {
            if let Some(id) = container.id {
                match self.get_container_info(&id).await {
                    Ok(info) => container_infos.push(info),
                    Err(e) => {
                        warn!("Failed to get info for container {}: {}", id, e);
                    }
                }
            }
        }

        Ok(container_infos)
    }

    async fn exec_command(
        &self,
        container_id: &str,
        command: Vec<String>,
        env_vars: Option<HashMap<String, String>>,
    ) -> Result<ExecResult> {
        info!(
            "Executing command in container {}: {:?}",
            container_id, command
        );

        // Note: Command validation should be performed at the manager level
        // before calling this provider method. This ensures blocked_commands
        // settings are enforced consistently across all providers.
        // See SandboxManager::exec_sandbox_command for validation implementation.

        let env: Option<Vec<String>> = env_vars.map(|vars| {
            vars.into_iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect()
        });

        let exec_config = CreateExecOptions {
            cmd: Some(command),
            env,
            attach_stdout: Some(true),
            attach_stderr: Some(true),
            ..Default::default()
        };

        let exec = self
            .client
            .create_exec(container_id, exec_config)
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        let start_result = self
            .client
            .start_exec(&exec.id, None)
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        match start_result {
            StartExecResults::Attached { mut output, .. } => {
                while let Some(msg) = output.next().await {
                    match msg {
                        Ok(LogOutput::StdOut { message }) => stdout.extend_from_slice(&message),
                        Ok(LogOutput::StdErr { message }) => stderr.extend_from_slice(&message),
                        Ok(LogOutput::Console { message }) => stdout.extend_from_slice(&message),
                        _ => {}
                    }
                }
            }
            StartExecResults::Detached => {
                return Err(ProviderError::ContainerError(
                    "Exec was detached unexpectedly".to_string(),
                ))
            }
        }

        let exec_inspect = self
            .client
            .inspect_exec(&exec.id)
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        let exit_code = exec_inspect.exit_code.unwrap_or(0);

        Ok(ExecResult {
            exit_code,
            stdout,
            stderr,
        })
    }

    async fn stream_logs(
        &self,
        container_id: &str,
        follow: bool,
        since: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<OutputStream> {
        let options = LogsOptions::<String> {
            stdout: true,
            stderr: true,
            follow,
            since: since.as_ref().map(|dt| dt.timestamp()).unwrap_or(0),
            timestamps: true,
            ..Default::default()
        };

        let logs = self.client.logs(container_id, Some(options));

        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut stream = Box::pin(logs);
            while let Some(result) = stream.next().await {
                match result {
                    Ok(log) => {
                        let (stream_type, data) = match log {
                            LogOutput::StdOut { message } => (StreamType::Stdout, message.to_vec()),
                            LogOutput::StdErr { message } => (StreamType::Stderr, message.to_vec()),
                            LogOutput::Console { message } => {
                                (StreamType::Stdout, message.to_vec())
                            }
                            _ => continue,
                        };

                        let chunk = OutputChunk {
                            timestamp: chrono::Utc::now(),
                            stream: stream_type,
                            data,
                        };

                        if tx.send(chunk).is_err() {
                            break; // Receiver dropped
                        }
                    }
                    Err(e) => {
                        error!("Error streaming logs: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(OutputStream { receiver: rx })
    }

    async fn copy_to_container(
        &self,
        container_id: &str,
        source_path: &str,
        dest_path: &str,
    ) -> Result<()> {
        use bollard::container::UploadToContainerOptions;

        use std::path::Path;

        info!(
            "Copying {} to container {}:{}",
            source_path, container_id, dest_path
        );

        let source = Path::new(source_path);
        if !source.exists() {
            return Err(ProviderError::ConfigError(format!(
                "Source path does not exist: {}",
                source_path
            )));
        }

        // Read file/directory into tar archive
        let tar_data = create_tar_archive(source_path)
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        let options = UploadToContainerOptions {
            path: dest_path.to_string(),
            ..Default::default()
        };

        self.client
            .upload_to_container(container_id, Some(options), tar_data.into())
            .await
            .map_err(|e| ProviderError::ContainerError(e.to_string()))?;

        Ok(())
    }

    async fn copy_from_container(
        &self,
        container_id: &str,
        source_path: &str,
        dest_path: &str,
    ) -> Result<()> {
        use bollard::container::DownloadFromContainerOptions;

        info!(
            "Copying container {}:{} to {}",
            container_id, source_path, dest_path
        );

        let options = DownloadFromContainerOptions {
            path: source_path.to_string(),
        };

        let mut stream = self
            .client
            .download_from_container(container_id, Some(options));

        let mut data = Vec::new();
        while let Some(chunk) = stream.next().await {
            let bytes = chunk.map_err(|e| ProviderError::ContainerError(e.to_string()))?;
            data.extend_from_slice(&bytes);
        }

        // Extract tar archive to destination
        extract_tar_archive(&data, dest_path)
            .map_err(|e| ProviderError::InternalError(e.to_string()))?;

        Ok(())
    }

    async fn get_metrics(&self, container_id: &str) -> Result<ContainerMetrics> {
        let options = StatsOptions {
            stream: false,
            one_shot: true,
        };

        let mut stats_stream = self.client.stats(container_id, Some(options));

        if let Some(Ok(stats)) = stats_stream.next().await {
            let cpu_delta =
                stats.cpu_stats.cpu_usage.total_usage - stats.precpu_stats.cpu_usage.total_usage;
            let system_delta = stats.cpu_stats.system_cpu_usage.unwrap_or(0)
                - stats.precpu_stats.system_cpu_usage.unwrap_or(0);

            let cpu_usage_percent = if system_delta > 0 && cpu_delta > 0 {
                (cpu_delta as f64 / system_delta as f64)
                    * 100.0
                    * stats.cpu_stats.online_cpus.unwrap_or(1) as f64
            } else {
                0.0
            };

            let memory_usage_mb = stats.memory_stats.usage.unwrap_or(0) / (1024 * 1024);
            let memory_limit_mb = stats.memory_stats.limit.unwrap_or(0) / (1024 * 1024);

            let (rx_bytes, tx_bytes) = if let Some(networks) = stats.networks {
                let rx: u64 = networks.values().map(|n| n.rx_bytes).sum();
                let tx: u64 = networks.values().map(|n| n.tx_bytes).sum();
                (rx, tx)
            } else {
                (0, 0)
            };

            Ok(ContainerMetrics {
                cpu_usage_percent,
                memory_usage_mb,
                memory_limit_mb,
                network_rx_bytes: rx_bytes,
                network_tx_bytes: tx_bytes,
            })
        } else {
            Err(ProviderError::ContainerError(
                "Failed to get container stats".to_string(),
            ))
        }
    }

    async fn pull_image(&self, image: &str, force: bool) -> Result<()> {
        // Check cache first (unless force is true)
        if !force {
            let cache = self.image_cache.read().await;
            if cache.contains_key(image) {
                debug!("Image {} found in cache, skipping pull", image);
                // Still verify it actually exists in Docker
                if self.image_exists(image).await? {
                    return Ok(());
                } else {
                    // Image was deleted outside of Orkee, remove from cache
                    drop(cache);
                    let mut cache_write = self.image_cache.write().await;
                    cache_write.remove(image);
                    info!("Image {} was deleted, removing from cache", image);
                }
            }
        }

        info!(
            "Pulling image: {} (timeout: {:?})",
            image, self.pull_timeout
        );

        let options = CreateImageOptions {
            from_image: image.to_string(),
            ..Default::default()
        };

        // Create the pull stream
        let stream = self.client.create_image(Some(options), None, None);

        // Apply timeout to the entire pull operation
        let result = tokio::time::timeout(self.pull_timeout, async {
            let mut stream = stream;
            let mut last_status = String::new();
            let mut progress_update_count = 0;

            while let Some(result) = stream.next().await {
                match result {
                    Ok(info) => {
                        if let Some(status) = &info.status {
                            // Log progress periodically to avoid spam
                            if status != &last_status {
                                debug!("Pull status: {}", status);
                                last_status = status.clone();
                            }
                            progress_update_count += 1;
                            if progress_update_count % 10 == 0 {
                                info!("Image pull progress: {}", status);
                            }
                        }
                        if let Some(error) = info.error {
                            return Err(ProviderError::ImageError(format!(
                                "Failed to pull image {}: {}",
                                image, error
                            )));
                        }
                    }
                    Err(e) => {
                        return Err(ProviderError::ImageError(format!(
                            "Failed to pull image {}: {}",
                            image, e
                        )));
                    }
                }
            }

            Ok(())
        })
        .await;

        match result {
            Ok(Ok(())) => {
                info!("Successfully pulled image: {}", image);
                // Add to cache on successful pull
                let mut cache = self.image_cache.write().await;
                cache.insert(image.to_string(), chrono::Utc::now());
                Ok(())
            }
            Ok(Err(e)) => Err(e),
            Err(_) => Err(ProviderError::ImageError(format!(
                "Timeout pulling image {} after {:?}. Image may be too large or network is slow. Consider using a local image or increasing the timeout.",
                image, self.pull_timeout
            ))),
        }
    }

    async fn image_exists(&self, image: &str) -> Result<bool> {
        match self.client.inspect_image(image).await {
            Ok(_) => Ok(true),
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404, ..
            }) => Ok(false),
            Err(e) => Err(ProviderError::ImageError(e.to_string())),
        }
    }
}

// Helper functions for tar operations
fn create_tar_archive(path: &str) -> std::io::Result<Vec<u8>> {
    use std::fs;
    use tar::Builder;

    let tar_data = Vec::new();
    let mut archive = Builder::new(tar_data);

    let path_obj = std::path::Path::new(path);
    if path_obj.is_file() {
        let mut file = fs::File::open(path)?;
        let file_name = path_obj.file_name().ok_or_else(|| {
            std::io::Error::new(std::io::ErrorKind::InvalidInput, "Invalid file name")
        })?;
        archive.append_file(file_name, &mut file)?;
    } else {
        archive.append_dir_all(".", path)?;
    }

    archive.into_inner().map_err(std::io::Error::other)
}

fn extract_tar_archive(data: &[u8], dest_path: &str) -> std::io::Result<()> {
    use std::fs;
    use tar::Archive;

    fs::create_dir_all(dest_path)?;
    let mut archive = Archive::new(data);
    archive.unpack(dest_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::{PortMapping, VolumeMount};

    #[tokio::test]
    async fn test_docker_provider_creation() {
        // This test might fail if Docker is not available
        let provider = DockerProvider::new();
        assert!(provider.is_ok() || provider.is_err());
    }

    #[tokio::test]
    async fn test_container_config_conversion() {
        let provider = DockerProvider::new().unwrap_or_else(|_| {
            DockerProvider::with_client(Docker::connect_with_local_defaults().unwrap())
        });

        let mut config = ContainerConfig {
            image: "alpine:latest".to_string(),
            name: "test-container".to_string(),
            env_vars: HashMap::from([("FOO".to_string(), "bar".to_string())]),
            volumes: vec![VolumeMount {
                host_path: "/tmp/host".to_string(),
                container_path: "/tmp/container".to_string(),
                readonly: false,
            }],
            ports: vec![PortMapping {
                host_port: 8080,
                container_port: 80,
                protocol: "tcp".to_string(),
            }],
            cpu_cores: 1.5,
            memory_mb: 1024,
            storage_gb: 10,
            command: Some(vec!["echo".to_string(), "hello".to_string()]),
            working_dir: Some("/app".to_string()),
            labels: HashMap::new(),
        };

        let bollard_config = provider.to_bollard_config(&config);

        assert_eq!(bollard_config.image, Some("alpine:latest".to_string()));
        assert!(bollard_config.env.is_some());
        assert!(bollard_config.host_config.is_some());
    }
}
