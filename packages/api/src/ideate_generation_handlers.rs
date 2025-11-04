// ABOUTME: HTTP request handlers for PRD generation and export operations
// ABOUTME: Handles PRD generation, section filling, export, and validation

use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json,
};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::convert::Infallible;
use tracing::info;

use super::response::{ok_or_internal_error, ok_or_not_found};
use orkee_ideate::{ExportFormat, PRDAggregator};
use orkee_projects::DbState;

/// Request body for generating PRD from session
#[derive(Deserialize)]
pub struct GeneratePRDRequest {
    #[serde(rename = "includeSkipped", default)]
    pub include_skipped: bool,
}

/// Generate PRD from collected session data
///
/// ⚠️ DEPRECATED: This handler is disabled - AI operations moved to frontend AI SDK.
/// Frontend should use `prd-ai.ts:generateFromSession()` instead.
#[allow(dead_code)]
pub async fn generate_prd(
    State(_db): State<DbState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<GeneratePRDRequest>,
) -> impl IntoResponse {
    // This handler is deprecated - route is commented out in lib.rs
    // Frontend should call prd-ai.ts:generateFromSession() directly
    ok_or_internal_error::<(), _>(
        Err("This endpoint has been deprecated. Use frontend AI SDK instead."),
        "This endpoint has been deprecated. Use frontend AI SDK instead.",
    )
}

/// Request body for filling skipped sections
#[derive(Deserialize)]
pub struct FillSkippedSectionsRequest {
    pub sections: Vec<String>,
}

/// AI-fill skipped sections with context
///
/// ⚠️ DEPRECATED: This handler is disabled - AI operations moved to frontend AI SDK.
/// Frontend should use `prd-ai.ts:fillSkippedSections()` instead.
#[allow(dead_code)]
pub async fn fill_skipped_sections(
    State(_db): State<DbState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<FillSkippedSectionsRequest>,
) -> impl IntoResponse {
    // This handler is deprecated - route is commented out in lib.rs
    // Frontend should call prd-ai.ts:fillSkippedSections() directly
    ok_or_internal_error::<(), _>(
        Err("This endpoint has been deprecated. Use frontend AI SDK instead."),
        "This endpoint has been deprecated. Use frontend AI SDK instead.",
    )
}

/// Request body for regenerating a section
#[derive(Deserialize)]
pub struct RegenerateSectionRequest {
    pub section: String,
}

/// Regenerate specific section with full context
///
/// ⚠️ DEPRECATED: This handler is disabled - AI operations moved to frontend AI SDK.
/// Frontend should use `prd-ai.ts:generateSectionWithContext()` instead.
#[allow(dead_code)]
pub async fn regenerate_section(
    State(_db): State<DbState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<RegenerateSectionRequest>,
) -> impl IntoResponse {
    // This handler is deprecated - route is commented out in lib.rs
    // Frontend should call prd-ai.ts:generateSectionWithContext() directly
    ok_or_internal_error::<(), _>(
        Err("This endpoint has been deprecated. Use frontend AI SDK instead."),
        "This endpoint has been deprecated. Use frontend AI SDK instead.",
    )
}

/// Get PRD preview (aggregated data)
pub async fn get_prd_preview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting PRD preview for session: {}", session_id);

    let aggregator = PRDAggregator::new(db.pool.clone());
    let result = aggregator.aggregate_session_data(&session_id).await;

    ok_or_not_found(result, "Session not found or incomplete")
}

/// Request body for exporting PRD
#[derive(Deserialize)]
pub struct ExportPRDRequest {
    pub format: ExportFormat,
    #[serde(rename = "includeToc", default = "default_true")]
    pub include_toc: bool,
    #[serde(rename = "includeMetadata", default = "default_true")]
    pub include_metadata: bool,
    #[serde(rename = "includePageNumbers", default)]
    pub include_page_numbers: bool,
    #[serde(rename = "customCss")]
    pub custom_css: Option<String>,
    pub title: Option<String>,
}

fn default_true() -> bool {
    true
}

/// Response for export operation
#[derive(Serialize)]
pub struct ExportPRDResponse {
    pub format: String,
    pub content: String,
    #[serde(rename = "fileName")]
    pub file_name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    #[serde(rename = "sizeBytes")]
    pub size_bytes: usize,
}

/// Export PRD in specified format
///
/// ⚠️ DEPRECATED: This handler is disabled - needs refactoring for frontend AI SDK pattern.
/// Export functionality will be re-implemented after PRD generation is handled by frontend.
#[allow(dead_code)]
pub async fn export_prd(
    State(_db): State<DbState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<ExportPRDRequest>,
) -> impl IntoResponse {
    // This handler is deprecated - route is commented out in lib.rs
    // Export functionality needs to be redesigned for frontend AI SDK pattern
    ok_or_internal_error::<ExportPRDResponse, _>(
        Err("This endpoint has been deprecated. Export will be re-implemented."),
        "This endpoint has been deprecated. Export will be re-implemented.",
    )
}

/// Get completeness metrics for session
pub async fn get_completeness(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting completeness metrics for session: {}", session_id);

    let aggregator = PRDAggregator::new(db.pool.clone());
    let result = aggregator.aggregate_session_data(&session_id).await;

    match result {
        Ok(data) => ok_or_internal_error::<orkee_ideate::CompletenessMetrics, String>(
            Ok(data.completeness),
            "",
        ),
        Err(e) => ok_or_internal_error::<orkee_ideate::CompletenessMetrics, _>(
            Err(e),
            "Failed to get completeness metrics",
        ),
    }
}

/// Response for generation history
#[derive(Serialize)]
pub struct GenerationHistoryItem {
    pub id: String,
    pub version: i32,
    #[serde(rename = "generationMethod")]
    pub generation_method: String,
    #[serde(rename = "validationStatus")]
    pub validation_status: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Get PRD generation history
pub async fn get_generation_history(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting generation history for session: {}", session_id);

    let result = sqlx::query_as::<_, (String, i32, String, String, String)>(
        "SELECT id, version, generation_method, validation_status, created_at
         FROM ideate_prd_generations
         WHERE session_id = ?
         ORDER BY version DESC",
    )
    .bind(&session_id)
    .fetch_all(&db.pool)
    .await;

    match result {
        Ok(rows) => {
            let history: Vec<GenerationHistoryItem> = rows
                .into_iter()
                .map(
                    |(id, version, method, status, created_at)| GenerationHistoryItem {
                        id,
                        version,
                        generation_method: method,
                        validation_status: status,
                        created_at,
                    },
                )
                .collect();
            ok_or_internal_error::<Vec<GenerationHistoryItem>, String>(Ok(history), "")
        }
        Err(e) => ok_or_internal_error::<Vec<GenerationHistoryItem>, _>(
            Err(orkee_ideate::IdeateError::Database(e)),
            "Failed to get generation history",
        ),
    }
}

/// Response for validation
#[derive(Serialize)]
pub struct ValidationResponse {
    pub status: String,
    pub errors: Vec<ValidationIssue>,
    pub warnings: Vec<ValidationIssue>,
}

#[derive(Serialize)]
pub struct ValidationIssue {
    pub rule: String,
    pub section: Option<String>,
    pub message: String,
}

/// Validate PRD against rules
pub async fn validate_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Validating PRD for session: {}", session_id);

    // Get aggregated data
    let aggregator = PRDAggregator::new(db.pool.clone());
    let data_result = aggregator.aggregate_session_data(&session_id).await;

    let data = match data_result {
        Ok(d) => d,
        Err(e) => {
            return ok_or_internal_error::<ValidationResponse, _>(
                Err(e),
                "Failed to get session data for validation",
            );
        }
    };

    // Get validation rules
    let rules_result = sqlx::query_as::<_, (String, String, Option<String>, String, String)>(
        "SELECT rule_name, rule_type, section, severity, error_message
         FROM ideate_validation_rules
         WHERE is_active = 1",
    )
    .fetch_all(&db.pool)
    .await;

    let rules = match rules_result {
        Ok(r) => r,
        Err(e) => {
            return ok_or_internal_error::<ValidationResponse, _>(
                Err(orkee_ideate::IdeateError::Database(e)),
                "Failed to get validation rules",
            );
        }
    };

    // Perform validation
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    for (rule_name, _rule_type, section, severity, message) in rules {
        // Simple validation logic - check if required sections exist
        let has_issue = match section.as_deref() {
            Some("overview") => data.overview.is_none(),
            Some("features") => false, // Features are in overview
            Some("ux") => data.ux.is_none(),
            Some("technical") => data.technical.is_none(),
            Some("roadmap") => data.roadmap.is_none(),
            Some("dependencies") => data.dependencies.is_none(),
            Some("risks") => data.risks.is_none(),
            Some("research") => data.research.is_none(),
            _ => false,
        };

        if has_issue {
            let issue = ValidationIssue {
                rule: rule_name,
                section,
                message,
            };

            if severity == "error" {
                errors.push(issue);
            } else {
                warnings.push(issue);
            }
        }
    }

    let status = if !errors.is_empty() {
        "invalid"
    } else if !warnings.is_empty() {
        "warnings"
    } else {
        "valid"
    };

    let response = ValidationResponse {
        status: status.to_string(),
        errors,
        warnings,
    };

    ok_or_internal_error::<ValidationResponse, String>(Ok(response), "")
}

/// Request body for regenerating PRD with new template
#[derive(Deserialize)]
pub struct RegeneratePRDWithTemplateRequest {
    #[serde(rename = "templateId")]
    pub template_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Regenerate PRD with a different template's style/format
///
/// ⚠️ DEPRECATED: This handler is disabled - AI operations moved to frontend AI SDK.
/// Frontend should use `prd-ai.ts:regenerateWithTemplateStream()` instead.
#[allow(dead_code)]
pub async fn regenerate_prd_with_template(
    State(_db): State<DbState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<RegeneratePRDWithTemplateRequest>,
) -> impl IntoResponse {
    // This handler is deprecated - route is commented out in lib.rs
    // Frontend should call prd-ai.ts:regenerateWithTemplateStream() directly
    ok_or_internal_error::<serde_json::Value, _>(
        Err("This endpoint has been deprecated. Use frontend AI SDK instead."),
        "This endpoint has been deprecated. Use frontend AI SDK instead.",
    )
}

/// Regenerate PRD with a different template's style/format (streaming version)
///
/// ⚠️ DEPRECATED: This handler is disabled - AI operations moved to frontend AI SDK.
/// Frontend should use `prd-ai.ts:regenerateWithTemplateStream()` instead.
#[allow(dead_code)]
pub async fn regenerate_prd_with_template_stream(
    State(_db): State<DbState>,
    Path(_session_id): Path<String>,
    Json(_request): Json<RegeneratePRDWithTemplateRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    // This handler is deprecated - route is commented out in lib.rs
    // Frontend should call prd-ai.ts:regenerateWithTemplateStream() directly
    let stream = async_stream::stream! {
        let error_event = Event::default()
            .json_data(json!({
                "type": "error",
                "message": "This endpoint has been deprecated. Use frontend AI SDK instead."
            }))
            .unwrap();
        yield Ok(error_event);
    };
    Sse::new(stream).keep_alive(KeepAlive::default())
}
