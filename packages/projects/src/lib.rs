//! # Orkee Projects
//!
//! A project management library for Orkee that provides CRUD operations
//! for managing development projects with persistent storage.

pub mod agents;
pub mod ai_service;
pub mod ai_usage_logs;
pub mod api;
pub mod context;
pub mod db;
pub mod executions;
pub mod manager;
pub mod models;
pub mod pagination;
pub mod tasks;

#[cfg(test)]
pub mod test_utils;

// Re-export main types from core
pub use orkee_core::{
    GitRepositoryInfo, ManualSubtask, ManualTask, Priority, Project, ProjectCreateInput,
    ProjectStatus, ProjectUpdateInput, ProjectsConfig, TaskSource, TaskStatus,
};

// Re-export manager functions
pub use manager::{
    create_project, delete_project, get_all_projects, get_project, get_project_by_name,
    get_project_by_path, get_storage_manager, initialize_storage, update_project, ManagerError,
    ManagerResult, ProjectsManager,
};

// Type alias for convenience
pub type ProjectManager = ProjectsManager;

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

// Re-export validator functions from core
pub use orkee_core::{truncate, validate_project_data, validate_project_update, ValidationError};

// Re-export utility functions from core
pub use orkee_core::generate_project_id;

// Re-export formatter functions
pub use formatter::{format_project_details, format_projects_table};

// Re-export settings types for backward compatibility
pub use settings::{
    validate_setting_value, BulkSettingUpdate, SettingCategory, SettingUpdate,
    SettingUpdateItem, SettingsResponse, SettingsStorage, SystemSetting,
    ValidationError as SettingsValidationError,
};

// Re-export constants from core
pub use orkee_core::{orkee_dir, projects_file, PROJECTS_VERSION};

// Re-export API routers
pub use api::{
    create_agents_router, create_ai_proxy_router, create_ai_router, create_ai_usage_router,
    create_changes_router, create_context_router, create_executions_router, create_graph_router,
    create_prds_router, create_projects_router, create_security_router, create_specs_router,
    create_tags_router, create_task_spec_router, create_tasks_router, create_users_router,
};

// Re-export database state
pub use db::DbState;

// Re-export pagination types
pub use pagination::{PaginatedResponse, PaginationMeta, PaginationParams};

// Re-export tags types
pub use tags::{Tag, TagCreateInput, TagStorage, TagUpdateInput};

// Re-export security types (API tokens, encryption, users)
pub use security::{
    ApiKeyEncryption, ApiToken, EncryptionError, MaskedUser, TokenGeneration, TokenStorage, User,
    UserStorage, UserUpdateInput,
};

// Re-export models and agents registry
pub use models::{
    Agent, AgentConfig, AgentModelRef, Model, ModelCapabilities, ModelPricing, ModelRegistry,
    REGISTRY,
};

// Re-export openspec module for backward compatibility
pub use openspec;

// Re-export storage module for backward compatibility
pub use storage;

// Re-export security module for backward compatibility
pub use security;
