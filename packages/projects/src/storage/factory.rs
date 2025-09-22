use async_trait::async_trait;
use std::sync::Arc;
use tracing::{debug, info};

use super::{
    sqlite::SqliteStorage, ProjectStorage, StorageConfig, StorageError, StorageProvider,
    StorageResult,
};

/// Factory for creating storage instances
pub struct StorageFactory;

impl StorageFactory {
    /// Create a new storage instance from configuration
    pub async fn create_storage(config: StorageConfig) -> StorageResult<Box<dyn ProjectStorage>> {
        debug!("Creating storage with provider: {:?}", config.provider);

        match &config.provider {
            StorageProvider::Sqlite { path } => {
                info!("Initializing SQLite storage at: {:?}", path);
                let storage = SqliteStorage::new(config).await?;
                storage.initialize().await?;
                Ok(Box::new(storage))
            }
            StorageProvider::Cloud {
                provider,
                local_cache: _,
            } => {
                // For future implementation
                match provider {
                    super::CloudProvider::S3 { bucket, region } => {
                        // TODO: Implement S3-backed storage
                        info!(
                            "S3 storage not yet implemented (bucket: {}, region: {})",
                            bucket, region
                        );
                        Err(StorageError::Database(
                            "S3 storage not yet implemented".to_string(),
                        ))
                    }
                    super::CloudProvider::Convex { deployment_url } => {
                        // TODO: Implement Convex-backed storage
                        info!(
                            "Convex storage not yet implemented (deployment: {})",
                            deployment_url
                        );
                        Err(StorageError::Database(
                            "Convex storage not yet implemented".to_string(),
                        ))
                    }
                }
            }
        }
    }

    /// Create a storage instance with default configuration
    pub async fn create_default_storage() -> StorageResult<Box<dyn ProjectStorage>> {
        let config = StorageConfig::default();
        Self::create_storage(config).await
    }

    /// Create a storage instance from a database URL
    pub async fn from_url(url: &str) -> StorageResult<Box<dyn ProjectStorage>> {
        if url.starts_with("sqlite:") {
            let path = url
                .strip_prefix("sqlite:")
                .ok_or_else(|| StorageError::Database("Invalid SQLite URL format".to_string()))?;
            let config = StorageConfig {
                provider: StorageProvider::Sqlite {
                    path: std::path::PathBuf::from(path),
                },
                ..StorageConfig::default()
            };
            Self::create_storage(config).await
        } else {
            Err(StorageError::Database(format!(
                "Unsupported database URL: {}",
                url
            )))
        }
    }

    /// Create an in-memory SQLite storage for testing
    #[cfg(test)]
    pub async fn create_memory_storage() -> StorageResult<Box<dyn ProjectStorage>> {
        let config = StorageConfig {
            provider: StorageProvider::Sqlite {
                path: std::path::PathBuf::from(":memory:"),
            },
            enable_wal: false, // WAL mode not supported for in-memory databases
            enable_fts: true,
            max_connections: 1,
            busy_timeout_seconds: 5,
        };
        Self::create_storage(config).await
    }
}

/// Storage manager that holds and manages the active storage instance
pub struct StorageManager {
    storage: Arc<Box<dyn ProjectStorage>>,
    config: StorageConfig,
}

impl StorageManager {
    /// Create a new storage manager with the given configuration
    pub async fn new(config: StorageConfig) -> StorageResult<Self> {
        let storage = Arc::new(StorageFactory::create_storage(config.clone()).await?);
        Ok(Self { storage, config })
    }

    /// Create a storage manager with default configuration
    pub async fn default() -> StorageResult<Self> {
        let config = StorageConfig::default();
        Self::new(config).await
    }

    /// Get a reference to the storage instance
    pub fn storage(&self) -> Arc<Box<dyn ProjectStorage>> {
        self.storage.clone()
    }

    /// Get the current configuration
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    /// Recreate the storage instance with new configuration
    pub async fn reconfigure(&mut self, config: StorageConfig) -> StorageResult<()> {
        info!("Reconfiguring storage manager");
        let new_storage = Arc::new(StorageFactory::create_storage(config.clone()).await?);
        self.storage = new_storage;
        self.config = config;
        Ok(())
    }

    /// Test the storage connection
    pub async fn test_connection(&self) -> StorageResult<()> {
        debug!("Testing storage connection");
        let _info = self.storage.get_storage_info().await?;
        info!("Storage connection test successful");
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> StorageResult<StorageStats> {
        let info = self.storage.get_storage_info().await?;
        let projects = self.storage.list_projects().await?;

        let active_count = projects
            .iter()
            .filter(|p| p.status == crate::types::ProjectStatus::Active)
            .count();

        let archived_count = projects
            .iter()
            .filter(|p| p.status == crate::types::ProjectStatus::Archived)
            .count();

        Ok(StorageStats {
            total_projects: info.total_projects,
            active_projects: active_count,
            archived_projects: archived_count,
            storage_size_bytes: info.size_bytes,
            last_modified: info.last_modified,
            provider: info.provider,
            capabilities: info.capabilities,
        })
    }
}

/// Statistics about the storage system
#[derive(Debug)]
pub struct StorageStats {
    pub total_projects: usize,
    pub active_projects: usize,
    pub archived_projects: usize,
    pub storage_size_bytes: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
    pub provider: String,
    pub capabilities: super::StorageCapabilities,
}

/// Wrapper trait that adds convenience methods to ProjectStorage
#[async_trait]
pub trait ProjectStorageExt: ProjectStorage {
    /// Create a project and return its ID
    async fn create_project_id(
        &self,
        input: crate::types::ProjectCreateInput,
    ) -> StorageResult<String> {
        let project = self.create_project(input).await?;
        Ok(project.id)
    }

    /// Check if a project exists by ID
    async fn project_exists(&self, id: &str) -> StorageResult<bool> {
        Ok(self.get_project(id).await?.is_some())
    }

    /// Check if a project name is available
    async fn name_available(&self, name: &str) -> StorageResult<bool> {
        Ok(self.get_project_by_name(name).await?.is_none())
    }

    /// Check if a project path is available
    async fn path_available(&self, path: &str) -> StorageResult<bool> {
        Ok(self.get_project_by_path(path).await?.is_none())
    }

    /// Get active projects only
    async fn list_active_projects(&self) -> StorageResult<Vec<crate::types::Project>> {
        let filter = super::ProjectFilter {
            status: Some(crate::types::ProjectStatus::Active),
            ..Default::default()
        };
        self.list_projects_with_filter(filter).await
    }

    /// Count projects by status
    async fn count_by_status(&self, status: crate::types::ProjectStatus) -> StorageResult<usize> {
        let filter = super::ProjectFilter {
            status: Some(status),
            ..Default::default()
        };
        let projects = self.list_projects_with_filter(filter).await?;
        Ok(projects.len())
    }
}

// Blanket implementation for all ProjectStorage types
impl<T: ProjectStorage + ?Sized> ProjectStorageExt for T {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ProjectCreateInput, ProjectStatus};
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_factory_create_sqlite_storage() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = StorageConfig {
            provider: StorageProvider::Sqlite { path: db_path },
            enable_wal: true,
            enable_fts: true,
            max_connections: 5,
            busy_timeout_seconds: 10,
        };

        let storage = StorageFactory::create_storage(config).await.unwrap();

        // Test that it works
        let projects = storage.list_projects().await.unwrap();
        assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn test_factory_create_default_storage() {
        // This should create a storage in the default location
        // For tests, we'll skip this as it would create files in the real directory
        // In a real test environment, you'd mock the default path
    }

    #[tokio::test]
    async fn test_factory_from_url() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let url = format!("sqlite:{}", db_path.display());

        let storage = StorageFactory::from_url(&url).await.unwrap();

        // Test that it works
        let projects = storage.list_projects().await.unwrap();
        assert_eq!(projects.len(), 0);
    }

    #[tokio::test]
    async fn test_storage_manager() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = StorageConfig {
            provider: StorageProvider::Sqlite { path: db_path },
            enable_wal: true,
            enable_fts: true,
            max_connections: 5,
            busy_timeout_seconds: 10,
        };

        let manager = StorageManager::new(config).await.unwrap();

        // Test connection
        manager.test_connection().await.unwrap();

        // Create a test project
        let input = ProjectCreateInput {
            name: "Test Project".to_string(),
            project_root: "/tmp/test".to_string(),
            description: Some("Test".to_string()),
            status: Some(ProjectStatus::Active),
            priority: None,
            rank: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            task_source: None,
            tags: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let storage = manager.storage();
        storage.create_project(input).await.unwrap();

        // Get stats
        let stats = manager.get_stats().await.unwrap();
        assert_eq!(stats.total_projects, 1);
        assert_eq!(stats.active_projects, 1);
        assert_eq!(stats.archived_projects, 0);
    }

    #[tokio::test]
    async fn test_storage_ext_methods() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = StorageConfig {
            provider: StorageProvider::Sqlite { path: db_path },
            enable_wal: true,
            enable_fts: true,
            max_connections: 5,
            busy_timeout_seconds: 10,
        };

        let storage = StorageFactory::create_storage(config).await.unwrap();

        // Test name availability
        assert!(storage.name_available("Test Project").await.unwrap());
        assert!(storage.path_available("/tmp/test").await.unwrap());

        // Create a project
        let input = ProjectCreateInput {
            name: "Test Project".to_string(),
            project_root: "/tmp/test".to_string(),
            description: None,
            status: Some(ProjectStatus::Active),
            priority: None,
            rank: None,
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            task_source: None,
            tags: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let project_id = storage.create_project_id(input).await.unwrap();

        // Test existence checks
        assert!(storage.project_exists(&project_id).await.unwrap());
        assert!(!storage.name_available("Test Project").await.unwrap());
        assert!(!storage.path_available("/tmp/test").await.unwrap());

        // Test counting
        let active_count = storage
            .count_by_status(ProjectStatus::Active)
            .await
            .unwrap();
        assert_eq!(active_count, 1);

        let archived_count = storage
            .count_by_status(ProjectStatus::Archived)
            .await
            .unwrap();
        assert_eq!(archived_count, 0);
    }
}
