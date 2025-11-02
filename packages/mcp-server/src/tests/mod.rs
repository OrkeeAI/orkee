#[cfg(test)]
mod protocol_tests;

#[cfg(test)]
mod tool_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
pub mod test_helpers {
    use crate::context::ToolContext;
    use orkee_projects::orkee_storage::{factory::StorageManager, StorageConfig, StorageProvider};
    use orkee_projects::ProjectsManager;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// Create a test context with isolated in-memory storage
    /// Each test gets its own isolated database that doesn't interfere with other tests
    pub async fn create_test_context(
    ) -> Result<ToolContext, Box<dyn std::error::Error + Send + Sync>> {
        let config = StorageConfig {
            provider: StorageProvider::Sqlite {
                path: PathBuf::from(":memory:"),
            },
            enable_wal: false, // WAL mode not supported for in-memory databases
            enable_fts: true,
            max_connections: 1,
            busy_timeout_seconds: 5,
        };

        let storage_manager = Arc::new(StorageManager::new(config).await?);
        let projects_manager = Arc::new(ProjectsManager::with_storage(storage_manager));

        Ok(ToolContext { projects_manager })
    }
}
