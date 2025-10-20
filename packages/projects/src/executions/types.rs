// ABOUTME: Agent execution and PR review type definitions
// ABOUTME: Structures for tracking AI agent work and code review cycles

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum PrStatus {
    Draft,
    Open,
    Closed,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
pub enum ReviewStatus {
    Pending,
    ChangesRequested,
    Approved,
    Dismissed,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
pub enum ReviewerType {
    Ai,
    Human,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecution {
    pub id: String,
    pub task_id: String,
    pub agent_id: Option<String>,
    pub model: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: ExecutionStatus,
    pub execution_time_seconds: Option<i32>,

    // Token and cost tracking
    pub tokens_input: Option<i32>,
    pub tokens_output: Option<i32>,
    pub total_cost: Option<f64>,

    // Execution details
    pub prompt: Option<String>,
    pub response: Option<String>,
    pub error_message: Option<String>,
    pub retry_attempt: i32,

    // File change tracking
    pub files_changed: Option<i32>,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub files_created: Option<Vec<String>>,
    pub files_modified: Option<Vec<String>>,
    pub files_deleted: Option<Vec<String>>,

    // Git/PR tracking
    pub branch_name: Option<String>,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
    pub pr_number: Option<i32>,
    pub pr_url: Option<String>,
    pub pr_title: Option<String>,
    pub pr_status: Option<PrStatus>,
    pub pr_created_at: Option<DateTime<Utc>>,
    pub pr_merged_at: Option<DateTime<Utc>>,
    pub pr_merge_commit: Option<String>,

    // PR review status
    pub review_status: Option<ReviewStatus>,
    pub review_comments: Option<i32>,

    // Performance metrics
    pub test_results: Option<serde_json::Value>,
    pub performance_metrics: Option<serde_json::Value>,

    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionCreateInput {
    pub task_id: String,
    pub agent_id: Option<String>,
    pub model: Option<String>,
    pub prompt: Option<String>,
    pub retry_attempt: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentExecutionUpdateInput {
    pub status: Option<ExecutionStatus>,
    pub completed_at: Option<DateTime<Utc>>,
    pub execution_time_seconds: Option<i32>,
    pub tokens_input: Option<i32>,
    pub tokens_output: Option<i32>,
    pub total_cost: Option<f64>,
    pub response: Option<String>,
    pub error_message: Option<String>,
    pub files_changed: Option<i32>,
    pub lines_added: Option<i32>,
    pub lines_removed: Option<i32>,
    pub files_created: Option<Vec<String>>,
    pub files_modified: Option<Vec<String>>,
    pub files_deleted: Option<Vec<String>>,
    pub branch_name: Option<String>,
    pub commit_hash: Option<String>,
    pub commit_message: Option<String>,
    pub pr_number: Option<i32>,
    pub pr_url: Option<String>,
    pub pr_title: Option<String>,
    pub pr_status: Option<PrStatus>,
    pub pr_created_at: Option<DateTime<Utc>>,
    pub pr_merged_at: Option<DateTime<Utc>>,
    pub pr_merge_commit: Option<String>,
    pub review_status: Option<ReviewStatus>,
    pub review_comments: Option<i32>,
    pub test_results: Option<serde_json::Value>,
    pub performance_metrics: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReview {
    pub id: String,
    pub execution_id: String,
    pub reviewer_id: Option<String>,
    pub reviewer_type: ReviewerType,
    pub review_status: ReviewStatus,
    pub review_body: Option<String>,
    pub comments: Option<serde_json::Value>,
    pub suggested_changes: Option<serde_json::Value>,
    pub approval_date: Option<DateTime<Utc>>,
    pub dismissal_reason: Option<String>,
    pub reviewed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReviewCreateInput {
    pub execution_id: String,
    pub reviewer_id: Option<String>,
    pub reviewer_type: ReviewerType,
    pub review_status: ReviewStatus,
    pub review_body: Option<String>,
    pub comments: Option<serde_json::Value>,
    pub suggested_changes: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrReviewUpdateInput {
    pub review_status: Option<ReviewStatus>,
    pub review_body: Option<String>,
    pub comments: Option<serde_json::Value>,
    pub suggested_changes: Option<serde_json::Value>,
    pub approval_date: Option<DateTime<Utc>>,
    pub dismissal_reason: Option<String>,
}
