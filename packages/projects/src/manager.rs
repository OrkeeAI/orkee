use crate::git_utils::get_git_repository_info;
use crate::storage::{factory::StorageManager, StorageError};
use crate::types::{Project, ProjectCreateInput, ProjectStatus, ProjectUpdateInput};
use crate::validator::{validate_project_data, validate_project_update, ValidationError};
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::OnceCell;
use tracing::{debug, info, warn};

/// Manager errors
#[derive(Error, Debug)]
pub enum ManagerError {
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("Validation errors: {0:?}")]
    Validation(Vec<ValidationError>),
    #[error("Project not found: {0}")]
    NotFound(String),
    #[error("Project with name '{0}' already exists")]
    DuplicateName(String),
    #[error("Project with path '{0}' already exists")]
    DuplicatePath(String),
}

/// Global storage manager instance
static STORAGE_MANAGER: OnceCell<Arc<StorageManager>> = OnceCell::const_new();

/// Initialize the global storage manager
pub async fn initialize_storage() -> ManagerResult<()> {
    let storage_manager = Arc::new(StorageManager::default().await?);
    STORAGE_MANAGER.set(storage_manager).map_err(|_| {
        ManagerError::Storage(StorageError::Database(
            "Storage already initialized".to_string(),
        ))
    })?;
    info!("Storage manager initialized successfully");
    Ok(())
}

/// Get the global storage manager instance
pub async fn get_storage_manager() -> ManagerResult<Arc<StorageManager>> {
    match STORAGE_MANAGER.get() {
        Some(manager) => Ok(manager.clone()),
        None => {
            warn!("Storage manager not initialized, initializing now");
            initialize_storage().await?;
            STORAGE_MANAGER
                .get()
                .ok_or_else(|| {
                    ManagerError::Storage(StorageError::Database(
                        "Failed to initialize storage manager".to_string(),
                    ))
                })
                .cloned()
        }
    }
}

/// Populate git repository information for projects
fn populate_git_info(projects: &mut Vec<Project>) {
    for project in projects {
        project.git_repository = get_git_repository_info(&project.project_root);
    }
}

pub type ManagerResult<T> = Result<T, ManagerError>;

/// Gets all projects
pub async fn get_all_projects() -> ManagerResult<Vec<Project>> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();
    let mut projects = storage.list_projects().await?;

    // Populate git repository information for each project
    populate_git_info(&mut projects);

    debug!("Retrieved {} projects", projects.len());
    Ok(projects)
}

/// Gets a project by ID
pub async fn get_project(id: &str) -> ManagerResult<Option<Project>> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();
    let mut project = storage.get_project(id).await?;

    // Populate git repository information if project exists
    if let Some(ref mut proj) = project {
        proj.git_repository = get_git_repository_info(&proj.project_root);
    }

    Ok(project)
}

/// Gets a project by name
pub async fn get_project_by_name(name: &str) -> ManagerResult<Option<Project>> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();
    let mut project = storage.get_project_by_name(name).await?;

    // Populate git repository information if project exists
    if let Some(ref mut proj) = project {
        proj.git_repository = get_git_repository_info(&proj.project_root);
    }

    Ok(project)
}

/// Gets a project by project root path
pub async fn get_project_by_path(project_root: &str) -> ManagerResult<Option<Project>> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();
    let mut project = storage.get_project_by_path(project_root).await?;

    // Populate git repository information if project exists
    if let Some(ref mut proj) = project {
        proj.git_repository = get_git_repository_info(&proj.project_root);
    }

    Ok(project)
}

/// Creates a new project
pub async fn create_project(data: ProjectCreateInput) -> ManagerResult<Project> {
    // Validate the input
    let validation_errors = validate_project_data(&data, true).await;
    if !validation_errors.is_empty() {
        return Err(ManagerError::Validation(validation_errors));
    }

    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();

    // Create project using storage layer (handles duplicate checks)
    let mut project = storage.create_project(data).await?;

    // Populate git repository information
    project.git_repository = get_git_repository_info(&project.project_root);

    info!("Created project '{}' with ID {}", project.name, project.id);
    Ok(project)
}

/// Updates an existing project
pub async fn update_project(id: &str, updates: ProjectUpdateInput) -> ManagerResult<Project> {
    // Validate the updates
    let validation_errors = validate_project_update(&updates, false).await;
    if !validation_errors.is_empty() {
        return Err(ManagerError::Validation(validation_errors));
    }

    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();

    // Update project using storage layer (handles duplicate checks)
    let mut project = storage.update_project(id, updates).await?;

    // Always refresh git repository info to ensure it's current
    project.git_repository = get_git_repository_info(&project.project_root);

    info!("Updated project '{}' (ID: {})", project.name, project.id);
    Ok(project)
}

/// Deletes a project
pub async fn delete_project(id: &str) -> ManagerResult<bool> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();

    // Get project info before deletion for logging
    if let Some(project) = storage.get_project(id).await? {
        storage.delete_project(id).await?;
        info!("Deleted project '{}' (ID: {})", project.name, project.id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Projects manager struct for compatibility with existing code
pub struct ProjectsManager {
    storage_manager: Arc<StorageManager>,
}

impl ProjectsManager {
    /// Create a new ProjectsManager with default storage
    pub async fn new() -> ManagerResult<Self> {
        let storage_manager = get_storage_manager().await?;
        Ok(Self { storage_manager })
    }

    /// Create a new ProjectsManager with custom storage
    pub fn with_storage(storage_manager: Arc<StorageManager>) -> Self {
        Self { storage_manager }
    }

    pub async fn list_projects(&self) -> ManagerResult<Vec<Project>> {
        let storage = self.storage_manager.storage();
        let mut projects = storage.list_projects().await?;
        populate_git_info(&mut projects);
        Ok(projects)
    }

    pub async fn get_project(&self, id: &str) -> ManagerResult<Option<Project>> {
        let storage = self.storage_manager.storage();
        let mut project = storage.get_project(id).await?;
        if let Some(ref mut proj) = project {
            proj.git_repository = get_git_repository_info(&proj.project_root);
        }
        Ok(project)
    }

    pub async fn get_project_by_name(&self, name: &str) -> ManagerResult<Option<Project>> {
        let storage = self.storage_manager.storage();
        let mut project = storage.get_project_by_name(name).await?;
        if let Some(ref mut proj) = project {
            proj.git_repository = get_git_repository_info(&proj.project_root);
        }
        Ok(project)
    }

    pub async fn get_project_by_path(&self, project_root: &str) -> ManagerResult<Option<Project>> {
        let storage = self.storage_manager.storage();
        let mut project = storage.get_project_by_path(project_root).await?;
        if let Some(ref mut proj) = project {
            proj.git_repository = get_git_repository_info(&proj.project_root);
        }
        Ok(project)
    }

    pub async fn create_project(&self, data: ProjectCreateInput) -> ManagerResult<Project> {
        // Validate the input
        let validation_errors = validate_project_data(&data, true).await;
        if !validation_errors.is_empty() {
            return Err(ManagerError::Validation(validation_errors));
        }

        let storage = self.storage_manager.storage();
        let mut project = storage.create_project(data).await?;
        project.git_repository = get_git_repository_info(&project.project_root);

        info!("Created project '{}' with ID {}", project.name, project.id);
        Ok(project)
    }

    pub async fn update_project(
        &self,
        id: &str,
        updates: ProjectUpdateInput,
    ) -> ManagerResult<Project> {
        // Validate the updates
        let validation_errors = validate_project_update(&updates, false).await;
        if !validation_errors.is_empty() {
            return Err(ManagerError::Validation(validation_errors));
        }

        let storage = self.storage_manager.storage();
        let mut project = storage.update_project(id, updates).await?;
        project.git_repository = get_git_repository_info(&project.project_root);

        info!("Updated project '{}' (ID: {})", project.name, project.id);
        Ok(project)
    }

    pub async fn delete_project(&self, id: &str) -> ManagerResult<bool> {
        let storage = self.storage_manager.storage();

        if let Some(project) = storage.get_project(id).await? {
            storage.delete_project(id).await?;
            info!("Deleted project '{}' (ID: {})", project.name, project.id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Search projects with text query
    pub async fn search_projects(&self, query: &str) -> ManagerResult<Vec<Project>> {
        let storage = self.storage_manager.storage();
        let mut projects = storage.search_projects(query).await?;
        populate_git_info(&mut projects);
        Ok(projects)
    }

    /// List projects with filters
    pub async fn list_projects_with_filter(
        &self,
        filter: crate::storage::ProjectFilter,
    ) -> ManagerResult<Vec<Project>> {
        let storage = self.storage_manager.storage();
        let mut projects = storage.list_projects_with_filter(filter).await?;
        populate_git_info(&mut projects);
        Ok(projects)
    }

    /// Get active projects only (Pre-Launch and Launched)
    pub async fn list_active_projects(&self) -> ManagerResult<Vec<Project>> {
        let filter = crate::storage::ProjectFilter {
            status: Some(ProjectStatus::Planning),
            ..Default::default()
        };
        let mut projects = self.list_projects_with_filter(filter).await?;

        let filter2 = crate::storage::ProjectFilter {
            status: Some(ProjectStatus::Launched),
            ..Default::default()
        };
        let launched = self.list_projects_with_filter(filter2).await?;
        projects.extend(launched);

        Ok(projects)
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> ManagerResult<crate::storage::factory::StorageStats> {
        self.storage_manager
            .get_stats()
            .await
            .map_err(ManagerError::Storage)
    }
}

/// Export database as a compressed snapshot
pub async fn export_database() -> ManagerResult<Vec<u8>> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();

    info!("Exporting database snapshot");
    let snapshot = storage.export_snapshot().await?;

    info!("Database exported successfully, {} bytes", snapshot.len());
    Ok(snapshot)
}

/// Import database from a compressed snapshot
pub async fn import_database(data: Vec<u8>) -> ManagerResult<crate::storage::ImportResult> {
    let storage_manager = get_storage_manager().await?;
    let storage = storage_manager.storage();

    info!("Importing database snapshot, {} bytes", data.len());
    let result = storage.import_snapshot(&data).await?;

    info!(
        "Database imported: {} projects imported, {} skipped, {} conflicts",
        result.projects_imported,
        result.projects_skipped,
        result.conflicts.len()
    );

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{StorageConfig, StorageProvider};
    use crate::types::ProjectStatus;
    use std::path::PathBuf;

    /// Create a test storage manager (not using the global singleton)
    async fn create_test_storage_manager() -> ManagerResult<Arc<StorageManager>> {
        let config = StorageConfig {
            provider: StorageProvider::Sqlite {
                path: PathBuf::from(":memory:"),
            },
            enable_wal: false,
            enable_fts: true,
            max_connections: 1,
            busy_timeout_seconds: 10,
        };

        Ok(Arc::new(StorageManager::new(config).await?))
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        let storage_manager = create_test_storage_manager().await.unwrap();
        let storage = storage_manager.storage();

        let input = ProjectCreateInput {
            name: "Test Project".to_string(),
            project_root: "/tmp/test".to_string(),
            setup_script: Some("npm install".to_string()),
            dev_script: Some("npm run dev".to_string()),
            cleanup_script: None,
            tags: Some(vec!["rust".to_string()]),
            description: Some("A test project".to_string()),
            status: Some(ProjectStatus::Planning),
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let project = storage.create_project(input).await.unwrap();
        assert_eq!(project.name, "Test Project");
        assert_eq!(project.project_root, "/tmp/test");

        let retrieved = storage.get_project(&project.id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Project");
    }

    #[tokio::test]
    async fn test_get_project_by_name() {
        let storage_manager = create_test_storage_manager().await.unwrap();
        let storage = storage_manager.storage();

        let input = ProjectCreateInput {
            name: "Unique Name".to_string(),
            project_root: "/tmp/unique".to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            tags: None,
            description: None,
            status: None,
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        storage.create_project(input).await.unwrap();

        let found = storage.get_project_by_name("Unique Name").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Unique Name");

        let not_found = storage.get_project_by_name("Nonexistent").await.unwrap();
        assert!(not_found.is_none());
    }

    #[tokio::test]
    async fn test_duplicate_name_error() {
        let storage_manager = create_test_storage_manager().await.unwrap();
        let storage = storage_manager.storage();

        let input1 = ProjectCreateInput {
            name: "Duplicate".to_string(),
            project_root: "/tmp/dup1".to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            tags: None,
            description: None,
            status: None,
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        storage.create_project(input1).await.unwrap();

        let input2 = ProjectCreateInput {
            name: "Duplicate".to_string(),
            project_root: "/tmp/dup2".to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            tags: None,
            description: None,
            status: None,
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let result = storage.create_project(input2).await;
        assert!(result.is_err());
        match result.unwrap_err() {
            StorageError::DuplicateName(name) => assert_eq!(name, "Duplicate"),
            _ => panic!("Expected DuplicateName error"),
        }
    }
}
