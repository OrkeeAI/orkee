// ABOUTME: PRD generator utilities for formatting and merging PRD content
// ABOUTME: Pure CRUD helpers - AI generation moved to frontend (AI SDK pattern)

use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tracing::warn;

use crate::prd_aggregator::AggregatedPRDData;

/// PRD generator utilities for formatting and merging PRD content
pub struct PRDGenerator {
    #[allow(dead_code)]
    pool: Pool<Sqlite>,
}

impl PRDGenerator {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Format a generated PRD into markdown (pure formatting helper)
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
                    markdown.push('\n');
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
                markdown.push('\n');
            }

            if let Some(infra) = &tech.infrastructure {
                markdown.push_str("### Infrastructure\n\n");
                if let Some(hosting) = &infra.hosting {
                    markdown.push_str(&format!("- **Hosting:** {}\n", hosting));
                }
                if let Some(db) = &infra.database {
                    markdown.push_str(&format!("- **Database:** {}\n", db));
                }
                markdown.push('\n');
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
                markdown.push('\n');
            }
        }

        // 6. Logical Dependency Chain
        markdown.push_str("## 6. Logical Dependency Chain\n\n");
        if let Some(deps) = &prd.dependencies {
            if let Some(foundation) = &deps.foundation_features {
                markdown.push_str("### Foundation Features (Build First)\n\n");
                for item in foundation {
                    markdown.push_str(&format!(
                        "- {} ({}): {}\n",
                        item.id, item.name, item.rationale
                    ));
                }
                markdown.push('\n');
            }

            if let Some(visible) = &deps.visible_features {
                markdown.push_str("### Visible Features (Quick Wins)\n\n");
                for item in visible {
                    markdown.push_str(&format!(
                        "- {} ({}): {}\n",
                        item.id, item.name, item.rationale
                    ));
                }
                markdown.push('\n');
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
                markdown.push('\n');
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
                markdown.push('\n');
            }
        }

        markdown
    }

    /// Build context string from aggregated data (helper for frontend AI calls)
    pub fn build_context_from_aggregated(&self, data: &AggregatedPRDData) -> String {
        let mut context = String::new();

        context.push_str(&format!(
            "Project: {}\n\n",
            data.session.initial_description
        ));

        if let Some(overview) = &data.overview {
            context.push_str("## Overview\n");
            if let Some(problem) = &overview.problem_statement {
                context.push_str(&format!("Problem: {}\n", problem));
            }
            if let Some(audience) = &overview.target_audience {
                context.push_str(&format!("Target Audience: {}\n", audience));
            }
            if let Some(value) = &overview.value_proposition {
                context.push_str(&format!("Value Proposition: {}\n", value));
            }
            context.push('\n');
        }

        if let Some(technical) = &data.technical {
            context.push_str("## Technical Context\n");
            if let Some(stack) = &technical.tech_stack_quick {
                context.push_str(&format!("Tech Stack: {}\n", stack));
            }
            context.push_str(&format!("Components: {}\n", technical.components.len()));
            context.push('\n');
        }

        if let Some(roadmap) = &data.roadmap {
            context.push_str("## Roadmap Context\n");
            context.push_str(&format!("MVP Features: {}\n", roadmap.mvp_scope.len()));
            context.push('\n');
        }

        if let Some(deps) = &data.dependencies {
            context.push_str("## Dependency Context\n");
            context.push_str(&format!(
                "Foundation Features: {}\n",
                deps.foundation_features.len()
            ));
            context.push_str(&format!(
                "Visible Features: {}\n",
                deps.visible_features.len()
            ));
            context.push('\n');
        }

        // Add research insights if available
        if let Some(research) = &data.research {
            context.push_str("## Research Context\n");
            context.push_str(&format!(
                "Competitors Analyzed: {}\n",
                research.competitors.len()
            ));
            context.push_str(&format!(
                "Similar Projects: {}\n",
                research.similar_projects.len()
            ));
            if let Some(findings) = &research.research_findings {
                context.push_str(&format!("Findings: {}\n", findings));
            }
            context.push('\n');
        }

        // Add expert roundtable insights
        if let Some(insights) = &data.roundtable_insights {
            if !insights.is_empty() {
                context.push_str("## Expert Insights\n");
                for insight in insights.iter().take(5) {
                    // Limit to top 5
                    context.push_str(&format!(
                        "- [{}] {}: {}\n",
                        insight.priority, insight.category, insight.content
                    ));
                }
                context.push('\n');
            }
        }

        context
    }

    /// Merge AI-generated content with existing manual content (helper for frontend)
    pub fn merge_content(
        &self,
        existing: &GeneratedPRD,
        ai_generated: &GeneratedPRD,
        sections_to_replace: Vec<String>,
    ) -> GeneratedPRD {
        let mut merged = existing.clone();

        for section in sections_to_replace {
            match section.as_str() {
                "overview" => {
                    if ai_generated.overview.is_some() {
                        merged.overview = ai_generated.overview.clone();
                    }
                }
                "features" => {
                    if ai_generated.features.is_some() {
                        merged.features = ai_generated.features.clone();
                    }
                }
                "ux" => {
                    if ai_generated.ux.is_some() {
                        merged.ux = ai_generated.ux.clone();
                    }
                }
                "technical" => {
                    if ai_generated.technical.is_some() {
                        merged.technical = ai_generated.technical.clone();
                    }
                }
                "roadmap" => {
                    if ai_generated.roadmap.is_some() {
                        merged.roadmap = ai_generated.roadmap.clone();
                    }
                }
                "dependencies" => {
                    if ai_generated.dependencies.is_some() {
                        merged.dependencies = ai_generated.dependencies.clone();
                    }
                }
                "risks" => {
                    if ai_generated.risks.is_some() {
                        merged.risks = ai_generated.risks.clone();
                    }
                }
                "research" => {
                    if ai_generated.research.is_some() {
                        merged.research = ai_generated.research.clone();
                    }
                }
                _ => {
                    warn!("Unknown section to merge: {}", section);
                }
            }
        }

        merged
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
    pub foundation_features: Option<Vec<DependencyFeature>>,
    #[serde(rename = "visibleFeatures")]
    pub visible_features: Option<Vec<DependencyFeature>>,
    #[serde(rename = "enhancementFeatures")]
    pub enhancement_features: Option<Vec<DependencyFeature>>,
    #[serde(rename = "dependencyGraph")]
    pub dependency_graph: Option<DependencyGraph>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyFeature {
    pub id: String,
    pub name: String,
    pub rationale: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocks: Vec<String>,
    #[serde(rename = "dependsOn", default, skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
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
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub node_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub edge_type: Option<String>,
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
