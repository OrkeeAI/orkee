//! Orkee Preview - Development server preview system
//!
//! This crate provides functionality for managing development servers
//! for various project types using SQLite-based persistence.

pub mod discovery;
pub mod manager;
pub mod registry;
pub mod storage;
pub mod types;

// Re-export key types and functions for easier use
pub use discovery::{
    discover_external_servers, load_env_from_directory, register_discovered_server,
    start_periodic_discovery, DiscoveredServer,
};
pub use manager::{PreviewManager, ServerInfo};
pub use registry::{is_process_running_validated, ServerRegistry};
pub use types::{
    ApiResponse, DevServerConfig, DevServerInstance, DevServerLog, DevServerStatus, Framework,
    LogType, PackageManager, PreviewError, PreviewResult, ProjectDetectionResult, ProjectType,
    ServerEvent, ServerLogsRequest, ServerLogsResponse, ServerSource, ServerStatusInfo,
    ServerStatusResponse, ServersResponse, StartServerRequest, StartServerResponse,
};

/// Initialize the preview service with a SQLite-based manager.
///
/// Creates a new preview manager instance that automatically recovers any
/// previously running development servers from the database registry.
///
/// This function also starts background tasks:
/// - External server discovery: Runs every 30 seconds to find manually launched servers
///
/// # Arguments
///
/// * `storage` - Reference to the shared SqliteStorage instance for database access
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
/// use orkee_storage::sqlite::SqliteStorage;
///
/// #[tokio::main]
/// async fn main() {
///     let storage = SqliteStorage::init(None).await.expect("Failed to initialize storage");
///     let manager = init(&storage).await.expect("Failed to initialize preview manager");
///     // Manager is now ready to start/stop development servers
///     // Background tasks run automatically
/// }
/// ```
pub async fn init(storage: &orkee_storage::sqlite::SqliteStorage) -> PreviewResult<PreviewManager> {
    // Create server registry from shared storage
    let registry =
        ServerRegistry::new(storage)
            .await
            .map_err(|e| PreviewError::DetectionFailed {
                reason: format!("Failed to initialize server registry: {}", e),
            })?;

    // Start periodic discovery of external servers
    start_periodic_discovery();

    Ok(PreviewManager::new_with_recovery(registry).await)
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
        let config = orkee_storage::StorageConfig {
            provider: orkee_storage::StorageProvider::Sqlite {
                path: std::path::PathBuf::from(":memory:"),
            },
            max_connections: 5,
            busy_timeout_seconds: 30,
            enable_wal: false, // WAL doesn't work with :memory:
            enable_fts: true,
        };
        let storage = orkee_storage::sqlite::SqliteStorage::new(config)
            .await
            .expect("Failed to initialize test storage");
        let manager = init(&storage).await.unwrap();
        // Basic smoke test - just ensure we can create a manager
        // We don't assert on server count as it might recover existing servers
        let _servers = manager.list_servers().await;
        // If we get here without panic, the test passes
    }
}
