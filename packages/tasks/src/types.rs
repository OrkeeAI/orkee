// ABOUTME: Task type definitions
// ABOUTME: Structures for tasks, subtasks, dependencies, and execution tracking

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "kebab-case")]
#[serde(rename_all = "kebab-case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Review,
    Done,
    Cancelled,
    Deferred,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TaskPriority {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TaskType {
    Task,
    Subtask,
}

impl Default for TaskType {
    fn default() -> Self {
        TaskType::Task
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT")]
#[serde(rename_all = "UPPERCASE")]
pub enum SizeEstimate {
    XS,
    S,
    M,
    L,
    XL,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub description: Option<String>,
    pub status: TaskStatus,
    pub priority: TaskPriority,

    // User and agent attribution
    pub created_by_user_id: String,
    pub assigned_agent_id: Option<String>,
    pub reviewed_by_agent_id: Option<String>,

    // Hierarchy
    pub parent_id: Option<String>,
    pub position: i32,
    pub subtasks: Option<Vec<Task>>,

    // Dependencies
    pub dependencies: Option<Vec<String>>,
    pub blockers: Option<Vec<String>>,

    // Planning
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub complexity_score: Option<i32>,

    // Rich content
    pub details: Option<String>,
    pub test_strategy: Option<String>,
    pub acceptance_criteria: Option<String>,

    // AI-specific fields
    pub prompt: Option<String>,
    pub context: Option<String>,
    pub output_format: Option<String>,
    pub validation_rules: Option<serde_json::Value>,

    // Execution tracking
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_log: Option<serde_json::Value>,
    pub error_log: Option<serde_json::Value>,
    pub retry_count: i32,

    // Categorization
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,

    // Metadata
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,

    // CCPM Epic Integration
    pub epic_id: Option<String>,
    pub github_issue_number: Option<i32>,
    pub github_issue_url: Option<String>,
    pub parallel_group: Option<String>,
    pub depends_on: Option<Vec<String>>, // JSON array of task IDs
    pub conflicts_with: Option<Vec<String>>, // JSON array of task IDs
    pub task_type: TaskType,
    pub size_estimate: Option<SizeEstimate>,
    pub technical_details: Option<String>,
    pub effort_hours: Option<i32>,
    pub can_parallel: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCreateInput {
    pub title: String,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assigned_agent_id: Option<String>,
    pub parent_id: Option<String>,
    pub position: Option<i32>,
    pub dependencies: Option<Vec<String>>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f64>,
    pub complexity_score: Option<i32>,
    pub details: Option<String>,
    pub test_strategy: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub prompt: Option<String>,
    pub context: Option<String>,
    pub tag_id: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,

    // CCPM Epic Integration
    pub epic_id: Option<String>,
    pub parallel_group: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub conflicts_with: Option<Vec<String>>,
    pub task_type: Option<TaskType>,
    pub size_estimate: Option<SizeEstimate>,
    pub technical_details: Option<String>,
    pub effort_hours: Option<i32>,
    pub can_parallel: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskUpdateInput {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<TaskStatus>,
    pub priority: Option<TaskPriority>,
    pub assigned_agent_id: Option<String>,
    pub position: Option<i32>,
    pub dependencies: Option<Vec<String>>,
    pub due_date: Option<DateTime<Utc>>,
    pub estimated_hours: Option<f64>,
    pub actual_hours: Option<f64>,
    pub complexity_score: Option<i32>,
    pub details: Option<String>,
    pub test_strategy: Option<String>,
    pub acceptance_criteria: Option<String>,
    pub tags: Option<Vec<String>>,
    pub category: Option<String>,

    // CCPM Epic Integration
    pub epic_id: Option<String>,
    pub parallel_group: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub conflicts_with: Option<Vec<String>>,
    pub task_type: Option<TaskType>,
    pub size_estimate: Option<SizeEstimate>,
    pub technical_details: Option<String>,
    pub effort_hours: Option<i32>,
    pub can_parallel: Option<bool>,
}
