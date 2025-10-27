// ABOUTME: Template management for PRD quickstart templates
// ABOUTME: Provides CRUD operations and template application logic

use crate::error::{IdeateError, Result};
use crate::types::{CreateTemplateInput, PRDTemplate};
use chrono::Utc;
use sqlx::{Row, SqlitePool};

/// Manager for PRD quickstart templates
pub struct TemplateManager {
    db: SqlitePool,
}

impl TemplateManager {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Get all available templates
    pub async fn get_templates(&self) -> Result<Vec<PRDTemplate>> {
        let templates = sqlx::query(
            "SELECT id, name, description, project_type, one_liner_prompts, default_features,
                    default_dependencies, is_system, created_at
             FROM prd_quickstart_templates
             ORDER BY is_system DESC, name ASC",
        )
        .fetch_all(&self.db)
        .await?;

        templates
            .into_iter()
            .map(|row| {
                Ok(PRDTemplate {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    project_type: row.get("project_type"),
                    one_liner_prompts: row
                        .get::<Option<String>, _>("one_liner_prompts")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    default_features: row
                        .get::<Option<String>, _>("default_features")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    default_dependencies: row
                        .get::<Option<String>, _>("default_dependencies")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    is_system: row.get::<i32, _>("is_system") == 1,
                    created_at: row.get("created_at"),
                })
            })
            .collect()
    }

    /// Get a specific template by ID
    pub async fn get_template(&self, template_id: &str) -> Result<PRDTemplate> {
        let template = sqlx::query(
            "SELECT id, name, description, project_type, one_liner_prompts, default_features,
                    default_dependencies, is_system, created_at
             FROM prd_quickstart_templates
             WHERE id = $1",
        )
        .bind(template_id)
        .fetch_optional(&self.db)
        .await?
        .ok_or_else(|| IdeateError::TemplateNotFound(template_id.to_string()))?;

        Ok(PRDTemplate {
            id: template.get("id"),
            name: template.get("name"),
            description: template.get("description"),
            project_type: template.get("project_type"),
            one_liner_prompts: template
                .get::<Option<String>, _>("one_liner_prompts")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_features: template
                .get::<Option<String>, _>("default_features")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_dependencies: template
                .get::<Option<String>, _>("default_dependencies")
                .and_then(|s| serde_json::from_str(&s).ok()),
            is_system: template.get::<i32, _>("is_system") == 1,
            created_at: template.get("created_at"),
        })
    }

    /// Create a new template (user-created)
    pub async fn create_template(&self, input: CreateTemplateInput) -> Result<PRDTemplate> {
        let id = nanoid::nanoid!(8);
        let now = Utc::now();

        let one_liner_prompts_json = input
            .one_liner_prompts
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let default_features_json = input
            .default_features
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let default_dependencies_json = input
            .default_dependencies
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());

        let template = sqlx::query(
            "INSERT INTO prd_quickstart_templates
             (id, name, description, project_type, one_liner_prompts, default_features,
              default_dependencies, is_system, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, 0, $8)
             RETURNING id, name, description, project_type, one_liner_prompts, default_features,
                       default_dependencies, is_system, created_at",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.project_type)
        .bind(&one_liner_prompts_json)
        .bind(&default_features_json)
        .bind(&default_dependencies_json)
        .bind(now)
        .fetch_one(&self.db)
        .await?;

        Ok(PRDTemplate {
            id: template.get("id"),
            name: template.get("name"),
            description: template.get("description"),
            project_type: template.get("project_type"),
            one_liner_prompts: template
                .get::<Option<String>, _>("one_liner_prompts")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_features: template
                .get::<Option<String>, _>("default_features")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_dependencies: template
                .get::<Option<String>, _>("default_dependencies")
                .and_then(|s| serde_json::from_str(&s).ok()),
            is_system: template.get::<i32, _>("is_system") == 1,
            created_at: template.get("created_at"),
        })
    }

    /// Delete a template (only user-created templates can be deleted)
    pub async fn delete_template(&self, template_id: &str) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM prd_quickstart_templates
             WHERE id = $1 AND is_system = 0",
        )
        .bind(template_id)
        .execute(&self.db)
        .await?;

        if result.rows_affected() == 0 {
            return Err(IdeateError::Forbidden(
                "Cannot delete system template or template not found".to_string(),
            ));
        }

        Ok(())
    }

    /// Get templates filtered by project type
    pub async fn get_templates_by_type(&self, project_type: &str) -> Result<Vec<PRDTemplate>> {
        let templates = sqlx::query(
            "SELECT id, name, description, project_type, one_liner_prompts, default_features,
                    default_dependencies, is_system, created_at
             FROM prd_quickstart_templates
             WHERE project_type = $1
             ORDER BY is_system DESC, name ASC",
        )
        .bind(project_type)
        .fetch_all(&self.db)
        .await?;

        templates
            .into_iter()
            .map(|row| {
                Ok(PRDTemplate {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    project_type: row.get("project_type"),
                    one_liner_prompts: row
                        .get::<Option<String>, _>("one_liner_prompts")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    default_features: row
                        .get::<Option<String>, _>("default_features")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    default_dependencies: row
                        .get::<Option<String>, _>("default_dependencies")
                        .and_then(|s| serde_json::from_str(&s).ok()),
                    is_system: row.get::<i32, _>("is_system") == 1,
                    created_at: row.get("created_at"),
                })
            })
            .collect()
    }

    /// Suggest best matching template based on initial description
    pub async fn suggest_template(&self, description: &str) -> Result<Option<PRDTemplate>> {
        let description_lower = description.to_lowercase();

        // Simple keyword matching for now
        let project_type = if description_lower.contains("saas")
            || description_lower.contains("web app")
            || description_lower.contains("subscription")
        {
            Some("saas")
        } else if description_lower.contains("mobile")
            || description_lower.contains("ios")
            || description_lower.contains("android")
        {
            Some("mobile")
        } else if description_lower.contains("api")
            || description_lower.contains("backend")
            || description_lower.contains("service")
        {
            Some("api")
        } else if description_lower.contains("marketplace")
            || description_lower.contains("platform")
            || description_lower.contains("two-sided")
        {
            Some("marketplace")
        } else if description_lower.contains("dashboard")
            || description_lower.contains("internal")
            || description_lower.contains("admin")
        {
            Some("internal-tool")
        } else {
            None
        };

        if let Some(project_type) = project_type {
            let templates = self.get_templates_by_type(project_type).await?;
            Ok(templates.into_iter().next())
        } else {
            Ok(None)
        }
    }
}
