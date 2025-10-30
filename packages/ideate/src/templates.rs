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

    /// Get all available templates (default: quickstart templates)
    pub async fn get_templates(&self) -> Result<Vec<PRDTemplate>> {
        self.get_templates_by_category("quickstart").await
    }

    /// Get templates filtered by category (quickstart or output)
    pub async fn get_templates_by_category(&self, category: &str) -> Result<Vec<PRDTemplate>> {
        let query_str = match category {
            "output" => "SELECT id, name, description, created_at
                         FROM prd_output_templates
                         ORDER BY name ASC".to_string(),
            _ => "SELECT id, name, description, project_type, one_liner_prompts, default_features,
                         default_dependencies, default_problem_statement, default_target_audience,
                         default_value_proposition, default_ui_considerations, default_ux_principles,
                         default_tech_stack_quick, default_mvp_scope, default_research_findings,
                         default_technical_specs, default_competitors, default_similar_projects,
                         is_system, created_at
                  FROM prd_quickstart_templates
                  ORDER BY is_system DESC, name ASC".to_string(),
        };

        let templates = sqlx::query(&query_str).fetch_all(&self.db).await?;

        templates
            .into_iter()
            .map(|row| {
                // For output templates, we only have id, name, description, created_at
                // For quickstart templates, we have all fields
                let project_type = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("project_type")
                };

                let one_liner_prompts = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("one_liner_prompts")
                        .and_then(|s| serde_json::from_str(&s).ok())
                };

                let default_features = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("default_features")
                        .and_then(|s| serde_json::from_str(&s).ok())
                };

                let default_dependencies = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("default_dependencies")
                        .and_then(|s| serde_json::from_str(&s).ok())
                };

                let is_system = if category == "output" {
                    false
                } else {
                    row.get::<i32, _>("is_system") == 1
                };

                let default_mvp_scope = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("default_mvp_scope")
                        .and_then(|s| serde_json::from_str(&s).ok())
                };

                let default_competitors = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("default_competitors")
                        .and_then(|s| serde_json::from_str(&s).ok())
                };

                let default_similar_projects = if category == "output" {
                    None
                } else {
                    row.get::<Option<String>, _>("default_similar_projects")
                        .and_then(|s| serde_json::from_str(&s).ok())
                };

                Ok(PRDTemplate {
                    id: row.get("id"),
                    name: row.get("name"),
                    description: row.get("description"),
                    project_type,
                    one_liner_prompts,
                    default_features,
                    default_dependencies,
                    default_problem_statement: if category == "output" {
                        None
                    } else {
                        row.get("default_problem_statement")
                    },
                    default_target_audience: if category == "output" {
                        None
                    } else {
                        row.get("default_target_audience")
                    },
                    default_value_proposition: if category == "output" {
                        None
                    } else {
                        row.get("default_value_proposition")
                    },
                    default_ui_considerations: if category == "output" {
                        None
                    } else {
                        row.get("default_ui_considerations")
                    },
                    default_ux_principles: if category == "output" {
                        None
                    } else {
                        row.get("default_ux_principles")
                    },
                    default_tech_stack_quick: if category == "output" {
                        None
                    } else {
                        row.get("default_tech_stack_quick")
                    },
                    default_mvp_scope,
                    default_research_findings: if category == "output" {
                        None
                    } else {
                        row.get("default_research_findings")
                    },
                    default_technical_specs: if category == "output" {
                        None
                    } else {
                        row.get("default_technical_specs")
                    },
                    default_competitors,
                    default_similar_projects,
                    is_system,
                    created_at: row.get("created_at"),
                })
            })
            .collect()
    }

    /// Get a specific template by ID
    pub async fn get_template(&self, template_id: &str) -> Result<PRDTemplate> {
        let template = sqlx::query(
            "SELECT id, name, description, project_type, one_liner_prompts, default_features,
                    default_dependencies, default_problem_statement, default_target_audience,
                    default_value_proposition, default_ui_considerations, default_ux_principles,
                    default_tech_stack_quick, default_mvp_scope, default_research_findings,
                    default_technical_specs, default_competitors, default_similar_projects,
                    is_system, created_at
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
            default_problem_statement: template.get("default_problem_statement"),
            default_target_audience: template.get("default_target_audience"),
            default_value_proposition: template.get("default_value_proposition"),
            default_ui_considerations: template.get("default_ui_considerations"),
            default_ux_principles: template.get("default_ux_principles"),
            default_tech_stack_quick: template.get("default_tech_stack_quick"),
            default_mvp_scope: template
                .get::<Option<String>, _>("default_mvp_scope")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_research_findings: template.get("default_research_findings"),
            default_technical_specs: template.get("default_technical_specs"),
            default_competitors: template
                .get::<Option<String>, _>("default_competitors")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_similar_projects: template
                .get::<Option<String>, _>("default_similar_projects")
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
        let default_mvp_scope_json = input
            .default_mvp_scope
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let default_competitors_json = input
            .default_competitors
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let default_similar_projects_json = input
            .default_similar_projects
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());

        let template = sqlx::query(
            "INSERT INTO prd_quickstart_templates
             (id, name, description, project_type, one_liner_prompts, default_features,
              default_dependencies, default_problem_statement, default_target_audience,
              default_value_proposition, default_ui_considerations, default_ux_principles,
              default_tech_stack_quick, default_mvp_scope, default_research_findings,
              default_technical_specs, default_competitors, default_similar_projects,
              is_system, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, 0, $19)
             RETURNING id, name, description, project_type, one_liner_prompts, default_features,
                       default_dependencies, default_problem_statement, default_target_audience,
                       default_value_proposition, default_ui_considerations, default_ux_principles,
                       default_tech_stack_quick, default_mvp_scope, default_research_findings,
                       default_technical_specs, default_competitors, default_similar_projects,
                       is_system, created_at",
        )
        .bind(&id)
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.project_type)
        .bind(&one_liner_prompts_json)
        .bind(&default_features_json)
        .bind(&default_dependencies_json)
        .bind(&input.default_problem_statement)
        .bind(&input.default_target_audience)
        .bind(&input.default_value_proposition)
        .bind(&input.default_ui_considerations)
        .bind(&input.default_ux_principles)
        .bind(&input.default_tech_stack_quick)
        .bind(&default_mvp_scope_json)
        .bind(&input.default_research_findings)
        .bind(&input.default_technical_specs)
        .bind(&default_competitors_json)
        .bind(&default_similar_projects_json)
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
            default_problem_statement: template.get("default_problem_statement"),
            default_target_audience: template.get("default_target_audience"),
            default_value_proposition: template.get("default_value_proposition"),
            default_ui_considerations: template.get("default_ui_considerations"),
            default_ux_principles: template.get("default_ux_principles"),
            default_tech_stack_quick: template.get("default_tech_stack_quick"),
            default_mvp_scope: template
                .get::<Option<String>, _>("default_mvp_scope")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_research_findings: template.get("default_research_findings"),
            default_technical_specs: template.get("default_technical_specs"),
            default_competitors: template
                .get::<Option<String>, _>("default_competitors")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_similar_projects: template
                .get::<Option<String>, _>("default_similar_projects")
                .and_then(|s| serde_json::from_str(&s).ok()),
            is_system: template.get::<i32, _>("is_system") == 1,
            created_at: template.get("created_at"),
        })
    }

    /// Update a template (only user-created templates can be updated)
    pub async fn update_template(
        &self,
        template_id: &str,
        input: CreateTemplateInput,
    ) -> Result<PRDTemplate> {
        // Check if template exists and is not a system template
        let existing = sqlx::query("SELECT is_system FROM prd_quickstart_templates WHERE id = $1")
            .bind(template_id)
            .fetch_optional(&self.db)
            .await?
            .ok_or_else(|| IdeateError::TemplateNotFound(template_id.to_string()))?;

        if existing.get::<i32, _>("is_system") == 1 {
            return Err(IdeateError::Forbidden(
                "Cannot update system template".to_string(),
            ));
        }

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
        let default_mvp_scope_json = input
            .default_mvp_scope
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let default_competitors_json = input
            .default_competitors
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());
        let default_similar_projects_json = input
            .default_similar_projects
            .as_ref()
            .map(|v| serde_json::to_string(v).unwrap());

        let template = sqlx::query(
            "UPDATE prd_quickstart_templates
             SET name = $1, description = $2, project_type = $3, one_liner_prompts = $4,
                 default_features = $5, default_dependencies = $6, default_problem_statement = $7,
                 default_target_audience = $8, default_value_proposition = $9,
                 default_ui_considerations = $10, default_ux_principles = $11,
                 default_tech_stack_quick = $12, default_mvp_scope = $13,
                 default_research_findings = $14, default_technical_specs = $15,
                 default_competitors = $16, default_similar_projects = $17
             WHERE id = $18
             RETURNING id, name, description, project_type, one_liner_prompts, default_features,
                       default_dependencies, default_problem_statement, default_target_audience,
                       default_value_proposition, default_ui_considerations, default_ux_principles,
                       default_tech_stack_quick, default_mvp_scope, default_research_findings,
                       default_technical_specs, default_competitors, default_similar_projects,
                       is_system, created_at",
        )
        .bind(&input.name)
        .bind(&input.description)
        .bind(&input.project_type)
        .bind(&one_liner_prompts_json)
        .bind(&default_features_json)
        .bind(&default_dependencies_json)
        .bind(&input.default_problem_statement)
        .bind(&input.default_target_audience)
        .bind(&input.default_value_proposition)
        .bind(&input.default_ui_considerations)
        .bind(&input.default_ux_principles)
        .bind(&input.default_tech_stack_quick)
        .bind(&default_mvp_scope_json)
        .bind(&input.default_research_findings)
        .bind(&input.default_technical_specs)
        .bind(&default_competitors_json)
        .bind(&default_similar_projects_json)
        .bind(template_id)
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
            default_problem_statement: template.get("default_problem_statement"),
            default_target_audience: template.get("default_target_audience"),
            default_value_proposition: template.get("default_value_proposition"),
            default_ui_considerations: template.get("default_ui_considerations"),
            default_ux_principles: template.get("default_ux_principles"),
            default_tech_stack_quick: template.get("default_tech_stack_quick"),
            default_mvp_scope: template
                .get::<Option<String>, _>("default_mvp_scope")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_research_findings: template.get("default_research_findings"),
            default_technical_specs: template.get("default_technical_specs"),
            default_competitors: template
                .get::<Option<String>, _>("default_competitors")
                .and_then(|s| serde_json::from_str(&s).ok()),
            default_similar_projects: template
                .get::<Option<String>, _>("default_similar_projects")
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
                    default_dependencies, default_problem_statement, default_target_audience,
                    default_value_proposition, default_ui_considerations, default_ux_principles,
                    default_tech_stack_quick, default_mvp_scope, default_research_findings,
                    default_technical_specs, default_competitors, default_similar_projects,
                    is_system, created_at
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
                let default_mvp_scope = row
                    .get::<Option<String>, _>("default_mvp_scope")
                    .and_then(|s| serde_json::from_str(&s).ok());

                let default_competitors = row
                    .get::<Option<String>, _>("default_competitors")
                    .and_then(|s| serde_json::from_str(&s).ok());

                let default_similar_projects = row
                    .get::<Option<String>, _>("default_similar_projects")
                    .and_then(|s| serde_json::from_str(&s).ok());

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
                    default_problem_statement: row.get("default_problem_statement"),
                    default_target_audience: row.get("default_target_audience"),
                    default_value_proposition: row.get("default_value_proposition"),
                    default_ui_considerations: row.get("default_ui_considerations"),
                    default_ux_principles: row.get("default_ux_principles"),
                    default_tech_stack_quick: row.get("default_tech_stack_quick"),
                    default_mvp_scope,
                    default_research_findings: row.get("default_research_findings"),
                    default_technical_specs: row.get("default_technical_specs"),
                    default_competitors,
                    default_similar_projects,
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
