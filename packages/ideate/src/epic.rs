// ABOUTME: Epic management types and data structures
// ABOUTME: Defines Epic entities, status tracking, and task breakdown metadata for CCPM workflow

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Epic status tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EpicStatus {
    /// Initial draft state
    Draft,
    /// Ready for implementation
    Ready,
    /// Work in progress
    InProgress,
    /// Blocked by external factors
    Blocked,
    /// Successfully completed
    Completed,
    /// Cancelled/abandoned
    Cancelled,
}

/// Epic complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EpicComplexity {
    Low,
    Medium,
    High,
    VeryHigh,
}

/// Estimated effort timeframe
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum EstimatedEffort {
    Days,
    Weeks,
    Months,
}

/// Architecture decision with rationale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchitectureDecision {
    pub decision: String,
    pub rationale: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alternatives: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tradeoffs: Option<String>,
}

/// External dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalDependency {
    pub name: String,
    #[serde(rename = "type")]
    pub dep_type: String, // 'library', 'service', 'api', etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    pub reason: String,
}

/// Success criterion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriterion {
    pub criterion: String,
    pub measurable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

/// Epic entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Epic {
    pub id: String,
    pub project_id: String,
    pub prd_id: String,
    pub name: String,

    // Epic content (markdown stored in DB)
    pub overview_markdown: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_decisions: Option<Vec<ArchitectureDecision>>,
    pub technical_approach: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<ExternalDependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<Vec<SuccessCriterion>>,

    // Task breakdown metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<EstimatedEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<EpicComplexity>,

    // Status tracking
    pub status: EpicStatus,
    pub progress_percentage: i32,

    // GitHub integration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_issue_number: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_issue_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_synced_at: Option<DateTime<Utc>>,

    // Phase 1 enhancement fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codebase_context: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub simplification_analysis: Option<String>,
    pub task_count_limit: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decomposition_phase: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_tasks: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality_validation: Option<serde_json::Value>,

    // Timestamps
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<DateTime<Utc>>,
}

/// Input for creating a new Epic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEpicInput {
    pub prd_id: String,
    pub name: String,
    pub overview_markdown: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_decisions: Option<Vec<ArchitectureDecision>>,
    pub technical_approach: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<ExternalDependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<Vec<SuccessCriterion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<EstimatedEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<EpicComplexity>,
}

/// Input for updating an Epic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateEpicInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overview_markdown: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture_decisions: Option<Vec<ArchitectureDecision>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub technical_approach: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub implementation_strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<ExternalDependency>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_criteria: Option<Vec<SuccessCriterion>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_categories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub estimated_effort: Option<EstimatedEffort>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub complexity: Option<EpicComplexity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<EpicStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress_percentage: Option<i32>,
}

/// Work stream for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkStream {
    pub name: String,
    pub description: String,
    pub tasks: Vec<String>, // task IDs
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_patterns: Option<Vec<String>>,
}

/// Dependency graph node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
}

/// Dependency graph edge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub edge_type: Option<String>,
}

/// Dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

/// Conflict between tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskConflict {
    pub task1: String,
    pub task2: String,
    pub reason: String,
}

/// Conflict analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictAnalysis {
    pub conflicts: Vec<TaskConflict>,
}

/// Work stream analysis for parallel execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkAnalysis {
    pub id: String,
    pub epic_id: String,

    // Analysis results (JSON in DB)
    pub parallel_streams: Vec<WorkStream>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_patterns: Option<std::collections::HashMap<String, Vec<String>>>,
    pub dependency_graph: DependencyGraph,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_analysis: Option<ConflictAnalysis>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallelization_strategy: Option<String>,

    // Metadata
    pub analyzed_at: DateTime<Utc>,
    pub is_current: bool,
    pub analysis_version: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence_score: Option<f64>,
}
