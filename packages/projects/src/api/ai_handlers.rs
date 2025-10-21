// ABOUTME: HTTP request handlers for AI operations using real Anthropic API calls
// ABOUTME: Implements structured generation for PRD analysis, spec generation, and task suggestions

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Json as ResponseJson},
};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use super::response::ApiResponse;
use crate::ai_service::{AIService, AIServiceError};
use crate::ai_usage_logs::AiUsageLog;
use crate::db::DbState;

// ============================================================================
// Shared Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecScenario {
    pub name: String,
    pub when: String,
    pub then: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub and: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecRequirement {
    pub name: String,
    pub content: String,
    pub scenarios: Vec<SpecScenario>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SpecCapability {
    pub id: String,
    pub name: String,
    pub purpose: String,
    pub requirements: Vec<SpecRequirement>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TaskSuggestion {
    pub title: String,
    pub description: String,
    #[serde(rename = "capabilityId")]
    pub capability_id: String,
    #[serde(rename = "requirementName")]
    pub requirement_name: String,
    pub complexity: u8,
    #[serde(rename = "estimatedHours", skip_serializing_if = "Option::is_none")]
    pub estimated_hours: Option<f32>,
    pub priority: String,
}

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
// PRD Analysis
// ============================================================================

#[derive(Deserialize)]
pub struct AnalyzePRDRequest {
    #[serde(rename = "prdId")]
    pub prd_id: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
}

#[derive(Serialize, Deserialize)]
pub struct PRDAnalysisData {
    pub summary: String,
    pub capabilities: Vec<SpecCapability>,
    #[serde(rename = "suggestedTasks")]
    pub suggested_tasks: Vec<TaskSuggestion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<Vec<String>>,
    #[serde(
        rename = "technicalConsiderations",
        skip_serializing_if = "Option::is_none"
    )]
    pub technical_considerations: Option<Vec<String>>,
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
    Json(request): Json<AnalyzePRDRequest>,
) -> impl IntoResponse {
    info!("AI PRD analysis requested for PRD: {}", request.prd_id);

    // Initialize AI service
    let ai_service = AIService::new();

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
        r#"You are an expert software architect analyzing Product Requirements Documents.

Your task is to:
1. Extract high-level capabilities (functional areas) from the PRD
2. For each capability, define specific requirements
3. For each requirement, create WHEN/THEN/AND scenarios
4. Suggest 5-10 actionable tasks to implement the capabilities
5. Identify dependencies and technical considerations

Important guidelines:
- Capability IDs must be kebab-case (e.g., "user-auth", "data-sync")
- Each requirement must have at least one scenario
- Scenarios must follow WHEN/THEN/AND structure
- Tasks should be specific, actionable, and include complexity scores (1-10)
- Tasks should reference the capability and requirement they implement
- Priority must be "low", "medium", or "high"
- Be specific and actionable
- Focus on testable behaviors

Respond with ONLY valid JSON. Do not include markdown formatting, code blocks, or any other text."#
            .to_string(),
    );

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

            // Log AI usage to database
            let usage_log = AiUsageLog {
                id: nanoid::nanoid!(10),
                project_id: request.prd_id.clone(), // Use PRD ID as project ID for now
                request_id: None,
                operation: "analyzePRD".to_string(),
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
                // Continue anyway - logging failure shouldn't block the response
            }

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
// Placeholder Implementations (TODO: Implement with real AI)
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
pub async fn suggest_tasks(Json(request): Json<SuggestTasksRequest>) -> impl IntoResponse {
    info!(
        "AI task suggestions requested for capability: {}",
        request.capability_id
    );

    let response = SuggestTasksResponse {
        capability_id: request.capability_id,
        suggested_tasks: vec![
            SuggestedTask {
                title: "Implement primary workflow".to_string(),
                description: "Build the core functionality as described in the requirements"
                    .to_string(),
                priority: "high".to_string(),
                complexity_score: 8,
                linked_requirements: vec!["req-1".to_string()],
            },
            SuggestedTask {
                title: "Add error handling".to_string(),
                description: "Implement error handling and user feedback mechanisms".to_string(),
                priority: "medium".to_string(),
                complexity_score: 5,
                linked_requirements: vec!["req-2".to_string()],
            },
            SuggestedTask {
                title: "Write tests for scenarios".to_string(),
                description: "Create automated tests for all WHEN/THEN scenarios".to_string(),
                priority: "high".to_string(),
                complexity_score: 6,
                linked_requirements: vec!["req-1".to_string(), "req-2".to_string()],
            },
        ],
        token_usage: TokenUsage {
            input: 200,
            output: 150,
            total: 350,
        },
        note: "AI integration not yet implemented - this is mock data".to_string(),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
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
pub async fn refine_spec(Json(request): Json<RefineSpecRequest>) -> impl IntoResponse {
    info!(
        "AI spec refinement requested for capability: {}",
        request.capability_id
    );

    let response = RefineSpecResponse {
        capability_id: request.capability_id.clone(),
        refined_spec_markdown: request.current_spec_markdown.clone() + "\n\n### Additional Considerations\n\nBased on feedback, added:\n- Performance optimization requirements\n- Security considerations\n- Accessibility requirements",
        changes_made: vec![
            "Added performance requirements".to_string(),
            "Clarified security constraints".to_string(),
            "Enhanced accessibility scenarios".to_string(),
        ],
        token_usage: TokenUsage {
            input: 350,
            output: 200,
            total: 550,
        },
        note: "AI integration not yet implemented - this is mock data".to_string(),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
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
    Json(request): Json<ValidateCompletionRequest>,
) -> impl IntoResponse {
    info!("AI validation requested for task: {}", request.task_id);

    let response = ValidateCompletionResponse {
        task_id: request.task_id,
        is_complete: true,
        validation_results: vec![
            ValidationResult {
                scenario: "User can complete primary action".to_string(),
                passed: true,
                confidence: 0.92,
                notes: Some("Implementation matches expected behavior".to_string()),
            },
            ValidationResult {
                scenario: "System handles errors gracefully".to_string(),
                passed: true,
                confidence: 0.88,
                notes: Some("Error handling appears comprehensive".to_string()),
            },
        ],
        overall_confidence: 0.90,
        recommendations: vec![
            "Consider adding more edge case handling".to_string(),
            "Add performance benchmarks".to_string(),
        ],
        token_usage: TokenUsage {
            input: 280,
            output: 120,
            total: 400,
        },
        note: "AI integration not yet implemented - this is mock data".to_string(),
    };

    (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
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
