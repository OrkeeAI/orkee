// ABOUTME: Command executor for running commands in sandboxes with streaming output
// ABOUTME: Handles command execution, output streaming, and execution tracking

use crate::manager::{ManagerError, SandboxManager};
use crate::providers::{ExecResult, OutputChunk, OutputStream, StreamType};
use crate::storage::{ExecutionStatus, SandboxExecution, SandboxStatus};
use chrono::Utc;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::mpsc;

#[derive(Error, Debug)]
pub enum ExecutorError {
    #[error("Manager error: {0}")]
    Manager(#[from] ManagerError),

    #[error("Sandbox not running: {0}")]
    NotRunning(String),

    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    #[error("Stream error: {0}")]
    StreamError(String),
}

pub type Result<T> = std::result::Result<T, ExecutorError>;

/// Request to execute a command in a sandbox
#[derive(Debug, Clone)]
pub struct ExecuteCommandRequest {
    pub sandbox_id: String,
    pub command: Vec<String>,
    pub working_dir: Option<String>,
    pub env_vars: HashMap<String, String>,
    pub created_by: Option<String>,
    pub agent_execution_id: Option<String>,
}

/// Result of command execution
#[derive(Debug)]
pub struct ExecutionResult {
    pub execution: SandboxExecution,
    pub exit_code: i64,
    pub stdout: String,
    pub stderr: String,
}

/// Command executor for sandboxes
pub struct CommandExecutor {
    manager: Arc<SandboxManager>,
}

impl CommandExecutor {
    pub fn new(manager: Arc<SandboxManager>) -> Self {
        Self { manager }
    }

    /// Execute a command and wait for completion
    pub async fn execute_command(
        &self,
        request: ExecuteCommandRequest,
    ) -> Result<ExecutionResult> {
        // Check sandbox status
        let sandbox = self.manager.get_sandbox(&request.sandbox_id).await?;
        if sandbox.status != SandboxStatus::Running {
            return Err(ExecutorError::NotRunning(format!(
                "Sandbox {} is in state {:?}",
                request.sandbox_id, sandbox.status
            )));
        }

        // Create execution record
        let command_str = request.command.join(" ");
        let mut execution = self
            .manager
            .create_execution(
                &request.sandbox_id,
                command_str.clone(),
                request.working_dir.clone(),
                request.created_by.clone(),
                request.agent_execution_id.clone(),
            )
            .await?;

        // Update status to running
        self.manager
            .update_execution_status(
                &execution.id,
                ExecutionStatus::Running,
                None,
                None,
                None,
            )
            .await?;

        // Get container ID
        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ExecutorError::ExecutionFailed("No container ID".to_string()))?;

        // Get provider and execute command
        let provider = self.manager.get_provider(&sandbox.provider).await?;
        let result = provider
            .exec_command(
                container_id,
                request.command,
                request.working_dir,
                request.env_vars,
            )
            .await
            .map_err(|e| ExecutorError::ExecutionFailed(e.to_string()))?;

        // Convert output to strings
        let stdout = String::from_utf8_lossy(&result.stdout).to_string();
        let stderr = String::from_utf8_lossy(&result.stderr).to_string();

        // Update execution with results
        let status = if result.exit_code == 0 {
            ExecutionStatus::Completed
        } else {
            ExecutionStatus::Failed
        };

        self.manager
            .update_execution_status(
                &execution.id,
                status.clone(),
                Some(result.exit_code as i32),
                Some(stdout.clone()),
                Some(stderr.clone()),
            )
            .await?;

        // Update execution object
        execution.status = status;
        execution.started_at = Some(Utc::now());
        execution.completed_at = Some(Utc::now());
        execution.exit_code = Some(result.exit_code as i32);
        execution.stdout = Some(stdout.clone());
        execution.stderr = Some(stderr.clone());

        Ok(ExecutionResult {
            execution,
            exit_code: result.exit_code,
            stdout,
            stderr,
        })
    }

    /// Execute a command with streaming output
    pub async fn execute_command_streaming(
        &self,
        request: ExecuteCommandRequest,
    ) -> Result<(SandboxExecution, mpsc::UnboundedReceiver<OutputChunk>)> {
        // Check sandbox status
        let sandbox = self.manager.get_sandbox(&request.sandbox_id).await?;
        if sandbox.status != SandboxStatus::Running {
            return Err(ExecutorError::NotRunning(format!(
                "Sandbox {} is in state {:?}",
                request.sandbox_id, sandbox.status
            )));
        }

        // Create execution record
        let command_str = request.command.join(" ");
        let execution = self
            .manager
            .create_execution(
                &request.sandbox_id,
                command_str.clone(),
                request.working_dir.clone(),
                request.created_by.clone(),
                request.agent_execution_id.clone(),
            )
            .await?;

        // Update status to running
        self.manager
            .update_execution_status(
                &execution.id,
                ExecutionStatus::Running,
                None,
                None,
                None,
            )
            .await?;

        // Get container ID
        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ExecutorError::ExecutionFailed("No container ID".to_string()))?
            .clone();

        // Get provider
        let provider = self.manager.get_provider(&sandbox.provider).await?;

        // Create output channel
        let (output_tx, output_rx) = mpsc::unbounded_channel();

        // Spawn task to execute command and capture output
        let execution_id = execution.id.clone();
        let manager = self.manager.clone();
        let provider_clone = provider.clone();
        let request_clone = request.clone();

        tokio::spawn(async move {
            // Execute command (this is a simplified version - in real implementation
            // we would need to stream output as it's generated)
            let result = provider_clone
                .exec_command(
                    &container_id,
                    request_clone.command,
                    request_clone.working_dir,
                    request_clone.env_vars,
                )
                .await;

            match result {
                Ok(exec_result) => {
                    // Send stdout chunks
                    if !exec_result.stdout.is_empty() {
                        let _ = output_tx.send(OutputChunk {
                            timestamp: Utc::now(),
                            stream: StreamType::Stdout,
                            data: exec_result.stdout.clone(),
                        });
                    }

                    // Send stderr chunks
                    if !exec_result.stderr.is_empty() {
                        let _ = output_tx.send(OutputChunk {
                            timestamp: Utc::now(),
                            stream: StreamType::Stderr,
                            data: exec_result.stderr.clone(),
                        });
                    }

                    // Update execution status
                    let stdout = String::from_utf8_lossy(&exec_result.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&exec_result.stderr).to_string();
                    let completed_at = Utc::now();
                    let status = if exec_result.exit_code == 0 {
                        ExecutionStatus::Completed
                    } else {
                        ExecutionStatus::Failed
                    };

                    let _ = manager
                        .update_execution_status(
                            &execution_id,
                            status,
                            Some(exec_result.exit_code as i32),
                            Some(stdout),
                            Some(stderr),
                        )
                        .await;
                }
                Err(e) => {
                    // Send error to stderr
                    let error_msg = format!("Execution failed: {}", e);
                    let _ = output_tx.send(OutputChunk {
                        timestamp: Utc::now(),
                        stream: StreamType::Stderr,
                        data: error_msg.as_bytes().to_vec(),
                    });

                    // Update execution status to failed
                    let _ = manager
                        .update_execution_status(
                            &execution_id,
                            ExecutionStatus::Failed,
                            Some(1),
                            None,
                            Some(error_msg),
                        )
                        .await;
                }
            }
        });

        Ok((execution, output_rx))
    }

    /// Stream logs from a sandbox
    pub async fn stream_logs(
        &self,
        sandbox_id: &str,
        follow: bool,
        timestamps: bool,
    ) -> Result<mpsc::UnboundedReceiver<OutputChunk>> {
        // Check sandbox exists
        let sandbox = self.manager.get_sandbox(sandbox_id).await?;

        let container_id = sandbox
            .container_id
            .as_ref()
            .ok_or_else(|| ExecutorError::ExecutionFailed("No container ID".to_string()))?
            .clone();

        // Get provider
        let provider = self.manager.get_provider(&sandbox.provider).await?;

        // Get log stream from provider
        let mut log_stream = provider
            .stream_logs(&container_id, follow, timestamps)
            .await
            .map_err(|e| ExecutorError::StreamError(e.to_string()))?;

        // Create output channel
        let (output_tx, output_rx) = mpsc::unbounded_channel();

        // Spawn task to forward log stream to output channel
        tokio::spawn(async move {
            while let Some(chunk) = log_stream.receiver.recv().await {
                if output_tx.send(chunk).is_err() {
                    break; // Receiver dropped
                }
            }
        });

        Ok(output_rx)
    }

    /// Cancel a running execution
    pub async fn cancel_execution(&self, execution_id: &str) -> Result<()> {
        // Update execution status to cancelled
        self.manager
            .update_execution_status(
                execution_id,
                ExecutionStatus::Cancelled,
                None,
                None,
                None,
            )
            .await?;

        Ok(())
    }

    /// Get execution by ID
    pub async fn get_execution(&self, execution_id: &str) -> Result<SandboxExecution> {
        Ok(self.manager.get_execution(execution_id).await?)
    }

    /// List executions for a sandbox
    pub async fn list_executions(&self, sandbox_id: &str) -> Result<Vec<SandboxExecution>> {
        Ok(self.manager.list_executions(sandbox_id).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: Full integration tests would require setting up test database
    // and mock provider. These are unit tests for the executor logic.
}
