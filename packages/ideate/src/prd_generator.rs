// ABOUTME: PRD generator service for Quick Mode AI-powered generation
// ABOUTME: Fetches settings from DB, uses Claude API to generate comprehensive PRDs

use ai::{AIResponse, AIService};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tracing::{error, info, warn};

use crate::error::{IdeateError, Result};
use crate::prompts;
use security::users::storage::UserStorage;
use settings::storage::SettingsStorage;

/// AI settings for PRD generation
#[derive(Debug, Clone)]
pub struct AISettings {
    pub max_tokens: u32,
    pub temperature: f32,
    pub model: String,
    pub timeout_seconds: u64,
    pub retry_attempts: u32,
}

impl Default for AISettings {
    fn default() -> Self {
        Self {
            max_tokens: 8000,
            temperature: 0.7,
            model: "claude-3-opus-20240229".to_string(),
            timeout_seconds: 120,
            retry_attempts: 3,
        }
    }
}

/// PRD generator responsible for AI-powered PRD creation
pub struct PRDGenerator {
    pool: Pool<Sqlite>,
}

impl PRDGenerator {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Fetch AI settings from database
    async fn get_ai_settings(&self) -> Result<AISettings> {
        let settings_storage = SettingsStorage::new(self.pool.clone());
        let ai_settings = settings_storage
            .get_by_category("ai")
            .await
            .map_err(|e| {
                error!("Failed to fetch AI settings: {}", e);
                IdeateError::InvalidInput(format!("Failed to fetch AI settings: {}", e))
            })?;

        // Convert settings to AISettings struct
        let mut settings = AISettings::default();

        for setting in ai_settings {
            match setting.key.as_str() {
                "ideate.max_tokens" => {
                    settings.max_tokens = setting.value.parse().unwrap_or(8000);
                }
                "ideate.temperature" => {
                    settings.temperature = setting.value.parse().unwrap_or(0.7);
                }
                "ideate.model" => {
                    settings.model = setting.value;
                }
                "ideate.timeout_seconds" => {
                    settings.timeout_seconds = setting.value.parse().unwrap_or(120);
                }
                "ideate.retry_attempts" => {
                    settings.retry_attempts = setting.value.parse().unwrap_or(3);
                }
                _ => {}
            }
        }

        Ok(settings)
    }

    /// Get user's API key from database
    async fn get_user_api_key(&self, user_id: &str) -> Result<String> {
        let user_storage = UserStorage::new(self.pool.clone())
            .map_err(|e| {
                error!("Failed to create user storage: {}", e);
                IdeateError::AIService(format!("Failed to access user storage: {}", e))
            })?;

        let user = user_storage.get_user(user_id).await.map_err(|e| {
            error!("Failed to fetch user: {}", e);
            IdeateError::AIService(format!("Failed to fetch user: {}", e))
        })?;

        user.anthropic_api_key.ok_or_else(|| {
            warn!("User {} has no Anthropic API key configured", user_id);
            IdeateError::InvalidInput("No Anthropic API key configured. Please add one in Settings -> API Keys".to_string())
        })
    }

    /// Generate a complete PRD from a description
    pub async fn generate_complete_prd(
        &self,
        user_id: &str,
        description: &str,
    ) -> Result<GeneratedPRD> {
        info!("Generating complete PRD for user {}", user_id);

        // Fetch configuration
        let settings = self.get_ai_settings().await?;
        let api_key = self.get_user_api_key(user_id).await?;

        // Create AI service
        let ai_service = AIService::with_api_key_and_model(api_key, settings.model.clone());

        // Generate complete PRD using the AI
        let prompt = prompts::complete_prd_prompt(description);
        let system_prompt = Some(prompts::SYSTEM_PROMPT.to_string());

        let response: AIResponse<serde_json::Value> = ai_service
            .generate_structured(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("AI generation failed: {}", e);
                IdeateError::AIService(format!("Failed to generate PRD: {}", e))
            })?;

        // Parse the response into GeneratedPRD
        let generated_prd: GeneratedPRD = serde_json::from_value(response.data).map_err(|e| {
            error!("Failed to parse AI response: {}", e);
            IdeateError::AIService(format!("Failed to parse AI response: {}", e))
        })?;

        info!(
            "Successfully generated complete PRD (tokens: {})",
            response.usage.total_tokens()
        );

        Ok(generated_prd)
    }

    /// Generate a specific section of the PRD
    pub async fn generate_section(
        &self,
        user_id: &str,
        section: &str,
        description: &str,
        context: Option<&str>, // Optional context from other sections
    ) -> Result<serde_json::Value> {
        info!("Generating section '{}' for user {}", section, user_id);

        // Fetch configuration
        let settings = self.get_ai_settings().await?;
        let api_key = self.get_user_api_key(user_id).await?;

        // Create AI service
        let ai_service = AIService::with_api_key_and_model(api_key, settings.model.clone());

        // Generate prompt based on section
        let prompt = match section {
            "overview" => prompts::overview_prompt(description),
            "features" => prompts::features_prompt(description),
            "ux" => prompts::ux_prompt(description),
            "technical" => prompts::technical_prompt(description),
            "roadmap" => prompts::roadmap_prompt(description, context.unwrap_or("")),
            "dependencies" => prompts::dependencies_prompt(description, context.unwrap_or("")),
            "risks" => prompts::risks_prompt(description),
            "research" => prompts::research_prompt(description),
            _ => {
                return Err(IdeateError::InvalidSection(format!(
                    "Unknown section: {}",
                    section
                )))
            }
        };

        let system_prompt = Some(prompts::SYSTEM_PROMPT.to_string());

        let response: AIResponse<serde_json::Value> = ai_service
            .generate_structured(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("AI generation failed for section '{}': {}", section, e);
                IdeateError::AIService(format!("Failed to generate section: {}", e))
            })?;

        info!(
            "Successfully generated section '{}' (tokens: {})",
            section,
            response.usage.total_tokens()
        );

        Ok(response.data)
    }

    /// Format a generated PRD into markdown
    pub fn format_prd_markdown(&self, prd: &GeneratedPRD) -> String {
        let mut markdown = String::new();

        // Title and overview
        markdown.push_str("# Product Requirements Document\n\n");

        // 1. Overview
        markdown.push_str("## 1. Overview\n\n");
        if let Some(overview) = &prd.overview {
            markdown.push_str("### Problem Statement\n");
            markdown.push_str(&overview.problem_statement);
            markdown.push_str("\n\n");

            markdown.push_str("### Target Audience\n");
            markdown.push_str(&overview.target_audience);
            markdown.push_str("\n\n");

            markdown.push_str("### Value Proposition\n");
            markdown.push_str(&overview.value_proposition);
            markdown.push_str("\n\n");

            if let Some(pitch) = &overview.one_line_pitch {
                markdown.push_str("### One-Line Pitch\n");
                markdown.push_str(pitch);
                markdown.push_str("\n\n");
            }
        }

        // 2. Core Features
        markdown.push_str("## 2. Core Features\n\n");
        if let Some(features) = &prd.features {
            for (idx, feature) in features.iter().enumerate() {
                markdown.push_str(&format!("### {}.{} {}\n\n", 2, idx + 1, feature.name));

                if let Some(what) = &feature.what {
                    markdown.push_str("**What:** ");
                    markdown.push_str(what);
                    markdown.push_str("\n\n");
                }

                if let Some(why) = &feature.why {
                    markdown.push_str("**Why:** ");
                    markdown.push_str(why);
                    markdown.push_str("\n\n");
                }

                if let Some(how) = &feature.how {
                    markdown.push_str("**How:** ");
                    markdown.push_str(how);
                    markdown.push_str("\n\n");
                }
            }
        }

        // 3. User Experience
        markdown.push_str("## 3. User Experience\n\n");
        if let Some(ux) = &prd.ux {
            if let Some(personas) = &ux.personas {
                markdown.push_str("### Personas\n\n");
                for persona in personas {
                    markdown.push_str(&format!("#### {}\n", persona.name));
                    markdown.push_str(&format!("**Role:** {}\n\n", persona.role));
                    markdown.push_str("**Goals:**\n");
                    for goal in &persona.goals {
                        markdown.push_str(&format!("- {}\n", goal));
                    }
                    markdown.push_str("\n**Pain Points:**\n");
                    for pain in &persona.pain_points {
                        markdown.push_str(&format!("- {}\n", pain));
                    }
                    markdown.push_str("\n");
                }
            }

            if let Some(ui_considerations) = &ux.ui_considerations {
                markdown.push_str("### UI Considerations\n");
                markdown.push_str(ui_considerations);
                markdown.push_str("\n\n");
            }

            if let Some(ux_principles) = &ux.ux_principles {
                markdown.push_str("### UX Principles\n");
                markdown.push_str(ux_principles);
                markdown.push_str("\n\n");
            }
        }

        // 4. Technical Architecture
        markdown.push_str("## 4. Technical Architecture\n\n");
        if let Some(tech) = &prd.technical {
            if let Some(stack) = &tech.tech_stack_quick {
                markdown.push_str(&format!("**Tech Stack:** {}\n\n", stack));
            }

            if let Some(components) = &tech.components {
                markdown.push_str("### Components\n\n");
                for comp in components {
                    markdown.push_str(&format!("- **{}**: {}\n", comp.name, comp.purpose));
                }
                markdown.push_str("\n");
            }

            if let Some(infra) = &tech.infrastructure {
                markdown.push_str("### Infrastructure\n\n");
                if let Some(hosting) = &infra.hosting {
                    markdown.push_str(&format!("- **Hosting:** {}\n", hosting));
                }
                if let Some(db) = &infra.database {
                    markdown.push_str(&format!("- **Database:** {}\n", db));
                }
                markdown.push_str("\n");
            }
        }

        // 5. Development Roadmap
        markdown.push_str("## 5. Development Roadmap\n\n");
        if let Some(roadmap) = &prd.roadmap {
            if let Some(mvp) = &roadmap.mvp_scope {
                markdown.push_str("### MVP Scope\n\n");
                for item in mvp {
                    markdown.push_str(&format!("- {}\n", item));
                }
                markdown.push_str("\n");
            }
        }

        // 6. Logical Dependency Chain
        markdown.push_str("## 6. Logical Dependency Chain\n\n");
        if let Some(deps) = &prd.dependencies {
            if let Some(foundation) = &deps.foundation_features {
                markdown.push_str("### Foundation Features (Build First)\n\n");
                for item in foundation {
                    markdown.push_str(&format!("- {}\n", item));
                }
                markdown.push_str("\n");
            }

            if let Some(visible) = &deps.visible_features {
                markdown.push_str("### Visible Features (Quick Wins)\n\n");
                for item in visible {
                    markdown.push_str(&format!("- {}\n", item));
                }
                markdown.push_str("\n");
            }
        }

        // 7. Risks and Mitigations
        markdown.push_str("## 7. Risks and Mitigations\n\n");
        if let Some(risks) = &prd.risks {
            if let Some(technical) = &risks.technical_risks {
                markdown.push_str("### Technical Risks\n\n");
                for risk in technical {
                    markdown.push_str(&format!(
                        "- **{}** (Severity: {}, Probability: {}): {}\n",
                        risk.description, risk.severity, risk.probability, risk.description
                    ));
                }
                markdown.push_str("\n");
            }
        }

        // 8. Research & References
        markdown.push_str("## 8. Research & References\n\n");
        if let Some(research) = &prd.research {
            if let Some(competitors) = &research.competitors {
                markdown.push_str("### Competitors\n\n");
                for comp in competitors {
                    markdown.push_str(&format!("#### {}\n", comp.name));
                    markdown.push_str(&format!("**URL:** {}\n\n", comp.url));
                }
                markdown.push_str("\n");
            }
        }

        markdown
    }
}

/// Generated PRD structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedPRD {
    pub overview: Option<Overview>,
    pub features: Option<Vec<Feature>>,
    pub ux: Option<UX>,
    pub technical: Option<Technical>,
    pub roadmap: Option<Roadmap>,
    pub dependencies: Option<Dependencies>,
    pub risks: Option<Risks>,
    pub research: Option<Research>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Overview {
    #[serde(rename = "problemStatement")]
    pub problem_statement: String,
    #[serde(rename = "targetAudience")]
    pub target_audience: String,
    #[serde(rename = "valueProposition")]
    pub value_proposition: String,
    #[serde(rename = "oneLinePitch")]
    pub one_line_pitch: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub name: String,
    pub what: Option<String>,
    pub why: Option<String>,
    pub how: Option<String>,
    #[serde(rename = "dependsOn")]
    pub depends_on: Option<Vec<String>>,
    pub enables: Option<Vec<String>>,
    #[serde(rename = "buildPhase")]
    pub build_phase: Option<u8>,
    #[serde(rename = "isVisible")]
    pub is_visible: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UX {
    pub personas: Option<Vec<Persona>>,
    #[serde(rename = "userFlows")]
    pub user_flows: Option<Vec<UserFlow>>,
    #[serde(rename = "uiConsiderations")]
    pub ui_considerations: Option<String>,
    #[serde(rename = "uxPrinciples")]
    pub ux_principles: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Persona {
    pub name: String,
    pub role: String,
    pub goals: Vec<String>,
    #[serde(rename = "painPoints")]
    pub pain_points: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFlow {
    pub name: String,
    pub steps: Vec<FlowStep>,
    pub touchpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowStep {
    pub action: String,
    pub screen: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Technical {
    pub components: Option<Vec<Component>>,
    #[serde(rename = "dataModels")]
    pub data_models: Option<Vec<DataModel>>,
    pub apis: Option<Vec<API>>,
    pub infrastructure: Option<Infrastructure>,
    #[serde(rename = "techStackQuick")]
    pub tech_stack_quick: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub name: String,
    pub purpose: String,
    pub technology: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataModel {
    pub name: String,
    pub fields: Vec<Field>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct API {
    pub name: String,
    pub purpose: String,
    pub endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Infrastructure {
    pub hosting: Option<String>,
    pub database: Option<String>,
    pub caching: Option<String>,
    #[serde(rename = "fileStorage")]
    pub file_storage: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Roadmap {
    #[serde(rename = "mvpScope")]
    pub mvp_scope: Option<Vec<String>>,
    #[serde(rename = "futurePhases")]
    pub future_phases: Option<Vec<Phase>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Phase {
    pub name: String,
    pub features: Vec<String>,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependencies {
    #[serde(rename = "foundationFeatures")]
    pub foundation_features: Option<Vec<String>>,
    #[serde(rename = "visibleFeatures")]
    pub visible_features: Option<Vec<String>>,
    #[serde(rename = "enhancementFeatures")]
    pub enhancement_features: Option<Vec<String>>,
    #[serde(rename = "dependencyGraph")]
    pub dependency_graph: Option<DependencyGraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub phase: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risks {
    #[serde(rename = "technicalRisks")]
    pub technical_risks: Option<Vec<Risk>>,
    #[serde(rename = "mvpScopingRisks")]
    pub mvp_scoping_risks: Option<Vec<Risk>>,
    #[serde(rename = "resourceRisks")]
    pub resource_risks: Option<Vec<Risk>>,
    pub mitigations: Option<Vec<Mitigation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub description: String,
    pub severity: String,
    pub probability: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mitigation {
    pub risk: String,
    pub strategy: String,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Research {
    pub competitors: Option<Vec<Competitor>>,
    #[serde(rename = "similarProjects")]
    pub similar_projects: Option<Vec<SimilarProject>>,
    #[serde(rename = "researchFindings")]
    pub research_findings: Option<String>,
    #[serde(rename = "technicalSpecs")]
    pub technical_specs: Option<String>,
    #[serde(rename = "referenceLinks")]
    pub reference_links: Option<Vec<ReferenceLink>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Competitor {
    pub name: String,
    pub url: String,
    pub strengths: Vec<String>,
    pub gaps: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarProject {
    pub name: String,
    pub url: String,
    #[serde(rename = "positiveAspects")]
    pub positive_aspects: Vec<String>,
    #[serde(rename = "negativeAspects")]
    pub negative_aspects: Vec<String>,
    #[serde(rename = "patternsToAdopt")]
    pub patterns_to_adopt: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReferenceLink {
    pub title: String,
    pub url: String,
    pub notes: Option<String>,
}
