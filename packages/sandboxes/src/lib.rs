// ABOUTME: Sandbox execution orchestration for Orkee
// ABOUTME: Manages containerized agent execution with Docker and Vibekit SDK integration

pub mod container;
pub mod error;
pub mod execution;
pub mod node_bridge;
pub mod storage;
pub mod types;

// Re-export commonly used types
pub use container::{ContainerInfo, ContainerManager};
pub use error::{Result, SandboxError};
pub use execution::ExecutionOrchestrator;
pub use node_bridge::{IPCResponse, NodeBridge};
pub use storage::ExecutionStorage;
pub use types::{
    Artifact, ContainerStatus, ExecutionRequest, ExecutionResponse, ExecutionStatus, LogEntry,
    ResourceLimits, ResourceUsage, SandboxProvider,
};
