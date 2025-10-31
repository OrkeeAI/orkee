// ABOUTME: PRD generator service for Quick Mode AI-powered generation
// ABOUTME: Fetches settings from DB, uses Claude API to generate comprehensive PRDs

use ai::{AIResponse, AIService};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};
use tracing::{error, info, warn};

use crate::error::{IdeateError, Result};
use crate::prd_aggregator::{AggregatedPRDData, PRDAggregator};
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
        let ai_settings = settings_storage.get_by_category("ai").await.map_err(|e| {
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
        let user_storage = UserStorage::new(self.pool.clone()).map_err(|e| {
            error!("Failed to create user storage: {}", e);
            IdeateError::AIService(format!("Failed to access user storage: {}", e))
        })?;

        let user = user_storage.get_user(user_id).await.map_err(|e| {
            error!("Failed to fetch user: {}", e);
            IdeateError::AIService(format!("Failed to fetch user: {}", e))
        })?;

        user.anthropic_api_key.ok_or_else(|| {
            warn!("User {} has no Anthropic API key configured", user_id);
            IdeateError::InvalidInput(
                "No Anthropic API key configured. Please add one in Settings -> API Keys"
                    .to_string(),
            )
        })
    }

    /// Generate a complete PRD from a description
    pub async fn generate_complete_prd(
        &self,
        user_id: &str,
        description: &str,
    ) -> Result<GeneratedPRD> {
        self.generate_complete_prd_with_model(user_id, description, None, None)
            .await
    }

    /// Generate a complete PRD with optional provider and model overrides
    pub async fn generate_complete_prd_with_model(
        &self,
        user_id: &str,
        description: &str,
        provider: Option<String>,
        model: Option<String>,
    ) -> Result<GeneratedPRD> {
        info!(
            "Generating complete PRD for user {} with provider: {:?}, model: {:?}",
            user_id, provider, model
        );

        // Fetch configuration (for defaults if not provided)
        let settings = self.get_ai_settings().await?;
        let api_key = self.get_user_api_key(user_id).await?;

        // Use provided model or fall back to settings
        let model_to_use = model.unwrap_or(settings.model.clone());

        info!("Using model: {}", model_to_use);

        // Create AI service with the selected model
        let ai_service = AIService::with_api_key_and_model(api_key, model_to_use);

        // Generate complete PRD using the AI
        let prompt = prompts::complete_prd_prompt(description)
            .map_err(|e| IdeateError::PromptError(e))?;
        let system_prompt = Some(prompts::get_system_prompt()
            .map_err(|e| IdeateError::PromptError(e))?);

        let response: AIResponse<serde_json::Value> = ai_service
            .generate_structured(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("AI generation failed: {}", e);
                IdeateError::AIService(format!("Failed to generate PRD: {}", e))
            })?;

        // Parse the response into GeneratedPRD
        info!(
            "AI response JSON: {}",
            serde_json::to_string_pretty(&response.data)
                .unwrap_or_else(|_| "Failed to serialize".to_string())
        );
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

        let system_prompt = Some(prompts::get_system_prompt());

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

    /// Generate PRD from aggregated session data (Guided/Comprehensive modes)
    pub async fn generate_from_session(
        &self,
        user_id: &str,
        session_id: &str,
    ) -> Result<GeneratedPRD> {
        info!("Generating PRD from session data: {}", session_id);

        // Aggregate all session data
        let aggregator = PRDAggregator::new(self.pool.clone());
        let aggregated = aggregator.aggregate_session_data(session_id).await?;

        // Fetch AI configuration
        let settings = self.get_ai_settings().await?;
        let api_key = self.get_user_api_key(user_id).await?;
        let ai_service = AIService::with_api_key_and_model(api_key, settings.model.clone());

        // Build context from all available sections
        let context = self.build_context_from_aggregated(&aggregated);

        // Generate PRD using AI with full context
        let prompt = self.build_session_prd_prompt(&aggregated, &context);
        let system_prompt = Some(prompts::get_system_prompt());

        let response: AIResponse<serde_json::Value> = ai_service
            .generate_structured(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("Failed to generate PRD from session: {}", e);
                IdeateError::AIService(format!("Failed to generate PRD: {}", e))
            })?;

        let generated_prd: GeneratedPRD = serde_json::from_value(response.data).map_err(|e| {
            error!("Failed to parse AI response: {}", e);
            IdeateError::AIService(format!("Failed to parse AI response: {}", e))
        })?;

        info!(
            "Successfully generated PRD from session (tokens: {})",
            response.usage.total_tokens()
        );

        Ok(generated_prd)
    }

    /// Fill skipped sections with AI-generated content
    pub async fn fill_skipped_sections(
        &self,
        user_id: &str,
        session_id: &str,
        sections_to_fill: Vec<String>,
    ) -> Result<Vec<(String, serde_json::Value)>> {
        info!(
            "Filling {} skipped sections for session: {}",
            sections_to_fill.len(),
            session_id
        );

        // Get aggregated data for context
        let aggregator = PRDAggregator::new(self.pool.clone());
        let aggregated = aggregator.aggregate_session_data(session_id).await?;
        let context = self.build_context_from_aggregated(&aggregated);

        let mut filled_sections = Vec::new();

        for section in sections_to_fill {
            info!("Filling section: {}", section);
            let filled_content = self
                .generate_section_with_context(
                    user_id,
                    &section,
                    &aggregated.session.initial_description,
                    &context,
                )
                .await?;

            filled_sections.push((section, filled_content));
        }

        info!("Successfully filled {} sections", filled_sections.len());

        Ok(filled_sections)
    }

    /// Generate a section with full context from other sections
    pub async fn generate_section_with_context(
        &self,
        user_id: &str,
        section: &str,
        description: &str,
        context: &str,
    ) -> Result<serde_json::Value> {
        info!("Generating section '{}' with context", section);

        let settings = self.get_ai_settings().await?;
        let api_key = self.get_user_api_key(user_id).await?;
        let ai_service = AIService::with_api_key_and_model(api_key, settings.model.clone());

        // Build enhanced prompt with context
        let prompt = self.build_section_prompt_with_context(section, description, context)?;
        let system_prompt = Some(prompts::get_system_prompt());

        let response: AIResponse<serde_json::Value> = ai_service
            .generate_structured(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("Failed to generate section '{}': {}", section, e);
                IdeateError::AIService(format!("Failed to generate section: {}", e))
            })?;

        info!(
            "Successfully generated section '{}' with context (tokens: {})",
            section,
            response.usage.total_tokens()
        );

        Ok(response.data)
    }

    /// Regenerate a specific section with updated context
    pub async fn regenerate_section_with_full_context(
        &self,
        user_id: &str,
        session_id: &str,
        section: &str,
    ) -> Result<serde_json::Value> {
        info!(
            "Regenerating section '{}' for session {}",
            section, session_id
        );

        // Get latest aggregated data
        let aggregator = PRDAggregator::new(self.pool.clone());
        let aggregated = aggregator.aggregate_session_data(session_id).await?;
        let context = self.build_context_from_aggregated(&aggregated);

        self.generate_section_with_context(
            user_id,
            section,
            &aggregated.session.initial_description,
            &context,
        )
        .await
    }

    /// Build context string from aggregated data
    fn build_context_from_aggregated(&self, data: &AggregatedPRDData) -> String {
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

    /// Build prompt for generating PRD from session data
    fn build_session_prd_prompt(&self, data: &AggregatedPRDData, context: &str) -> String {
        let mut prompt = String::new();

        prompt.push_str("# Generate Complete PRD from Session Data\n\n");
        prompt.push_str("You are generating a Product Requirements Document (PRD) based on collected session data.\n\n");
        prompt.push_str("## Context\n");
        prompt.push_str(context);
        prompt.push_str("\n## Your Task\n");
        prompt.push_str("Generate a complete, cohesive PRD that:\n");
        prompt.push_str("1. Synthesizes all available information\n");
        prompt.push_str("2. Fills in any gaps with reasonable assumptions\n");
        prompt.push_str("3. Maintains consistency across all sections\n");
        prompt.push_str("4. Follows the standard 8-section PRD structure\n\n");

        // Add notes about skipped sections
        if !data.skipped_sections.is_empty() {
            prompt.push_str("## Skipped Sections (Generate These)\n");
            for section in &data.skipped_sections {
                prompt.push_str(&format!("- {}\n", section));
            }
            prompt.push('\n');
        }

        prompt.push_str("Generate the PRD in the standard GeneratedPRD JSON format.\n");

        prompt
    }

    /// Build enhanced prompt for section generation with context
    fn build_section_prompt_with_context(
        &self,
        section: &str,
        description: &str,
        context: &str,
    ) -> Result<String> {
        let mut prompt = String::new();

        prompt.push_str(&format!("# Generate {} Section\n\n", section));
        prompt.push_str("## Project Description\n");
        prompt.push_str(description);
        prompt.push_str("\n\n## Context from Other Sections\n");
        prompt.push_str(context);
        prompt.push_str("\n\n## Your Task\n");
        prompt.push_str(&format!(
            "Generate the {} section of the PRD that is consistent with the context above.\n",
            section
        ));
        prompt.push_str("Ensure your output:\n");
        prompt.push_str("1. Aligns with information from other sections\n");
        prompt.push_str("2. Fills any gaps identified in the context\n");
        prompt.push_str("3. Maintains consistency in terminology and approach\n");
        prompt.push_str("4. Provides specific, actionable information\n\n");

        // Add section-specific guidance
        match section {
            "overview" => {
                prompt.push_str("Focus on: problem statement, target audience, value proposition, one-line pitch.\n");
            }
            "ux" => {
                prompt.push_str(
                    "Focus on: user personas, user flows, UI considerations, UX principles.\n",
                );
            }
            "technical" => {
                prompt.push_str(
                    "Focus on: system components, data models, APIs, infrastructure, tech stack.\n",
                );
            }
            "roadmap" => {
                prompt.push_str(
                    "Focus on: MVP scope (no timelines, just features), future phases.\n",
                );
            }
            "dependencies" => {
                prompt.push_str("Focus on: foundation features, visible features, enhancement features, build order.\n");
            }
            "risks" => {
                prompt.push_str("Focus on: technical risks, MVP scoping risks, resource risks, mitigation strategies.\n");
            }
            "research" => {
                prompt.push_str("Focus on: competitors, similar projects, research findings, technical specs.\n");
            }
            _ => {
                return Err(IdeateError::InvalidSection(format!(
                    "Unknown section: {}",
                    section
                )))
            }
        }

        Ok(prompt)
    }

    /// Merge AI-generated content with existing manual content
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

    /// Regenerate PRD sections to match a different template's style/format (streaming version)
    pub async fn regenerate_with_template_stream(
        &self,
        user_id: &str,
        session_id: &str,
        template_id: &str,
        provider: Option<&str>,
        model: Option<&str>,
    ) -> Result<impl futures::stream::Stream<Item = Result<String>>> {
        info!(
            "Streaming PRD regeneration for session {} with template {} (provider: {:?}, model: {:?})",
            session_id, template_id, provider, model
        );

        // Fetch existing session data
        let aggregator = PRDAggregator::new(self.pool.clone());
        let aggregated = aggregator.aggregate_session_data(session_id).await?;

        // Fetch template details
        let template = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM prd_output_templates WHERE id = ?",
        )
        .bind(template_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch template: {}", e);
            IdeateError::Database(e)
        })?
        .ok_or_else(|| {
            warn!("Template {} not found", template_id);
            IdeateError::InvalidInput(format!("Template not found: {}", template_id))
        })?;

        let template_name = template.1;

        // Get API key and determine model to use
        let api_key = self.get_user_api_key(user_id).await?;

        // Use provided model or fall back to settings
        let model_to_use = if let Some(m) = model {
            m.to_string()
        } else {
            let settings = self.get_ai_settings().await?;
            settings.model
        };

        let ai_service = AIService::with_api_key_and_model(api_key, model_to_use);

        // Build context from current sections
        let context = self.build_context_from_aggregated(&aggregated);

        // Build prompt for intelligent template reformatting
        let prompt = self.build_template_regeneration_prompt(&aggregated, &context, &template_name);
        let system_prompt = Some(prompts::get_system_prompt());

        // Call Claude with streaming
        let text_stream = ai_service
            .generate_text_stream(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("AI streaming failed for template regeneration: {}", e);
                IdeateError::AIService(format!("Failed to stream regeneration: {}", e))
            })?;

        // Map AIServiceError to IdeateError in the stream
        use futures::stream::StreamExt;
        let mapped_stream = text_stream.map(|result| {
            result.map_err(|e| IdeateError::AIService(format!("Stream error: {}", e)))
        });

        Ok(mapped_stream)
    }

    /// Regenerate PRD sections to match a different template's style/format
    pub async fn regenerate_with_template(
        &self,
        user_id: &str,
        session_id: &str,
        template_id: &str,
        provider: Option<&str>,
        model: Option<&str>,
    ) -> Result<String> {
        info!(
            "Regenerating PRD for session {} with template {} (provider: {:?}, model: {:?})",
            session_id, template_id, provider, model
        );

        // Fetch existing session data
        let aggregator = PRDAggregator::new(self.pool.clone());
        let aggregated = aggregator.aggregate_session_data(session_id).await?;

        // Fetch template details from output templates (formatting templates)
        let template = sqlx::query_as::<_, (String, String)>(
            "SELECT id, name FROM prd_output_templates WHERE id = ?",
        )
        .bind(template_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            error!("Failed to fetch template: {}", e);
            IdeateError::Database(e)
        })?
        .ok_or_else(|| {
            warn!("Template {} not found", template_id);
            IdeateError::InvalidInput(format!("Template not found: {}", template_id))
        })?;

        let template_name = template.1;

        // Get API key and determine model to use
        let api_key = self.get_user_api_key(user_id).await?;

        // Use provided model or fall back to settings
        let model_to_use = if let Some(m) = model {
            m.to_string()
        } else {
            let settings = self.get_ai_settings().await?;
            settings.model
        };

        let ai_service = AIService::with_api_key_and_model(api_key, model_to_use);

        // Build context from current sections
        let context = self.build_context_from_aggregated(&aggregated);

        // Build prompt for intelligent template reformatting
        let prompt = self.build_template_regeneration_prompt(&aggregated, &context, &template_name);
        let system_prompt = Some(prompts::get_system_prompt());

        // Call Claude to intelligently reformat the data as markdown
        let response = ai_service
            .generate_text(prompt, system_prompt)
            .await
            .map_err(|e| {
                error!("AI generation failed for template regeneration: {}", e);
                IdeateError::AIService(format!("Failed to regenerate for template: {}", e))
            })?;

        info!(
            "Successfully regenerated PRD for template {} (tokens: {})",
            template_name,
            response.usage.total_tokens()
        );

        Ok(response.data)
    }

    /// Build prompt for intelligent template regeneration
    fn build_template_regeneration_prompt(
        &self,
        _data: &AggregatedPRDData,
        context: &str,
        template_name: &str,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("# Intelligently Reformat PRD for Different Template\n\n");
        prompt.push_str(&format!(
            "You are reformatting an existing Product Requirements Document to match the style and structure of the \"{}\" template.\n\n",
            template_name
        ));

        prompt.push_str("## Current PRD Data\n");
        prompt.push_str(context);
        prompt.push_str("\n## Task\n");
        prompt.push_str(
            "Reformat and restructure the PRD data to match the target template's style:\n",
        );
        prompt.push_str("1. Maintain all essential information from the original PRD\n");
        prompt.push_str("2. Reorganize content to fit the template's structure\n");
        prompt.push_str("3. Adjust tone and emphasis to match the template's approach\n");
        prompt.push_str("4. Fill any template-specific requirements with intelligent extrapolation from existing data\n");
        prompt.push_str("5. Ensure consistency across all sections\n");
        prompt.push_str("6. Preserve all technical accuracy and feature details\n\n");

        prompt.push_str("Output the reformatted PRD as markdown with clear section headers and formatting. Return only the markdown content without any code blocks or JSON wrapper.\n");

        prompt
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
