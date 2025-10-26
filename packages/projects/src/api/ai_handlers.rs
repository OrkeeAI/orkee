// ABOUTME: HTTP request handlers for AI operations using real Anthropic API calls
// ABOUTME: Implements structured generation for PRD analysis, spec generation, and task suggestions

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};

use super::auth::CurrentUser;
use super::response::ApiResponse;
use crate::ai_service::{AIService, AIServiceError};
use crate::ai_usage_logs::AiUsageLog;
use crate::db::DbState;
use openspec::db as openspec_db;
use tasks::{TaskCreateInput, TaskPriority};

// ============================================================================
// Shared Types (Re-exported from openspec for backward compatibility)
// ============================================================================

// Re-export AI analysis types from openspec module
pub use openspec::ai_types::{
    PRDAnalysisData, SpecCapability, SpecRequirement, SpecScenario, TaskSuggestion,
};

// ============================================================================
// Spec Generation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct SpecGenerationRequirement {
    name: String,
    description: String,
    scenarios: Vec<SpecScenario>,
}

#[derive(Debug, Serialize, Deserialize)]
struct SpecGenerationData {
    requirements: Vec<SpecGenerationRequirement>,
}

// ============================================================================
// Task Suggestions
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct TaskSuggestionItem {
    title: String,
    description: String,
    priority: String,
    complexity_score: i32,
    linked_requirements: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskSuggestionsData {
    tasks: Vec<TaskSuggestionItem>,
}

// ============================================================================
// Spec Refinement
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct SpecRefinementData {
    refined_spec: String,
    changes_made: Vec<String>,
}

// ============================================================================
// Completion Validation
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
struct ValidationItem {
    scenario: String,
    passed: bool,
    confidence: f32,
    notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CompletionValidationData {
    is_complete: bool,
    validation_results: Vec<ValidationItem>,
    overall_confidence: f32,
    recommendations: Vec<String>,
}

// ============================================================================
// PRD Analysis
// ============================================================================

#[derive(Deserialize)]
pub struct AnalyzePRDRequest {
    #[serde(rename = "prdId")]
    pub prd_id: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
    pub provider: String,
    pub model: String,
}

#[derive(Serialize)]
pub struct AnalyzePRDResponse {
    #[serde(rename = "prdId")]
    pub prd_id: String,
    pub analysis: PRDAnalysisData,
    #[serde(rename = "tokenUsage")]
    pub token_usage: TokenUsage,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenUsage {
    pub input: u32,
    pub output: u32,
    pub total: u32,
}

/// Analyze PRD with AI and extract capabilities
pub async fn analyze_prd(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(request): Json<AnalyzePRDRequest>,
) -> impl IntoResponse {
    info!("AI PRD analysis requested for PRD: {}", request.prd_id);

    // Get API key from database (with environment variable fallback)
    let api_key = match db
        .user_storage
        .get_api_key(&current_user.id, "anthropic")
        .await
    {
        Ok(Some(key)) => {
            info!(
                "Using Anthropic API key from database (starts with: {})",
                &key.chars().take(15).collect::<String>()
            );
            key
        }
        Ok(None) => {
            // Fallback to environment variable
            match env::var("ANTHROPIC_API_KEY") {
                Ok(key) => {
                    info!("Using Anthropic API key from environment variable");
                    key
                }
                Err(_) => {
                    error!("No Anthropic API key found in database or environment");
                    return (
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::<()>::error(
                            "Anthropic API key not configured. Please set it in Settings → Security or via ANTHROPIC_API_KEY environment variable.".to_string()
                        ))
                    ).into_response();
                }
            }
        }
        Err(e) => {
            error!("Failed to fetch API key from database: {}", e);
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error(format!(
                    "Failed to retrieve API key: {}",
                    e
                ))),
            )
                .into_response();
        }
    };

    // Initialize AI service with the API key and selected model
    info!(
        "Using model: {} from provider: {}",
        request.model, request.provider
    );
    let ai_service = AIService::with_api_key_and_model(api_key, request.model.clone());

    // Build the prompt matching TypeScript implementation
    let user_prompt = format!(
        r#"Analyze the following Product Requirements Document (PRD) and extract structured information.

PRD Content:
{}

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{{
  "summary": "Executive summary of the PRD",
  "capabilities": [
    {{
      "id": "kebab-case-id",
      "name": "Human Readable Name",
      "purpose": "Purpose and context",
      "requirements": [
        {{
          "name": "Requirement Name",
          "content": "Detailed requirement description",
          "scenarios": [
            {{
              "name": "Scenario name",
              "when": "WHEN condition",
              "then": "THEN outcome",
              "and": ["AND condition 1", "AND condition 2"]
            }}
          ]
        }}
      ]
    }}
  ],
  "suggestedTasks": [
    {{
      "title": "Task title",
      "description": "Task description",
      "capabilityId": "capability-id",
      "requirementName": "Requirement Name",
      "complexity": 5,
      "estimatedHours": 8.0,
      "priority": "high"
    }}
  ],
  "dependencies": ["External dependency 1"],
  "technicalConsiderations": ["Technical consideration 1"]
}}"#,
        request.content_markdown
    );

    let system_prompt = Some(
        r#"You are an expert software architect creating OpenSpec change proposals from PRDs.

CRITICAL FORMAT REQUIREMENTS:
1. Every requirement MUST use: ### Requirement: [Name]
2. Every scenario MUST use: #### Scenario: [Name] (exactly 4 hashtags)
3. Scenarios MUST follow this bullet format:
   - **WHEN** [condition]
   - **THEN** [outcome]
   - **AND** [additional] (optional)
4. Requirements MUST use SHALL or MUST (never should/may)
5. Every requirement MUST have at least one scenario

Generate:
1. Executive summary for proposal
2. Capability specifications using:
   ## ADDED Requirements
   [requirements with proper format]
3. Implementation tasks (specific and actionable)
4. Technical considerations (if complex)

Example of correct format:
## ADDED Requirements
### Requirement: User Authentication
The system SHALL provide secure user authentication using JWT tokens.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
- **AND** the token expires after 24 hours

Rules:
- Use kebab-case for capability IDs (e.g., "user-auth")
- Complexity scores: 1-10 (1=trivial, 10=very complex)
- Priority: low, medium, or high
- Be specific and testable

RESPOND WITH ONLY VALID JSON."#
            .to_string(),
    );

    // Get the model being used for logging
    let model_name = ai_service.model().to_string();

    // Make AI call
    match ai_service
        .generate_structured::<PRDAnalysisData>(user_prompt, system_prompt)
        .await
    {
        Ok(ai_response) => {
            let token_usage = TokenUsage {
                input: ai_response.usage.input_tokens,
                output: ai_response.usage.output_tokens,
                total: ai_response.usage.total_tokens(),
            };

            // Get the PRD to obtain project_id
            let prd = match openspec_db::get_prd(&db.pool, &request.prd_id).await {
                Ok(prd) => prd,
                Err(e) => {
                    error!("Failed to fetch PRD {}: {}", request.prd_id, e);
                    return (
                        StatusCode::NOT_FOUND,
                        ResponseJson(ApiResponse::<()>::error(format!("PRD not found: {}", e))),
                    )
                        .into_response();
                }
            };

            let project_id = &prd.project_id;
            info!(
                "Creating OpenSpec change proposal for project: {}",
                project_id
            );

            // Create change proposal from analysis using OpenSpec workflow
            let change = match openspec::create_change_from_analysis(
                &db.pool,
                project_id,
                &request.prd_id,
                &ai_response.data,
                &current_user.id,
            )
            .await
            {
                Ok(change) => {
                    info!(
                        "Created change proposal: {} (status: {:?})",
                        change.id, change.status
                    );
                    change
                }
                Err(e) => {
                    error!("Failed to create change proposal: {}", e);
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        ResponseJson(ApiResponse::<()>::error(format!(
                            "Failed to create change proposal: {}",
                            e
                        ))),
                    )
                        .into_response();
                }
            };

            // Validate the created change deltas
            use openspec::OpenSpecMarkdownValidator;
            let validator = OpenSpecMarkdownValidator::new(false); // Use relaxed mode for now

            let deltas = match openspec_db::get_deltas_by_change(&db.pool, &change.id).await {
                Ok(deltas) => deltas,
                Err(e) => {
                    error!("Failed to fetch deltas for validation: {}", e);
                    vec![] // Continue without validation if fetch fails
                }
            };

            let mut validation_errors = Vec::new();
            for delta in &deltas {
                let errors = validator.validate_delta_markdown(&delta.delta_markdown);
                if !errors.is_empty() {
                    error!(
                        "Validation errors in delta for {}: {:?}",
                        delta.capability_name, errors
                    );
                    validation_errors.extend(errors);
                }
            }

            // Update change validation status
            let validation_status = if validation_errors.is_empty() {
                "valid"
            } else {
                "invalid"
            };

            if let Err(e) = sqlx::query(
                "UPDATE spec_changes SET validation_status = ?, validation_errors = ? WHERE id = ?",
            )
            .bind(validation_status)
            .bind(if validation_errors.is_empty() {
                None
            } else {
                Some(serde_json::to_string(&validation_errors).unwrap_or_default())
            })
            .bind(&change.id)
            .execute(&db.pool)
            .await
            {
                error!("Failed to update change validation status: {}", e);
            }

            // Save suggested tasks to database
            info!(
                "Creating {} suggested tasks",
                ai_response.data.suggested_tasks.len()
            );
            for (task_index, task_suggestion) in ai_response.data.suggested_tasks.iter().enumerate()
            {
                // Convert priority string to TaskPriority enum
                let priority = match task_suggestion.priority.to_lowercase().as_str() {
                    "high" => TaskPriority::High,
                    "low" => TaskPriority::Low,
                    _ => TaskPriority::Medium,
                };

                let task_input = TaskCreateInput {
                    title: task_suggestion.title.clone(),
                    description: Some(task_suggestion.description.clone()),
                    status: None, // Will default to Pending
                    priority: Some(priority),
                    assigned_agent_id: None,
                    parent_id: None,
                    position: Some(task_index as i32),
                    dependencies: None,
                    due_date: None,
                    estimated_hours: task_suggestion.estimated_hours.map(|h| h as f64),
                    complexity_score: Some(task_suggestion.complexity as i32),
                    details: None,
                    test_strategy: None,
                    acceptance_criteria: None,
                    prompt: None,
                    context: None,
                    tag_id: None,
                    tags: None,
                    category: Some(task_suggestion.capability_id.clone()),
                };

                match db
                    .task_storage
                    .create_task(project_id, &current_user.id, task_input)
                    .await
                {
                    Ok(task) => {
                        info!("Created task: {} ({})", task_suggestion.title, task.id);
                    }
                    Err(e) => {
                        error!("Failed to create task {}: {}", task_suggestion.title, e);
                    }
                }
            }

            // Log AI usage to database
            let usage_log = AiUsageLog {
                id: nanoid::nanoid!(10),
                project_id: project_id.to_string(),
                request_id: None,
                operation: "analyzePRD".to_string(),
                provider: "anthropic".to_string(),
                model: model_name,
                input_tokens: Some(ai_response.usage.input_tokens as i64),
                output_tokens: Some(ai_response.usage.output_tokens as i64),
                total_tokens: Some(ai_response.usage.total_tokens() as i64),
                estimated_cost: Some(calculate_cost(
                    ai_response.usage.input_tokens,
                    ai_response.usage.output_tokens,
                )),
                duration_ms: Some(0), // TODO: Track actual duration
                error: None,
                created_at: chrono::Utc::now(),
            };

            // Save to database (non-blocking)
            if let Err(e) = db.ai_usage_log_storage.create_log(&usage_log).await {
                error!("Failed to log AI usage: {}", e);
                // Continue anyway - logging failure shouldn't block the response
            }

            info!("Successfully saved PRD analysis to database");

            let response = AnalyzePRDResponse {
                prd_id: request.prd_id,
                analysis: ai_response.data,
                token_usage,
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("AI PRD analysis failed: {}", e);
            let error_message = match e {
                AIServiceError::NoApiKey => {
                    "Anthropic API key not configured. Please set ANTHROPIC_API_KEY environment variable.".to_string()
                }
                AIServiceError::ApiError(msg) => format!("Anthropic API error: {}", msg),
                AIServiceError::ParseError(msg) => {
                    format!("Failed to parse AI response. The model may have returned invalid JSON: {}", msg)
                }
                AIServiceError::RequestFailed(e) => format!("Request failed: {}", e),
                AIServiceError::InvalidResponse => {
                    "Invalid response from AI service".to_string()
                }
            };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(error_message)),
            )
                .into_response()
        }
    }
}

/// Calculate cost for Anthropic API usage
/// Claude 3.5 Sonnet pricing: $3/MTok input, $15/MTok output
fn calculate_cost(input_tokens: u32, output_tokens: u32) -> f64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * 3.0;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * 15.0;
    input_cost + output_cost
}

// ============================================================================
// Spec and Task Management Endpoints
// ============================================================================

/// Generate spec from requirements
pub async fn generate_spec(
    State(db): State<DbState>,
    Json(request): Json<GenerateSpecRequest>,
) -> impl IntoResponse {
    info!(
        "AI spec generation requested for: {}",
        request.capability_name
    );

    // Initialize AI service
    let ai_service = AIService::new();

    // Build requirements list for the prompt
    let requirements_text = if request.requirements.is_empty() {
        "No specific requirements provided".to_string()
    } else {
        request
            .requirements
            .iter()
            .enumerate()
            .map(|(i, req)| format!("{}. {}", i + 1, req))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let user_prompt = format!(
        r#"Generate a detailed specification for the following capability.

Capability: {}
Purpose: {}

Requirements to address:
{}

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{{
  "requirements": [
    {{
      "name": "Requirement Name",
      "description": "Detailed description of what this requirement addresses",
      "scenarios": [
        {{
          "name": "Scenario name",
          "when": "WHEN condition",
          "then": "THEN expected outcome",
          "and": ["AND additional condition 1", "AND additional condition 2"]
        }}
      ]
    }}
  ]
}}"#,
        request.capability_name,
        request
            .purpose
            .as_ref()
            .unwrap_or(&"Core functionality".to_string()),
        requirements_text
    );

    let system_prompt = Some(
        r#"You are an expert software architect creating detailed specifications.

Your task is to:
1. Create specific, testable requirements for the capability
2. For each requirement, define WHEN/THEN/AND scenarios
3. Make scenarios concrete and actionable
4. Ensure all scenarios follow the Given-When-Then pattern
5. Include both happy path and error scenarios

Important guidelines:
- Each requirement must have at least 2 scenarios
- Scenarios must be specific and testable
- Use clear, precise language
- Focus on user-facing behavior
- Include edge cases and error handling

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text."#
            .to_string(),
    );

    // Make AI call
    match ai_service
        .generate_structured::<SpecGenerationData>(user_prompt, system_prompt)
        .await
    {
        Ok(ai_response) => {
            // Convert JSON response to markdown format
            let mut spec_markdown = format!("# {}\n\n", request.capability_name);

            if let Some(purpose) = &request.purpose {
                spec_markdown.push_str(&format!("## Purpose\n{}\n\n", purpose));
            }

            spec_markdown.push_str("## Requirements\n\n");

            let mut total_scenarios = 0;
            for req in &ai_response.data.requirements {
                spec_markdown.push_str(&format!("### {}\n{}\n\n", req.name, req.description));

                for scenario in &req.scenarios {
                    spec_markdown.push_str(&format!("**{}**\n", scenario.name));
                    spec_markdown.push_str(&format!("WHEN {}\n", scenario.when));
                    spec_markdown.push_str(&format!("THEN {}\n", scenario.then));

                    if let Some(and_conditions) = &scenario.and {
                        for condition in and_conditions {
                            spec_markdown.push_str(&format!("AND {}\n", condition));
                        }
                    }
                    spec_markdown.push('\n');
                    total_scenarios += 1;
                }
            }

            let token_usage = TokenUsage {
                input: ai_response.usage.input_tokens,
                output: ai_response.usage.output_tokens,
                total: ai_response.usage.total_tokens(),
            };

            // Log AI usage to database
            let usage_log = AiUsageLog {
                id: nanoid::nanoid!(10),
                project_id: request.capability_name.clone(), // Use capability name as project ID
                request_id: None,
                operation: "generateSpec".to_string(),
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_tokens: Some(ai_response.usage.input_tokens as i64),
                output_tokens: Some(ai_response.usage.output_tokens as i64),
                total_tokens: Some(ai_response.usage.total_tokens() as i64),
                estimated_cost: Some(calculate_cost(
                    ai_response.usage.input_tokens,
                    ai_response.usage.output_tokens,
                )),
                duration_ms: Some(0), // TODO: Track actual duration
                error: None,
                created_at: chrono::Utc::now(),
            };

            // Save to database (non-blocking)
            if let Err(e) = db.ai_usage_log_storage.create_log(&usage_log).await {
                error!("Failed to log AI usage: {}", e);
            }

            let response = GenerateSpecResponse {
                capability_name: request.capability_name,
                spec_markdown,
                requirement_count: ai_response.data.requirements.len(),
                scenario_count: total_scenarios,
                token_usage,
                note: String::new(),
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("AI spec generation failed: {}", e);
            let error_message = match e {
                AIServiceError::NoApiKey => {
                    "Anthropic API key not configured. Please set ANTHROPIC_API_KEY environment variable.".to_string()
                }
                AIServiceError::ApiError(msg) => format!("Anthropic API error: {}", msg),
                AIServiceError::ParseError(msg) => {
                    format!("Failed to parse AI response: {}", msg)
                }
                AIServiceError::RequestFailed(e) => format!("Request failed: {}", e),
                AIServiceError::InvalidResponse => {
                    "Invalid response from AI service".to_string()
                }
            };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(error_message)),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct GenerateSpecRequest {
    #[serde(rename = "capabilityName")]
    pub capability_name: String,
    pub purpose: Option<String>,
    pub requirements: Vec<String>,
}

#[derive(Serialize)]
pub struct GenerateSpecResponse {
    #[serde(rename = "capabilityName")]
    pub capability_name: String,
    #[serde(rename = "specMarkdown")]
    pub spec_markdown: String,
    #[serde(rename = "requirementCount")]
    pub requirement_count: usize,
    #[serde(rename = "scenarioCount")]
    pub scenario_count: usize,
    #[serde(rename = "tokenUsage")]
    pub token_usage: TokenUsage,
    pub note: String,
}

/// Suggest tasks from spec
pub async fn suggest_tasks(
    State(db): State<DbState>,
    Json(request): Json<SuggestTasksRequest>,
) -> impl IntoResponse {
    info!(
        "AI task suggestions requested for capability: {}",
        request.capability_id
    );

    // Initialize AI service
    let ai_service = AIService::new();

    let user_prompt = format!(
        r#"Analyze the following specification and generate actionable development tasks.

Specification:
{}

Your tasks should:
1. Cover all requirements and scenarios in the spec
2. Be specific and actionable for developers
3. Include both implementation tasks and testing tasks
4. Have realistic complexity scores (1-10, where 10 is most complex)
5. Reference specific requirement IDs from the spec
6. Use priorities: "high", "medium", or "low"

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{{
  "tasks": [
    {{
      "title": "Task title",
      "description": "Detailed description of what needs to be done",
      "priority": "high",
      "complexity_score": 7,
      "linked_requirements": ["req-1", "req-2"]
    }}
  ]
}}"#,
        request.spec_markdown
    );

    let system_prompt = Some(
        r#"You are an expert software development project manager breaking down specifications into actionable tasks.

Your task is to:
1. Read the specification carefully
2. Identify all requirements and scenarios
3. Create specific, actionable tasks for developers
4. Ensure tasks cover implementation, testing, and documentation
5. Assign realistic complexity scores based on:
   - 1-3: Simple changes, configuration
   - 4-6: Medium features, moderate complexity
   - 7-9: Complex features, significant work
   - 10: Very complex, architectural changes
6. Set priorities based on:
   - high: Core functionality, critical path
   - medium: Important but not blocking
   - low: Nice-to-have, can be deferred
7. Link tasks to specific requirements they address

Important guidelines:
- Tasks should be granular enough to be assigned to individual developers
- Each task should be completable in 1-3 days
- Include both happy path and error handling tasks
- Don't forget testing and documentation tasks
- Use requirement IDs that match the spec (e.g., "req-1", "req-2")

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text."#
            .to_string(),
    );

    // Make AI call
    match ai_service
        .generate_structured::<TaskSuggestionsData>(user_prompt, system_prompt)
        .await
    {
        Ok(ai_response) => {
            // Convert AI response to API format
            let suggested_tasks: Vec<SuggestedTask> = ai_response
                .data
                .tasks
                .into_iter()
                .map(|task| SuggestedTask {
                    title: task.title,
                    description: task.description,
                    priority: task.priority,
                    complexity_score: task.complexity_score,
                    linked_requirements: task.linked_requirements,
                })
                .collect();

            let token_usage = TokenUsage {
                input: ai_response.usage.input_tokens,
                output: ai_response.usage.output_tokens,
                total: ai_response.usage.total_tokens(),
            };

            // Log AI usage to database
            let usage_log = AiUsageLog {
                id: nanoid::nanoid!(10),
                project_id: request.capability_id.clone(),
                request_id: None,
                operation: "suggestTasks".to_string(),
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_tokens: Some(ai_response.usage.input_tokens as i64),
                output_tokens: Some(ai_response.usage.output_tokens as i64),
                total_tokens: Some(ai_response.usage.total_tokens() as i64),
                estimated_cost: Some(calculate_cost(
                    ai_response.usage.input_tokens,
                    ai_response.usage.output_tokens,
                )),
                duration_ms: Some(0),
                error: None,
                created_at: chrono::Utc::now(),
            };

            // Save to database (non-blocking)
            if let Err(e) = db.ai_usage_log_storage.create_log(&usage_log).await {
                error!("Failed to log AI usage: {}", e);
            }

            let response = SuggestTasksResponse {
                capability_id: request.capability_id,
                suggested_tasks,
                token_usage,
                note: String::new(),
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("AI task suggestion failed: {}", e);
            let error_message = match e {
                AIServiceError::NoApiKey => {
                    "Anthropic API key not configured. Please set ANTHROPIC_API_KEY environment variable.".to_string()
                }
                AIServiceError::ApiError(msg) => format!("Anthropic API error: {}", msg),
                AIServiceError::ParseError(msg) => {
                    format!("Failed to parse AI response: {}", msg)
                }
                AIServiceError::RequestFailed(e) => format!("Request failed: {}", e),
                AIServiceError::InvalidResponse => {
                    "Invalid response from AI service".to_string()
                }
            };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(error_message)),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct SuggestTasksRequest {
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "specMarkdown")]
    pub spec_markdown: String,
}

#[derive(Serialize)]
pub struct SuggestTasksResponse {
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "suggestedTasks")]
    pub suggested_tasks: Vec<SuggestedTask>,
    #[serde(rename = "tokenUsage")]
    pub token_usage: TokenUsage,
    pub note: String,
}

#[derive(Serialize)]
pub struct SuggestedTask {
    pub title: String,
    pub description: String,
    pub priority: String,
    #[serde(rename = "complexityScore")]
    pub complexity_score: i32,
    #[serde(rename = "linkedRequirements")]
    pub linked_requirements: Vec<String>,
}

/// Refine spec with feedback
pub async fn refine_spec(
    State(db): State<DbState>,
    Json(request): Json<RefineSpecRequest>,
) -> impl IntoResponse {
    info!(
        "AI spec refinement requested for capability: {}",
        request.capability_id
    );

    // Initialize AI service
    let ai_service = AIService::new();

    let user_prompt = format!(
        r#"Refine the following specification based on user feedback.

Current Specification:
{}

User Feedback:
{}

Your task is to:
1. Analyze the current specification and the feedback
2. Update the specification to address all feedback points
3. Keep the same markdown structure and format
4. Preserve existing content that isn't affected by the feedback
5. Track what changes you made

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{{
  "refined_spec": "The complete refined specification in markdown format",
  "changes_made": ["Description of change 1", "Description of change 2"]
}}"#,
        request.current_spec_markdown, request.feedback
    );

    let system_prompt = Some(
        r#"You are an expert technical writer refining software specifications based on feedback.

Your task is to:
1. Carefully read the current specification
2. Understand the user's feedback and concerns
3. Update the specification to address the feedback while maintaining quality
4. Preserve the spec's structure, clarity, and testability
5. Track all changes made for transparency

Important guidelines:
- Keep the WHEN/THEN/AND scenario format
- Maintain markdown formatting
- Don't remove content unless explicitly requested
- Add clarifications, not just rephrasing
- Ensure all changes directly address the feedback
- List specific changes made (e.g., "Added error handling scenario for X", "Clarified requirement Y")

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text."#
            .to_string(),
    );

    // Make AI call
    match ai_service
        .generate_structured::<SpecRefinementData>(user_prompt, system_prompt)
        .await
    {
        Ok(ai_response) => {
            let token_usage = TokenUsage {
                input: ai_response.usage.input_tokens,
                output: ai_response.usage.output_tokens,
                total: ai_response.usage.total_tokens(),
            };

            // Log AI usage to database
            let usage_log = AiUsageLog {
                id: nanoid::nanoid!(10),
                project_id: request.capability_id.clone(),
                request_id: None,
                operation: "refineSpec".to_string(),
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_tokens: Some(ai_response.usage.input_tokens as i64),
                output_tokens: Some(ai_response.usage.output_tokens as i64),
                total_tokens: Some(ai_response.usage.total_tokens() as i64),
                estimated_cost: Some(calculate_cost(
                    ai_response.usage.input_tokens,
                    ai_response.usage.output_tokens,
                )),
                duration_ms: Some(0),
                error: None,
                created_at: chrono::Utc::now(),
            };

            // Save to database (non-blocking)
            if let Err(e) = db.ai_usage_log_storage.create_log(&usage_log).await {
                error!("Failed to log AI usage: {}", e);
            }

            let response = RefineSpecResponse {
                capability_id: request.capability_id,
                refined_spec_markdown: ai_response.data.refined_spec,
                changes_made: ai_response.data.changes_made,
                token_usage,
                note: String::new(),
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("AI spec refinement failed: {}", e);
            let error_message = match e {
                AIServiceError::NoApiKey => {
                    "Anthropic API key not configured. Please set ANTHROPIC_API_KEY environment variable.".to_string()
                }
                AIServiceError::ApiError(msg) => format!("Anthropic API error: {}", msg),
                AIServiceError::ParseError(msg) => {
                    format!("Failed to parse AI response: {}", msg)
                }
                AIServiceError::RequestFailed(e) => format!("Request failed: {}", e),
                AIServiceError::InvalidResponse => {
                    "Invalid response from AI service".to_string()
                }
            };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(error_message)),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct RefineSpecRequest {
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "currentSpecMarkdown")]
    pub current_spec_markdown: String,
    pub feedback: String,
}

#[derive(Serialize)]
pub struct RefineSpecResponse {
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "refinedSpecMarkdown")]
    pub refined_spec_markdown: String,
    #[serde(rename = "changesMade")]
    pub changes_made: Vec<String>,
    #[serde(rename = "tokenUsage")]
    pub token_usage: TokenUsage,
    pub note: String,
}

/// Validate task completion against spec
pub async fn validate_completion(
    State(db): State<DbState>,
    Json(request): Json<ValidateCompletionRequest>,
) -> impl IntoResponse {
    info!("AI validation requested for task: {}", request.task_id);

    // Initialize AI service
    let ai_service = AIService::new();

    // Build scenario list for validation
    let scenarios_text = if request.linked_scenarios.is_empty() {
        "No specific scenarios linked".to_string()
    } else {
        request
            .linked_scenarios
            .iter()
            .enumerate()
            .map(|(i, scenario)| format!("{}. {}", i + 1, scenario))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let user_prompt = format!(
        r#"Validate whether the following implementation completes the specified scenarios.

Implementation Details:
{}

Scenarios to Validate:
{}

Your task is to:
1. Analyze the implementation details
2. Check if each scenario is adequately addressed
3. Assess confidence in completion for each scenario
4. Provide an overall assessment
5. Suggest improvements if needed

Respond with ONLY valid JSON matching this exact structure (no markdown, no code blocks):
{{
  "is_complete": true,
  "validation_results": [
    {{
      "scenario": "Scenario description",
      "passed": true,
      "confidence": 0.95,
      "notes": "Optional notes about this validation"
    }}
  ],
  "overall_confidence": 0.93,
  "recommendations": ["Recommendation 1", "Recommendation 2"]
}}"#,
        request.implementation_details, scenarios_text
    );

    let system_prompt = Some(
        r#"You are an expert QA engineer validating task completion against specifications.

Your task is to:
1. Carefully read the implementation details
2. Compare against each specified scenario
3. Determine if the scenario is fully implemented
4. Assign confidence scores (0.0 to 1.0):
   - 1.0: Perfectly matches, no doubt
   - 0.9-0.99: Excellent match, minor uncertainties
   - 0.8-0.89: Good match, some assumptions made
   - 0.7-0.79: Adequate match, several assumptions
   - Below 0.7: Insufficient evidence or concerns
5. Provide specific notes for failed or uncertain scenarios
6. Give actionable recommendations for improvement

Important guidelines:
- Be thorough but fair in assessment
- If implementation is incomplete, set passed: false
- Consider edge cases and error handling
- Recommendations should be specific and actionable
- Overall confidence should reflect the weakest link
- If no scenarios provided, evaluate based on implementation quality

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text."#
            .to_string(),
    );

    // Make AI call
    match ai_service
        .generate_structured::<CompletionValidationData>(user_prompt, system_prompt)
        .await
    {
        Ok(ai_response) => {
            // Convert AI response to API format
            let validation_results: Vec<ValidationResult> = ai_response
                .data
                .validation_results
                .into_iter()
                .map(|item| ValidationResult {
                    scenario: item.scenario,
                    passed: item.passed,
                    confidence: item.confidence,
                    notes: item.notes,
                })
                .collect();

            let token_usage = TokenUsage {
                input: ai_response.usage.input_tokens,
                output: ai_response.usage.output_tokens,
                total: ai_response.usage.total_tokens(),
            };

            // Log AI usage to database
            let usage_log = AiUsageLog {
                id: nanoid::nanoid!(10),
                project_id: request.task_id.clone(),
                request_id: None,
                operation: "validateCompletion".to_string(),
                provider: "anthropic".to_string(),
                model: "claude-3-5-sonnet-20241022".to_string(),
                input_tokens: Some(ai_response.usage.input_tokens as i64),
                output_tokens: Some(ai_response.usage.output_tokens as i64),
                total_tokens: Some(ai_response.usage.total_tokens() as i64),
                estimated_cost: Some(calculate_cost(
                    ai_response.usage.input_tokens,
                    ai_response.usage.output_tokens,
                )),
                duration_ms: Some(0),
                error: None,
                created_at: chrono::Utc::now(),
            };

            // Save to database (non-blocking)
            if let Err(e) = db.ai_usage_log_storage.create_log(&usage_log).await {
                error!("Failed to log AI usage: {}", e);
            }

            let response = ValidateCompletionResponse {
                task_id: request.task_id,
                is_complete: ai_response.data.is_complete,
                validation_results,
                overall_confidence: ai_response.data.overall_confidence,
                recommendations: ai_response.data.recommendations,
                token_usage,
                note: String::new(),
            };

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("AI validation failed: {}", e);
            let error_message = match e {
                AIServiceError::NoApiKey => {
                    "Anthropic API key not configured. Please set ANTHROPIC_API_KEY environment variable.".to_string()
                }
                AIServiceError::ApiError(msg) => format!("Anthropic API error: {}", msg),
                AIServiceError::ParseError(msg) => {
                    format!("Failed to parse AI response: {}", msg)
                }
                AIServiceError::RequestFailed(e) => format!("Request failed: {}", e),
                AIServiceError::InvalidResponse => {
                    "Invalid response from AI service".to_string()
                }
            };

            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(error_message)),
            )
                .into_response()
        }
    }
}

#[derive(Deserialize)]
pub struct ValidateCompletionRequest {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "implementationDetails")]
    pub implementation_details: String,
    #[serde(rename = "linkedScenarios")]
    pub linked_scenarios: Vec<String>,
}

#[derive(Serialize)]
pub struct ValidateCompletionResponse {
    #[serde(rename = "taskId")]
    pub task_id: String,
    #[serde(rename = "isComplete")]
    pub is_complete: bool,
    #[serde(rename = "validationResults")]
    pub validation_results: Vec<ValidationResult>,
    #[serde(rename = "overallConfidence")]
    pub overall_confidence: f32,
    pub recommendations: Vec<String>,
    #[serde(rename = "tokenUsage")]
    pub token_usage: TokenUsage,
    pub note: String,
}

#[derive(Serialize)]
pub struct ValidationResult {
    pub scenario: String,
    pub passed: bool,
    pub confidence: f32,
    pub notes: Option<String>,
}
