// ABOUTME: HTTP request handlers for PRD generation and export operations
// ABOUTME: Handles PRD generation, section filling, export, and validation

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use super::response::{ok_or_internal_error, ok_or_not_found};
use ideate::{ExportFormat, ExportOptions, PRDAggregator, PRDGenerator};
use ideate::prd_generator::GeneratedPRD;
use orkee_projects::DbState;

// TODO: Replace with proper user authentication
const DEFAULT_USER_ID: &str = "default-user";

/// Request body for generating PRD from session
#[derive(Deserialize)]
pub struct GeneratePRDRequest {
    #[serde(rename = "includeSkipped", default)]
    pub include_skipped: bool,
}

/// Generate PRD from collected session data
pub async fn generate_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<GeneratePRDRequest>,
) -> impl IntoResponse {
    info!(
        "Generating PRD for session: {} (include_skipped: {})",
        session_id, request.include_skipped
    );

    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .generate_from_session(DEFAULT_USER_ID, &session_id)
        .await;

    ok_or_internal_error(result, "Failed to generate PRD")
}

/// Request body for filling skipped sections
#[derive(Deserialize)]
pub struct FillSkippedSectionsRequest {
    pub sections: Vec<String>,
}

/// AI-fill skipped sections with context
pub async fn fill_skipped_sections(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<FillSkippedSectionsRequest>,
) -> impl IntoResponse {
    info!(
        "Filling skipped sections for session: {} (sections: {:?})",
        session_id, request.sections
    );

    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .fill_skipped_sections(DEFAULT_USER_ID, &session_id, request.sections)
        .await;

    ok_or_internal_error(result, "Failed to fill skipped sections")
}

/// Request body for regenerating a section
#[derive(Deserialize)]
pub struct RegenerateSectionRequest {
    pub section: String,
}

/// Regenerate specific section with full context
pub async fn regenerate_section(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<RegenerateSectionRequest>,
) -> impl IntoResponse {
    info!(
        "Regenerating section '{}' for session: {}",
        request.section, session_id
    );

    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .regenerate_section_with_full_context(DEFAULT_USER_ID, &session_id, &request.section)
        .await;

    ok_or_internal_error(result, "Failed to regenerate section")
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
pub async fn export_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<ExportPRDRequest>,
) -> impl IntoResponse {
    info!(
        "Exporting PRD for session: {} (format: {:?})",
        session_id, request.format
    );

    // First generate the PRD
    let generator = PRDGenerator::new(db.pool.clone());
    let prd_result = generator
        .generate_from_session(DEFAULT_USER_ID, &session_id)
        .await;

    let prd = match prd_result {
        Ok(p) => p,
        Err(e) => {
            return ok_or_internal_error::<ExportPRDResponse, _>(
                Err(e),
                "Failed to generate PRD for export",
            );
        }
    };

    // Now export it
    let export_service = ideate::ExportService::new(db.pool.clone());
    let options = ExportOptions {
        format: request.format,
        include_toc: request.include_toc,
        include_metadata: request.include_metadata,
        include_page_numbers: request.include_page_numbers,
        custom_css: request.custom_css,
        title: request.title,
    };

    let result = export_service
        .export_prd(&prd, options, Some(&session_id))
        .await;

    match result {
        Ok(export_result) => {
            let response = ExportPRDResponse {
                format: export_result.format.to_string(),
                content: export_result.content,
                file_name: export_result.file_name,
                mime_type: export_result.mime_type,
                size_bytes: export_result.size_bytes,
            };
            ok_or_internal_error::<ExportPRDResponse, String>(Ok(response), "")
        }
        Err(e) => ok_or_internal_error::<ExportPRDResponse, _>(Err(e), "Failed to export PRD"),
    }
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
        Ok(data) => {
            ok_or_internal_error::<ideate::CompletenessMetrics, String>(Ok(data.completeness), "")
        }
        Err(e) => ok_or_internal_error::<ideate::CompletenessMetrics, _>(
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
            Err(ideate::IdeateError::Database(e)),
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
                Err(ideate::IdeateError::Database(e)),
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
}

/// Regenerate PRD with a different template's style/format
pub async fn regenerate_prd_with_template(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<RegeneratePRDWithTemplateRequest>,
) -> impl IntoResponse {
    info!(
        "Regenerating PRD for session: {} with template: {}",
        session_id, request.template_id
    );

    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .regenerate_with_template(DEFAULT_USER_ID, &session_id, &request.template_id)
        .await;

    match result {
        Ok(prd) => ok_or_internal_error::<GeneratedPRD, String>(Ok(prd), ""),
        Err(e) => ok_or_internal_error::<GeneratedPRD, _>(
            Err(e),
            "Failed to regenerate PRD with template",
        ),
    }
}
