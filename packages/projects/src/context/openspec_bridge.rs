// ABOUTME: Bridge between OpenSpec system and Context generation
// ABOUTME: Generates context from PRDs, tasks, and validates spec coverage

use crate::context::spec_context::SpecContextBuilder;
use crate::context::types::Symbol;
use crate::openspec::types::{SpecCapability, SpecRequirement, SpecScenario, PRD};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::{error, info};

pub struct OpenSpecContextBridge {
    pool: SqlitePool,
}

impl OpenSpecContextBridge {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Generate context for a specific PRD
    pub async fn generate_prd_context(
        &self,
        prd_id: &str,
        project_root: &str,
    ) -> Result<String, String> {
        info!("Generating context for PRD: {}", prd_id);

        // 1. Load PRD from database
        let prd = self.load_prd(prd_id).await?;

        // 2. Load all associated capabilities
        let capabilities = self.load_capabilities_for_prd(prd_id).await?;

        // 3. Generate context
        let mut context = String::new();

        // PRD Header
        context.push_str(&format!(
            r#"# PRD Context: {}

## Content
{}

## Target Capabilities
"#,
            prd.title, prd.content_markdown
        ));

        // For each capability, find implementing code
        let mut spec_builder = SpecContextBuilder::new(self.pool.clone());
        for capability in capabilities {
            let cap_context = spec_builder
                .build_capability_context(&capability, project_root)
                .await
                .map_err(|e| format!("Failed to build capability context: {}", e))?;
            context.push_str(&cap_context);

            // Find test files related to this capability
            let test_files = self.find_test_files(&capability, project_root).await?;
            if !test_files.is_empty() {
                context.push_str("\n### Related Tests:\n");
                for test_file in test_files {
                    context.push_str(&format!("- {}\n", test_file));
                }
            }
        }

        Ok(context)
    }

    /// Generate context for a specific task
    pub async fn generate_task_context(
        &self,
        task_id: &str,
        project_root: &str,
    ) -> Result<String, String> {
        info!("Generating context for task: {}", task_id);

        // 1. Load task details
        let task = sqlx::query!(
            "SELECT id, title, description, requirement_id, acceptance_criteria FROM tasks WHERE id = ?",
            task_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Task not found".to_string())?;

        // 2. Find the requirement this task implements
        let requirement_id = task.requirement_id.unwrap_or_default();
        let requirement = self.load_requirement(&requirement_id).await?;

        // 3. Find existing implementations related to this requirement
        let related_code = self
            .find_requirement_implementations(&requirement.id, project_root)
            .await?;

        // 4. Build focused context
        let mut context = String::new();

        context.push_str(&format!(
            r#"# Task Context: {}

## Description
{}

## Requirement
{}

## Acceptance Criteria
{}

## Related Code
"#,
            task.title,
            task.description.unwrap_or_default(),
            requirement.content_markdown,
            task.acceptance_criteria.unwrap_or_default()
        ));

        for (file, symbols) in related_code {
            context.push_str(&format!("\n### {}\n", file));
            for symbol in symbols {
                context.push_str(&format!(
                    "- {} (lines {}-{})\n",
                    symbol.name, symbol.line_start, symbol.line_end
                ));
            }
        }

        // 5. Include WHEN/THEN scenarios
        let scenarios = self.load_scenarios(&requirement.id).await?;
        if !scenarios.is_empty() {
            context.push_str("\n## Test Scenarios\n");
            for scenario in scenarios {
                context.push_str(&format!(
                    "- WHEN {} THEN {}\n",
                    scenario.when_clause, scenario.then_clause
                ));
            }
        }

        Ok(context)
    }

    /// Validate that code matches spec requirements
    pub async fn validate_spec_coverage(
        &self,
        capability_id: &str,
        project_root: &str,
    ) -> Result<SpecValidationReport, String> {
        info!("Validating spec coverage for capability: {}", capability_id);

        let capability = self.load_capability(capability_id).await?;
        let requirements = self.load_requirements(capability_id).await?;

        let mut report = SpecValidationReport {
            capability_name: capability.name.clone(),
            total_requirements: requirements.len(),
            implemented: 0,
            partially_implemented: 0,
            not_implemented: 0,
            details: vec![],
        };

        for requirement in requirements {
            // Check if code exists for this requirement
            let implementations = self
                .find_requirement_implementations(&requirement.id, project_root)
                .await?;

            let status = if implementations.is_empty() {
                RequirementStatus::NotImplemented
            } else if self
                .validates_scenarios(&requirement, &implementations)
                .await
            {
                RequirementStatus::Implemented
            } else {
                RequirementStatus::PartiallyImplemented
            };

            match status {
                RequirementStatus::Implemented => report.implemented += 1,
                RequirementStatus::PartiallyImplemented => report.partially_implemented += 1,
                RequirementStatus::NotImplemented => report.not_implemented += 1,
            }

            report.details.push(RequirementValidation {
                requirement: requirement.content_markdown.clone(),
                status,
                code_references: implementations.keys().cloned().collect(),
            });
        }

        Ok(report)
    }

    // Helper methods

    async fn load_prd(&self, prd_id: &str) -> Result<PRD, String> {
        sqlx::query_as!(
            PRD,
            r#"
            SELECT 
                id,
                project_id,
                title,
                content_markdown,
                version,
                status as "status: PRDStatus",
                source as "source: PRDSource",
                created_at as "created_at: chrono::DateTime<chrono::Utc>",
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>",
                created_by,
                deleted_at as "deleted_at: Option<chrono::DateTime<chrono::Utc>>"
            FROM prds
            WHERE id = ? AND deleted_at IS NULL
            "#,
            prd_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "PRD not found".to_string())
    }

    async fn load_capabilities_for_prd(&self, prd_id: &str) -> Result<Vec<SpecCapability>, String> {
        sqlx::query_as!(
            SpecCapability,
            r#"
            SELECT 
                id,
                project_id,
                prd_id,
                name,
                purpose_markdown,
                spec_markdown,
                design_markdown,
                requirement_count,
                version,
                status as "status: CapabilityStatus",
                deleted_at as "deleted_at: Option<chrono::DateTime<chrono::Utc>>",
                created_at as "created_at: chrono::DateTime<chrono::Utc>",
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM spec_capabilities
            WHERE prd_id = ? AND deleted_at IS NULL
            ORDER BY created_at
            "#,
            prd_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }

    async fn load_capability(&self, capability_id: &str) -> Result<SpecCapability, String> {
        sqlx::query_as!(
            SpecCapability,
            r#"
            SELECT 
                id,
                project_id,
                prd_id,
                name,
                purpose_markdown,
                spec_markdown,
                design_markdown,
                requirement_count,
                version,
                status as "status: CapabilityStatus",
                deleted_at as "deleted_at: Option<chrono::DateTime<chrono::Utc>>",
                created_at as "created_at: chrono::DateTime<chrono::Utc>",
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM spec_capabilities
            WHERE id = ? AND deleted_at IS NULL
            "#,
            capability_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Capability not found".to_string())
    }

    async fn load_requirements(&self, capability_id: &str) -> Result<Vec<SpecRequirement>, String> {
        sqlx::query_as!(
            SpecRequirement,
            r#"
            SELECT 
                id,
                capability_id,
                name,
                content_markdown,
                position,
                created_at as "created_at: chrono::DateTime<chrono::Utc>",
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM spec_requirements
            WHERE capability_id = ?
            ORDER BY position
            "#,
            capability_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }

    async fn load_requirement(&self, requirement_id: &str) -> Result<SpecRequirement, String> {
        sqlx::query_as!(
            SpecRequirement,
            r#"
            SELECT 
                id,
                capability_id,
                name,
                content_markdown,
                position,
                created_at as "created_at: chrono::DateTime<chrono::Utc>",
                updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM spec_requirements
            WHERE id = ?
            "#,
            requirement_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Requirement not found".to_string())
    }

    async fn load_scenarios(&self, requirement_id: &str) -> Result<Vec<SpecScenario>, String> {
        sqlx::query_as!(
            SpecScenario,
            r#"
            SELECT 
                id,
                requirement_id,
                name,
                when_clause,
                then_clause,
                and_clauses as "and_clauses: Option<Vec<String>>",
                position,
                created_at as "created_at: chrono::DateTime<chrono::Utc>"
            FROM spec_scenarios
            WHERE requirement_id = ?
            ORDER BY position
            "#,
            requirement_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }

    async fn find_test_files(
        &self,
        _capability: &SpecCapability,
        project_root: &str,
    ) -> Result<Vec<String>, String> {
        // TODO: Implement test file discovery based on capability name
        // For now, return common test file patterns
        use std::path::PathBuf;
        let project_path = PathBuf::from(project_root);

        let test_patterns = vec![
            "**/*.test.ts",
            "**/*.test.js",
            "**/*.spec.ts",
            "**/*.spec.js",
            "**/test_*.py",
            "**/*_test.rs",
        ];

        // In a real implementation, we'd walk the directory and filter
        // based on the capability name being mentioned in the files
        Ok(vec![])
    }

    async fn find_requirement_implementations(
        &self,
        _requirement_id: &str,
        _project_root: &str,
    ) -> Result<HashMap<String, Vec<Symbol>>, String> {
        // TODO: Use AST analyzer to find code that implements this requirement
        // This would involve:
        // 1. Searching for keywords from requirement description
        // 2. Finding functions/classes that match the requirement
        // 3. Building a map of files to symbols
        Ok(HashMap::new())
    }

    async fn validates_scenarios(
        &self,
        _requirement: &SpecRequirement,
        _implementations: &HashMap<String, Vec<Symbol>>,
    ) -> bool {
        // TODO: Check if implementations handle WHEN/THEN scenarios
        // This would involve:
        // 1. Loading scenarios for the requirement
        // 2. Checking if test files exist that validate the scenarios
        // 3. Analyzing test coverage
        false
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct SpecValidationReport {
    pub capability_name: String,
    pub total_requirements: usize,
    pub implemented: usize,
    pub partially_implemented: usize,
    pub not_implemented: usize,
    pub details: Vec<RequirementValidation>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RequirementValidation {
    pub requirement: String,
    pub status: RequirementStatus,
    pub code_references: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RequirementStatus {
    Implemented,
    PartiallyImplemented,
    NotImplemented,
}
