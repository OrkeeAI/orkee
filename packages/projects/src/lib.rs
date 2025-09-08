//! # Orkee Projects
//!
//! A project management library for Orkee that provides CRUD operations
//! for managing development projects with persistent storage.

pub mod api;
pub mod constants;
pub mod formatter;
pub mod git_utils;
pub mod manager;
pub mod storage;
pub mod types;
pub mod validator;

#[cfg(test)]
pub mod test_utils;

// Re-export main types
pub use types::{
    GitRepositoryInfo, ManualSubtask, ManualTask, Priority, Project, ProjectCreateInput, ProjectStatus,
    ProjectUpdateInput, ProjectsConfig, TaskSource, TaskStatus,
};

// Re-export manager functions
pub use manager::{
    create_project, delete_project, get_all_projects, get_project, get_project_by_name,
    get_project_by_path, update_project, ManagerError, ManagerResult, ProjectsManager,
    initialize_storage, get_storage_manager,
};

// Re-export storage types and traits
pub use storage::{
    ProjectStorage, StorageConfig, StorageProvider, CloudProvider, StorageError,
    StorageResult, StorageInfo, StorageCapabilities, ProjectFilter,
    factory::{StorageFactory, StorageManager, StorageStats, ProjectStorageExt},
    // Legacy JSON storage functions for backward compatibility
    ensure_projects_file, path_exists, read_projects_config, write_projects_config,
};

// Re-export validator functions
pub use validator::{
    truncate, validate_project_data, validate_project_update,
    ValidationError,
};

// Re-export storage utility functions
pub use storage::generate_project_id;

// Re-export formatter functions
pub use formatter::{format_project_details, format_projects_table};

// Re-export constants
pub use constants::{orkee_dir, projects_file, PROJECTS_VERSION};

// Re-export API router
pub use api::create_projects_router;