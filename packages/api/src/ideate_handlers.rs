// ABOUTME: HTTP request handlers for PRD ideation operations
// ABOUTME: Handles session management, section updates, and completion status

use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use tracing::{info, warn};

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};
use orkee_ideate::prd_generator::GeneratedPRD;
use orkee_ideate::{
    CreateIdeateSessionInput, CreateTemplateInput, IdeateDependencies, IdeateManager, IdeateMode,
    IdeateOverview, IdeateResearch, IdeateRisks, IdeateRoadmap, IdeateStatus, IdeateTechnical,
    IdeateUX, PRDGenerator, SkipSectionRequest, TemplateManager, UpdateIdeateSessionInput,
};
use orkee_projects::{self as projects, DbState};

// TODO: Replace with proper user authentication
const DEFAULT_USER_ID: &str = "default-user";

/// Request body for starting a ideateing session
#[derive(Deserialize)]
pub struct StartIdeateRequest {
    #[serde(rename = "projectId")]
    pub project_id: String,
    #[serde(rename = "initialDescription")]
    pub initial_description: String,
    pub mode: IdeateMode,
    #[serde(rename = "templateId")]
    pub template_id: Option<String>,
}

/// Start a new ideateing session
pub async fn start_ideate(
    State(db): State<DbState>,
    Json(request): Json<StartIdeateRequest>,
) -> impl IntoResponse {
    info!(
        "Starting ideate session for project: {} (mode: {:?}), template: {:?}",
        request.project_id, request.mode, request.template_id
    );

    let manager = IdeateManager::new(db.pool.clone());
    let input = CreateIdeateSessionInput {
        project_id: request.project_id,
        initial_description: request.initial_description,
        mode: request.mode,
        template_id: request.template_id,
    };

    let result = manager.create_session(input).await;
    created_or_internal_error(result, "Failed to start ideate session")
}

/// Get a ideateing session by ID
pub async fn get_ideate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting ideate session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_session(&session_id).await;
    ok_or_not_found(result, "Ideate session not found")
}

/// List all ideateing sessions for a project
pub async fn list_ideates(
    State(db): State<DbState>,
    Path(project_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing ideate sessions for project: {}", project_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.list_sessions(&project_id).await;
    ok_or_internal_error(result, "Failed to list ideate sessions")
}

/// Request body for updating a session
#[derive(Deserialize)]
pub struct UpdateIdeateRequest {
    #[serde(rename = "initialDescription")]
    pub initial_description: Option<String>,
    pub mode: Option<IdeateMode>,
    pub status: Option<IdeateStatus>,
    #[serde(rename = "skippedSections")]
    pub skipped_sections: Option<Vec<String>>,
}

/// Update a ideateing session
pub async fn update_ideate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<UpdateIdeateRequest>,
) -> impl IntoResponse {
    info!("Updating ideate session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let input = UpdateIdeateSessionInput {
        initial_description: request.initial_description,
        mode: request.mode,
        status: request.status,
        skipped_sections: request.skipped_sections,
        current_section: None,
    };

    let result = manager.update_session(&session_id, input).await;
    ok_or_internal_error(result, "Failed to update ideate session")
}

/// Delete a ideateing session
pub async fn delete_ideate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting ideate session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_session(&session_id).await;
    ok_or_internal_error(result, "Failed to delete ideate session")
}

/// Skip a section with optional AI fill
pub async fn skip_section(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<SkipSectionRequest>,
) -> impl IntoResponse {
    info!(
        "Skipping section '{}' for session: {} (AI fill: {})",
        request.section, session_id, request.ai_fill
    );

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.skip_section(&session_id, request).await;
    ok_or_internal_error(result, "Failed to skip section")
}

/// Get session completion status
pub async fn get_status(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting completion status for session: {}", session_id);

    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_completion_status(&session_id).await;
    ok_or_internal_error(result, "Failed to get session status")
}

// ============================================================================
// QUICK MODE ENDPOINTS
// ============================================================================

/// Request body for quick PRD generation
#[derive(Deserialize)]
pub struct QuickGenerateRequest {
    pub provider: Option<String>,
    pub model: Option<String>,
}

/// Generate a complete PRD from the session's initial description (Quick Mode)
pub async fn quick_generate(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<QuickGenerateRequest>,
) -> impl IntoResponse {
    info!(
        "Generating complete PRD for session: {} with provider: {:?}, model: {:?}",
        session_id, request.provider, request.model
    );

    // Get the session to retrieve the description
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Generate PRD using the generator with optional provider and model
    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .generate_complete_prd_with_model(
            DEFAULT_USER_ID,
            &session.initial_description,
            request.provider,
            request.model,
        )
        .await;

    match result {
        Ok(prd) => {
            // Persist section data to ideate_ tables
            if let Err(e) = persist_generated_prd(&manager, &session_id, &prd).await {
                warn!(
                    "Failed to persist PRD sections for session {}: {}",
                    session_id, e
                );
                return ok_or_internal_error::<serde_json::Value, _>(
                    Err(format!("PRD generated but failed to persist: {}", e)),
                    "Failed to persist PRD",
                );
            }

            // Convert to JSON for response
            let json_value = serde_json::to_value(&prd).unwrap_or_else(|_| serde_json::json!({}));
            ok_or_internal_error::<_, String>(Ok(json_value), "Failed to generate PRD")
        }
        Err(e) => ok_or_internal_error::<serde_json::Value, _>(Err(e), "Failed to generate PRD"),
    }
}

/// Persist generated PRD sections to ideate_ database tables
async fn persist_generated_prd(
    manager: &IdeateManager,
    session_id: &str,
    prd: &GeneratedPRD,
) -> Result<(), String> {
    use chrono::Utc;

    // Persist overview section
    if let Some(overview) = &prd.overview {
        let ideate_overview = IdeateOverview {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            problem_statement: Some(overview.problem_statement.clone()),
            target_audience: Some(overview.target_audience.clone()),
            value_proposition: Some(overview.value_proposition.clone()),
            one_line_pitch: overview.one_line_pitch.clone(),
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_overview(session_id, ideate_overview)
            .await
            .map_err(|e| format!("Failed to save overview: {}", e))?;
    }

    // Persist UX section - convert prd_generator types to ideate types using serde
    if let Some(ux) = &prd.ux {
        let personas = ux
            .personas
            .as_ref()
            .and_then(|p| serde_json::from_value(serde_json::to_value(p).ok()?).ok());
        let user_flows = ux
            .user_flows
            .as_ref()
            .and_then(|f| serde_json::from_value(serde_json::to_value(f).ok()?).ok());

        let ideate_ux = IdeateUX {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            personas,
            user_flows,
            ui_considerations: ux.ui_considerations.clone(),
            ux_principles: ux.ux_principles.clone(),
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_ux(session_id, ideate_ux)
            .await
            .map_err(|e| format!("Failed to save UX: {}", e))?;
    }

    // Persist Technical section - convert prd_generator types to ideate types using serde
    if let Some(technical) = &prd.technical {
        let components = technical
            .components
            .as_ref()
            .and_then(|c| serde_json::from_value(serde_json::to_value(c).ok()?).ok());
        let data_models = technical
            .data_models
            .as_ref()
            .and_then(|d| serde_json::from_value(serde_json::to_value(d).ok()?).ok());
        let apis = technical
            .apis
            .as_ref()
            .and_then(|a| serde_json::from_value(serde_json::to_value(a).ok()?).ok());
        let infrastructure = technical
            .infrastructure
            .as_ref()
            .and_then(|i| serde_json::from_value(serde_json::to_value(i).ok()?).ok());

        let ideate_technical = IdeateTechnical {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            components,
            data_models,
            apis,
            infrastructure,
            tech_stack_quick: technical.tech_stack_quick.clone(),
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_technical(session_id, ideate_technical)
            .await
            .map_err(|e| format!("Failed to save technical: {}", e))?;
    }

    // Persist Roadmap section - convert prd_generator types to ideate types using serde
    if let Some(roadmap) = &prd.roadmap {
        let future_phases = roadmap
            .future_phases
            .as_ref()
            .and_then(|p| serde_json::from_value(serde_json::to_value(p).ok()?).ok());

        let ideate_roadmap = IdeateRoadmap {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            mvp_scope: roadmap.mvp_scope.clone(),
            future_phases,
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_roadmap(session_id, ideate_roadmap)
            .await
            .map_err(|e| format!("Failed to save roadmap: {}", e))?;
    }

    // Persist Dependencies section - preserve full DependencyFeature data as JSON strings
    if let Some(dependencies) = &prd.dependencies {
        let dependency_graph = dependencies
            .dependency_graph
            .as_ref()
            .and_then(|g| serde_json::from_value(serde_json::to_value(g).ok()?).ok());

        // Convert DependencyFeature objects to JSON strings to preserve full data
        let foundation_features = dependencies
            .foundation_features
            .as_ref()
            .map(|features| {
                features
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
            })
            .and_then(|v| if v.is_empty() { None } else { Some(v) });

        let visible_features = dependencies
            .visible_features
            .as_ref()
            .map(|features| {
                features
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
            })
            .and_then(|v| if v.is_empty() { None } else { Some(v) });

        let enhancement_features = dependencies
            .enhancement_features
            .as_ref()
            .map(|features| {
                features
                    .iter()
                    .filter_map(|f| serde_json::to_string(f).ok())
                    .collect::<Vec<String>>()
            })
            .and_then(|v| if v.is_empty() { None } else { Some(v) });

        let ideate_dependencies = IdeateDependencies {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            foundation_features,
            visible_features,
            enhancement_features,
            dependency_graph,
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_dependencies(session_id, ideate_dependencies)
            .await
            .map_err(|e| format!("Failed to save dependencies: {}", e))?;
    }

    // Persist Risks section - convert prd_generator types to ideate types using serde
    if let Some(risks) = &prd.risks {
        let technical_risks = risks
            .technical_risks
            .as_ref()
            .and_then(|t| serde_json::from_value(serde_json::to_value(t).ok()?).ok());
        let mvp_scoping_risks = risks
            .mvp_scoping_risks
            .as_ref()
            .and_then(|m| serde_json::from_value(serde_json::to_value(m).ok()?).ok());
        let resource_risks = risks
            .resource_risks
            .as_ref()
            .and_then(|r| serde_json::from_value(serde_json::to_value(r).ok()?).ok());
        let mitigations = risks
            .mitigations
            .as_ref()
            .and_then(|m| serde_json::from_value(serde_json::to_value(m).ok()?).ok());

        let ideate_risks = IdeateRisks {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            technical_risks,
            mvp_scoping_risks,
            resource_risks,
            mitigations,
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_risks(session_id, ideate_risks)
            .await
            .map_err(|e| format!("Failed to save risks: {}", e))?;
    }

    // Persist Research section - convert prd_generator types to ideate types using serde
    if let Some(research) = &prd.research {
        let competitors = research
            .competitors
            .as_ref()
            .and_then(|c| serde_json::from_value(serde_json::to_value(c).ok()?).ok());
        let similar_projects = research
            .similar_projects
            .as_ref()
            .and_then(|s| serde_json::from_value(serde_json::to_value(s).ok()?).ok());
        let reference_links = research
            .reference_links
            .as_ref()
            .and_then(|r| serde_json::from_value(serde_json::to_value(r).ok()?).ok());

        let ideate_research = IdeateResearch {
            id: uuid::Uuid::new_v4().to_string(),
            session_id: session_id.to_string(),
            competitors,
            similar_projects,
            research_findings: research.research_findings.clone(),
            technical_specs: research.technical_specs.clone(),
            reference_links,
            ai_generated: true,
            created_at: Utc::now(),
        };
        manager
            .save_research(session_id, ideate_research)
            .await
            .map_err(|e| format!("Failed to save research: {}", e))?;
    }

    Ok(())
}

/// Request body for expanding a specific section
#[derive(Deserialize)]
pub struct QuickExpandRequest {
    pub section: String,
    #[serde(rename = "context")]
    pub context: Option<String>,
}

/// Expand a specific section of the PRD
pub async fn quick_expand(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<QuickExpandRequest>,
) -> impl IntoResponse {
    info!(
        "Expanding section '{}' for session: {}",
        request.section, session_id
    );

    // Get the session to retrieve the description
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Generate specific section
    let generator = PRDGenerator::new(db.pool.clone());
    let result = generator
        .generate_section(
            DEFAULT_USER_ID,
            &request.section,
            &session.initial_description,
            request.context.as_deref(),
        )
        .await;

    ok_or_internal_error(result, "Failed to expand section")
}

/// Get a preview of the generated PRD in markdown format
pub async fn get_preview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting PRD preview for session: {}", session_id);

    // Get the session
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // If session has a saved PRD, retrieve it instead of regenerating
    if let Some(prd_id) = &session.generated_prd_id {
        info!("Session has saved PRD, retrieving it: {}", prd_id);
        match projects::get_prd(&db.pool, prd_id).await {
            Ok(saved_prd) => {
                info!("Successfully retrieved saved PRD: {}", prd_id);

                // Load structured section data from ideate_ tables instead of parsing markdown
                let mut sections = serde_json::Map::new();

                // Load each section from the database
                if let Ok(Some(overview)) = manager.get_overview(&session_id).await {
                    sections.insert(
                        "overview".to_string(),
                        serde_json::to_value(overview).unwrap_or(serde_json::Value::Null),
                    );
                }

                if let Ok(Some(ux)) = manager.get_ux(&session_id).await {
                    sections.insert(
                        "ux".to_string(),
                        serde_json::to_value(ux).unwrap_or(serde_json::Value::Null),
                    );
                }

                if let Ok(Some(technical)) = manager.get_technical(&session_id).await {
                    sections.insert(
                        "technical".to_string(),
                        serde_json::to_value(technical).unwrap_or(serde_json::Value::Null),
                    );
                }

                if let Ok(Some(roadmap)) = manager.get_roadmap(&session_id).await {
                    sections.insert(
                        "roadmap".to_string(),
                        serde_json::to_value(roadmap).unwrap_or(serde_json::Value::Null),
                    );
                }

                if let Ok(Some(dependencies)) = manager.get_dependencies(&session_id).await {
                    sections.insert(
                        "dependencies".to_string(),
                        serde_json::to_value(dependencies).unwrap_or(serde_json::Value::Null),
                    );
                }

                if let Ok(Some(risks)) = manager.get_risks(&session_id).await {
                    sections.insert(
                        "risks".to_string(),
                        serde_json::to_value(risks).unwrap_or(serde_json::Value::Null),
                    );
                }

                if let Ok(Some(research)) = manager.get_research(&session_id).await {
                    sections.insert(
                        "appendix".to_string(),
                        serde_json::to_value(research).unwrap_or(serde_json::Value::Null),
                    );
                }

                let response = serde_json::json!({
                    "markdown": saved_prd.content_markdown,
                    "content": saved_prd.content_markdown,
                    "sections": sections
                });
                return ok_or_internal_error::<_, String>(Ok(response), "Failed to get preview");
            }
            Err(e) => {
                info!("Failed to retrieve saved PRD, will regenerate: {}", e);
            }
        }
    }

    // Check if session has generated sections (Quick Mode stores sections in ideate tables)
    info!(
        "Checking for existing sections in ideate tables for session: {}",
        session_id
    );

    let mut sections = serde_json::Map::new();
    let mut has_sections = false;

    // Load each section from the database
    if let Ok(Some(overview)) = manager.get_overview(&session_id).await {
        info!("Found overview section for session: {}", session_id);
        sections.insert(
            "overview".to_string(),
            serde_json::to_value(overview).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    if let Ok(Some(ux)) = manager.get_ux(&session_id).await {
        info!("Found ux section for session: {}", session_id);
        sections.insert(
            "ux".to_string(),
            serde_json::to_value(ux).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    if let Ok(Some(technical)) = manager.get_technical(&session_id).await {
        info!("Found technical section for session: {}", session_id);
        sections.insert(
            "technical".to_string(),
            serde_json::to_value(technical).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    if let Ok(Some(roadmap)) = manager.get_roadmap(&session_id).await {
        info!("Found roadmap section for session: {}", session_id);
        sections.insert(
            "roadmap".to_string(),
            serde_json::to_value(roadmap).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    if let Ok(Some(dependencies)) = manager.get_dependencies(&session_id).await {
        info!("Found dependencies section for session: {}", session_id);
        sections.insert(
            "dependencies".to_string(),
            serde_json::to_value(dependencies).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    if let Ok(Some(risks)) = manager.get_risks(&session_id).await {
        info!("Found risks section for session: {}", session_id);
        sections.insert(
            "risks".to_string(),
            serde_json::to_value(risks).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    if let Ok(Some(research)) = manager.get_research(&session_id).await {
        info!("Found research section for session: {}", session_id);
        sections.insert(
            "appendix".to_string(),
            serde_json::to_value(research).unwrap_or(serde_json::Value::Null),
        );
        has_sections = true;
    }

    info!(
        "Section check complete for session {}: has_sections={}, section_count={}",
        session_id,
        has_sections,
        sections.len()
    );

    // If we have sections, return them
    if has_sections {
        info!("Found existing sections for session, returning them");

        // Generate markdown from sections
        let mut markdown_parts = Vec::new();
        if let Some(overview) = sections.get("overview") {
            markdown_parts.push(format!("## Overview\n\n{}", overview));
        }
        if let Some(ux) = sections.get("ux") {
            markdown_parts.push(format!("## UX\n\n{}", ux));
        }
        if let Some(technical) = sections.get("technical") {
            markdown_parts.push(format!("## Technical\n\n{}", technical));
        }
        if let Some(roadmap) = sections.get("roadmap") {
            markdown_parts.push(format!("## Roadmap\n\n{}", roadmap));
        }
        if let Some(dependencies) = sections.get("dependencies") {
            markdown_parts.push(format!("## Dependencies\n\n{}", dependencies));
        }
        if let Some(risks) = sections.get("risks") {
            markdown_parts.push(format!("## Risks\n\n{}", risks));
        }
        if let Some(appendix) = sections.get("appendix") {
            markdown_parts.push(format!("## Appendix\n\n{}", appendix));
        }

        let markdown = markdown_parts.join("\n\n");

        let response = serde_json::json!({
            "markdown": markdown,
            "content": markdown,
            "sections": sections
        });
        return ok_or_internal_error::<_, String>(Ok(response), "Failed to get preview");
    }

    info!(
        "No sections found for session {}, will attempt to generate or retrieve saved PRD",
        session_id
    );

    // No saved PRD and no sections, generate on-demand
    info!(
        "No saved PRD or sections, generating new one for session: {}",
        session_id
    );
    let generator = PRDGenerator::new(db.pool.clone());
    let prd = match generator
        .generate_complete_prd(DEFAULT_USER_ID, &session.initial_description)
        .await
    {
        Ok(p) => p,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Failed to generate PRD")
        }
    };

    // Format as markdown
    let markdown = generator.format_prd_markdown(&prd);

    // Return as JSON with markdown string
    let response = serde_json::json!({
        "markdown": markdown,
        "prd": prd
    });

    ok_or_internal_error::<_, String>(Ok(response), "Failed to get preview")
}

/// Request body for saving PRD
#[derive(Deserialize)]
pub struct SaveAsPRDRequest {
    pub title: String,
    #[serde(rename = "contentMarkdown")]
    pub content_markdown: String,
}

/// Save the generated PRD to the OpenSpec system
pub async fn save_as_prd(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<SaveAsPRDRequest>,
) -> impl IntoResponse {
    info!("Saving PRD for session: {}", session_id);

    // Get the session
    let manager = IdeateManager::new(db.pool.clone());
    let session = match manager.get_session(&session_id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Ideate session not found")
        }
    };

    // Create PRD in projects system
    use projects::{create_prd, PRDSource, PRDStatus};

    let prd = match create_prd(
        &db.pool,
        &session.project_id,
        &request.title,
        &request.content_markdown,
        PRDStatus::Draft,
        PRDSource::Generated,
        Some(DEFAULT_USER_ID),
    )
    .await
    {
        Ok(p) => p,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(
                Err(orkee_ideate::IdeateError::AIService(e.to_string())),
                "Failed to save PRD",
            )
        }
    };

    // Update session with generated PRD ID
    let update_input = UpdateIdeateSessionInput {
        initial_description: None,
        mode: None,
        status: Some(IdeateStatus::Completed),
        skipped_sections: None,
        current_section: None,
    };

    if let Err(e) = manager.update_session(&session_id, update_input).await {
        tracing::warn!("Failed to update session status: {}", e);
    }

    // Also update the generated_prd_id in the database directly
    let _ = sqlx::query("UPDATE ideate_sessions SET generated_prd_id = ? WHERE id = ?")
        .bind(&prd.id)
        .bind(&session_id)
        .execute(&db.pool)
        .await;

    // Return the created PRD
    let response = serde_json::to_value(&prd).unwrap_or_else(|_| serde_json::json!({}));
    ok_or_internal_error::<_, String>(Ok(response), "Failed to save PRD")
}

// ============================================================================
// GUIDED MODE - SECTION ENDPOINTS
// ============================================================================

// Overview Section
pub async fn save_overview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(overview): Json<orkee_ideate::IdeateOverview>,
) -> impl IntoResponse {
    info!("Saving overview section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_overview(&session_id, overview).await;
    ok_or_internal_error(result, "Failed to save overview section")
}

pub async fn get_overview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting overview section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_overview(&session_id).await;
    ok_or_internal_error(result, "Failed to get overview section")
}

pub async fn delete_overview(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting overview section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_overview(&session_id).await;
    ok_or_internal_error(result, "Failed to delete overview section")
}

// UX Section
pub async fn save_ux(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(ux): Json<orkee_ideate::IdeateUX>,
) -> impl IntoResponse {
    info!("Saving UX section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_ux(&session_id, ux).await;
    ok_or_internal_error(result, "Failed to save UX section")
}

pub async fn get_ux(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting UX section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_ux(&session_id).await;
    ok_or_internal_error(result, "Failed to get UX section")
}

pub async fn delete_ux(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting UX section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_ux(&session_id).await;
    ok_or_internal_error(result, "Failed to delete UX section")
}

// Technical Section
pub async fn save_technical(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(technical): Json<orkee_ideate::IdeateTechnical>,
) -> impl IntoResponse {
    info!("Saving technical section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_technical(&session_id, technical).await;
    ok_or_internal_error(result, "Failed to save technical section")
}

pub async fn get_technical(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting technical section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_technical(&session_id).await;
    ok_or_internal_error(result, "Failed to get technical section")
}

pub async fn delete_technical(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting technical section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_technical(&session_id).await;
    ok_or_internal_error(result, "Failed to delete technical section")
}

// Roadmap Section
pub async fn save_roadmap(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(roadmap): Json<orkee_ideate::IdeateRoadmap>,
) -> impl IntoResponse {
    info!("Saving roadmap section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_roadmap(&session_id, roadmap).await;
    ok_or_internal_error(result, "Failed to save roadmap section")
}

pub async fn get_roadmap(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting roadmap section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_roadmap(&session_id).await;
    ok_or_internal_error(result, "Failed to get roadmap section")
}

pub async fn delete_roadmap(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting roadmap section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_roadmap(&session_id).await;
    ok_or_internal_error(result, "Failed to delete roadmap section")
}

// Dependencies Section
pub async fn save_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(deps): Json<orkee_ideate::IdeateDependencies>,
) -> impl IntoResponse {
    info!("Saving dependencies section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_dependencies(&session_id, deps).await;
    ok_or_internal_error(result, "Failed to save dependencies section")
}

pub async fn get_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting dependencies section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to get dependencies section")
}

pub async fn delete_dependencies(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting dependencies section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_dependencies(&session_id).await;
    ok_or_internal_error(result, "Failed to delete dependencies section")
}

// Risks Section
pub async fn save_risks(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(risks): Json<orkee_ideate::IdeateRisks>,
) -> impl IntoResponse {
    info!("Saving risks section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_risks(&session_id, risks).await;
    ok_or_internal_error(result, "Failed to save risks section")
}

pub async fn get_risks(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting risks section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_risks(&session_id).await;
    ok_or_internal_error(result, "Failed to get risks section")
}

pub async fn delete_risks(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting risks section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_risks(&session_id).await;
    ok_or_internal_error(result, "Failed to delete risks section")
}

// Research Section
pub async fn save_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(research): Json<orkee_ideate::IdeateResearch>,
) -> impl IntoResponse {
    info!("Saving research section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.save_research(&session_id, research).await;
    ok_or_internal_error(result, "Failed to save research section")
}

pub async fn get_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting research section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_research(&session_id).await;
    ok_or_internal_error(result, "Failed to get research section")
}

pub async fn delete_research(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting research section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.delete_research(&session_id).await;
    ok_or_internal_error(result, "Failed to delete research section")
}

// Navigation Endpoints
pub async fn get_next_section(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting next section for session: {}", session_id);
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.get_next_section(&session_id).await;
    ok_or_internal_error(result, "Failed to get next section")
}

#[derive(Deserialize)]
pub struct NavigateToRequest {
    pub section: String,
}

pub async fn navigate_to(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<NavigateToRequest>,
) -> impl IntoResponse {
    info!(
        "Navigating to section '{}' for session: {}",
        request.section, session_id
    );
    let manager = IdeateManager::new(db.pool.clone());
    let result = manager.navigate_to(&session_id, &request.section).await;
    ok_or_internal_error(result, "Failed to navigate to section")
}

// ===================================
// Template Endpoints
// ===================================

/// Get all available templates (optionally filtered by category)
pub async fn list_templates(
    State(db): State<DbState>,
    axum::extract::Query(params): axum::extract::Query<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let category = params
        .get("category")
        .map(|s| s.as_str())
        .unwrap_or("quickstart");
    info!("Listing PRD templates for category: {}", category);
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.get_templates_by_category(category).await;
    ok_or_internal_error(result, "Failed to list templates")
}

/// Get a specific template by ID
pub async fn get_template(
    State(db): State<DbState>,
    Path(template_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting template: {}", template_id);
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.get_template(&template_id).await;
    ok_or_not_found(result, "Template not found")
}

/// Get templates by project type
pub async fn get_templates_by_type(
    State(db): State<DbState>,
    Path(project_type): Path<String>,
) -> impl IntoResponse {
    info!("Getting templates for project type: {}", project_type);
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.get_templates_by_type(&project_type).await;
    ok_or_internal_error(result, "Failed to get templates by type")
}

#[derive(Deserialize)]
pub struct SuggestTemplateRequest {
    pub description: String,
}

/// Suggest a template based on description
pub async fn suggest_template(
    State(db): State<DbState>,
    Json(request): Json<SuggestTemplateRequest>,
) -> impl IntoResponse {
    info!("Suggesting template for description");
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.suggest_template(&request.description).await;
    ok_or_internal_error(result, "Failed to suggest template")
}

// ============================================================================
// QUICK MODE - SAVE SECTIONS ENDPOINT
// ============================================================================

/// Request body for saving all PRD sections at once (for frontend streaming)
/// Note: Uses separate DTOs without database fields (id, session_id, created_at, ai_generated)
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSectionsRequest {
    pub overview: Option<OverviewInput>,
    pub ux: Option<UXInput>,
    pub technical: Option<TechnicalInput>,
    pub roadmap: Option<RoadmapInput>,
    pub dependencies: Option<DependenciesInput>,
    pub risks: Option<RisksInput>,
    pub research: Option<ResearchInput>,
    pub template_id: Option<String>,
}

// DTOs for API input (match frontend TypeScript schemas with camelCase)
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewInput {
    pub problem_statement: Option<String>,
    pub target_audience: Option<String>,
    pub value_proposition: Option<String>,
    pub one_line_pitch: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UXInput {
    pub personas: Option<serde_json::Value>,
    pub user_flows: Option<serde_json::Value>,
    pub ui_considerations: Option<String>,
    pub ux_principles: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TechnicalInput {
    pub components: Option<serde_json::Value>,
    pub data_models: Option<serde_json::Value>,
    pub apis: Option<serde_json::Value>,
    pub infrastructure: Option<serde_json::Value>,
    pub tech_stack_quick: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RoadmapInput {
    pub mvp_scope: Option<Vec<String>>,
    pub future_phases: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DependenciesInput {
    pub foundation_features: Option<Vec<String>>,
    pub visible_features: Option<Vec<String>>,
    pub enhancement_features: Option<Vec<String>>,
    pub dependency_graph: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RisksInput {
    pub technical_risks: Option<serde_json::Value>,
    pub mvp_scoping_risks: Option<serde_json::Value>,
    pub resource_risks: Option<serde_json::Value>,
    pub mitigations: Option<serde_json::Value>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchInput {
    pub competitors: Option<serde_json::Value>,
    pub similar_projects: Option<serde_json::Value>,
    pub research_findings: Option<String>,
    pub technical_specs: Option<String>,
    pub reference_links: Option<serde_json::Value>,
}

/// Save generated PRD sections to database (used after frontend streaming)
pub async fn save_sections(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<SaveSectionsRequest>,
) -> impl IntoResponse {
    info!("Saving PRD sections for session: {}", session_id);

    use chrono::Utc;
    use nanoid::nanoid;

    let manager = IdeateManager::new(db.pool.clone());
    let mut saved_sections = Vec::new();
    let mut errors = Vec::new();

    // Convert DTOs to full entities and save each section if provided
    if let Some(input) = request.overview {
        let entity = orkee_ideate::IdeateOverview {
            id: nanoid!(),
            session_id: session_id.clone(),
            problem_statement: input.problem_statement,
            target_audience: input.target_audience,
            value_proposition: input.value_proposition,
            one_line_pitch: input.one_line_pitch,
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_overview(&session_id, entity).await {
            Ok(_) => saved_sections.push("overview"),
            Err(e) => errors.push(format!("overview: {}", e)),
        }
    }

    if let Some(input) = request.ux {
        let entity = orkee_ideate::IdeateUX {
            id: nanoid!(),
            session_id: session_id.clone(),
            personas: input.personas.and_then(|v| serde_json::from_value(v).ok()),
            user_flows: input
                .user_flows
                .and_then(|v| serde_json::from_value(v).ok()),
            ui_considerations: input.ui_considerations,
            ux_principles: input.ux_principles,
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_ux(&session_id, entity).await {
            Ok(_) => saved_sections.push("ux"),
            Err(e) => errors.push(format!("ux: {}", e)),
        }
    }

    if let Some(input) = request.technical {
        let entity = orkee_ideate::IdeateTechnical {
            id: nanoid!(),
            session_id: session_id.clone(),
            components: input
                .components
                .and_then(|v| serde_json::from_value(v).ok()),
            data_models: input
                .data_models
                .and_then(|v| serde_json::from_value(v).ok()),
            apis: input.apis.and_then(|v| serde_json::from_value(v).ok()),
            infrastructure: input
                .infrastructure
                .and_then(|v| serde_json::from_value(v).ok()),
            tech_stack_quick: input.tech_stack_quick,
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_technical(&session_id, entity).await {
            Ok(_) => saved_sections.push("technical"),
            Err(e) => errors.push(format!("technical: {}", e)),
        }
    }

    if let Some(input) = request.roadmap {
        let entity = orkee_ideate::IdeateRoadmap {
            id: nanoid!(),
            session_id: session_id.clone(),
            mvp_scope: input.mvp_scope,
            future_phases: input
                .future_phases
                .and_then(|v| serde_json::from_value(v).ok()),
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_roadmap(&session_id, entity).await {
            Ok(_) => saved_sections.push("roadmap"),
            Err(e) => errors.push(format!("roadmap: {}", e)),
        }
    }

    if let Some(input) = request.dependencies {
        let entity = orkee_ideate::IdeateDependencies {
            id: nanoid!(),
            session_id: session_id.clone(),
            foundation_features: input.foundation_features,
            visible_features: input.visible_features,
            enhancement_features: input.enhancement_features,
            dependency_graph: input
                .dependency_graph
                .and_then(|v| serde_json::from_value(v).ok()),
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_dependencies(&session_id, entity).await {
            Ok(_) => saved_sections.push("dependencies"),
            Err(e) => errors.push(format!("dependencies: {}", e)),
        }
    }

    if let Some(input) = request.risks {
        let entity = orkee_ideate::IdeateRisks {
            id: nanoid!(),
            session_id: session_id.clone(),
            technical_risks: input
                .technical_risks
                .and_then(|v| serde_json::from_value(v).ok()),
            mvp_scoping_risks: input
                .mvp_scoping_risks
                .and_then(|v| serde_json::from_value(v).ok()),
            resource_risks: input
                .resource_risks
                .and_then(|v| serde_json::from_value(v).ok()),
            mitigations: input
                .mitigations
                .and_then(|v| serde_json::from_value(v).ok()),
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_risks(&session_id, entity).await {
            Ok(_) => saved_sections.push("risks"),
            Err(e) => errors.push(format!("risks: {}", e)),
        }
    }

    if let Some(input) = request.research {
        let entity = orkee_ideate::IdeateResearch {
            id: nanoid!(),
            session_id: session_id.clone(),
            competitors: input
                .competitors
                .and_then(|v| serde_json::from_value(v).ok()),
            similar_projects: input
                .similar_projects
                .and_then(|v| serde_json::from_value(v).ok()),
            research_findings: input.research_findings,
            technical_specs: input.technical_specs,
            reference_links: input
                .reference_links
                .and_then(|v| serde_json::from_value(v).ok()),
            ai_generated: true,
            created_at: Utc::now(),
        };
        match manager.save_research(&session_id, entity).await {
            Ok(_) => saved_sections.push("research"),
            Err(e) => errors.push(format!("research: {}", e)),
        }
    }

    // Update session with template_id if provided
    if let Some(template_id) = request.template_id {
        match sqlx::query("UPDATE ideate_sessions SET template_id = ? WHERE id = ?")
            .bind(&template_id)
            .bind(&session_id)
            .execute(&db.pool)
            .await
        {
            Ok(_) => {
                info!(
                    "Updated session {} with template_id: {}",
                    session_id, template_id
                );
            }
            Err(e) => {
                tracing::warn!("Failed to update session template_id: {}", e);
                errors.push(format!("template_id: {}", e));
            }
        }
    }

    // Return result with saved sections and any errors
    let response = serde_json::json!({
        "message": "Sections saved",
        "saved": saved_sections,
        "errors": if errors.is_empty() { None } else { Some(errors) }
    });

    ok_or_internal_error::<_, String>(Ok(response), "Failed to save sections")
}

/// Create a new template
pub async fn create_template(
    State(db): State<DbState>,
    Json(input): Json<CreateTemplateInput>,
) -> impl IntoResponse {
    info!("Creating new template: {}", input.name);
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.create_template(input).await;
    created_or_internal_error(result, "Failed to create template")
}

/// Update an existing template
pub async fn update_template(
    State(db): State<DbState>,
    Path(template_id): Path<String>,
    Json(input): Json<CreateTemplateInput>,
) -> impl IntoResponse {
    info!("Updating template: {}", template_id);
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.update_template(&template_id, input).await;
    ok_or_internal_error(result, "Failed to update template")
}

/// Delete a template (only user-created templates can be deleted)
pub async fn delete_template(
    State(db): State<DbState>,
    Path(template_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting template: {}", template_id);
    let manager = TemplateManager::new(db.pool.clone());
    let result = manager.delete_template(&template_id).await;
    ok_or_internal_error(result, "Failed to delete template")
}
