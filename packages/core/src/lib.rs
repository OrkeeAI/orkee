// ABOUTME: Core types, traits, and utilities for Orkee
// ABOUTME: Foundational package providing shared functionality across all Orkee packages

pub mod constants;
pub mod types;
pub mod utils;
pub mod validation;

// Re-export main types
pub use types::{
    GitRepositoryInfo, ManualSubtask, ManualTask, Priority, Project, ProjectCreateInput,
    ProjectStatus, ProjectUpdateInput, ProjectsConfig, TaskSource, TaskStatus,
};

// Re-export constants
pub use constants::{orkee_dir, projects_file, PROJECTS_VERSION};

// Re-export utilities
pub use utils::{compress_data, decompress_data, generate_project_id, path_exists};

// Re-export validation
pub use validation::{truncate, validate_project_data, validate_project_update, ValidationError};
