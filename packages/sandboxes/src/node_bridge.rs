// ABOUTME: Node.js child process bridge for Vibekit SDK integration
// ABOUTME: Manages IPC communication with TypeScript bridge via stdin/stdout

use crate::{error::SandboxError, types::*, Result};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufRead, BufReader, Write},
    path::PathBuf,
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// IPC request types sent to Node.js bridge
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub(crate) enum IPCRequest {
    Ping,
    Execute { request: ExecutionRequest },
    Stop { execution_id: String },
}

/// IPC response types received from Node.js bridge
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)] // Fields will be used in Phase 3+ for execution orchestration
pub enum IPCResponse {
    Pong,
    ExecutionStarted { response: ExecutionResponse },
    Log { log: LogEntry },
    Artifact { artifact: Artifact },
    ResourceUpdate { execution_id: String, usage: ResourceUsage },
    ExecutionComplete { execution_id: String, status: ExecutionStatus, error_message: Option<String> },
    Error { execution_id: Option<String>, error: String, details: Option<String> },
}

/// Node.js bridge process manager
pub struct NodeBridge {
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<ChildStdin>>>,
    bridge_path: PathBuf,
    response_tx: mpsc::UnboundedSender<IPCResponse>,
    response_rx: Arc<Mutex<mpsc::UnboundedReceiver<IPCResponse>>>,
}

impl NodeBridge {
    /// Create a new Node.js bridge instance
    pub fn new() -> Result<Self> {
        // Determine bridge script path
        let bridge_path = Self::find_bridge_path()?;

        // Create channel for IPC responses
        let (response_tx, response_rx) = mpsc::unbounded_channel();

        Ok(Self {
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            bridge_path,
            response_tx,
            response_rx: Arc::new(Mutex::new(response_rx)),
        })
    }

    /// Find the path to the Vibekit bridge script
    fn find_bridge_path() -> Result<PathBuf> {
        // Try current directory structure first
        let candidates = vec![
            PathBuf::from("packages/sandboxes/vibekit/src/index.ts"),
            PathBuf::from("../sandboxes/vibekit/src/index.ts"),
            PathBuf::from("./vibekit/src/index.ts"),
        ];

        for path in candidates {
            if path.exists() {
                info!("Found Vibekit bridge at: {}", path.display());
                return Ok(path);
            }
        }

        Err(SandboxError::BridgeNotFound(
            "Could not find Vibekit bridge script. Tried: packages/sandboxes/vibekit/src/index.ts".to_string(),
        ))
    }

    /// Start the Node.js bridge process
    pub async fn start(&self) -> Result<()> {
        info!("Starting Node.js Vibekit bridge...");

        // Check if already running
        if self.is_running() {
            warn!("Bridge process already running, restarting...");
            self.stop().await?;
        }

        // Spawn Node.js process with tsx
        let mut child = Command::new("npx")
            .arg("tsx")
            .arg(&self.bridge_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| SandboxError::BridgeStartFailed(e.to_string()))?;

        // Extract stdin and stdout handles
        let stdin = child.stdin.take().ok_or_else(|| {
            SandboxError::BridgeStartFailed("Failed to capture stdin".to_string())
        })?;

        let stdout = child.stdout.take().ok_or_else(|| {
            SandboxError::BridgeStartFailed("Failed to capture stdout".to_string())
        })?;

        // Store stdin handle
        *self.stdin.lock().unwrap() = Some(stdin);

        // Start stdout reader task
        let response_tx = self.response_tx.clone();
        tokio::task::spawn_blocking(move || {
            Self::read_responses(stdout, response_tx);
        });

        // Store process handle
        *self.process.lock().unwrap() = Some(child);

        // Wait for initial pong to confirm bridge is ready
        self.wait_for_ready().await?;

        info!("Node.js Vibekit bridge started successfully");
        Ok(())
    }

    /// Wait for the bridge to be ready (receives pong)
    async fn wait_for_ready(&self) -> Result<()> {
        debug!("Waiting for bridge to be ready...");

        let timeout = Duration::from_secs(10);
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            // Check for responses
            if let Ok(mut rx) = self.response_rx.try_lock() {
                if let Ok(response) = rx.try_recv() {
                    match response {
                        IPCResponse::Pong => {
                            debug!("Bridge is ready");
                            return Ok(());
                        }
                        IPCResponse::Error { error, details, .. } => {
                            return Err(SandboxError::BridgeStartFailed(format!(
                                "{}: {}",
                                error,
                                details.unwrap_or_default()
                            )));
                        }
                        _ => {
                            warn!("Unexpected response while waiting for ready: {:?}", response);
                        }
                    }
                }
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        Err(SandboxError::BridgeStartFailed(
            "Timeout waiting for bridge to become ready".to_string(),
        ))
    }

    /// Stop the Node.js bridge process
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Node.js Vibekit bridge...");

        let mut process = self.process.lock().unwrap();
        if let Some(mut child) = process.take() {
            // Try graceful shutdown first
            if let Err(e) = child.kill() {
                warn!("Failed to kill bridge process: {}", e);
            }

            // Wait for process to exit
            match child.wait() {
                Ok(status) => {
                    info!("Bridge process exited with status: {}", status);
                }
                Err(e) => {
                    warn!("Failed to wait for bridge process: {}", e);
                }
            }
        }

        // Clear stdin handle
        *self.stdin.lock().unwrap() = None;

        Ok(())
    }

    /// Restart the bridge process
    pub async fn restart(&self) -> Result<()> {
        info!("Restarting Node.js Vibekit bridge...");
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Check if the bridge process is running
    pub fn is_running(&self) -> bool {
        let process = self.process.lock().unwrap();
        process.is_some()
    }

    /// Send an execution request to the bridge
    pub async fn execute(&self, request: ExecutionRequest) -> Result<()> {
        debug!("Sending execution request: {}", request.execution_id);

        let message = IPCRequest::Execute { request };
        self.send_request(message).await
    }

    /// Send a stop request to the bridge
    pub async fn stop_execution(&self, execution_id: String) -> Result<()> {
        debug!("Sending stop request for execution: {}", execution_id);

        let message = IPCRequest::Stop { execution_id };
        self.send_request(message).await
    }

    /// Send a ping to check if bridge is alive
    pub async fn ping(&self) -> Result<()> {
        debug!("Pinging bridge...");

        let message = IPCRequest::Ping;
        self.send_request(message).await
    }

    /// Receive the next response from the bridge
    pub async fn receive_response(&self) -> Option<IPCResponse> {
        let mut rx = self.response_rx.lock().unwrap();
        rx.recv().await
    }

    /// Send a request to the bridge via stdin
    async fn send_request(&self, request: IPCRequest) -> Result<()> {
        let message = serde_json::to_string(&request)
            .map_err(|e| SandboxError::SerializationError(e.to_string()))?;

        let mut stdin = self.stdin.lock().unwrap();
        if let Some(stdin) = stdin.as_mut() {
            writeln!(stdin, "{}", message)
                .map_err(|e| SandboxError::BridgeCommunicationError(e.to_string()))?;

            stdin.flush()
                .map_err(|e| SandboxError::BridgeCommunicationError(e.to_string()))?;

            Ok(())
        } else {
            Err(SandboxError::BridgeNotRunning)
        }
    }

    /// Read responses from bridge stdout (runs in blocking thread)
    fn read_responses(stdout: ChildStdout, tx: mpsc::UnboundedSender<IPCResponse>) {
        let reader = BufReader::new(stdout);

        for line in reader.lines() {
            match line {
                Ok(line) => {
                    if line.trim().is_empty() {
                        continue;
                    }

                    match serde_json::from_str::<IPCResponse>(&line) {
                        Ok(response) => {
                            if tx.send(response).is_err() {
                                error!("Failed to send response, receiver dropped");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Failed to parse bridge response: {} - Line: {}", e, line);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to read from bridge stdout: {}", e);
                    break;
                }
            }
        }

        debug!("Bridge stdout reader task ended");
    }
}

impl Drop for NodeBridge {
    fn drop(&mut self) {
        // Synchronous cleanup - just kill the process
        if let Some(mut child) = self.process.lock().unwrap().take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_path_detection() {
        let path = NodeBridge::find_bridge_path();
        assert!(path.is_ok(), "Should find bridge path");
    }

    #[test]
    fn test_ipc_request_serialization() {
        let request = IPCRequest::Ping;
        let json = serde_json::to_string(&request).unwrap();
        assert_eq!(json, r#"{"type":"ping"}"#);
    }

    #[test]
    fn test_ipc_response_deserialization() {
        let json = r#"{"type":"pong"}"#;
        let response: IPCResponse = serde_json::from_str(json).unwrap();
        assert!(matches!(response, IPCResponse::Pong));
    }
}
