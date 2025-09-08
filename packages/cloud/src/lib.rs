//! Orkee Cloud Sync Package
//! 
//! Provides cloud synchronization functionality for Orkee AI agent orchestration platform.
//! Supports AWS S3, Cloudflare R2, and other S3-compatible storage providers.

pub mod providers;
pub mod auth;
pub mod encryption;
pub mod sync;
pub mod config;
pub mod state;
pub mod types;

// Re-export commonly used types and traits
pub use providers::{CloudProvider, CloudProviderFactory};
pub use auth::{CredentialProvider, CredentialStore};
pub use encryption::{EncryptedSnapshotManager, EncryptionConfig};
pub use sync::{SyncEngine, SyncEngineConfig, SyncEngineFactory};
pub use config::{CloudConfig, CloudConfigManager, ProviderConfig};
pub use state::CloudSyncStateManager;
pub use types::*;

/// Cloud sync error types
pub use providers::CloudError;
pub type CloudResult<T> = Result<T, CloudError>;

/// Version of the cloud sync protocol
pub const CLOUD_SYNC_VERSION: u32 = 1;

/// Default configuration for cloud sync
pub fn default_config() -> CloudConfig {
    CloudConfig::default()
}