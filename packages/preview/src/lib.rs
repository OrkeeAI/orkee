//! Orkee Preview - Development server preview system
//!
//! This crate provides functionality for managing development servers
//! for various project types with crash-resistant operation.

pub mod manager;
pub mod types;

// Re-export key types and functions for easier use
pub use manager::{PreviewManager, ServerInfo};
pub use types::{
    ApiResponse, DevServerConfig, DevServerInstance, DevServerLog, DevServerStatus, Framework,
    LogType, PackageManager, PreviewError, PreviewResult, ProjectDetectionResult, ProjectType,
    ServerLockData, ServerLogsRequest, ServerLogsResponse, ServerStatusResponse,
    StartServerRequest, StartServerResponse,
};

/// Initialize the preview service with a crash-resistant manager
pub async fn init() -> PreviewResult<PreviewManager> {
    Ok(PreviewManager::new_with_recovery().await)
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;


    #[tokio::test]
    async fn test_init() {
        let manager = init().await.unwrap();
        // Basic smoke test - just ensure we can create a manager
        // We don't assert on server count as it might recover existing servers
        let _servers = manager.list_servers().await;
        // If we get here without panic, the test passes
    }
}
