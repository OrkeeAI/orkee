// ABOUTME: Aggregates data from all ideate_* tables into unified PRD structure
// ABOUTME: Handles missing/skipped sections and formats data for generation or export

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Row, Sqlite};
use tracing::{error, info};

use crate::error::{IdeateError, Result};
use crate::types::IdeateSession;

/// Complete aggregated PRD data from all sections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedPRDData {
    /// Session metadata
    pub session: IdeateSession,

    /// Overview section (may be empty if skipped)
    pub overview: Option<OverviewData>,

    /// User experience section
    pub ux: Option<UXData>,

    /// Technical architecture
    pub technical: Option<TechnicalData>,

    /// Development roadmap
    pub roadmap: Option<RoadmapData>,

    /// Dependency chain
    pub dependencies: Option<DependencyData>,

    /// Risks and mitigations
    pub risks: Option<RisksData>,

    /// Research findings (comprehensive mode)
    pub research: Option<ResearchData>,

    /// Expert roundtable insights (comprehensive mode)
    pub roundtable_insights: Option<Vec<RoundtableInsight>>,

    /// Sections that were skipped
    pub skipped_sections: Vec<String>,

    /// Completeness metrics
    pub completeness: CompletenessMetrics,
}

/// Overview section data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverviewData {
    pub problem_statement: Option<String>,
    pub target_audience: Option<String>,
    pub value_proposition: Option<String>,
    pub one_line_pitch: Option<String>,
    pub ai_generated: bool,
}

/// User experience data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UXData {
    pub personas: Vec<serde_json::Value>,
    pub user_flows: Vec<serde_json::Value>,
    pub ui_considerations: Option<String>,
    pub ux_principles: Option<String>,
    pub ai_generated: bool,
}

/// Technical architecture data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalData {
    pub components: Vec<serde_json::Value>,
    pub data_models: Vec<serde_json::Value>,
    pub apis: Vec<serde_json::Value>,
    pub infrastructure: Option<serde_json::Value>,
    pub tech_stack_quick: Option<String>,
    pub ai_generated: bool,
}

/// Development roadmap data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoadmapData {
    pub mvp_scope: Vec<String>,
    pub future_phases: Vec<serde_json::Value>,
}

/// Dependency chain data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyData {
    pub foundation_features: Vec<String>,
    pub visible_features: Vec<String>,
    pub enhancement_features: Vec<String>,
    pub dependency_graph: Option<serde_json::Value>,
    pub build_order: Option<Vec<BuildPhase>>,
}

/// Build phase with features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildPhase {
    pub phase_number: i32,
    pub phase_name: String,
    pub features: Vec<String>,
    pub can_parallelize: Vec<Vec<String>>,
}

/// Risks and mitigations data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RisksData {
    pub technical_risks: Vec<serde_json::Value>,
    pub mvp_scoping_risks: Vec<serde_json::Value>,
    pub resource_risks: Vec<serde_json::Value>,
    pub mitigations: Vec<serde_json::Value>,
}

/// Research findings data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchData {
    pub competitors: Vec<serde_json::Value>,
    pub similar_projects: Vec<serde_json::Value>,
    pub research_findings: Option<String>,
    pub technical_specs: Option<String>,
    pub reference_links: Vec<String>,
}

/// Expert roundtable insight
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableInsight {
    pub roundtable_id: String,
    pub topic: String,
    pub category: String,
    pub content: String,
    pub priority: String,
    pub source_expert: Option<String>,
}

/// Completeness metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessMetrics {
    pub total_sections: usize,
    pub completed_sections: usize,
    pub skipped_sections: usize,
    pub ai_filled_sections: usize,
    pub completion_percentage: f32,
    pub is_complete: bool,
}

/// Helper struct to group section data for completeness calculation
struct SectionData<'a> {
    overview: &'a Option<OverviewData>,
    ux: &'a Option<UXData>,
    technical: &'a Option<TechnicalData>,
    roadmap: &'a Option<RoadmapData>,
    dependencies: &'a Option<DependencyData>,
    risks: &'a Option<RisksData>,
    research: &'a Option<ResearchData>,
    skipped_sections: &'a [String],
}

/// PRD Aggregator service
pub struct PRDAggregator {
    pool: Pool<Sqlite>,
}

impl PRDAggregator {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Aggregate all session data into unified structure
    pub async fn aggregate_session_data(&self, session_id: &str) -> Result<AggregatedPRDData> {
        info!("Aggregating data for session: {}", session_id);

        // Get session
        let session = self.get_session(session_id).await?;

        // Determine skipped sections
        let skipped_sections = session.skipped_sections.clone().unwrap_or_default();

        // Aggregate each section (handling missing/skipped gracefully)
        let overview = self.get_overview_data(session_id).await.ok();
        let ux = self.get_ux_data(session_id).await.ok();
        let technical = self.get_technical_data(session_id).await.ok();
        let roadmap = self.get_roadmap_data(session_id).await.ok();
        let dependencies = self.get_dependency_data(session_id).await.ok();
        let risks = self.get_risks_data(session_id).await.ok();
        let research = self.get_research_data(session_id).await.ok();
        let roundtable_insights = self.get_roundtable_insights(session_id).await.ok();

        // Calculate completeness
        let completeness = self.calculate_completeness(SectionData {
            overview: &overview,
            ux: &ux,
            technical: &technical,
            roadmap: &roadmap,
            dependencies: &dependencies,
            risks: &risks,
            research: &research,
            skipped_sections: &skipped_sections,
        });

        info!(
            "Session {} aggregation complete: {:.1}% complete",
            session_id, completeness.completion_percentage
        );

        Ok(AggregatedPRDData {
            session,
            overview,
            ux,
            technical,
            roadmap,
            dependencies,
            risks,
            research,
            roundtable_insights,
            skipped_sections,
            completeness,
        })
    }

    /// Get session by ID
    async fn get_session(&self, session_id: &str) -> Result<IdeateSession> {
        let row = sqlx::query(
            "SELECT id, project_id, initial_description, mode, status, skipped_sections, current_section, generated_prd_id,
             created_at, updated_at FROM ideate_sessions WHERE id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch session {}: {}", session_id, e);
            IdeateError::SectionNotFound(format!("Session not found: {}", session_id))
        })?;

        Ok(IdeateSession {
            id: row
                .try_get("id")
                .map_err(|e| IdeateError::AIService(e.to_string()))?,
            project_id: row
                .try_get("project_id")
                .map_err(|e| IdeateError::AIService(e.to_string()))?,
            initial_description: row
                .try_get("initial_description")
                .map_err(|e| IdeateError::AIService(e.to_string()))?,
            mode: {
                let mode_str: String = row
                    .try_get("mode")
                    .map_err(|e| IdeateError::AIService(e.to_string()))?;
                serde_json::from_str(&format!(r#""{}""#, mode_str))
                    .map_err(|e| IdeateError::AIService(e.to_string()))?
            },
            status: {
                let status_str: String = row
                    .try_get("status")
                    .map_err(|e| IdeateError::AIService(e.to_string()))?;
                serde_json::from_str(&format!(r#""{}""#, status_str))
                    .map_err(|e| IdeateError::AIService(e.to_string()))?
            },
            skipped_sections: row
                .try_get::<Option<String>, _>("skipped_sections")
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok()),
            current_section: row.try_get("current_section").ok(),
            generated_prd_id: row.try_get("generated_prd_id").ok(),
            created_at: row
                .try_get::<String, _>("created_at")
                .ok()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            updated_at: row
                .try_get::<String, _>("updated_at")
                .ok()
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
        })
    }

    /// Get overview section data
    async fn get_overview_data(&self, session_id: &str) -> Result<OverviewData> {
        let row = sqlx::query(
            "SELECT problem_statement, target_audience, value_proposition, one_line_pitch, ai_generated
             FROM ideate_overview WHERE session_id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Overview not found".to_string()))?;

        Ok(OverviewData {
            problem_statement: row.try_get("problem_statement").ok(),
            target_audience: row.try_get("target_audience").ok(),
            value_proposition: row.try_get("value_proposition").ok(),
            one_line_pitch: row.try_get("one_line_pitch").ok(),
            ai_generated: row.try_get::<i64, _>("ai_generated").unwrap_or(0) == 1,
        })
    }

    /// Get UX section data
    async fn get_ux_data(&self, session_id: &str) -> Result<UXData> {
        let row = sqlx::query(
            "SELECT personas, user_flows, ui_considerations, ux_principles, ai_generated
             FROM ideate_ux WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("UX data not found".to_string()))?;

        Ok(UXData {
            personas: row
                .try_get::<Option<String>, _>("personas")
                .ok()
                .flatten()
                .and_then(|p| serde_json::from_str(&p).ok())
                .unwrap_or_default(),
            user_flows: row
                .try_get::<Option<String>, _>("user_flows")
                .ok()
                .flatten()
                .and_then(|f| serde_json::from_str(&f).ok())
                .unwrap_or_default(),
            ui_considerations: row.try_get("ui_considerations").ok(),
            ux_principles: row.try_get("ux_principles").ok(),
            ai_generated: row.try_get::<i64, _>("ai_generated").unwrap_or(0) == 1,
        })
    }

    /// Get technical architecture data
    async fn get_technical_data(&self, session_id: &str) -> Result<TechnicalData> {
        let row = sqlx::query(
            "SELECT components, data_models, apis, infrastructure, tech_stack_quick, ai_generated
             FROM ideate_technical WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Technical data not found".to_string()))?;

        Ok(TechnicalData {
            components: row
                .try_get::<Option<String>, _>("components")
                .ok()
                .flatten()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default(),
            data_models: row
                .try_get::<Option<String>, _>("data_models")
                .ok()
                .flatten()
                .and_then(|d| serde_json::from_str(&d).ok())
                .unwrap_or_default(),
            apis: row
                .try_get::<Option<String>, _>("apis")
                .ok()
                .flatten()
                .and_then(|a| serde_json::from_str(&a).ok())
                .unwrap_or_default(),
            infrastructure: row
                .try_get::<Option<String>, _>("infrastructure")
                .ok()
                .flatten()
                .and_then(|i| serde_json::from_str(&i).ok()),
            tech_stack_quick: row.try_get("tech_stack_quick").ok(),
            ai_generated: row.try_get::<i64, _>("ai_generated").unwrap_or(0) == 1,
        })
    }

    /// Get roadmap data
    async fn get_roadmap_data(&self, session_id: &str) -> Result<RoadmapData> {
        let row =
            sqlx::query("SELECT mvp_scope, future_phases FROM ideate_roadmap WHERE session_id = ?")
                .bind(session_id)
                .fetch_one(&self.pool)
                .await
                .map_err(|_| IdeateError::SectionNotFound("Roadmap not found".to_string()))?;

        Ok(RoadmapData {
            mvp_scope: row
                .try_get::<Option<String>, _>("mvp_scope")
                .ok()
                .flatten()
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default(),
            future_phases: row
                .try_get::<Option<String>, _>("future_phases")
                .ok()
                .flatten()
                .and_then(|f| serde_json::from_str(&f).ok())
                .unwrap_or_default(),
        })
    }

    /// Get dependency chain data
    async fn get_dependency_data(&self, session_id: &str) -> Result<DependencyData> {
        let row = sqlx::query(
            "SELECT foundation_features, visible_features, enhancement_features, dependency_graph
             FROM ideate_dependencies WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Dependencies not found".to_string()))?;

        // Try to fetch build order from optimization table
        let build_order = self.get_build_order(session_id).await.ok();

        Ok(DependencyData {
            foundation_features: row
                .try_get::<Option<String>, _>("foundation_features")
                .ok()
                .flatten()
                .and_then(|f| serde_json::from_str(&f).ok())
                .unwrap_or_default(),
            visible_features: row
                .try_get::<Option<String>, _>("visible_features")
                .ok()
                .flatten()
                .and_then(|v| serde_json::from_str(&v).ok())
                .unwrap_or_default(),
            enhancement_features: row
                .try_get::<Option<String>, _>("enhancement_features")
                .ok()
                .flatten()
                .and_then(|e| serde_json::from_str(&e).ok())
                .unwrap_or_default(),
            dependency_graph: row
                .try_get::<Option<String>, _>("dependency_graph")
                .ok()
                .flatten()
                .and_then(|g| serde_json::from_str(&g).ok()),
            build_order,
        })
    }

    /// Get risks and mitigations data
    async fn get_risks_data(&self, session_id: &str) -> Result<RisksData> {
        let row = sqlx::query(
            "SELECT technical_risks, mvp_scoping_risks, resource_risks, mitigations
             FROM ideate_risks WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Risks not found".to_string()))?;

        Ok(RisksData {
            technical_risks: row
                .try_get::<Option<String>, _>("technical_risks")
                .ok()
                .flatten()
                .and_then(|t| serde_json::from_str(&t).ok())
                .unwrap_or_default(),
            mvp_scoping_risks: row
                .try_get::<Option<String>, _>("mvp_scoping_risks")
                .ok()
                .flatten()
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default(),
            resource_risks: row
                .try_get::<Option<String>, _>("resource_risks")
                .ok()
                .flatten()
                .and_then(|r| serde_json::from_str(&r).ok())
                .unwrap_or_default(),
            mitigations: row
                .try_get::<Option<String>, _>("mitigations")
                .ok()
                .flatten()
                .and_then(|m| serde_json::from_str(&m).ok())
                .unwrap_or_default(),
        })
    }

    /// Get research findings
    async fn get_research_data(&self, session_id: &str) -> Result<ResearchData> {
        let row = sqlx::query(
            "SELECT competitors, similar_projects, research_findings, technical_specs, reference_links
             FROM ideate_research WHERE session_id = ?"
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Research not found".to_string()))?;

        Ok(ResearchData {
            competitors: row
                .try_get::<Option<String>, _>("competitors")
                .ok()
                .flatten()
                .and_then(|c| serde_json::from_str(&c).ok())
                .unwrap_or_default(),
            similar_projects: row
                .try_get::<Option<String>, _>("similar_projects")
                .ok()
                .flatten()
                .and_then(|s| serde_json::from_str(&s).ok())
                .unwrap_or_default(),
            research_findings: row.try_get("research_findings").ok(),
            technical_specs: row.try_get("technical_specs").ok(),
            reference_links: row
                .try_get::<Option<String>, _>("reference_links")
                .ok()
                .flatten()
                .and_then(|r| serde_json::from_str(&r).ok())
                .unwrap_or_default(),
        })
    }

    /// Get expert roundtable insights
    async fn get_roundtable_insights(&self, session_id: &str) -> Result<Vec<RoundtableInsight>> {
        let rows = sqlx::query(
            "SELECT i.id, i.roundtable_id, r.topic, i.category, i.content, i.priority, i.source_expert
             FROM roundtable_insights i
             INNER JOIN roundtable_sessions r ON i.roundtable_id = r.id
             WHERE r.ideate_session_id = ?
             ORDER BY i.priority DESC, i.created_at"
        )
        .bind(session_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Roundtable insights not found".to_string()))?;

        Ok(rows
            .into_iter()
            .filter_map(|row| {
                Some(RoundtableInsight {
                    roundtable_id: row.try_get("roundtable_id").ok()?,
                    topic: row.try_get("topic").ok()?,
                    category: row.try_get("category").ok()?,
                    content: row.try_get("content").ok()?,
                    priority: row.try_get("priority").ok()?,
                    source_expert: row.try_get("source_expert").ok(),
                })
            })
            .collect())
    }

    /// Get build order from optimization table
    async fn get_build_order(&self, session_id: &str) -> Result<Vec<BuildPhase>> {
        let row = sqlx::query(
            "SELECT phases FROM build_order_optimization WHERE session_id = ? ORDER BY created_at DESC LIMIT 1"
        )
        .bind(session_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|_| IdeateError::SectionNotFound("Build order not found".to_string()))?;

        row.try_get::<Option<String>, _>("phases")
            .ok()
            .flatten()
            .and_then(|p| serde_json::from_str(&p).ok())
            .ok_or_else(|| IdeateError::AIService("Invalid build order JSON".to_string()))
    }

    /// Calculate completeness metrics
    fn calculate_completeness(&self, sections: SectionData) -> CompletenessMetrics {
        let total_sections = 7; // overview, ux, technical, roadmap, dependencies, risks, research
        let mut completed_sections = 0;
        let mut ai_filled = 0;

        if let Some(o) = sections.overview {
            completed_sections += 1;
            if o.ai_generated {
                ai_filled += 1;
            }
        }
        if sections.ux.is_some() {
            completed_sections += 1;
        }
        if sections.technical.is_some() {
            completed_sections += 1;
        }
        if sections.roadmap.is_some() {
            completed_sections += 1;
        }
        if sections.dependencies.is_some() {
            completed_sections += 1;
        }
        if sections.risks.is_some() {
            completed_sections += 1;
        }
        if sections.research.is_some() {
            completed_sections += 1;
        }

        let completion_percentage = (completed_sections as f32 / total_sections as f32) * 100.0;
        let is_complete = completed_sections == total_sections;

        CompletenessMetrics {
            total_sections,
            completed_sections,
            skipped_sections: sections.skipped_sections.len(),
            ai_filled_sections: ai_filled,
            completion_percentage,
            is_complete,
        }
    }
}
