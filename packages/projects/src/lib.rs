//! # Orkee Projects
//!
//! A project management library for Orkee that provides CRUD operations
//! for managing development projects with persistent storage.

pub mod agents;
pub mod api;
pub mod constants;
pub mod db;
pub mod executions;
pub mod formatter;
pub mod git_utils;
pub mod manager;
pub mod openspec;
pub mod storage;
pub mod tags;
pub mod tasks;
pub mod types;
pub mod users;
pub mod validator;

#[cfg(test)]
pub mod test_utils;

// Re-export main types
pub use types::{
    GitRepositoryInfo, ManualSubtask, ManualTask, Priority, Project, ProjectCreateInput,
    ProjectStatus, ProjectUpdateInput, ProjectsConfig, TaskSource, TaskStatus,
};

// Re-export manager functions
pub use manager::{
    create_project, delete_project, get_all_projects, get_project, get_project_by_name,
    get_project_by_path, get_storage_manager, initialize_storage, update_project, ManagerError,
    ManagerResult, ProjectsManager,
};

// Re-export storage types and traits
pub use storage::{
    // Legacy JSON storage functions for backward compatibility
    ensure_projects_file,
    factory::{ProjectStorageExt, StorageFactory, StorageManager, StorageStats},
    path_exists,
    read_projects_config,
    write_projects_config,
    CloudProvider,
    ProjectFilter,
    ProjectStorage,
    StorageCapabilities,
    StorageConfig,
    StorageError,
    StorageInfo,
    StorageProvider,
    StorageResult,
};

// Re-export validator functions
pub use validator::{truncate, validate_project_data, validate_project_update, ValidationError};

// Re-export storage utility functions
pub use storage::generate_project_id;

// Re-export formatter functions
pub use formatter::{format_project_details, format_projects_table};

// Re-export constants
pub use constants::{orkee_dir, projects_file, PROJECTS_VERSION};

// Re-export API routers
pub use api::{create_agents_router, create_executions_router, create_prds_router, create_projects_router, create_tags_router, create_tasks_router, create_users_router};

// Re-export database state
pub use db::DbState;
