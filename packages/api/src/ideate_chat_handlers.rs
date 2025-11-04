// ABOUTME: HTTP request handlers for chat mode PRD discovery
// ABOUTME: Handles chat messages, streaming responses, insights, quality metrics, and PRD generation

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{error, info, warn};

use super::response::ok_or_internal_error;
use orkee_ideate::{
    ChatManager, CreateInsightInput, DiscoveryQuestion, DiscoveryStatus, GeneratePRDFromChatInput,
    GeneratePRDFromChatResult, MessageRole, QualityMetrics, QuestionCategory, SendMessageInput,
    TopicCoverage, ValidationResult,
};
use orkee_projects::DbState;

// TODO: AI functionality moved to frontend - see packages/dashboard/src/services/chat-ai.ts

/// Get chat history for a session
pub async fn get_history(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting chat history for session: {}", session_id);

    let manager = ChatManager::new(db.pool.clone());
    let result = manager.get_history(&session_id).await;

    ok_or_internal_error(result, "Failed to get chat history")
}

/// Send a message in the chat
pub async fn send_message(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(input): Json<SendMessageInput>,
) -> impl IntoResponse {
    info!(
        "Sending message to session: {} (type: {:?})",
        session_id, input.message_type
    );

    let manager = ChatManager::new(db.pool.clone());

    // Determine role - if no role specified, default to User
    // Frontend can send Assistant messages after AI streaming
    let role = input.role.unwrap_or(MessageRole::User);

    let message_result = manager
        .add_message(
            &session_id,
            role.clone(),
            input.content.clone(),
            input.message_type,
            None,
        )
        .await;

    // Note: Insight extraction is now handled by the frontend after AI streaming completes
    // This ensures the user's selected model is used for extraction (Phase 6)

    ok_or_internal_error(message_result, "Failed to send message")
}

/// Query parameters for suggested questions
#[derive(Debug, Deserialize)]
pub struct QuestionsQuery {
    category: Option<String>,
}

/// Get discovery questions, optionally filtered by category
pub async fn get_discovery_questions(
    State(db): State<DbState>,
    Query(query): Query<QuestionsQuery>,
) -> impl IntoResponse {
    info!(
        "Getting discovery questions (category: {:?})",
        query.category
    );

    let category = if let Some(cat_str) = query.category {
        match cat_str.to_lowercase().as_str() {
            "problem" => Some(QuestionCategory::Problem),
            "users" => Some(QuestionCategory::Users),
            "features" => Some(QuestionCategory::Features),
            "technical" => Some(QuestionCategory::Technical),
            "risks" => Some(QuestionCategory::Risks),
            "constraints" => Some(QuestionCategory::Constraints),
            "success" => Some(QuestionCategory::Success),
            _ => {
                warn!("Invalid question category: {}", cat_str);
                None
            }
        }
    } else {
        None
    };

    let manager = ChatManager::new(db.pool.clone());
    let result = manager.get_discovery_questions(category).await;

    ok_or_internal_error(result, "Failed to get discovery questions")
}

/// Get suggested questions based on chat context
pub async fn get_suggested_questions(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting suggested questions for session: {}", session_id);

    let manager = ChatManager::new(db.pool.clone());

    // Get chat history to analyze context
    let _history = match manager.get_history(&session_id).await {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to get chat history: {}", e);
            return ok_or_internal_error(
                Err::<Vec<DiscoveryQuestion>, _>(e),
                "Failed to get chat history",
            );
        }
    };

    // TODO: Use AI SDK to analyze chat and suggest contextual questions
    // For now, return high-priority required questions that haven't been covered
    let all_questions = match manager.get_discovery_questions(None).await {
        Ok(q) => q,
        Err(e) => {
            error!("Failed to get discovery questions: {}", e);
            return ok_or_internal_error(
                Err::<Vec<DiscoveryQuestion>, _>(e),
                "Failed to get discovery questions",
            );
        }
    };

    // Filter to required questions with high priority
    let suggested: Vec<DiscoveryQuestion> = all_questions
        .into_iter()
        .filter(|q| q.is_required && q.priority >= 8)
        .take(3)
        .collect();

    ok_or_internal_error(
        Ok::<_, orkee_ideate::IdeateError>(suggested),
        "Failed to get suggested questions",
    )
}

/// Get insights extracted from the chat
pub async fn get_insights(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting insights for session: {}", session_id);

    let manager = ChatManager::new(db.pool.clone());
    let result = manager.get_insights(&session_id).await;

    ok_or_internal_error(result, "Failed to get insights")
}

/// Create a new insight
pub async fn create_insight(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(input): Json<CreateInsightInput>,
) -> impl IntoResponse {
    info!(
        "Creating insight for session: {} (type: {:?})",
        session_id, input.insight_type
    );

    let manager = ChatManager::new(db.pool.clone());
    let result = manager.create_insight(&session_id, input).await;

    ok_or_internal_error(result, "Failed to create insight")
}

/// Calculate quality metrics for the chat
pub async fn get_quality_metrics(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting quality metrics for session: {}", session_id);

    let manager = ChatManager::new(db.pool.clone());

    // Get insights to calculate coverage
    let insights = match manager.get_insights(&session_id).await {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to get insights: {}", e);
            return ok_or_internal_error(Err::<QualityMetrics, _>(e), "Failed to get insights");
        }
    };

    // TODO: Use AI SDK to calculate more sophisticated quality metrics
    // For now, basic coverage based on insights

    let has_requirement = insights
        .iter()
        .any(|i| matches!(i.insight_type, orkee_ideate::InsightType::Requirement));
    let has_constraint = insights
        .iter()
        .any(|i| matches!(i.insight_type, orkee_ideate::InsightType::Constraint));
    let has_risk = insights
        .iter()
        .any(|i| matches!(i.insight_type, orkee_ideate::InsightType::Risk));

    let coverage = TopicCoverage {
        problem: has_requirement,
        users: has_requirement,
        features: has_requirement,
        technical: has_constraint,
        risks: has_risk,
        constraints: has_constraint,
        success: false, // TODO: Detect from insights
    };

    let covered_areas = [
        coverage.problem,
        coverage.users,
        coverage.features,
        coverage.technical,
        coverage.risks,
        coverage.constraints,
        coverage.success,
    ]
    .iter()
    .filter(|&&x| x)
    .count();

    let quality_score = ((covered_areas as f32 / 7.0) * 100.0) as i32;

    let mut missing_areas = Vec::new();
    if !coverage.problem {
        missing_areas.push("problem".to_string());
    }
    if !coverage.users {
        missing_areas.push("users".to_string());
    }
    if !coverage.features {
        missing_areas.push("features".to_string());
    }
    if !coverage.technical {
        missing_areas.push("technical".to_string());
    }
    if !coverage.risks {
        missing_areas.push("risks".to_string());
    }
    if !coverage.constraints {
        missing_areas.push("constraints".to_string());
    }
    if !coverage.success {
        missing_areas.push("success".to_string());
    }

    let is_ready_for_prd = quality_score >= 60 && covered_areas >= 5;

    let metrics = QualityMetrics {
        quality_score,
        missing_areas,
        coverage,
        is_ready_for_prd,
    };

    ok_or_internal_error(
        Ok::<_, orkee_ideate::IdeateError>(metrics),
        "Failed to calculate quality metrics",
    )
}

/// Update discovery status
#[derive(Debug, Deserialize)]
pub struct UpdateStatusRequest {
    status: String,
}

pub async fn update_status(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateStatusRequest>,
) -> impl IntoResponse {
    info!(
        "Updating discovery status for session: {} to {}",
        session_id, request.status
    );

    let status = match request.status.to_lowercase().as_str() {
        "draft" => DiscoveryStatus::Draft,
        "brainstorming" => DiscoveryStatus::Brainstorming,
        "refining" => DiscoveryStatus::Refining,
        "validating" => DiscoveryStatus::Validating,
        "finalized" => DiscoveryStatus::Finalized,
        _ => {
            return ok_or_internal_error(
                Err::<(), _>(orkee_ideate::IdeateError::InvalidInput(format!(
                    "Invalid discovery status: {}",
                    request.status
                ))),
                "Invalid discovery status",
            );
        }
    };

    let manager = ChatManager::new(db.pool.clone());
    let result = manager.update_discovery_status(&session_id, status).await;

    ok_or_internal_error(result, "Failed to update discovery status")
}

/// Generate PRD from chat
pub async fn generate_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(input): Json<GeneratePRDFromChatInput>,
) -> impl IntoResponse {
    info!(
        "Generating PRD from chat for session: {} (title: {})",
        session_id, input.title
    );

    let manager = ChatManager::new(db.pool.clone());

    // Get chat history
    let history = match manager.get_history(&session_id).await {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to get chat history: {}", e);
            return ok_or_internal_error(
                Err::<GeneratePRDFromChatResult, _>(e),
                "Failed to get chat history",
            );
        }
    };

    // Get insights
    let insights = match manager.get_insights(&session_id).await {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to get insights: {}", e);
            return ok_or_internal_error(
                Err::<GeneratePRDFromChatResult, _>(e),
                "Failed to get insights",
            );
        }
    };

    // TODO: Use AI SDK streamObject to generate structured PRD from chat history
    // For now, return a placeholder
    let prd_id = nanoid::nanoid!(12);
    let content_markdown = format!(
        "# {}\n\n## Overview\n\nGenerated from {} messages and {} insights.\n\n",
        input.title,
        history.len(),
        insights.len()
    );

    let result = GeneratePRDFromChatResult {
        prd_id,
        content_markdown,
        quality_score: 75,
    };

    ok_or_internal_error(
        Ok::<_, orkee_ideate::IdeateError>(result),
        "Failed to generate PRD",
    )
}

/// Validate chat readiness for PRD generation
pub async fn validate_for_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Validating chat for PRD generation: {}", session_id);

    let manager = ChatManager::new(db.pool.clone());

    // Get chat history
    let history = match manager.get_history(&session_id).await {
        Ok(h) => h,
        Err(e) => {
            error!("Failed to get chat history: {}", e);
            return ok_or_internal_error(
                Err::<ValidationResult, _>(e),
                "Failed to get chat history",
            );
        }
    };

    // Get insights to check coverage
    let insights = match manager.get_insights(&session_id).await {
        Ok(i) => i,
        Err(e) => {
            error!("Failed to get insights: {}", e);
            return ok_or_internal_error(Err::<ValidationResult, _>(e), "Failed to get insights");
        }
    };

    let has_requirement = insights
        .iter()
        .any(|i| matches!(i.insight_type, orkee_ideate::InsightType::Requirement));
    let _has_constraint = insights
        .iter()
        .any(|i| matches!(i.insight_type, orkee_ideate::InsightType::Constraint));

    let mut missing_required = Vec::new();
    let mut warnings = Vec::new();

    if history.len() < 3 {
        missing_required.push("Need at least 3 messages in chat".to_string());
    }

    if !has_requirement {
        missing_required.push("No requirements identified yet".to_string());
    }

    if insights.len() < 3 {
        warnings.push("Consider exploring more areas to improve PRD quality".to_string());
    }

    let is_valid = missing_required.is_empty();

    let validation = ValidationResult {
        is_valid,
        missing_required,
        warnings,
    };

    ok_or_internal_error(
        Ok::<_, orkee_ideate::IdeateError>(validation),
        "Failed to validate chat",
    )
}

// TODO: reanalyze_insights and extract_and_save_insights handlers removed
// AI functionality moved to frontend (chat-ai.ts:extractInsights)
