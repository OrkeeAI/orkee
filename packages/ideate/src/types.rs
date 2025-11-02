// ABOUTME: Type definitions for PRD ideation
// ABOUTME: Defines session modes, statuses, and data structures for all PRD sections

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Ideating session mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum IdeateMode {
    /// Quick mode: one-liner → complete PRD
    Quick,
    /// Guided mode: step-by-step with optional sections and advanced research tools
    Guided,
    /// Conversational mode: chat-based PRD discovery through conversation
    Conversational,
}

/// Ideating session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum IdeateStatus {
    /// Initial state
    Draft,
    /// User is actively working on it
    InProgress,
    /// Ready to generate PRD
    ReadyForPrd,
    /// PRD has been generated
    Completed,
}

/// Main brainstorming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateSession {
    pub id: String,
    pub project_id: String,
    pub initial_description: String,
    pub mode: IdeateMode,
    pub status: IdeateStatus,
    pub skipped_sections: Option<Vec<String>>,
    pub current_section: Option<String>,
    pub research_tools_enabled: bool,
    pub generated_prd_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Input for creating a new brainstorming session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIdeateSessionInput {
    pub project_id: String,
    pub initial_description: String,
    pub mode: IdeateMode,
    pub template_id: Option<String>,
    #[serde(default)]
    pub research_tools_enabled: bool,
}

/// Input for updating a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateIdeateSessionInput {
    pub initial_description: Option<String>,
    pub mode: Option<IdeateMode>,
    pub status: Option<IdeateStatus>,
    pub skipped_sections: Option<Vec<String>>,
    pub current_section: Option<String>,
    pub research_tools_enabled: Option<bool>,
}

/// Overview section data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateOverview {
    pub id: String,
    pub session_id: String,
    pub problem_statement: Option<String>,
    pub target_audience: Option<String>,
    pub value_proposition: Option<String>,
    pub one_line_pitch: Option<String>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// Feature with dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateFeature {
    pub id: String,
    pub session_id: String,
    pub feature_name: String,
    pub what_it_does: Option<String>,
    pub why_important: Option<String>,
    pub how_it_works: Option<String>,
    pub depends_on: Option<Vec<String>>,
    pub enables: Option<Vec<String>>,
    pub build_phase: i32, // 1=foundation, 2=visible, 3=enhancement
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
}

/// User experience section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateUX {
    pub id: String,
    pub session_id: String,
    pub personas: Option<Vec<Persona>>,
    pub user_flows: Option<Vec<UserFlow>>,
    pub ui_considerations: Option<String>,
    pub ux_principles: Option<String>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// User persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub role: String,
    pub goals: Vec<String>,
    pub pain_points: Vec<String>,
}

/// User flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFlow {
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub touchpoints: Vec<String>,
}

/// Step in a user flow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub action: String,
    pub screen: String,
    pub notes: Option<String>,
}

/// Technical architecture section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateTechnical {
    pub id: String,
    pub session_id: String,
    pub components: Option<Vec<Component>>,
    pub data_models: Option<Vec<DataModel>>,
    pub apis: Option<Vec<API>>,
    pub infrastructure: Option<Infrastructure>,
    pub tech_stack_quick: Option<String>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// System component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub purpose: String,
    pub technology: Option<String>,
}

/// Data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModel {
    pub name: String,
    pub fields: Vec<Field>,
}

/// Field in a data model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub field_type: String,
    pub required: bool,
}

/// API definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct API {
    pub name: String,
    pub purpose: String,
    pub endpoints: Vec<String>,
}

/// Infrastructure requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Infrastructure {
    pub hosting: Option<String>,
    pub database: Option<String>,
    pub caching: Option<String>,
    pub file_storage: Option<String>,
}

/// Development roadmap section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateRoadmap {
    pub id: String,
    pub session_id: String,
    pub mvp_scope: Option<Vec<String>>,
    pub future_phases: Option<Vec<Phase>>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// Development phase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub features: Vec<String>,
    pub goals: Vec<String>,
}

/// Logical dependency chain section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateDependencies {
    pub id: String,
    pub session_id: String,
    pub foundation_features: Option<Vec<String>>,
    pub visible_features: Option<Vec<String>>,
    pub enhancement_features: Option<Vec<String>>,
    pub dependency_graph: Option<DependencyGraph>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// Dependency graph for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub phase: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
}

/// Risks and mitigations section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateRisks {
    pub id: String,
    pub session_id: String,
    pub technical_risks: Option<Vec<Risk>>,
    pub mvp_scoping_risks: Option<Vec<Risk>>,
    pub resource_risks: Option<Vec<Risk>>,
    pub mitigations: Option<Vec<Mitigation>>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// Risk severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk probability levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskProbability {
    Low,
    Medium,
    High,
}

/// Risk definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub severity: RiskSeverity,
    pub probability: RiskProbability,
}

/// Mitigation strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mitigation {
    pub risk: String,
    pub strategy: String,
    pub owner: Option<String>,
}

/// Research and appendix section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeateResearch {
    pub id: String,
    pub session_id: String,
    pub competitors: Option<Vec<Competitor>>,
    pub similar_projects: Option<Vec<SimilarProject>>,
    pub research_findings: Option<String>,
    pub technical_specs: Option<String>,
    pub reference_links: Option<Vec<Reference>>,
    pub ai_generated: bool,
    pub created_at: DateTime<Utc>,
}

/// Competitor information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competitor {
    pub name: String,
    pub url: Option<String>,
    pub strengths: Vec<String>,
    pub gaps: Vec<String>,
    pub features: Vec<String>,
}

/// Similar project reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarProject {
    pub name: String,
    pub url: Option<String>,
    pub positive_aspects: Vec<String>,
    pub negative_aspects: Vec<String>,
    pub patterns_to_adopt: Vec<String>,
}

/// Reference or resource
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    pub title: String,
    pub url: Option<String>,
    pub notes: Option<String>,
}

/// Expert roundtable session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtableSession {
    pub id: String,
    pub session_id: String,
    pub experts: Option<Vec<Expert>>,
    pub discussion_log: Option<String>,
    pub key_insights: Option<Vec<String>>,
    pub recommendations: Option<Vec<String>>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Expert persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Expert {
    pub name: String,
    pub role: String,
    pub expertise: String,
    pub personality: Option<String>,
}

/// Request to skip a section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipSectionRequest {
    pub section: String,
    pub ai_fill: bool,
}

/// Session completion status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionCompletionStatus {
    pub session_id: String,
    pub total_sections: u8,
    pub completed_sections: u8,
    pub skipped_sections: Vec<String>,
    pub is_ready_for_prd: bool,
    pub missing_required_sections: Vec<String>,
}

/// PRD Quickstart Template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDTemplate {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub one_liner_prompts: Option<Vec<String>>,
    pub default_features: Option<Vec<String>>,
    pub default_dependencies: Option<serde_json::Value>,
    // Overview section defaults
    pub default_problem_statement: Option<String>,
    pub default_target_audience: Option<String>,
    pub default_value_proposition: Option<String>,
    // UX section defaults
    pub default_ui_considerations: Option<String>,
    pub default_ux_principles: Option<String>,
    // Technical section defaults
    pub default_tech_stack_quick: Option<String>,
    // Roadmap section defaults
    pub default_mvp_scope: Option<Vec<String>>,
    // Research section defaults
    pub default_research_findings: Option<String>,
    pub default_technical_specs: Option<String>,
    pub default_competitors: Option<Vec<String>>,
    pub default_similar_projects: Option<Vec<String>>,
    pub is_system: bool,
    pub created_at: DateTime<Utc>,
}

/// Input for creating a new template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTemplateInput {
    pub name: String,
    pub description: Option<String>,
    pub project_type: Option<String>,
    pub one_liner_prompts: Option<Vec<String>>,
    pub default_features: Option<Vec<String>>,
    pub default_dependencies: Option<serde_json::Value>,
    // Overview section defaults
    pub default_problem_statement: Option<String>,
    pub default_target_audience: Option<String>,
    pub default_value_proposition: Option<String>,
    // UX section defaults
    pub default_ui_considerations: Option<String>,
    pub default_ux_principles: Option<String>,
    // Technical section defaults
    pub default_tech_stack_quick: Option<String>,
    // Roadmap section defaults
    pub default_mvp_scope: Option<Vec<String>>,
    // Research section defaults
    pub default_research_findings: Option<String>,
    pub default_technical_specs: Option<String>,
    pub default_competitors: Option<Vec<String>>,
    pub default_similar_projects: Option<Vec<String>>,
}
