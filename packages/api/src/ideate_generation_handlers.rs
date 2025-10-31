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
use tracing::{error, info};

use super::response::{ok_or_internal_error, ok_or_not_found};
use ideate::{ExportFormat, ExportOptions, PRDAggregator, PRDGenerator};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
}

/// Regenerate PRD with a different template's style/format
pub async fn regenerate_prd_with_template(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<RegeneratePRDWithTemplateRequest>,
) -> impl IntoResponse {
    info!(
        "Regenerating PRD for session: {} with template: {} (provider: {:?}, model: {:?})",
        session_id, request.template_id, request.provider, request.model
    );

    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .regenerate_with_template(
            DEFAULT_USER_ID,
            &session_id,
            &request.template_id,
            request.provider.as_deref(),
            request.model.as_deref(),
        )
        .await;

    let markdown = match result {
        Ok(md) => md,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(
                Err(e),
                "Failed to regenerate PRD with template",
            )
        }
    };

    // Get the session to retrieve project_id and title
    use ideate::IdeateManager;
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Create PRD in projects system
    use orkee_projects::{create_prd, PRDSource, PRDStatus};

    // Generate a title based on the template and current timestamp
    let title = format!("{} ({})", session_id, request.template_id);

    let prd = match create_prd(
        &db.pool,
        &session.project_id,
        &title,
        &markdown,
        PRDStatus::Draft,
        PRDSource::Generated,
        Some(DEFAULT_USER_ID),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(
                Err(ideate::IdeateError::AIService(e.to_string())),
                "Failed to save regenerated PRD",
            )
        }
    };

    // Link the PRD back to the ideate session by setting ideate_session_id
    let _ = sqlx::query("UPDATE prds SET ideate_session_id = ? WHERE id = ?")
        .bind(&session_id)
        .bind(&prd.id)
        .execute(&db.pool)
        .await;

    // Return the created PRD
    let response = serde_json::to_value(&prd).unwrap_or_else(|_| serde_json::json!({}));
    ok_or_internal_error::<_, String>(Ok(response), "Failed to save regenerated PRD")
}

/// Regenerate PRD with a different template's style/format (streaming version)
pub async fn regenerate_prd_with_template_stream(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<RegeneratePRDWithTemplateRequest>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!(
        "Streaming PRD regeneration for session: {} with template: {} (provider: {:?}, model: {:?})",
        session_id, request.template_id, request.provider, request.model
    );

    let stream = async_stream::stream! {
        let generator = PRDGenerator::new(db.pool.clone());

        // Get the streaming text from AI
        let text_stream_result = generator
            .regenerate_with_template_stream(
                DEFAULT_USER_ID,
                &session_id,
                &request.template_id,
                request.provider.as_deref(),
                request.model.as_deref(),
            )
            .await;

        let text_stream = match text_stream_result {
            Ok(stream) => stream,
            Err(e) => {
                error!("Failed to start PRD regeneration stream: {:?}", e);
                let error_event = Event::default()
                    .json_data(json!({
                        "type": "error",
                        "message": format!("Failed to regenerate PRD: {}", e)
                    }))
                    .unwrap();
                yield Ok(error_event);
                return;
            }
        };

        // Stream the markdown chunks as they arrive from AI
        let mut accumulated_markdown = String::new();

        use futures::StreamExt;
        tokio::pin!(text_stream);
        while let Some(chunk_result) = text_stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    accumulated_markdown.push_str(&chunk);

                    let event = Event::default()
                        .json_data(json!({
                            "type": "chunk",
                            "content": chunk
                        }))
                        .unwrap();

                    yield Ok(event);
                }
                Err(e) => {
                    error!("Error in stream: {:?}", e);
                    let error_event = Event::default()
                        .json_data(json!({
                            "type": "error",
                            "message": format!("Streaming error: {}", e)
                        }))
                        .unwrap();
                    yield Ok(error_event);
                    return;
                }
            }
        }

        let markdown = accumulated_markdown;

        // Get the session to retrieve project_id and title
        use ideate::IdeateManager;
        let manager = IdeateManager::new(db.pool.clone());
        let session = match manager.get_session(&session_id).await {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to get session: {:?}", e);
                let error_event = Event::default()
                    .json_data(json!({
                        "type": "error",
                        "message": format!("Failed to get session: {}", e)
                    }))
                    .unwrap();
                yield Ok(error_event);
                return;
            }
        };

        // Create PRD in projects system
        use orkee_projects::{create_prd, PRDSource, PRDStatus};

        let title = format!("{} ({})", session_id, request.template_id);

        let prd = match create_prd(
            &db.pool,
            &session.project_id,
            &title,
            &markdown,
            PRDStatus::Draft,
            PRDSource::Generated,
            Some(DEFAULT_USER_ID),
        )
        .await
        {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to save PRD: {:?}", e);
                let error_event = Event::default()
                    .json_data(json!({
                        "type": "error",
                        "message": format!("Failed to save PRD: {}", e)
                    }))
                    .unwrap();
                yield Ok(error_event);
                return;
            }
        };

        // Link the PRD back to the ideate session
        let _ = sqlx::query("UPDATE prds SET ideate_session_id = ? WHERE id = ?")
            .bind(&session_id)
            .bind(&prd.id)
            .execute(&db.pool)
            .await;

        // Send completion event
        let complete_event = Event::default()
            .json_data(json!({
                "type": "complete",
                "prd_id": prd.id,
                "markdown": markdown
            }))
            .unwrap();
        yield Ok(complete_event);

        // Send done marker
        let done_event = Event::default().data("[DONE]");
        yield Ok(done_event);
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}
