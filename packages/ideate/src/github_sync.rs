// ABOUTME: GitHub synchronization service for Epics and Tasks
// ABOUTME: Handles creating, updating, and syncing GitHub issues with local Epic/Task data

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

use crate::epic::{Epic, EpicStatus};
use git_utils::{GitHubCli, UpdateIssueParams};

#[derive(Debug, Error)]
pub enum GitHubSyncError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("GitHub API error: {0}")]
    ApiError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Missing GitHub token")]
    MissingToken,

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, GitHubSyncError>;

/// GitHub issue creation request
#[derive(Debug, Serialize)]
struct CreateIssueRequest {
    title: String,
    body: String,
    labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignees: Option<Vec<String>>,
}

/// GitHub issue update request
#[derive(Debug, Serialize)]
struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
}

/// GitHub issue response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubIssue {
    number: i32,
    html_url: String,
    title: String,
    body: Option<String>,
    state: String,
    updated_at: DateTime<Utc>,
}

/// GitHub API error response
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct GitHubErrorResponse {
    message: String,
    #[serde(default)]
    errors: Vec<serde_json::Value>,
}

/// GitHub sync configuration for a project
#[derive(Debug, Clone)]
pub struct GitHubConfig {
    pub owner: String,
    pub repo: String,
    pub token: String,
    pub labels_config: Option<HashMap<String, String>>,
    pub default_assignee: Option<String>,
}

/// Sync status tracking
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Pending,
    Syncing,
    Synced,
    Failed,
    Conflict,
}

/// Sync direction
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    LocalToGithub,
    GithubToLocal,
    Bidirectional,
}

/// Entity type for sync tracking
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "TEXT", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum EntityType {
    Epic,
    Task,
    Comment,
    Status,
}

/// GitHub sync record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubSync {
    pub id: String,
    pub project_id: String,
    pub entity_type: EntityType,
    pub entity_id: String,
    pub github_issue_number: Option<i32>,
    pub github_issue_url: Option<String>,
    pub sync_status: SyncStatus,
    pub sync_direction: Option<SyncDirection>,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub last_sync_hash: Option<String>,
    pub last_sync_error: Option<String>,
    pub retry_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Result of a sync operation
#[derive(Debug, Serialize, Deserialize)]
pub struct SyncResult {
    pub issue_number: i32,
    pub issue_url: String,
    pub synced_at: DateTime<Utc>,
}

/// Sync method preference
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyncMethod {
    /// Use gh CLI when available (preferred)
    GhCli,
    /// Use REST API directly
    RestApi,
    /// Auto-detect and use gh CLI if available, fallback to REST API
    Auto,
}

/// GitHub sync service
pub struct GitHubSyncService {
    client: Client,
    gh_cli: Option<GitHubCli>,
    sync_method: SyncMethod,
}

impl GitHubSyncService {
    /// Create new sync service with auto-detection of gh CLI
    pub fn new() -> Self {
        Self::with_method(SyncMethod::Auto)
    }

    /// Create sync service with specific method
    pub fn with_method(method: SyncMethod) -> Self {
        let gh_cli = if method == SyncMethod::RestApi {
            None
        } else {
            GitHubCli::new().ok()
        };

        let effective_method = match method {
            SyncMethod::Auto => {
                if gh_cli.is_some() {
                    SyncMethod::GhCli
                } else {
                    SyncMethod::RestApi
                }
            }
            other => other,
        };

        Self {
            client: Client::new(),
            gh_cli,
            sync_method: effective_method,
        }
    }

    /// Get the active sync method
    pub fn sync_method(&self) -> SyncMethod {
        self.sync_method
    }

    /// Check if gh CLI is available and authenticated
    pub fn has_gh_cli(&self) -> bool {
        self.gh_cli.is_some()
    }

    /// Create an Epic issue on GitHub
    pub async fn create_epic_issue(
        &self,
        epic: &Epic,
        config: &GitHubConfig,
        pool: &sqlx::SqlitePool,
    ) -> Result<SyncResult> {
        // Format Epic as GitHub issue
        let body = self.format_epic_body(epic);

        // Get labels from config or use defaults
        let mut labels = vec!["epic".to_string()];
        if let Some(label_config) = &config.labels_config {
            if let Some(epic_label) = label_config.get("epic") {
                labels = vec![epic_label.clone()];
            }
        }

        // Add status label
        labels.push(format!(
            "status:{}",
            self.epic_status_to_label(&epic.status)
        ));

        // Create issue request
        let request = CreateIssueRequest {
            title: epic.name.clone(),
            body,
            labels,
            assignees: config.default_assignee.as_ref().map(|a| vec![a.clone()]),
        };

        // Call GitHub API
        let issue = self
            .create_github_issue(&config.owner, &config.repo, &config.token, &request)
            .await?;

        // Update Epic with GitHub info
        sqlx::query(
            "UPDATE epics SET github_issue_number = ?, github_issue_url = ?, github_synced_at = ? WHERE id = ?"
        )
        .bind(issue.number)
        .bind(&issue.html_url)
        .bind(Utc::now().to_rfc3339())
        .bind(&epic.id)
        .execute(pool)
        .await?;

        // Create sync record
        self.create_sync_record(
            pool,
            &epic.project_id,
            EntityType::Epic,
            &epic.id,
            issue.number,
            &issue.html_url,
        )
        .await?;

        Ok(SyncResult {
            issue_number: issue.number,
            issue_url: issue.html_url,
            synced_at: Utc::now(),
        })
    }

    /// Sync Epic updates to GitHub
    pub async fn sync_epic_to_github(
        &self,
        epic: &Epic,
        config: &GitHubConfig,
        pool: &sqlx::SqlitePool,
    ) -> Result<SyncResult> {
        let issue_number = epic.github_issue_number.ok_or_else(|| {
            GitHubSyncError::InvalidConfig("Epic not synced to GitHub".to_string())
        })?;

        // Format Epic body
        let body = self.format_epic_body(epic);

        // Get labels
        let mut labels = vec!["epic".to_string()];
        labels.push(format!(
            "status:{}",
            self.epic_status_to_label(&epic.status)
        ));

        // Determine GitHub state
        let state = match epic.status {
            EpicStatus::Completed | EpicStatus::Cancelled => Some("closed".to_string()),
            _ => Some("open".to_string()),
        };

        // Create update request
        let request = UpdateIssueRequest {
            title: Some(epic.name.clone()),
            body: Some(body),
            state,
            labels: Some(labels),
        };

        // Call GitHub API
        let issue = self
            .update_github_issue(
                &config.owner,
                &config.repo,
                &config.token,
                issue_number,
                &request,
            )
            .await?;

        // Update Epic sync timestamp
        sqlx::query("UPDATE epics SET github_synced_at = ? WHERE id = ?")
            .bind(Utc::now().to_rfc3339())
            .bind(&epic.id)
            .execute(pool)
            .await?;

        // Update sync record
        self.update_sync_record(pool, EntityType::Epic, &epic.id)
            .await?;

        Ok(SyncResult {
            issue_number: issue.number,
            issue_url: issue.html_url,
            synced_at: Utc::now(),
        })
    }

    /// Create task issues for an Epic
    pub async fn create_task_issues(
        &self,
        epic_id: &str,
        project_id: &str,
        config: &GitHubConfig,
        pool: &sqlx::SqlitePool,
    ) -> Result<Vec<SyncResult>> {
        // Get Epic issue number using EpicManager
        let epic_manager = crate::epic_manager::EpicManager::new(pool.clone());
        let epic = epic_manager
            .get_epic(project_id, epic_id)
            .await
            .map_err(|_| GitHubSyncError::InvalidConfig("Failed to fetch Epic".to_string()))?
            .ok_or_else(|| GitHubSyncError::InvalidConfig("Epic not found".to_string()))?;

        let epic_issue_number = epic.github_issue_number.ok_or_else(|| {
            GitHubSyncError::InvalidConfig("Epic must be synced to GitHub first".to_string())
        })?;

        // Get tasks for Epic
        let tasks = sqlx::query(
            "SELECT id, name, description, status, priority, github_issue_number, github_issue_url FROM tasks WHERE epic_id = ?"
        )
        .bind(epic_id)
        .fetch_all(pool)
        .await?;

        let mut results = Vec::new();
        let mut task_list = String::new();

        for task in tasks {
            use sqlx::Row;

            let task_id: String = task.get("id");
            let task_name: String = task.get("name");
            let task_description: Option<String> = task.get("description");
            let github_issue_number: Option<i32> = task.get("github_issue_number");

            // Skip if already synced
            if github_issue_number.is_some() {
                continue;
            }

            // Format task body
            let body = format!(
                "Part of Epic #{}\n\n{}",
                epic_issue_number,
                task_description.unwrap_or_default()
            );

            // Create issue
            let request = CreateIssueRequest {
                title: task_name.clone(),
                body,
                labels: vec!["task".to_string()],
                assignees: config.default_assignee.as_ref().map(|a| vec![a.clone()]),
            };

            let issue = self
                .create_github_issue(&config.owner, &config.repo, &config.token, &request)
                .await?;

            // Update task with GitHub info
            sqlx::query(
                "UPDATE tasks SET github_issue_number = ?, github_issue_url = ? WHERE id = ?",
            )
            .bind(issue.number)
            .bind(&issue.html_url)
            .bind(&task_id)
            .execute(pool)
            .await?;

            // Create sync record
            self.create_sync_record(
                pool,
                project_id,
                EntityType::Task,
                &task_id,
                issue.number,
                &issue.html_url,
            )
            .await?;

            // Add to task list
            task_list.push_str(&format!("- [ ] #{} {}\n", issue.number, task_name));

            results.push(SyncResult {
                issue_number: issue.number,
                issue_url: issue.html_url.clone(),
                synced_at: Utc::now(),
            });
        }

        // Update Epic issue with task list
        if !task_list.is_empty() {
            let epic_body = self.format_epic_body(&epic);
            let updated_body = format!("{}\n\n## Tasks\n{}", epic_body, task_list);

            let request = UpdateIssueRequest {
                title: None,
                body: Some(updated_body),
                state: None,
                labels: None,
            };

            self.update_github_issue(
                &config.owner,
                &config.repo,
                &config.token,
                epic_issue_number,
                &request,
            )
            .await?;
        }

        Ok(results)
    }

    /// Format Epic as GitHub issue body (markdown)
    fn format_epic_body(&self, epic: &Epic) -> String {
        let mut body = String::new();

        // Overview
        body.push_str("## Overview\n\n");
        body.push_str(&epic.overview_markdown);
        body.push_str("\n\n");

        // Technical Approach
        body.push_str("## Technical Approach\n\n");
        body.push_str(&epic.technical_approach);
        body.push_str("\n\n");

        // Implementation Strategy
        if let Some(strategy) = &epic.implementation_strategy {
            body.push_str("## Implementation Strategy\n\n");
            body.push_str(strategy);
            body.push_str("\n\n");
        }

        // Architecture Decisions
        if let Some(decisions) = &epic.architecture_decisions {
            if !decisions.is_empty() {
                body.push_str("## Architecture Decisions\n\n");
                for decision in decisions {
                    body.push_str(&format!("### {}\n\n", decision.decision));
                    body.push_str(&format!("**Rationale:** {}\n\n", decision.rationale));

                    if let Some(alternatives) = &decision.alternatives {
                        body.push_str("**Alternatives considered:**\n");
                        for alt in alternatives {
                            body.push_str(&format!("- {}\n", alt));
                        }
                        body.push('\n');
                    }

                    if let Some(tradeoffs) = &decision.tradeoffs {
                        body.push_str(&format!("**Trade-offs:** {}\n\n", tradeoffs));
                    }
                }
            }
        }

        // Dependencies
        if let Some(deps) = &epic.dependencies {
            if !deps.is_empty() {
                body.push_str("## Dependencies\n\n");
                for dep in deps {
                    let version_str = dep
                        .version
                        .as_ref()
                        .map(|v| format!(" ({})", v))
                        .unwrap_or_default();
                    body.push_str(&format!(
                        "- **{}**{} - {} - {}\n",
                        dep.name, version_str, dep.dep_type, dep.reason
                    ));
                }
                body.push('\n');
            }
        }

        // Success Criteria
        if let Some(criteria) = &epic.success_criteria {
            if !criteria.is_empty() {
                body.push_str("## Success Criteria\n\n");
                for criterion in criteria {
                    let target_str = criterion
                        .target
                        .as_ref()
                        .map(|t| format!(" (Target: {})", t))
                        .unwrap_or_default();
                    let measurable = if criterion.measurable { "📊" } else { "📝" };
                    body.push_str(&format!(
                        "- {} {}{}\n",
                        measurable, criterion.criterion, target_str
                    ));
                }
                body.push('\n');
            }
        }

        // Metadata
        body.push_str("---\n\n");
        body.push_str(&format!("**Status:** {:?} | ", epic.status));
        body.push_str(&format!("**Progress:** {}% | ", epic.progress_percentage));
        if let Some(effort) = &epic.estimated_effort {
            body.push_str(&format!("**Effort:** {:?} | ", effort));
        }
        if let Some(complexity) = &epic.complexity {
            body.push_str(&format!("**Complexity:** {:?}", complexity));
        }

        body
    }

    /// Convert Epic status to GitHub label
    fn epic_status_to_label(&self, status: &EpicStatus) -> String {
        match status {
            EpicStatus::Draft => "draft",
            EpicStatus::Ready => "ready",
            EpicStatus::InProgress => "in-progress",
            EpicStatus::Blocked => "blocked",
            EpicStatus::Completed => "completed",
            EpicStatus::Cancelled => "cancelled",
        }
        .to_string()
    }

    /// Create GitHub issue (uses gh CLI if available, falls back to REST API)
    async fn create_github_issue(
        &self,
        owner: &str,
        repo: &str,
        token: &str,
        request: &CreateIssueRequest,
    ) -> Result<GitHubIssue> {
        // Try gh CLI first if available
        if let Some(gh) = &self.gh_cli {
            match gh
                .create_issue(
                    owner,
                    repo,
                    &request.title,
                    &request.body,
                    Some(request.labels.clone()),
                    request.assignees.clone(),
                )
                .await
            {
                Ok(gh_issue) => {
                    // Convert gh CLI issue to GitHubIssue
                    return Ok(GitHubIssue {
                        number: gh_issue.number,
                        html_url: gh_issue.url,
                        title: gh_issue.title,
                        body: gh_issue.body,
                        state: gh_issue.state,
                        updated_at: gh_issue
                            .updated_at
                            .parse::<DateTime<Utc>>()
                            .unwrap_or_else(|_| Utc::now()),
                    });
                }
                Err(e) => {
                    // Log gh CLI error and fall back to REST API
                    eprintln!("gh CLI failed, falling back to REST API: {}", e);
                }
            }
        }

        // Fallback to REST API
        let url = format!("https://api.github.com/repos/{}/{}/issues", owner, repo);

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Orkee-CCPM")
            .header("Accept", "application/vnd.github.v3+json")
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: GitHubErrorResponse = response.json().await?;
            return Err(GitHubSyncError::ApiError(error.message));
        }

        Ok(response.json().await?)
    }

    /// Update GitHub issue (uses gh CLI if available, falls back to REST API)
    async fn update_github_issue(
        &self,
        owner: &str,
        repo: &str,
        token: &str,
        issue_number: i32,
        request: &UpdateIssueRequest,
    ) -> Result<GitHubIssue> {
        // Try gh CLI first if available
        if let Some(gh) = &self.gh_cli {
            let params = UpdateIssueParams {
                title: request.title.clone(),
                body: request.body.clone(),
                state: request.state.clone(),
                labels: request.labels.clone(),
            };

            match gh.update_issue(owner, repo, issue_number, params).await {
                Ok(gh_issue) => {
                    // Convert gh CLI issue to GitHubIssue
                    return Ok(GitHubIssue {
                        number: gh_issue.number,
                        html_url: gh_issue.url,
                        title: gh_issue.title,
                        body: gh_issue.body,
                        state: gh_issue.state,
                        updated_at: gh_issue
                            .updated_at
                            .parse::<DateTime<Utc>>()
                            .unwrap_or_else(|_| Utc::now()),
                    });
                }
                Err(e) => {
                    // Log gh CLI error and fall back to REST API
                    eprintln!("gh CLI failed, falling back to REST API: {}", e);
                }
            }
        }

        // Fallback to REST API
        let url = format!(
            "https://api.github.com/repos/{}/{}/issues/{}",
            owner, repo, issue_number
        );

        let response = self
            .client
            .patch(&url)
            .header("Authorization", format!("Bearer {}", token))
            .header("User-Agent", "Orkee-CCPM")
            .header("Accept", "application/vnd.github.v3+json")
            .json(request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: GitHubErrorResponse = response.json().await?;
            return Err(GitHubSyncError::ApiError(error.message));
        }

        Ok(response.json().await?)
    }

    /// Create sync record in database
    async fn create_sync_record(
        &self,
        pool: &sqlx::SqlitePool,
        project_id: &str,
        entity_type: EntityType,
        entity_id: &str,
        issue_number: i32,
        issue_url: &str,
    ) -> Result<()> {
        let id = nanoid::nanoid!(12);
        let entity_type_str = match entity_type {
            EntityType::Epic => "epic",
            EntityType::Task => "task",
            EntityType::Comment => "comment",
            EntityType::Status => "status",
        };

        sqlx::query(
            "INSERT INTO github_sync (
                id, project_id, entity_type, entity_id,
                github_issue_number, github_issue_url,
                sync_status, sync_direction, last_synced_at
            ) VALUES (?, ?, ?, ?, ?, ?, 'synced', 'local_to_github', ?)",
        )
        .bind(id)
        .bind(project_id)
        .bind(entity_type_str)
        .bind(entity_id)
        .bind(issue_number)
        .bind(issue_url)
        .bind(Utc::now().to_rfc3339())
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Update sync record timestamp
    async fn update_sync_record(
        &self,
        pool: &sqlx::SqlitePool,
        entity_type: EntityType,
        entity_id: &str,
    ) -> Result<()> {
        let entity_type_str = match entity_type {
            EntityType::Epic => "epic",
            EntityType::Task => "task",
            EntityType::Comment => "comment",
            EntityType::Status => "status",
        };

        sqlx::query(
            "UPDATE github_sync SET last_synced_at = ?, sync_status = 'synced' WHERE entity_type = ? AND entity_id = ?"
        )
        .bind(Utc::now().to_rfc3339())
        .bind(entity_type_str)
        .bind(entity_id)
        .execute(pool)
        .await?;

        Ok(())
    }

    /// Get sync status for a project
    pub async fn get_sync_status(
        &self,
        pool: &sqlx::SqlitePool,
        project_id: &str,
    ) -> Result<Vec<GitHubSync>> {
        use sqlx::Row;

        let records = sqlx::query(
            "SELECT id, project_id, entity_type, entity_id,
                    github_issue_number, github_issue_url,
                    sync_status, sync_direction,
                    last_synced_at, last_sync_hash, last_sync_error, retry_count,
                    created_at, updated_at
            FROM github_sync WHERE project_id = ?
            ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?;

        let syncs = records
            .into_iter()
            .map(|r| {
                let entity_type_str: String = r.get("entity_type");
                let entity_type = match entity_type_str.as_str() {
                    "epic" => EntityType::Epic,
                    "task" => EntityType::Task,
                    "comment" => EntityType::Comment,
                    _ => EntityType::Status,
                };

                let sync_status_str: String = r.get("sync_status");
                let sync_status = match sync_status_str.as_str() {
                    "pending" => SyncStatus::Pending,
                    "syncing" => SyncStatus::Syncing,
                    "synced" => SyncStatus::Synced,
                    "failed" => SyncStatus::Failed,
                    _ => SyncStatus::Conflict,
                };

                let sync_direction_str: Option<String> = r.get("sync_direction");
                let sync_direction = sync_direction_str.and_then(|d| match d.as_str() {
                    "local_to_github" => Some(SyncDirection::LocalToGithub),
                    "github_to_local" => Some(SyncDirection::GithubToLocal),
                    "bidirectional" => Some(SyncDirection::Bidirectional),
                    _ => None,
                });

                let last_synced_str: Option<String> = r.get("last_synced_at");
                let created_str: String = r.get("created_at");
                let updated_str: String = r.get("updated_at");

                GitHubSync {
                    id: r.get("id"),
                    project_id: r.get("project_id"),
                    entity_type,
                    entity_id: r.get("entity_id"),
                    github_issue_number: r.get("github_issue_number"),
                    github_issue_url: r.get("github_issue_url"),
                    sync_status,
                    sync_direction,
                    last_synced_at: last_synced_str.and_then(|s| s.parse::<DateTime<Utc>>().ok()),
                    last_sync_hash: r.get("last_sync_hash"),
                    last_sync_error: r.get("last_sync_error"),
                    retry_count: r.get("retry_count"),
                    created_at: created_str
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                    updated_at: updated_str
                        .parse::<DateTime<Utc>>()
                        .unwrap_or_else(|_| Utc::now()),
                }
            })
            .collect();

        Ok(syncs)
    }
}

impl Default for GitHubSyncService {
    fn default() -> Self {
        Self::new()
    }
}
