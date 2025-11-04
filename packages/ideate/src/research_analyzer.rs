// ABOUTME: Research analyzer utilities for competitor and project data management
// ABOUTME: Handles web scraping, data storage, and CRUD operations (AI moved to frontend)

use crate::error::{IdeateError, Result};
use crate::types::{Competitor, SimilarProject};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
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

    /// Scrape URL and return text content (helper for frontend AI)
    pub async fn scrape_competitor_url(&self, url: &str) -> Result<String> {
        let html = self.scrape_url(url).await?;
        Ok(self.extract_text_from_html(&html))
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


}
