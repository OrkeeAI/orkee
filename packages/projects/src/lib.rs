//! # Orkee Projects
//!
//! A project management library for Orkee that provides CRUD operations
//! for managing development projects with persistent storage.

pub mod db;
pub mod manager;
pub mod pagination;
pub mod prd;

#[cfg(any(test, feature = "test-utils"))]
pub mod test_utils;

// Re-export main types from core
pub use orkee_core::{
    GitRepositoryInfo, ManualSubtask, ManualTask, Priority, Project, ProjectCreateInput,
    ProjectStatus, ProjectUpdateInput, ProjectsConfig, TaskSource,
};

// Re-export manager functions
pub use manager::{
    create_project, delete_project, export_database, get_all_projects, get_project,
    get_project_by_name, get_project_by_path, get_storage_manager, import_database,
    initialize_storage, update_project, ManagerError, ManagerResult, ProjectsManager,
};

// Type alias for convenience
pub type ProjectManager = ProjectsManager;

// Re-export storage types and traits
pub use orkee_storage::{
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
pub use orkee_formatter::{format_project_details, format_projects_table};

// Re-export settings types for backward compatibility
pub use orkee_settings::{
    validate_setting_value, BulkSettingUpdate, SettingCategory, SettingUpdate, SettingUpdateItem,
    SettingsResponse, SettingsStorage, SystemSetting, ValidationError as SettingsValidationError,
};

// Re-export constants from core
pub use orkee_core::{orkee_dir, projects_file, PROJECTS_VERSION};

// Re-export database state
pub use db::DbState;

// Re-export pagination types
pub use pagination::{PaginatedResponse, PaginationMeta, PaginationParams};

// Re-export tags types
pub use orkee_tags::{Tag, TagCreateInput, TagStorage, TagUpdateInput};

// Re-export security types (API tokens, encryption, users)
pub use orkee_security::{
    ApiKeyEncryption, ApiToken, EncryptionError, MaskedUser, TokenGeneration, TokenStorage, User,
    UserStorage, UserUpdateInput,
};

// Re-export PRD types (used by API handlers and CCPM)
pub use prd::{
    create_prd, delete_prd, get_prd, get_prds_by_project, get_prds_by_project_paginated,
    hard_delete_prd, restore_prd, update_prd, DbError as PrdDbError, DbResult as PrdDbResult,
    PRDSource, PRDStatus, PRD,
};

// Re-export storage module (used by API handlers)
pub use orkee_storage;

// Re-export security module (used by API handlers)
pub use orkee_security;

// Re-export context module (used by API handlers)
pub use orkee_context;

// Re-export tasks types (used by API handlers)
pub use orkee_tasks::{
    storage::TaskStorage, Task, TaskCreateInput, TaskPriority, TaskStatus, TaskUpdateInput,
};
