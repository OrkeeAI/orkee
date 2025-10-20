// ABOUTME: HTTP request handlers for agent execution operations
// ABOUTME: Handles CRUD operations for executions and PR reviews with database integration

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
    Json,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::info;

use super::response::ApiResponse;
use crate::db::DbState;
use crate::executions::{
    AgentExecutionCreateInput, AgentExecutionUpdateInput, ExecutionStatus, PrReviewCreateInput,
    PrReviewUpdateInput, PrStatus, ReviewStatus, ReviewerType,
};

// ==================== Agent Executions ====================

/// List all executions for a task
pub async fn list_executions(
    State(db): State<DbState>,
    Path(task_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing executions for task: {}", task_id);

    match db.execution_storage.list_executions(&task_id).await {
        Ok(executions) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(executions))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Get a single execution by ID
pub async fn get_execution(
    State(db): State<DbState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting execution: {}", execution_id);

    match db.execution_storage.get_execution(&execution_id).await {
        Ok(execution) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(execution))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Request body for creating an execution
#[derive(Deserialize)]
pub struct CreateExecutionRequest {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "agentId")]
    pub agent_id: Option<String>,
    pub model: Option<String>,
    pub prompt: Option<String>,
    #[serde(rename = "retryAttempt")]
    pub retry_attempt: Option<i32>,
}

/// Create a new execution
pub async fn create_execution(
    State(db): State<DbState>,
    Json(request): Json<CreateExecutionRequest>,
) -> impl IntoResponse {
    info!("Creating execution for task: {}", request.task_id);

    let input = AgentExecutionCreateInput {
        task_id: request.task_id,
        agent_id: request.agent_id,
        model: request.model,
        prompt: request.prompt,
        retry_attempt: request.retry_attempt,
    };

    match db.execution_storage.create_execution(input).await {
        Ok(execution) => {
            (StatusCode::CREATED, ResponseJson(ApiResponse::success(execution))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Request body for updating an execution
#[derive(Deserialize)]
pub struct UpdateExecutionRequest {
    pub status: Option<ExecutionStatus>,
    #[serde(rename = "completedAt")]
    pub completed_at: Option<String>,
    #[serde(rename = "executionTimeSeconds")]
    pub execution_time_seconds: Option<i32>,
    #[serde(rename = "tokensInput")]
    pub tokens_input: Option<i32>,
    #[serde(rename = "tokensOutput")]
    pub tokens_output: Option<i32>,
    #[serde(rename = "totalCost")]
    pub total_cost: Option<f64>,
    pub response: Option<String>,
    #[serde(rename = "errorMessage")]
    pub error_message: Option<String>,
    #[serde(rename = "filesChanged")]
    pub files_changed: Option<i32>,
    #[serde(rename = "linesAdded")]
    pub lines_added: Option<i32>,
    #[serde(rename = "linesRemoved")]
    pub lines_removed: Option<i32>,
    #[serde(rename = "filesCreated")]
    pub files_created: Option<Vec<String>>,
    #[serde(rename = "filesModified")]
    pub files_modified: Option<Vec<String>>,
    #[serde(rename = "filesDeleted")]
    pub files_deleted: Option<Vec<String>>,
    #[serde(rename = "branchName")]
    pub branch_name: Option<String>,
    #[serde(rename = "commitHash")]
    pub commit_hash: Option<String>,
    #[serde(rename = "commitMessage")]
    pub commit_message: Option<String>,
    #[serde(rename = "prNumber")]
    pub pr_number: Option<i32>,
    #[serde(rename = "prUrl")]
    pub pr_url: Option<String>,
    #[serde(rename = "prTitle")]
    pub pr_title: Option<String>,
    #[serde(rename = "prStatus")]
    pub pr_status: Option<PrStatus>,
    #[serde(rename = "prCreatedAt")]
    pub pr_created_at: Option<String>,
    #[serde(rename = "prMergedAt")]
    pub pr_merged_at: Option<String>,
    #[serde(rename = "prMergeCommit")]
    pub pr_merge_commit: Option<String>,
    #[serde(rename = "reviewStatus")]
    pub review_status: Option<ReviewStatus>,
    #[serde(rename = "reviewComments")]
    pub review_comments: Option<i32>,
    #[serde(rename = "testResults")]
    pub test_results: Option<serde_json::Value>,
    #[serde(rename = "performanceMetrics")]
    pub performance_metrics: Option<serde_json::Value>,
    pub metadata: Option<serde_json::Value>,
}

/// Update an execution
pub async fn update_execution(
    State(db): State<DbState>,
    Path(execution_id): Path<String>,
    Json(request): Json<UpdateExecutionRequest>,
) -> impl IntoResponse {
    info!("Updating execution: {}", execution_id);

    let parse_date = |s: &str| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc));

    let input = AgentExecutionUpdateInput {
        status: request.status,
        completed_at: request.completed_at.as_deref().and_then(parse_date),
        execution_time_seconds: request.execution_time_seconds,
        tokens_input: request.tokens_input,
        tokens_output: request.tokens_output,
        total_cost: request.total_cost,
        response: request.response,
        error_message: request.error_message,
        files_changed: request.files_changed,
        lines_added: request.lines_added,
        lines_removed: request.lines_removed,
        files_created: request.files_created,
        files_modified: request.files_modified,
        files_deleted: request.files_deleted,
        branch_name: request.branch_name,
        commit_hash: request.commit_hash,
        commit_message: request.commit_message,
        pr_number: request.pr_number,
        pr_url: request.pr_url,
        pr_title: request.pr_title,
        pr_status: request.pr_status,
        pr_created_at: request.pr_created_at.as_deref().and_then(parse_date),
        pr_merged_at: request.pr_merged_at.as_deref().and_then(parse_date),
        pr_merge_commit: request.pr_merge_commit,
        review_status: request.review_status,
        review_comments: request.review_comments,
        test_results: request.test_results,
        performance_metrics: request.performance_metrics,
        metadata: request.metadata,
    };

    match db
        .execution_storage
        .update_execution(&execution_id, input)
        .await
    {
        Ok(execution) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(execution))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Delete an execution
pub async fn delete_execution(
    State(db): State<DbState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting execution: {}", execution_id);

    match db.execution_storage.delete_execution(&execution_id).await {
        Ok(_) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success("Execution deleted successfully")),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}

// ==================== PR Reviews ====================

/// List all reviews for an execution
pub async fn list_reviews(
    State(db): State<DbState>,
    Path(execution_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing reviews for execution: {}", execution_id);

    match db.execution_storage.list_reviews(&execution_id).await {
        Ok(reviews) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(reviews))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Get a single review by ID
pub async fn get_review(
    State(db): State<DbState>,
    Path(review_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting review: {}", review_id);

    match db.execution_storage.get_review(&review_id).await {
        Ok(review) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(review))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Request body for creating a review
#[derive(Deserialize)]
pub struct CreateReviewRequest {
    #[serde(rename = "executionId")]
    pub execution_id: String,
    #[serde(rename = "reviewerId")]
    pub reviewer_id: Option<String>,
    #[serde(rename = "reviewerType")]
    pub reviewer_type: ReviewerType,
    #[serde(rename = "reviewStatus")]
    pub review_status: ReviewStatus,
    #[serde(rename = "reviewBody")]
    pub review_body: Option<String>,
    pub comments: Option<serde_json::Value>,
    #[serde(rename = "suggestedChanges")]
    pub suggested_changes: Option<serde_json::Value>,
}

/// Create a new review
pub async fn create_review(
    State(db): State<DbState>,
    Json(request): Json<CreateReviewRequest>,
) -> impl IntoResponse {
    info!("Creating review for execution: {}", request.execution_id);

    let input = PrReviewCreateInput {
        execution_id: request.execution_id,
        reviewer_id: request.reviewer_id,
        reviewer_type: request.reviewer_type,
        review_status: request.review_status,
        review_body: request.review_body,
        comments: request.comments,
        suggested_changes: request.suggested_changes,
    };

    match db.execution_storage.create_review(input).await {
        Ok(review) => {
            (StatusCode::CREATED, ResponseJson(ApiResponse::success(review))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Request body for updating a review
#[derive(Deserialize)]
pub struct UpdateReviewRequest {
    #[serde(rename = "reviewStatus")]
    pub review_status: Option<ReviewStatus>,
    #[serde(rename = "reviewBody")]
    pub review_body: Option<String>,
    pub comments: Option<serde_json::Value>,
    #[serde(rename = "suggestedChanges")]
    pub suggested_changes: Option<serde_json::Value>,
    #[serde(rename = "approvalDate")]
    pub approval_date: Option<String>,
    #[serde(rename = "dismissalReason")]
    pub dismissal_reason: Option<String>,
}

/// Update a review
pub async fn update_review(
    State(db): State<DbState>,
    Path(review_id): Path<String>,
    Json(request): Json<UpdateReviewRequest>,
) -> impl IntoResponse {
    info!("Updating review: {}", review_id);

    let parse_date = |s: &str| DateTime::parse_from_rfc3339(s).ok().map(|dt| dt.with_timezone(&Utc));

    let input = PrReviewUpdateInput {
        review_status: request.review_status,
        review_body: request.review_body,
        comments: request.comments,
        suggested_changes: request.suggested_changes,
        approval_date: request.approval_date.as_deref().and_then(parse_date),
        dismissal_reason: request.dismissal_reason,
    };

    match db.execution_storage.update_review(&review_id, input).await {
        Ok(review) => {
            (StatusCode::OK, ResponseJson(ApiResponse::success(review))).into_response()
        }
        Err(e) => e.into_response(),
    }
}

/// Delete a review
pub async fn delete_review(
    State(db): State<DbState>,
    Path(review_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting review: {}", review_id);

    match db.execution_storage.delete_review(&review_id).await {
        Ok(_) => (
            StatusCode::OK,
            ResponseJson(ApiResponse::success("Review deleted successfully")),
        )
            .into_response(),
        Err(e) => e.into_response(),
    }
}
