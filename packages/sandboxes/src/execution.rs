// ABOUTME: Execution orchestration for containerized agent runs
// ABOUTME: Manages complete execution lifecycle from container creation to cleanup

use std::collections::HashMap;
use std::sync::Arc;

use chrono::Utc;
use tracing::{error, info, warn};

use crate::container::ContainerManager;
use crate::error::{Result, SandboxError};
use crate::node_bridge::NodeBridge;
use crate::storage::ExecutionStorage;
use crate::types::{
    Artifact, ContainerStatus, ExecutionEvent, ExecutionRequest, ExecutionResponse,
    ExecutionStatus, LogEntry, ResourceUsage,
};

/// Default capacity for SSE broadcast channel
/// Can be overridden via ORKEE_EXECUTION_EVENT_CHANNEL_SIZE environment variable
const DEFAULT_EVENT_CHANNEL_SIZE: usize = 200;

/// Orchestrates execution lifecycle from container creation to cleanup
pub struct ExecutionOrchestrator {
    container_manager: Arc<ContainerManager>,
    node_bridge: Arc<NodeBridge>,
    storage: Arc<ExecutionStorage>,
    /// Broadcast channel for real-time events
    event_tx: tokio::sync::broadcast::Sender<ExecutionEvent>,
}

impl ExecutionOrchestrator {
    /// Create a new execution orchestrator
    pub fn new(
        container_manager: Arc<ContainerManager>,
        node_bridge: Arc<NodeBridge>,
        storage: Arc<ExecutionStorage>,
    ) -> Self {
        // Read channel size from environment with validation
        let channel_size = std::env::var("ORKEE_EXECUTION_EVENT_CHANNEL_SIZE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .filter(|&v| v >= 10 && v <= 10000)
            .unwrap_or(DEFAULT_EVENT_CHANNEL_SIZE);

        let (event_tx, _) = tokio::sync::broadcast::channel(channel_size);

        Self {
            container_manager,
            node_bridge,
            storage,
            event_tx,
        }
    }

    /// Subscribe to execution events for SSE streaming
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<ExecutionEvent> {
        self.event_tx.subscribe()
    }

    /// Broadcast an event to all SSE subscribers
    fn broadcast_event(&self, event: ExecutionEvent) {
        // Log errors but don't fail - SSE is best-effort
        if let Err(e) = self.event_tx.send(event) {
            // Only log if there are no receivers (normal case when no SSE clients)
            if self.event_tx.receiver_count() > 0 {
                warn!("Failed to broadcast execution event: {}", e);
            }
        }
    }

    /// Start a new execution
    ///
    /// Execution flow:
    /// 1. Validate request and check quotas
    /// 2. Create execution record in database (done externally)
    /// 3. Spawn Vibekit bridge process
    /// 4. Create and start container
    /// 5. Execute agent prompt
    /// 6. Stream logs to database
    /// 7. Collect artifacts
    /// 8. Update execution status
    /// 9. Cleanup resources
    pub async fn start_execution(&self, request: ExecutionRequest) -> Result<ExecutionResponse> {
        info!(
            "Starting execution {} for task {}",
            request.execution_id, request.task_id
        );

        // Validate request
        self.validate_request(&request)?;

        // Create container
        let container_id = match self
            .create_container(&request)
            .await
        {
            Ok(id) => {
                self.storage
                    .update_container_status(
                        &request.execution_id,
                        &id,
                        &ContainerStatus::Creating.to_string(),
                    )
                    .await?;
                id
            }
            Err(e) => {
                error!("Failed to create container: {}", e);
                self.storage
                    .update_execution_status(
                        &request.execution_id,
                        &ExecutionStatus::Failed.to_string(),
                        Some(&e.to_string()),
                    )
                    .await?;
                return Err(e);
            }
        };

        // Start container
        if let Err(e) = self.container_manager.start_container(&container_id).await {
            error!("Failed to start container {}: {}", container_id, e);
            self.storage
                .update_container_status(
                    &request.execution_id,
                    &container_id,
                    &ContainerStatus::Error.to_string(),
                )
                .await?;
            self.storage
                .update_execution_status(
                    &request.execution_id,
                    &ExecutionStatus::Failed.to_string(),
                    Some(&e.to_string()),
                )
                .await?;
            return Err(e);
        }

        // Update to running status
        self.storage
            .update_container_status(
                &request.execution_id,
                &container_id,
                &ContainerStatus::Running.to_string(),
            )
            .await?;

        self.storage
            .update_execution_status(&request.execution_id, &ExecutionStatus::Running.to_string(), None)
            .await?;

        // Broadcast status events
        self.broadcast_event(ExecutionEvent::ContainerStatus {
            execution_id: request.execution_id.clone(),
            container_id: container_id.clone(),
            status: ContainerStatus::Running,
        });

        self.broadcast_event(ExecutionEvent::Status {
            execution_id: request.execution_id.clone(),
            status: ExecutionStatus::Running,
            error_message: None,
        });

        info!(
            "Container {} started successfully for execution {}",
            container_id, request.execution_id
        );

        // Return initial response
        // The actual execution will continue in a background task
        Ok(ExecutionResponse {
            execution_id: request.execution_id.clone(),
            container_id,
            status: ExecutionStatus::Running,
            container_status: ContainerStatus::Running,
            vibekit_session_id: None,
            error_message: None,
        })
    }

    /// Stop a running execution
    pub async fn stop_execution(&self, execution_id: &str, container_id: &str) -> Result<()> {
        info!("Stopping execution {}", execution_id);

        // Stop the container
        self.container_manager
            .stop_container(container_id, Some(10))
            .await?;

        // Update status
        self.storage
            .update_container_status(
                execution_id,
                container_id,
                &ContainerStatus::Stopped.to_string(),
            )
            .await?;

        self.storage
            .update_execution_status(
                execution_id,
                &ExecutionStatus::Cancelled.to_string(),
                Some("Stopped by user"),
            )
            .await?;

        // Broadcast status events
        self.broadcast_event(ExecutionEvent::ContainerStatus {
            execution_id: execution_id.to_string(),
            container_id: container_id.to_string(),
            status: ContainerStatus::Stopped,
        });

        self.broadcast_event(ExecutionEvent::Status {
            execution_id: execution_id.to_string(),
            status: ExecutionStatus::Cancelled,
            error_message: Some("Stopped by user".to_string()),
        });

        info!("Execution {} stopped successfully", execution_id);
        Ok(())
    }

    /// Monitor execution progress and resource usage
    pub async fn monitor_execution(
        &self,
        execution_id: &str,
        container_id: &str,
    ) -> Result<ResourceUsage> {
        // Get container stats
        let stats = self.container_manager.get_container_stats(container_id).await?;

        // Update resource usage in database
        self.storage
            .update_resource_usage(execution_id, stats.memory_used_mb, stats.cpu_usage_percent)
            .await?;

        Ok(stats)
    }

    /// Finalize execution - cleanup and save results
    pub async fn finalize_execution(
        &self,
        execution_id: &str,
        container_id: &str,
        status: ExecutionStatus,
        error_message: Option<&str>,
    ) -> Result<()> {
        info!("Finalizing execution {}", execution_id);

        // Update final status
        self.storage
            .update_execution_status(execution_id, &status.to_string(), error_message)
            .await?;

        // Clean up container
        if let Err(e) = self.container_manager.remove_container(container_id, false).await {
            warn!(
                "Failed to remove container {} for execution {}: {}",
                container_id, execution_id, e
            );
        } else {
            info!(
                "Container {} removed for execution {}",
                container_id, execution_id
            );
        }

        Ok(())
    }

    /// Collect artifacts from container workspace
    pub async fn collect_artifacts(
        &self,
        execution_id: &str,
        _container_id: &str,
        output_paths: Vec<String>,
    ) -> Result<Vec<Artifact>> {
        info!("Collecting artifacts for execution {}", execution_id);

        let mut artifacts = Vec::new();

        for path in output_paths {
            // For Phase 4, we'll just create placeholder artifact records
            // Full artifact collection will be implemented in Phase 5
            let artifact = Artifact {
                id: nanoid::nanoid!(12),
                execution_id: execution_id.to_string(),
                artifact_type: "file".to_string(),
                file_path: path.clone(),
                file_name: path.split('/').last().unwrap_or(&path).to_string(),
                file_size_bytes: None,
                mime_type: None,
                stored_path: None,
                storage_backend: "local".to_string(),
                description: None,
                metadata: None,
                checksum: None,
                created_at: Utc::now().to_rfc3339(),
            };

            // Save artifact to database
            self.storage.create_artifact(artifact.clone()).await?;
            artifacts.push(artifact);
        }

        info!(
            "Collected {} artifacts for execution {}",
            artifacts.len(),
            execution_id
        );
        Ok(artifacts)
    }

    /// Stream logs from container to database
    ///
    /// Note: This is a simplified implementation for Phase 4.
    /// Full log streaming will be implemented in Phase 5.
    pub async fn stream_logs(
        &self,
        execution_id: &str,
        container_id: &str,
    ) -> Result<Vec<LogEntry>> {
        info!("Streaming logs for execution {}", execution_id);

        use futures_util::StreamExt;

        // Stream logs from container (ContainerManager returns a Stream<Item = Result<LogEntry>>)
        let log_stream = self
            .container_manager
            .stream_container_logs(container_id.to_string(), execution_id.to_string(), 0)
            .await;

        // Box the stream to make it Unpin
        let mut log_stream = Box::pin(log_stream);
        let mut logs = Vec::new();

        // Collect logs from stream
        while let Some(log_result) = log_stream.next().await {
            let log_entry = log_result?;

            // Insert log into database
            self.storage.insert_log(log_entry.clone()).await?;

            // Broadcast log event to SSE subscribers
            self.broadcast_event(ExecutionEvent::Log {
                log: log_entry.clone(),
            });

            logs.push(log_entry);
        }

        info!(
            "Streamed {} log entries for execution {}",
            logs.len(),
            execution_id
        );
        Ok(logs)
    }

    // ==================== Private Helper Methods ====================

    /// Validate execution request
    fn validate_request(&self, request: &ExecutionRequest) -> Result<()> {
        if request.execution_id.is_empty() {
            return Err(SandboxError::InvalidRequest(
                "execution_id cannot be empty".to_string(),
            ));
        }

        if request.task_id.is_empty() {
            return Err(SandboxError::InvalidRequest(
                "task_id cannot be empty".to_string(),
            ));
        }

        if request.container_image.is_empty() {
            return Err(SandboxError::InvalidRequest(
                "container_image cannot be empty".to_string(),
            ));
        }

        if request.resource_limits.memory_mb == 0 {
            return Err(SandboxError::InvalidRequest(
                "memory_mb must be greater than 0".to_string(),
            ));
        }

        if request.resource_limits.cpu_cores <= 0.0 {
            return Err(SandboxError::InvalidRequest(
                "cpu_cores must be greater than 0".to_string(),
            ));
        }

        Ok(())
    }

    /// Create a container for the execution
    async fn create_container(&self, request: &ExecutionRequest) -> Result<String> {
        // Create container with resource limits
        let container_id = self
            .container_manager
            .create_container(
                &request.execution_id,
                Some(&request.task_id),
                &request.container_image,
                &request.resource_limits,
                request.workspace_path.as_deref(),
                request.environment_variables.clone(),
            )
            .await?;

        Ok(container_id)
    }
}

impl ExecutionStatus {
    /// Convert enum to string representation
    fn to_string(&self) -> String {
        match self {
            ExecutionStatus::Pending => "pending".to_string(),
            ExecutionStatus::Running => "running".to_string(),
            ExecutionStatus::Completed => "completed".to_string(),
            ExecutionStatus::Failed => "failed".to_string(),
            ExecutionStatus::Cancelled => "cancelled".to_string(),
        }
    }
}

impl ContainerStatus {
    /// Convert enum to string representation
    fn to_string(&self) -> String {
        match self {
            ContainerStatus::Creating => "creating".to_string(),
            ContainerStatus::Running => "running".to_string(),
            ContainerStatus::Stopped => "stopped".to_string(),
            ContainerStatus::Error => "error".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request() {
        // Tests would go here
        // For now, placeholder to demonstrate structure
    }
}
