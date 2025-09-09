//! Context management for MCP server tools
//!
//! This module provides dependency injection support for testing,
//! allowing tests to use isolated in-memory storage while production
//! code continues to use the global storage manager.

use orkee_projects::ProjectsManager;
use std::sync::Arc;

/// Context for tool execution that holds dependencies
/// This enables dependency injection for testing while maintaining
/// backward compatibility with production code.
#[derive(Clone)]
pub struct ToolContext {
    pub(crate) projects_manager: Arc<ProjectsManager>,
}

impl ToolContext {
    /// Create a new ToolContext with the global storage manager (production use)
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let projects_manager = Arc::new(ProjectsManager::new().await?);
        Ok(Self { projects_manager })
    }

    /// Get the projects manager
    pub fn projects_manager(&self) -> &Arc<ProjectsManager> {
        &self.projects_manager
    }
}

// Note: No Default implementation since ToolContext requires async initialization
// Tool functions will create the context if None is provided
