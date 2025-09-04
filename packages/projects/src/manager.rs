use crate::storage::{read_projects_config, write_projects_config, StorageError};
use crate::types::{Project, ProjectCreateInput, ProjectUpdateInput};
use crate::validator::{generate_project_id, validate_project_data, validate_project_update, ValidationError};
use crate::git_utils::get_git_repository_info;
use chrono::Utc;
use thiserror::Error;
use tracing::{debug, info};

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

pub type ManagerResult<T> = Result<T, ManagerError>;

/// Gets all projects
pub async fn get_all_projects() -> ManagerResult<Vec<Project>> {
    let config = read_projects_config().await?;
    let mut projects: Vec<_> = config.projects.into_values().collect();
    
    // Populate git repository information for each project
    for project in &mut projects {
        project.git_repository = get_git_repository_info(&project.project_root);
    }
    
    // Sort by rank (ascending) and then by name
    projects.sort_by(|a, b| {
        match (a.rank, b.rank) {
            (Some(rank_a), Some(rank_b)) => rank_a.cmp(&rank_b),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => a.name.cmp(&b.name),
        }
    });
    
    debug!("Retrieved {} projects", projects.len());
    Ok(projects)
}

/// Gets a project by ID
pub async fn get_project(id: &str) -> ManagerResult<Option<Project>> {
    let config = read_projects_config().await?;
    let mut project = config.projects.get(id).cloned();
    
    // Populate git repository information if project exists
    if let Some(ref mut proj) = project {
        proj.git_repository = get_git_repository_info(&proj.project_root);
    }
    
    Ok(project)
}

/// Gets a project by name
pub async fn get_project_by_name(name: &str) -> ManagerResult<Option<Project>> {
    let projects = get_all_projects().await?;
    Ok(projects.into_iter().find(|p| p.name == name))
}

/// Gets a project by project root path
pub async fn get_project_by_path(project_root: &str) -> ManagerResult<Option<Project>> {
    let projects = get_all_projects().await?;
    Ok(projects.into_iter().find(|p| p.project_root == project_root))
}

/// Creates a new project
pub async fn create_project(data: ProjectCreateInput) -> ManagerResult<Project> {
    // Validate the input
    let validation_errors = validate_project_data(&data, true).await;
    if !validation_errors.is_empty() {
        return Err(ManagerError::Validation(validation_errors));
    }
    
    let mut config = read_projects_config().await?;
    
    // Check for duplicate name
    if let Some(_) = config.projects.values().find(|p| p.name == data.name) {
        return Err(ManagerError::DuplicateName(data.name));
    }
    
    // Check for duplicate path
    if let Some(_) = config.projects.values().find(|p| p.project_root == data.project_root) {
        return Err(ManagerError::DuplicatePath(data.project_root));
    }
    
    let now = Utc::now();
    
    // Get git repository info before moving data
    let git_repository = get_git_repository_info(&data.project_root);
    
    // Calculate rank for new project (add to end)
    let max_rank = config.projects.values()
        .map(|p| p.rank.unwrap_or(0))
        .max()
        .unwrap_or(0);
    
    let project = Project {
        id: generate_project_id(),
        name: data.name,
        project_root: data.project_root,
        setup_script: data.setup_script.and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
        dev_script: data.dev_script.and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
        cleanup_script: data.cleanup_script.and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
        tags: data.tags.and_then(|t| if t.is_empty() { None } else { Some(t) }),
        description: data.description.and_then(|d| if d.trim().is_empty() { None } else { Some(d) }),
        status: data.status.unwrap_or_default(),
        rank: data.rank.or(Some(max_rank + 1)),
        priority: data.priority.unwrap_or_default(),
        task_source: data.task_source,
        manual_tasks: data.manual_tasks,
        mcp_servers: data.mcp_servers,
        git_repository,
        created_at: now,
        updated_at: now,
    };
    
    config.projects.insert(project.id.clone(), project.clone());
    write_projects_config(&config).await?;
    
    info!("Created project '{}' with ID {}", project.name, project.id);
    Ok(project)
}

/// Updates an existing project
pub async fn update_project(id: &str, updates: ProjectUpdateInput) -> ManagerResult<Project> {
    let mut config = read_projects_config().await?;
    
    // Check if project exists first
    if !config.projects.contains_key(id) {
        return Err(ManagerError::NotFound(id.to_string()));
    }
    
    // Validate the updates
    let validation_errors = validate_project_update(&updates, false).await;
    if !validation_errors.is_empty() {
        return Err(ManagerError::Validation(validation_errors));
    }
    
    // Get current project for comparison (clone to avoid borrowing issues)
    let current_project = config.projects[id].clone();
    
    // Check for duplicate name if name is being changed
    if let Some(ref new_name) = updates.name {
        if new_name != &current_project.name {
            if let Some(_) = config.projects.values().find(|p| p.name == *new_name && p.id != id) {
                return Err(ManagerError::DuplicateName(new_name.clone()));
            }
        }
    }
    
    // Check for duplicate path if path is being changed
    if let Some(ref new_path) = updates.project_root {
        if new_path != &current_project.project_root {
            if let Some(_) = config.projects.values().find(|p| p.project_root == *new_path && p.id != id) {
                return Err(ManagerError::DuplicatePath(new_path.clone()));
            }
        }
    }
    
    // Now we can safely get the mutable reference and apply updates
    let project = config.projects.get_mut(id).unwrap(); // Safe because we checked existence above
    
    // Apply updates
    if let Some(name) = updates.name {
        project.name = name;
    }
    if let Some(project_root) = updates.project_root {
        project.project_root = project_root;
    }
    if let Some(setup_script) = updates.setup_script {
        project.setup_script = if setup_script.trim().is_empty() { None } else { Some(setup_script) };
    }
    if let Some(dev_script) = updates.dev_script {
        project.dev_script = if dev_script.trim().is_empty() { None } else { Some(dev_script) };
    }
    if let Some(cleanup_script) = updates.cleanup_script {
        project.cleanup_script = if cleanup_script.trim().is_empty() { None } else { Some(cleanup_script) };
    }
    if let Some(tags) = updates.tags {
        project.tags = if tags.is_empty() { None } else { Some(tags) };
    }
    if let Some(description) = updates.description {
        project.description = if description.trim().is_empty() { None } else { Some(description) };
    }
    if let Some(status) = updates.status {
        project.status = status;
    }
    if let Some(rank) = updates.rank {
        project.rank = Some(rank);
    }
    if let Some(priority) = updates.priority {
        project.priority = priority;
    }
    if let Some(task_source) = updates.task_source {
        project.task_source = Some(task_source);
    }
    if let Some(manual_tasks) = updates.manual_tasks {
        project.manual_tasks = Some(manual_tasks);
    }
    if let Some(mcp_servers) = updates.mcp_servers {
        project.mcp_servers = Some(mcp_servers);
    }
    
    project.updated_at = Utc::now();
    
    // Clone the updated project before writing to avoid borrowing issues
    let updated_project = project.clone();
    
    write_projects_config(&config).await?;
    
    info!("Updated project '{}' (ID: {})", updated_project.name, updated_project.id);
    Ok(updated_project)
}

/// Deletes a project
pub async fn delete_project(id: &str) -> ManagerResult<bool> {
    let mut config = read_projects_config().await?;
    
    if let Some(project) = config.projects.remove(id) {
        write_projects_config(&config).await?;
        info!("Deleted project '{}' (ID: {})", project.name, project.id);
        Ok(true)
    } else {
        Ok(false)
    }
}

/// Projects manager struct for compatibility with existing code
pub struct ProjectsManager;

impl ProjectsManager {
    pub fn new() -> Self {
        ProjectsManager
    }
    
    pub async fn list_projects(&self) -> ManagerResult<Vec<Project>> {
        get_all_projects().await
    }
    
    pub async fn get_project(&self, id: &str) -> ManagerResult<Option<Project>> {
        get_project(id).await
    }
    
    pub async fn get_project_by_name(&self, name: &str) -> ManagerResult<Option<Project>> {
        get_project_by_name(name).await
    }
    
    pub async fn get_project_by_path(&self, project_root: &str) -> ManagerResult<Option<Project>> {
        get_project_by_path(project_root).await
    }
    
    pub async fn create_project(&self, data: ProjectCreateInput) -> ManagerResult<Project> {
        create_project(data).await
    }
    
    pub async fn update_project(&self, id: &str, updates: ProjectUpdateInput) -> ManagerResult<Project> {
        update_project(id, updates).await
    }
    
    pub async fn delete_project(&self, id: &str) -> ManagerResult<bool> {
        delete_project(id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ProjectStatus;
    use tempfile::TempDir;
    use std::env;

    async fn with_temp_home<F, Fut>(test: F) 
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let temp_dir = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();
        
        env::set_var("HOME", temp_dir.path());
        
        test().await;
        
        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }

    #[tokio::test]
    async fn test_create_and_get_project() {
        with_temp_home(|| async {
            // Clean slate
            if crate::constants::projects_file().exists() {
                tokio::fs::remove_file(crate::constants::projects_file()).await.ok();
            }
            if crate::constants::orkee_dir().exists() {
                tokio::fs::remove_dir_all(crate::constants::orkee_dir()).await.ok();
            }
            
            let input = ProjectCreateInput {
                name: "Test Project".to_string(),
                project_root: "/tmp/test".to_string(),
                setup_script: Some("npm install".to_string()),
                dev_script: Some("npm run dev".to_string()),
                cleanup_script: None,
                tags: Some(vec!["rust".to_string()]),
                description: Some("A test project".to_string()),
                status: Some(ProjectStatus::Active),
                rank: None,
                priority: None,
                task_source: None,
                manual_tasks: None,
                mcp_servers: None,
            };
            
            let project = create_project(input).await.unwrap();
            assert_eq!(project.name, "Test Project");
            assert_eq!(project.project_root, "/tmp/test");
            
            let retrieved = get_project(&project.id).await.unwrap();
            assert!(retrieved.is_some());
            assert_eq!(retrieved.unwrap().name, "Test Project");
        }).await;
    }

    #[tokio::test]
    async fn test_get_project_by_name() {
        with_temp_home(|| async {
            // Clean slate
            if crate::constants::projects_file().exists() {
                tokio::fs::remove_file(crate::constants::projects_file()).await.ok();
            }
            if crate::constants::orkee_dir().exists() {
                tokio::fs::remove_dir_all(crate::constants::orkee_dir()).await.ok();
            }
            
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
            
            create_project(input).await.unwrap();
            
            let found = get_project_by_name("Unique Name").await.unwrap();
            assert!(found.is_some());
            assert_eq!(found.unwrap().name, "Unique Name");
            
            let not_found = get_project_by_name("Nonexistent").await.unwrap();
            assert!(not_found.is_none());
        }).await;
    }

    #[tokio::test]
    async fn test_duplicate_name_error() {
        with_temp_home(|| async {
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
            
            create_project(input1).await.unwrap();
            
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
            
            let result = create_project(input2).await;
            assert!(result.is_err());
            match result.unwrap_err() {
                ManagerError::DuplicateName(name) => assert_eq!(name, "Duplicate"),
                _ => panic!("Expected DuplicateName error"),
            }
        }).await;
    }
}