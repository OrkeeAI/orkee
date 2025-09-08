use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Git repository information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitRepositoryInfo {
    pub owner: String,
    pub repo: String,
    pub url: String,
    pub branch: Option<String>,
}

/// Status options for projects
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    Active,
    Archived,
}

impl Default for ProjectStatus {
    fn default() -> Self {
        ProjectStatus::Active
    }
}

impl fmt::Display for ProjectStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProjectStatus::Active => write!(f, "Active"),
            ProjectStatus::Archived => write!(f, "Archived"),
        }
    }
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,
    Medium,
    Low,
}

impl Default for Priority {
    fn default() -> Self {
        Priority::Medium
    }
}

impl fmt::Display for Priority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Priority::High => write!(f, "High"),
            Priority::Medium => write!(f, "Medium"),
            Priority::Low => write!(f, "Low"),
        }
    }
}

/// Task source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TaskSource {
    Taskmaster,
    Manual,
}

/// Task status options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Pending,
    Done,
    InProgress,
    Review,
    Deferred,
    Cancelled,
}

impl Default for TaskStatus {
    fn default() -> Self {
        TaskStatus::Pending
    }
}

/// A manual subtask
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualSubtask {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub dependencies: Vec<u32>,
    pub details: Option<String>,
    #[serde(default)]
    pub status: TaskStatus,
    #[serde(rename = "testStrategy")]
    pub test_strategy: Option<String>,
}

/// A manual task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManualTask {
    pub id: u32,
    pub title: String,
    pub description: String,
    pub details: Option<String>,
    #[serde(rename = "testStrategy")]
    pub test_strategy: Option<String>,
    #[serde(default)]
    pub priority: Priority,
    pub dependencies: Vec<u32>,
    #[serde(default)]
    pub status: TaskStatus,
    pub subtasks: Vec<ManualSubtask>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
}

/// A project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(rename = "projectRoot")]
    pub project_root: String,
    #[serde(rename = "setupScript")]
    pub setup_script: Option<String>,
    #[serde(rename = "devScript")]
    pub dev_script: Option<String>,
    #[serde(rename = "cleanupScript")]
    pub cleanup_script: Option<String>,
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updatedAt")]
    pub updated_at: DateTime<Utc>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    #[serde(default)]
    pub status: ProjectStatus,
    pub rank: Option<u32>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(rename = "taskSource")]
    pub task_source: Option<TaskSource>,
    #[serde(rename = "manualTasks")]
    pub manual_tasks: Option<Vec<ManualTask>>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Option<HashMap<String, bool>>,
    #[serde(rename = "gitRepository")]
    pub git_repository: Option<GitRepositoryInfo>,
}

/// Configuration structure for projects.json
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectsConfig {
    pub version: String,
    pub projects: HashMap<String, Project>,
}

impl Default for ProjectsConfig {
    fn default() -> Self {
        ProjectsConfig {
            version: "1.0.0".to_string(),
            projects: HashMap::new(),
        }
    }
}

/// Input for creating a new project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectCreateInput {
    pub name: String,
    #[serde(rename = "projectRoot")]
    pub project_root: String,
    #[serde(rename = "setupScript")]
    pub setup_script: Option<String>,
    #[serde(rename = "devScript")]
    pub dev_script: Option<String>,
    #[serde(rename = "cleanupScript")]
    pub cleanup_script: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
    pub rank: Option<u32>,
    pub priority: Option<Priority>,
    #[serde(rename = "taskSource")]
    pub task_source: Option<TaskSource>,
    #[serde(rename = "manualTasks")]
    pub manual_tasks: Option<Vec<ManualTask>>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Option<HashMap<String, bool>>,
}

/// Input for updating an existing project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUpdateInput {
    pub name: Option<String>,
    #[serde(rename = "projectRoot")]
    pub project_root: Option<String>,
    #[serde(rename = "setupScript")]
    pub setup_script: Option<String>,
    #[serde(rename = "devScript")]
    pub dev_script: Option<String>,
    #[serde(rename = "cleanupScript")]
    pub cleanup_script: Option<String>,
    pub tags: Option<Vec<String>>,
    pub description: Option<String>,
    pub status: Option<ProjectStatus>,
    pub rank: Option<u32>,
    pub priority: Option<Priority>,
    #[serde(rename = "taskSource")]
    pub task_source: Option<TaskSource>,
    #[serde(rename = "manualTasks")]
    pub manual_tasks: Option<Vec<ManualTask>>,
    #[serde(rename = "mcpServers")]
    pub mcp_servers: Option<HashMap<String, bool>>,
}
