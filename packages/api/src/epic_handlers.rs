// ABOUTME: HTTP request handlers for Epic operations (CCPM workflow)
// ABOUTME: Handles CRUD operations, generation, task decomposition, and progress tracking for Epics

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};
use orkee_ideate::{
    ComplexityAnalyzer, CreateEpicInput, Epic, EpicComplexity, EpicManager, EpicStatus,
    EstimatedEffort, ExecutionTracker, UpdateEpicInput,
};
use orkee_projects::DbState;

/// List all Epics for a project
pub async fn list_epics(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing epics for project: {}", project_id);

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.list_epics(&project_id).await;

    ok_or_internal_error(result, "Failed to list epics")
}

/// Get a single Epic by ID
pub async fn get_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Getting epic: {} for project: {}", epic_id, project_id);

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.get_epic(&project_id, &epic_id).await;

    ok_or_not_found(result, "Epic not found")
}

/// List Epics by PRD
pub async fn list_epics_by_prd(
    State(db): State<DbState>,
    Path((project_id, prd_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Listing epics for PRD: {} in project: {}",
        prd_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.list_epics_by_prd(&project_id, &prd_id).await;

    ok_or_internal_error(result, "Failed to list epics for PRD")
}

/// Request body for creating an Epic
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateEpicRequest {
    pub prd_id: String,
    pub name: String,
    pub overview_markdown: String,
    pub architecture_decisions: Option<Vec<orkee_ideate::ArchitectureDecision>>,
    pub technical_approach: String,
    pub implementation_strategy: Option<String>,
    pub dependencies: Option<Vec<orkee_ideate::ExternalDependency>>,
    pub success_criteria: Option<Vec<orkee_ideate::SuccessCriterion>>,
    pub task_categories: Option<Vec<String>>,
    pub estimated_effort: Option<EstimatedEffort>,
    pub complexity: Option<EpicComplexity>,
}

/// Create a new Epic
pub async fn create_epic(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
    Json(request): Json<CreateEpicRequest>,
) -> impl IntoResponse {
    info!(
        "Creating epic '{}' for project: {}",
        request.name, project_id
    );

    let input = CreateEpicInput {
        prd_id: request.prd_id,
        name: request.name,
        overview_markdown: request.overview_markdown,
        architecture_decisions: request.architecture_decisions,
        technical_approach: request.technical_approach,
        implementation_strategy: request.implementation_strategy,
        dependencies: request.dependencies,
        success_criteria: request.success_criteria,
        task_categories: request.task_categories,
        estimated_effort: request.estimated_effort,
        complexity: request.complexity,
    };

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.create_epic(&project_id, input).await;

    created_or_internal_error(result, "Failed to create epic")
}

/// Request body for updating an Epic
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEpicRequest {
    pub name: Option<String>,
    pub overview_markdown: Option<String>,
    pub architecture_decisions: Option<Vec<orkee_ideate::ArchitectureDecision>>,
    pub technical_approach: Option<String>,
    pub implementation_strategy: Option<String>,
    pub dependencies: Option<Vec<orkee_ideate::ExternalDependency>>,
    pub success_criteria: Option<Vec<orkee_ideate::SuccessCriterion>>,
    pub task_categories: Option<Vec<String>>,
    pub estimated_effort: Option<EstimatedEffort>,
    pub complexity: Option<EpicComplexity>,
    pub status: Option<EpicStatus>,
    pub progress_percentage: Option<i32>,
}

/// Update an existing Epic
pub async fn update_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
    Json(request): Json<UpdateEpicRequest>,
) -> impl IntoResponse {
    info!("Updating epic: {} for project: {}", epic_id, project_id);

    let input = UpdateEpicInput {
        name: request.name,
        overview_markdown: request.overview_markdown,
        architecture_decisions: request.architecture_decisions,
        technical_approach: request.technical_approach,
        implementation_strategy: request.implementation_strategy,
        dependencies: request.dependencies,
        success_criteria: request.success_criteria,
        task_categories: request.task_categories,
        estimated_effort: request.estimated_effort,
        complexity: request.complexity,
        status: request.status,
        progress_percentage: request.progress_percentage,
    };

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.update_epic(&project_id, &epic_id, input).await;

    ok_or_internal_error(result, "Failed to update epic")
}

/// Delete an Epic
pub async fn delete_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Deleting epic: {} from project: {}", epic_id, project_id);

    let manager = EpicManager::new(db.pool.clone());
    let result = manager.delete_epic(&project_id, &epic_id).await;

    ok_or_internal_error(result, "Failed to delete epic")
}

/// Get tasks for an Epic
pub async fn get_epic_tasks(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting tasks for epic: {} in project: {}",
        epic_id, project_id
    );

    // Query tasks table for this epic
    let result = sqlx::query_as::<_, (String,)>(
        r#"
        SELECT id FROM tasks WHERE epic_id = ?
        "#,
    )
    .bind(&epic_id)
    .fetch_all(&db.pool)
    .await
    .map(|rows| rows.into_iter().map(|(id,)| id).collect::<Vec<_>>());

    ok_or_internal_error(result, "Failed to get epic tasks")
}

/// Calculate Epic progress
#[derive(Serialize)]
pub struct ProgressResponse {
    progress: i32,
}

pub async fn calculate_epic_progress(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Calculating progress for epic: {} in project: {}",
        epic_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let result = manager
        .calculate_progress(&project_id, &epic_id)
        .await
        .map(|progress| ProgressResponse { progress });

    ok_or_internal_error(result, "Failed to calculate epic progress")
}

/// Request body for generating an Epic from a PRD
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateEpicRequest {
    pub prd_id: String,
    pub include_task_breakdown: Option<bool>,
}

/// Response for Epic generation
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GenerateEpicResponse {
    pub epic_id: String,
    pub tasks_created: Option<usize>,
}

/// Generate an Epic from a PRD (placeholder - AI generation to be implemented)
pub async fn generate_epic_from_prd(
    State(_db): State<DbState>,
    Path(_project_id): Path<String>,
    Json(_request): Json<GenerateEpicRequest>,
) -> impl IntoResponse {
    info!("Epic generation from PRD - not yet implemented");

    // TODO: Implement AI-powered Epic generation
    // This will:
    // 1. Load PRD content
    // 2. Analyze technical requirements
    // 3. Generate architecture decisions
    // 4. Create implementation strategy
    // 5. Optionally decompose to tasks

    let error = orkee_ideate::IdeateError::NotImplemented(
        "Epic generation from PRD is not yet implemented".to_string(),
    );
    ok_or_internal_error(Err::<Epic, _>(error), "Epic generation not implemented")
}

/// Analyze work streams for an Epic (placeholder - to be implemented)
pub async fn analyze_work_streams(
    State(_db): State<DbState>,
    Path((_project_id, _epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!("Work stream analysis - not yet implemented");

    // TODO: Implement work stream analysis
    // This will:
    // 1. Load tasks for epic
    // 2. Analyze file patterns
    // 3. Group tasks into work streams
    // 4. Detect conflicts
    // 5. Generate parallelization strategy

    let error = orkee_ideate::IdeateError::NotImplemented(
        "Work stream analysis is not yet implemented".to_string(),
    );
    ok_or_internal_error(
        Err::<orkee_ideate::WorkAnalysis, _>(error),
        "Work stream analysis not implemented",
    )
}

/// Analyze Epic complexity
pub async fn analyze_complexity(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Analyzing complexity for epic: {} in project: {}",
        epic_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let epic_result = manager.get_epic(&project_id, &epic_id).await;

    let epic = match epic_result {
        Ok(Some(epic)) => epic,
        Ok(None) => {
            return ok_or_not_found::<orkee_ideate::ComplexityReport, orkee_ideate::IdeateError>(
                Err(orkee_ideate::IdeateError::NotFound("Epic not found".to_string())),
                "Epic not found",
            )
        }
        Err(e) => {
            return ok_or_internal_error::<orkee_ideate::ComplexityReport, orkee_ideate::IdeateError>(
                Err(e),
                "Failed to get epic",
            )
        }
    };

    let analyzer = ComplexityAnalyzer::new();
    let user_limit = epic.task_count_limit;
    let result = analyzer.analyze_epic(&epic, user_limit);

    ok_or_internal_error(result, "Failed to analyze complexity")
}

/// Request body for simplification analysis
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimplifyRequest {
    pub current_task_count: usize,
}

/// Response for simplification analysis
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimplifyResponse {
    pub suggestions: Vec<SimplificationSuggestion>,
    pub target_task_count: usize,
    pub potential_savings: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimplificationSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub task_ids: Vec<String>,
    pub estimated_reduction: usize,
}

/// Get simplification suggestions for an Epic
pub async fn simplify_epic(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
    Json(request): Json<SimplifyRequest>,
) -> impl IntoResponse {
    info!(
        "Getting simplification suggestions for epic: {} in project: {}",
        epic_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let epic_result = manager.get_epic(&project_id, &epic_id).await;

    let epic = match epic_result {
        Ok(Some(epic)) => epic,
        Ok(None) => {
            return ok_or_not_found::<SimplifyResponse, orkee_ideate::IdeateError>(
                Err(orkee_ideate::IdeateError::NotFound("Epic not found".to_string())),
                "Epic not found",
            )
        }
        Err(e) => {
            return ok_or_internal_error::<SimplifyResponse, orkee_ideate::IdeateError>(
                Err(e),
                "Failed to get epic",
            )
        }
    };

    let target_limit = epic.task_count_limit.unwrap_or(20) as usize;
    let mut suggestions = Vec::new();
    let mut potential_savings = 0;

    // Suggest combining similar tasks
    if request.current_task_count > target_limit {
        let overhead = request.current_task_count - target_limit;
        suggestions.push(SimplificationSuggestion {
            suggestion_type: "combine_similar".to_string(),
            description: format!(
                "Combine similar tasks to reduce count by approximately {} tasks",
                overhead / 2
            ),
            task_ids: Vec::new(), // Would be populated by actual task analysis
            estimated_reduction: overhead / 2,
        });
        potential_savings += overhead / 2;
    }

    // Suggest leveraging existing code
    if let Some(context) = &epic.codebase_context {
        if context.get("similar_features").is_some() {
            suggestions.push(SimplificationSuggestion {
                suggestion_type: "leverage_existing".to_string(),
                description: "Use existing similar features to reduce implementation tasks"
                    .to_string(),
                task_ids: Vec::new(),
                estimated_reduction: 2,
            });
            potential_savings += 2;
        }
    }

    // Suggest deferring non-critical work
    suggestions.push(SimplificationSuggestion {
        suggestion_type: "defer_non_critical".to_string(),
        description: "Move nice-to-have features to a future phase".to_string(),
        task_ids: Vec::new(),
        estimated_reduction: 3,
    });
    potential_savings += 3;

    let response = SimplifyResponse {
        suggestions,
        target_task_count: target_limit,
        potential_savings: potential_savings.min(request.current_task_count - target_limit),
    };

    ok_or_internal_error::<SimplifyResponse, orkee_ideate::IdeateError>(
        Ok(response),
        "Failed to generate simplification suggestions",
    )
}

/// Response for leverage analysis
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LeverageAnalysisResponse {
    pub reusable_components: Vec<ReusableComponent>,
    pub similar_features: Vec<SimilarFeature>,
    pub existing_patterns: Vec<ExistingPattern>,
    pub estimated_time_savings: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReusableComponent {
    pub name: String,
    pub file_path: String,
    pub description: String,
    pub usage_example: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarFeature {
    pub name: String,
    pub location: String,
    pub similarity_score: u8,
    pub adaptation_notes: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExistingPattern {
    pub pattern_name: String,
    pub description: String,
    pub example_location: String,
    pub recommended_usage: String,
}

/// Get leverage analysis for an Epic
pub async fn get_leverage_analysis(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Getting leverage analysis for epic: {} in project: {}",
        epic_id, project_id
    );

    let manager = EpicManager::new(db.pool.clone());
    let epic_result = manager.get_epic(&project_id, &epic_id).await;

    let epic = match epic_result {
        Ok(Some(epic)) => epic,
        Ok(None) => {
            return ok_or_not_found::<LeverageAnalysisResponse, orkee_ideate::IdeateError>(
                Err(orkee_ideate::IdeateError::NotFound("Epic not found".to_string())),
                "Epic not found",
            )
        }
        Err(e) => {
            return ok_or_internal_error::<LeverageAnalysisResponse, orkee_ideate::IdeateError>(
                Err(e),
                "Failed to get epic",
            )
        }
    };

    let mut reusable_components = Vec::new();
    let mut similar_features = Vec::new();
    let mut existing_patterns = Vec::new();

    // Parse codebase_context if available
    if let Some(context) = &epic.codebase_context {
        // Extract reusable components
        if let Some(components) = context.get("reusable_components").and_then(|c| c.as_array()) {
            for component in components {
                if let (Some(name), Some(path)) = (
                    component.get("name").and_then(|n| n.as_str()),
                    component.get("path").and_then(|p| p.as_str()),
                ) {
                    reusable_components.push(ReusableComponent {
                        name: name.to_string(),
                        file_path: path.to_string(),
                        description: component
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("Reusable component")
                            .to_string(),
                        usage_example: component
                            .get("usage")
                            .and_then(|u| u.as_str())
                            .unwrap_or("See documentation")
                            .to_string(),
                    });
                }
            }
        }

        // Extract similar features
        if let Some(features) = context.get("similar_features").and_then(|f| f.as_array()) {
            for feature in features {
                if let (Some(name), Some(location)) = (
                    feature.get("name").and_then(|n| n.as_str()),
                    feature.get("location").and_then(|l| l.as_str()),
                ) {
                    similar_features.push(SimilarFeature {
                        name: name.to_string(),
                        location: location.to_string(),
                        similarity_score: feature
                            .get("similarity")
                            .and_then(|s| s.as_u64())
                            .unwrap_or(70) as u8,
                        adaptation_notes: feature
                            .get("notes")
                            .and_then(|n| n.as_str())
                            .unwrap_or("Can be adapted for this use case")
                            .to_string(),
                    });
                }
            }
        }

        // Extract existing patterns
        if let Some(patterns) = context.get("patterns").and_then(|p| p.as_array()) {
            for pattern in patterns {
                if let Some(name) = pattern.get("name").and_then(|n| n.as_str()) {
                    existing_patterns.push(ExistingPattern {
                        pattern_name: name.to_string(),
                        description: pattern
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("Established pattern in codebase")
                            .to_string(),
                        example_location: pattern
                            .get("example")
                            .and_then(|e| e.as_str())
                            .unwrap_or("See codebase")
                            .to_string(),
                        recommended_usage: pattern
                            .get("usage")
                            .and_then(|u| u.as_str())
                            .unwrap_or("Follow this pattern for consistency")
                            .to_string(),
                    });
                }
            }
        }
    }

    // Estimate time savings
    let total_opportunities =
        reusable_components.len() + similar_features.len() + existing_patterns.len();
    let estimated_time_savings = if total_opportunities > 0 {
        format!(
            "Approximately {}-{} hours by leveraging existing code",
            total_opportunities * 2,
            total_opportunities * 4
        )
    } else {
        "No significant reuse opportunities identified yet".to_string()
    };

    let response = LeverageAnalysisResponse {
        reusable_components,
        similar_features,
        existing_patterns,
        estimated_time_savings,
    };

    ok_or_internal_error::<LeverageAnalysisResponse, orkee_ideate::IdeateError>(
        Ok(response),
        "Failed to get leverage analysis",
    )
}

/// Generate execution checkpoints for an Epic
pub async fn generate_epic_checkpoints(
    State(db): State<DbState>,
    Path((project_id, epic_id)): Path<(String, String)>,
) -> impl IntoResponse {
    info!(
        "Generating checkpoints for epic: {} in project: {}",
        epic_id, project_id
    );

    let tracker = ExecutionTracker::new(db.pool.clone());
    let result = tracker.generate_checkpoints(&epic_id).await;

    ok_or_internal_error(result, "Failed to generate checkpoints")
}
