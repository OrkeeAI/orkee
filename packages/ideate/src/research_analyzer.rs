// ABOUTME: Research analyzer for competitor analysis and similar project research
// ABOUTME: Scrapes URLs, extracts features, and uses AI for intelligent analysis

use crate::error::{IdeateError, Result};
use crate::research_prompts;
use crate::types::{Competitor, SimilarProject};
use reqwest::Client;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::env;
use std::time::Duration;
use tracing::{debug, error, info};

/// Maximum number of concurrent URL analyses
const _MAX_CONCURRENT_ANALYSES: usize = 3;

/// Rate limiting delay between requests (milliseconds)
const _RATE_LIMIT_DELAY_MS: u64 = 2000;

/// Cache expiration for competitor analysis (24 hours)
const _COMPETITOR_CACHE_HOURS: i64 = 24;

/// Cache expiration for pattern extraction (1 hour)
const _PATTERN_CACHE_HOURS: i64 = 1;

#[derive(Debug, serde::Deserialize)]
struct AnthropicResponse {
    #[allow(dead_code)]
    id: String,
    content: Vec<ContentBlock>,
    usage: Usage,
    #[allow(dead_code)]
    stop_reason: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    content_type: String,
    text: String,
}

#[derive(Debug, serde::Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl Usage {
    fn total_tokens(&self) -> u32 {
        self.input_tokens + self.output_tokens
    }
}

/// UI/UX pattern extracted from analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIPattern {
    pub pattern_type: String, // layout, navigation, interaction, visual, content
    pub name: String,
    pub description: String,
    pub benefits: String,
    pub adoption_notes: String,
}

/// Gap analysis opportunity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Opportunity {
    pub opportunity_type: String, // differentiation, improvement, gap
    pub title: String,
    pub description: String,
    pub competitor_context: String,
    pub recommendation: String,
}

/// Result of gap analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapAnalysis {
    pub opportunities: Vec<Opportunity>,
    pub summary: String,
}

/// Lesson learned from similar projects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lesson {
    pub category: String, // design, implementation, feature, ux, technical
    pub insight: String,
    pub application: String,
    pub priority: String, // high, medium, low
}

/// Research synthesisresult
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSynthesis {
    pub key_findings: Vec<String>,
    pub market_position: String,
    pub differentiators: Vec<String>,
    pub risks: Vec<String>,
    pub recommendations: Vec<String>,
}

/// Research analyzer with web scraping and AI
pub struct ResearchAnalyzer {
    db: SqlitePool,
    http_client: reqwest::Client,
}

impl ResearchAnalyzer {
    pub fn new(db: SqlitePool) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (compatible; OrkeeBot/1.0; +https://orkee.ai)")
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        Self { db, http_client }
    }

    /// Scrape URL and extract HTML content
    async fn scrape_url(&self, url: &str) -> Result<String> {
        info!("Scraping URL: {}", url);

        // Validate URL
        let parsed_url = url::Url::parse(url).map_err(|e| {
            error!("Invalid URL {}: {}", url, e);
            IdeateError::InvalidInput(format!("Invalid URL: {}", e))
        })?;

        // Check robots.txt compliance (basic check - just log for now)
        debug!("Fetching content from: {}", parsed_url);

        // Fetch the page
        let response = self.http_client.get(url).send().await.map_err(|e| {
            error!("Failed to fetch URL {}: {}", url, e);
            IdeateError::InvalidInput(format!("Failed to fetch URL: {}", e))
        })?;

        if !response.status().is_success() {
            return Err(IdeateError::InvalidInput(format!(
                "HTTP error {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let html = response.text().await.map_err(|e| {
            error!("Failed to read response body: {}", e);
            IdeateError::InvalidInput(format!("Failed to read response: {}", e))
        })?;

        Ok(html)
    }

    /// Extract plain text from HTML for analysis
    fn extract_text_from_html(&self, html: &str) -> String {
        let document = Html::parse_document(html);

        // Remove script and style tags
        let mut text_parts = Vec::new();

        // Extract text from common content tags
        let selectors = [
            "h1", "h2", "h3", "h4", "h5", "h6", "p", "li", "span", "div", "section", "article",
        ];

        for selector_str in &selectors {
            if let Ok(selector) = Selector::parse(selector_str) {
                for element in document.select(&selector) {
                    let text = element.text().collect::<Vec<_>>().join(" ");
                    if !text.trim().is_empty() {
                        text_parts.push(text.trim().to_string());
                    }
                }
            }
        }

        text_parts.join("\n")
    }

    /// Check cache for competitor analysis
    async fn get_cached_competitor(
        &self,
        session_id: &str,
        url: &str,
    ) -> Result<Option<Competitor>> {
        let result = sqlx::query(
            "SELECT name, url, strengths, gaps, features, created_at
             FROM competitor_analysis_cache
             WHERE session_id = ? AND url = ? AND created_at > datetime('now', '-24 hours')",
        )
        .bind(session_id)
        .bind(url)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = result {
            debug!("Cache hit for competitor: {}", url);
            Ok(Some(Competitor {
                name: row.get("name"),
                url: row.get("url"),
                strengths: serde_json::from_str(row.get("strengths")).unwrap_or_default(),
                gaps: serde_json::from_str(row.get("gaps")).unwrap_or_default(),
                features: serde_json::from_str(row.get("features")).unwrap_or_default(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Store competitor in cache
    async fn cache_competitor(&self, session_id: &str, competitor: &Competitor) -> Result<()> {
        let strengths_json = serde_json::to_string(&competitor.strengths)?;
        let gaps_json = serde_json::to_string(&competitor.gaps)?;
        let features_json = serde_json::to_string(&competitor.features)?;

        sqlx::query(
            "INSERT INTO competitor_analysis_cache
             (session_id, url, name, strengths, gaps, features, created_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))
             ON CONFLICT(session_id, url) DO UPDATE SET
             name = excluded.name,
             strengths = excluded.strengths,
             gaps = excluded.gaps,
             features = excluded.features,
             created_at = excluded.created_at",
        )
        .bind(session_id)
        .bind(&competitor.url)
        .bind(&competitor.name)
        .bind(&strengths_json)
        .bind(&gaps_json)
        .bind(&features_json)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Analyze competitor URL
    pub async fn analyze_competitor(
        &self,
        session_id: &str,
        project_description: &str,
        url: &str,
        user_id: &str,
        model: &str,
    ) -> Result<Competitor> {
        info!("Analyzing competitor: {}", url);

        // Check cache first
        if let Some(cached) = self.get_cached_competitor(session_id, url).await? {
            info!("Returning cached competitor analysis for: {}", url);
            return Ok(cached);
        }

        // Scrape the URL
        let html = self.scrape_url(url).await?;
        let text_content = self.extract_text_from_html(&html);

        // Use AI to analyze
        let prompt =
            research_prompts::competitor_analysis_prompt(project_description, &text_content, url)
                .map_err(|e| {
                error!("Failed to load competitor analysis prompt: {}", e);
                IdeateError::InvalidSection(format!("Failed to load prompt: {}", e))
            })?;

        let system_prompt = research_prompts::get_research_system_prompt().map_err(|e| {
            error!("Failed to load research system prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load system prompt: {}", e))
        })?;

        let client = Client::new();

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 64000,
            "temperature": 0.7,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "system": system_prompt
        });

        let api_port = env::var("ORKEE_API_PORT").unwrap_or_else(|_| "4001".to_string());
        let proxy_url = format!("http://localhost:{}/api/ai/anthropic/v1/messages", api_port);

        let response = client
            .post(&proxy_url)
            .header("x-user-id", user_id)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to call AI proxy: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IdeateError::AIService(format!(
                "AI proxy returned error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to parse AI response: {}", e)))?;

        let data_text = anthropic_response
            .content
            .first()
            .map(|c| c.text.as_str())
            .ok_or_else(|| IdeateError::AIService("Empty response from AI".to_string()))?;

        let competitor: Competitor = serde_json::from_str(data_text)
            .map_err(|e| IdeateError::AIService(format!("Failed to parse structured data: {}", e)))?;

        // Cache the result
        self.cache_competitor(session_id, &competitor).await?;

        // Store in research table
        self.store_competitor(session_id, &competitor).await?;

        Ok(competitor)
    }

    /// Store competitor in ideate_research table
    async fn store_competitor(&self, session_id: &str, competitor: &Competitor) -> Result<()> {
        // Fetch existing research
        let existing = sqlx::query("SELECT competitors FROM ideate_research WHERE session_id = ?")
            .bind(session_id)
            .fetch_optional(&self.db)
            .await?;

        let mut competitors: Vec<Competitor> = if let Some(row) = existing {
            let competitors_json: String = row.get("competitors");
            serde_json::from_str(&competitors_json).unwrap_or_default()
        } else {
            vec![]
        };

        // Add or update competitor
        if let Some(pos) = competitors.iter().position(|c| c.url == competitor.url) {
            competitors[pos] = competitor.clone();
        } else {
            competitors.push(competitor.clone());
        }

        // Update database
        let competitors_json = serde_json::to_string(&competitors)?;
        sqlx::query(
            "UPDATE ideate_research SET competitors = ?, updated_at = datetime('now')
             WHERE session_id = ?",
        )
        .bind(&competitors_json)
        .bind(session_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get all competitors for a session
    pub async fn get_competitors(&self, session_id: &str) -> Result<Vec<Competitor>> {
        let result = sqlx::query("SELECT competitors FROM ideate_research WHERE session_id = ?")
            .bind(session_id)
            .fetch_optional(&self.db)
            .await?;

        if let Some(row) = result {
            let competitors_json: String = row.get("competitors");
            let competitors: Vec<Competitor> =
                serde_json::from_str(&competitors_json).unwrap_or_default();
            Ok(competitors)
        } else {
            Ok(vec![])
        }
    }

    /// Perform gap analysis across competitors
    pub async fn analyze_gaps(
        &self,
        session_id: &str,
        project_description: &str,
        your_features: Vec<String>,
        user_id: &str,
        model: &str,
    ) -> Result<GapAnalysis> {
        info!("Performing gap analysis for session: {}", session_id);

        let competitors = self.get_competitors(session_id).await?;

        if competitors.is_empty() {
            return Ok(GapAnalysis {
                opportunities: vec![],
                summary: "No competitors analyzed yet.".to_string(),
            });
        }

        // Prepare competitor features
        let competitor_data: Vec<(String, Vec<String>)> = competitors
            .iter()
            .map(|c| (c.name.clone(), c.features.clone()))
            .collect();

        let competitor_refs: Vec<&(String, Vec<String>)> = competitor_data.iter().collect();

        let prompt = research_prompts::gap_analysis_prompt(
            project_description,
            &your_features,
            &competitor_refs,
        )
        .map_err(|e| {
            error!("Failed to load gap analysis prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load prompt: {}", e))
        })?;

        let system_prompt = research_prompts::get_research_system_prompt().map_err(|e| {
            error!("Failed to load research system prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load system prompt: {}", e))
        })?;

        let client = Client::new();

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 64000,
            "temperature": 0.7,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "system": system_prompt
        });

        let api_port = env::var("ORKEE_API_PORT").unwrap_or_else(|_| "4001".to_string());
        let proxy_url = format!("http://localhost:{}/api/ai/anthropic/v1/messages", api_port);

        let response = client
            .post(&proxy_url)
            .header("x-user-id", user_id)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to call AI proxy: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IdeateError::AIService(format!(
                "AI proxy returned error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to parse AI response: {}", e)))?;

        let data_text = anthropic_response
            .content
            .first()
            .map(|c| c.text.as_str())
            .ok_or_else(|| IdeateError::AIService("Empty response from AI".to_string()))?;

        let data: GapAnalysis = serde_json::from_str(data_text)
            .map_err(|e| IdeateError::AIService(format!("Failed to parse structured data: {}", e)))?;

        Ok(data)
    }

    /// Extract UI/UX patterns from URL
    pub async fn extract_ui_patterns(
        &self,
        project_description: &str,
        url: &str,
        user_id: &str,
        model: &str,
    ) -> Result<Vec<UIPattern>> {
        info!("Extracting UI patterns from: {}", url);

        // Scrape the URL
        let html = self.scrape_url(url).await?;

        // Extract structural information (simplified - just get text for now)
        let structure = self.extract_text_from_html(&html);

        let prompt =
            research_prompts::ui_pattern_prompt(project_description, &structure).map_err(|e| {
                error!("Failed to load UI pattern prompt: {}", e);
                IdeateError::InvalidSection(format!("Failed to load prompt: {}", e))
            })?;

        let system_prompt = research_prompts::get_research_system_prompt().map_err(|e| {
            error!("Failed to load research system prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load system prompt: {}", e))
        })?;

        #[derive(Deserialize)]
        struct PatternResponse {
            patterns: Vec<UIPattern>,
        }

        let client = Client::new();

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 64000,
            "temperature": 0.7,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "system": system_prompt
        });

        let api_port = env::var("ORKEE_API_PORT").unwrap_or_else(|_| "4001".to_string());
        let proxy_url = format!("http://localhost:{}/api/ai/anthropic/v1/messages", api_port);

        let response = client
            .post(&proxy_url)
            .header("x-user-id", user_id)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to call AI proxy: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IdeateError::AIService(format!(
                "AI proxy returned error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to parse AI response: {}", e)))?;

        let data_text = anthropic_response
            .content
            .first()
            .map(|c| c.text.as_str())
            .ok_or_else(|| IdeateError::AIService("Empty response from AI".to_string()))?;

        let data: PatternResponse = serde_json::from_str(data_text)
            .map_err(|e| IdeateError::AIService(format!("Failed to parse structured data: {}", e)))?;

        Ok(data.patterns)
    }

    /// Add similar project
    pub async fn add_similar_project(
        &self,
        session_id: &str,
        project: SimilarProject,
    ) -> Result<()> {
        info!(
            "Adding similar project: {} for session: {}",
            project.name, session_id
        );

        // Fetch existing research
        let existing =
            sqlx::query("SELECT similar_projects FROM ideate_research WHERE session_id = ?")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        let mut projects: Vec<SimilarProject> = if let Some(row) = existing {
            let projects_json: String = row.get("similar_projects");
            serde_json::from_str(&projects_json).unwrap_or_default()
        } else {
            vec![]
        };

        // Add or update project
        if let Some(pos) = projects.iter().position(|p| p.url == project.url) {
            projects[pos] = project;
        } else {
            projects.push(project);
        }

        // Update database
        let projects_json = serde_json::to_string(&projects)?;
        sqlx::query(
            "UPDATE ideate_research SET similar_projects = ?, updated_at = datetime('now')
             WHERE session_id = ?",
        )
        .bind(&projects_json)
        .bind(session_id)
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get similar projects for a session
    pub async fn get_similar_projects(&self, session_id: &str) -> Result<Vec<SimilarProject>> {
        let result =
            sqlx::query("SELECT similar_projects FROM ideate_research WHERE session_id = ?")
                .bind(session_id)
                .fetch_optional(&self.db)
                .await?;

        if let Some(row) = result {
            let projects_json: String = row.get("similar_projects");
            let projects: Vec<SimilarProject> =
                serde_json::from_str(&projects_json).unwrap_or_default();
            Ok(projects)
        } else {
            Ok(vec![])
        }
    }

    /// Extract lessons from similar project
    pub async fn extract_lessons(
        &self,
        project_description: &str,
        similar_project: &SimilarProject,
        user_id: &str,
        model: &str,
    ) -> Result<Vec<Lesson>> {
        info!("Extracting lessons from: {}", similar_project.name);

        let prompt = research_prompts::lessons_learned_prompt(
            project_description,
            &similar_project.name,
            &similar_project.positive_aspects,
            &similar_project.negative_aspects,
        )
        .map_err(|e| {
            error!("Failed to load lessons learned prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load prompt: {}", e))
        })?;

        let system_prompt = research_prompts::get_research_system_prompt().map_err(|e| {
            error!("Failed to load research system prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load system prompt: {}", e))
        })?;

        #[derive(Deserialize)]
        struct LessonResponse {
            lessons: Vec<Lesson>,
        }

        let client = Client::new();

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 64000,
            "temperature": 0.7,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "system": system_prompt
        });

        let api_port = env::var("ORKEE_API_PORT").unwrap_or_else(|_| "4001".to_string());
        let proxy_url = format!("http://localhost:{}/api/ai/anthropic/v1/messages", api_port);

        let response = client
            .post(&proxy_url)
            .header("x-user-id", user_id)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to call AI proxy: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IdeateError::AIService(format!(
                "AI proxy returned error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to parse AI response: {}", e)))?;

        let data_text = anthropic_response
            .content
            .first()
            .map(|c| c.text.as_str())
            .ok_or_else(|| IdeateError::AIService("Empty response from AI".to_string()))?;

        let data: LessonResponse = serde_json::from_str(data_text)
            .map_err(|e| IdeateError::AIService(format!("Failed to parse structured data: {}", e)))?;

        Ok(data.lessons)
    }

    /// Synthesize all research findings
    pub async fn synthesize_research(
        &self,
        session_id: &str,
        project_description: &str,
        user_id: &str,
        model: &str,
    ) -> Result<ResearchSynthesis> {
        info!("Synthesizing research for session: {}", session_id);

        let competitors = self.get_competitors(session_id).await?;
        let similar_projects = self.get_similar_projects(session_id).await?;

        let competitor_data: Vec<(String, Vec<String>, Vec<String>)> = competitors
            .iter()
            .map(|c| (c.name.clone(), c.strengths.clone(), c.gaps.clone()))
            .collect();

        let prompt = research_prompts::research_synthesis_prompt(
            project_description,
            &competitor_data,
            similar_projects.len(),
        )
        .map_err(|e| {
            error!("Failed to load research synthesis prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load prompt: {}", e))
        })?;

        let system_prompt = research_prompts::get_research_system_prompt().map_err(|e| {
            error!("Failed to load research system prompt: {}", e);
            IdeateError::InvalidSection(format!("Failed to load system prompt: {}", e))
        })?;

        let client = Client::new();

        let request_body = serde_json::json!({
            "model": model,
            "max_tokens": 64000,
            "temperature": 0.7,
            "messages": [{
                "role": "user",
                "content": prompt
            }],
            "system": system_prompt
        });

        let api_port = env::var("ORKEE_API_PORT").unwrap_or_else(|_| "4001".to_string());
        let proxy_url = format!("http://localhost:{}/api/ai/anthropic/v1/messages", api_port);

        let response = client
            .post(&proxy_url)
            .header("x-user-id", user_id)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to call AI proxy: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(IdeateError::AIService(format!(
                "AI proxy returned error {}: {}",
                status, error_text
            )));
        }

        let anthropic_response: AnthropicResponse = response
            .json()
            .await
            .map_err(|e| IdeateError::AIService(format!("Failed to parse AI response: {}", e)))?;

        let data_text = anthropic_response
            .content
            .first()
            .map(|c| c.text.as_str())
            .ok_or_else(|| IdeateError::AIService("Empty response from AI".to_string()))?;

        let data: ResearchSynthesis = serde_json::from_str(data_text)
            .map_err(|e| IdeateError::AIService(format!("Failed to parse structured data: {}", e)))?;

        Ok(data)
    }
}
