//! Orkee Preview - Development server preview system
//! 
//! This crate provides functionality for detecting project types,
//! managing development servers, and serving static files for preview purposes.

pub mod api;
pub mod detector;
pub mod manager;
pub mod simple_manager;
pub mod static_server;
pub mod types;

// Re-export key types and functions for easier use
pub use api::create_preview_router;
pub use detector::ProjectDetector;
pub use manager::DevServerManager;
pub use simple_manager::{SimplePreviewManager, ServerInfo};
pub use static_server::{StaticServer, StaticServerConfig};
pub use types::{
    ApiResponse, DevServerConfig, DevServerInstance, DevServerLog, DevServerStatus,
    Framework, LogType, PackageManager, PreviewError, PreviewResult,
    ProjectDetectionResult, ProjectType, ServerLockData, StartServerRequest,
    StartServerResponse, ServerStatusResponse, ServerLogsRequest, ServerLogsResponse,
};

/// Initialize the preview service with a simple, crash-resistant manager
pub async fn init() -> PreviewResult<SimplePreviewManager> {
    Ok(SimplePreviewManager::new_with_recovery().await)
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_init() {
        let manager = init().unwrap();
        // Basic smoke test - just ensure we can create a manager
        assert!(manager.lock_dir.to_string_lossy().contains(".orkee"));
    }
}