//! Context management for MCP server tools
//! 
//! This module provides dependency injection support for testing,
//! allowing tests to use isolated in-memory storage while production
//! code continues to use the global storage manager.

use orkee_projects::{
    storage::{
        factory::StorageManager,
        StorageConfig, StorageProvider,
    },
    ProjectsManager,
};
use std::path::PathBuf;
use std::sync::Arc;

/// Context for tool execution that holds dependencies
/// This enables dependency injection for testing while maintaining
/// backward compatibility with production code.
#[derive(Clone)]
pub struct ToolContext {
    projects_manager: Arc<ProjectsManager>,
}

impl ToolContext {
    /// Create a new ToolContext with the global storage manager (production use)
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let projects_manager = Arc::new(ProjectsManager::new().await?);
        Ok(Self { projects_manager })
    }

    /// Create a new ToolContext with a custom ProjectsManager (testing use)
    pub fn with_manager(projects_manager: Arc<ProjectsManager>) -> Self {
        Self { projects_manager }
    }

    /// Get the projects manager
    pub fn projects_manager(&self) -> &Arc<ProjectsManager> {
        &self.projects_manager
    }

    /// Create a test context with an isolated in-memory database
    /// This ensures complete test isolation without shared state
    #[cfg(test)]
    pub async fn test_context() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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
        
        Ok(Self::with_manager(projects_manager))
    }
}

// Note: No Default implementation since ToolContext requires async initialization
// Tool functions will create the context if None is provided