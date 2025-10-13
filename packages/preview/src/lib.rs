//! Orkee Preview - Development server preview system
//!
//! This crate provides functionality for managing development servers
//! for various project types with crash-resistant operation.

pub mod env;
pub mod manager;
pub mod registry;
pub mod types;

// Re-export key types and functions for easier use
pub use manager::{PreviewManager, ServerInfo};
pub use registry::{is_process_running_validated, start_periodic_cleanup};
pub use types::{
    ApiResponse, DevServerConfig, DevServerInstance, DevServerLog, DevServerStatus, Framework,
    LogType, PackageManager, PreviewError, PreviewResult, ProjectDetectionResult, ProjectType,
    ServerLockData, ServerLogsRequest, ServerLogsResponse, ServerStatusResponse,
    StartServerRequest, StartServerResponse,
};

/// Initialize the preview service with a crash-resistant manager.
///
/// Creates a new preview manager instance that automatically recovers any
/// previously running development servers from lock files. This ensures
/// that servers started in previous sessions are properly tracked.
///
/// This function also starts a background task that periodically cleans up
/// stale server entries from the registry (runs every 5 minutes by default).
///
/// # Returns
///
/// Returns a `PreviewResult<PreviewManager>` containing the initialized manager,
/// or an error if initialization fails.
///
/// # Examples
///
/// ```no_run
/// use orkee_preview::init;
///
/// #[tokio::main]
/// async fn main() {
///     let manager = init().await.expect("Failed to initialize preview manager");
///     // Manager is now ready to start/stop development servers
///     // Periodic cleanup runs automatically in the background
/// }
/// ```
pub async fn init() -> PreviewResult<PreviewManager> {
    // Start periodic cleanup task for stale registry entries
    start_periodic_cleanup();

    Ok(PreviewManager::new_with_recovery().await)
}

/// Version information for the preview crate.
///
/// This constant contains the version string from Cargo.toml at compile time.
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
