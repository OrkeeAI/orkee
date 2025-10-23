// ABOUTME: Bridge between OpenSpec system and Context generation
// ABOUTME: Generates context from PRDs, tasks, and validates spec coverage

use crate::context::ast_analyzer::Symbol;
use crate::context::spec_context::SpecContextBuilder;
use crate::openspec::types::{
    SpecCapability, SpecRequirement, SpecScenario, PRD,
};
use sqlx::SqlitePool;
use std::collections::HashMap;
use tracing::info;

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
        let mut spec_builder = SpecContextBuilder::new();
        for capability in capabilities {
            // Get requirements for this capability
            let requirements = vec![]; // TODO: Fetch actual requirements from database
            let cap_context = spec_builder
                .build_capability_context(&capability, &requirements, project_root)
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
        _project_root: &str,
    ) -> Result<String, String> {
        info!("Generating context for task: {}", task_id);

        // 1. Load task details
        let task = sqlx::query!(
            "SELECT id, title, description, acceptance_criteria FROM tasks WHERE id = ?",
            task_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Task not found".to_string())?;

        // 2. TODO: Find the requirement this task implements
        // For now, return a simple context without requirement integration
        Ok(format!(
            r#"# Task Context: {}

## Description
{}

## Acceptance Criteria
{}
"#,
            task.title,
            task.description.unwrap_or_default(),
            task.acceptance_criteria.unwrap_or_default()
        ))
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
        sqlx::query_as::<_, PRD>(
            "SELECT 
                id,
                project_id,
                title,
                content_markdown,
                version,
                status,
                source,
                created_at,
                updated_at,
                created_by,
                deleted_at
            FROM prds
            WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(prd_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "PRD not found".to_string())
    }

    async fn load_capabilities_for_prd(&self, prd_id: &str) -> Result<Vec<SpecCapability>, String> {
        sqlx::query_as::<_, SpecCapability>(
            "SELECT 
                id,
                project_id,
                prd_id,
                name,
                purpose_markdown,
                spec_markdown,
                design_markdown,
                requirement_count,
                version,
                status,
                deleted_at,
                created_at,
                updated_at
            FROM spec_capabilities
            WHERE prd_id = ? AND deleted_at IS NULL
            ORDER BY created_at",
        )
        .bind(prd_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }

    async fn load_capability(&self, capability_id: &str) -> Result<SpecCapability, String> {
        sqlx::query_as::<_, SpecCapability>(
            "SELECT 
                id,
                project_id,
                prd_id,
                name,
                purpose_markdown,
                spec_markdown,
                design_markdown,
                requirement_count,
                version,
                status,
                deleted_at,
                created_at,
                updated_at
            FROM spec_capabilities
            WHERE id = ? AND deleted_at IS NULL",
        )
        .bind(capability_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Capability not found".to_string())
    }

    async fn load_requirements(&self, capability_id: &str) -> Result<Vec<SpecRequirement>, String> {
        sqlx::query_as::<_, SpecRequirement>(
            "SELECT 
                id,
                capability_id,
                name,
                content_markdown,
                position,
                created_at,
                updated_at
            FROM spec_requirements
            WHERE capability_id = ?
            ORDER BY position",
        )
        .bind(capability_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }

    #[allow(dead_code)]
    async fn load_requirement(&self, requirement_id: &str) -> Result<SpecRequirement, String> {
        sqlx::query_as::<_, SpecRequirement>(
            "SELECT 
                id,
                capability_id,
                name,
                content_markdown,
                position,
                created_at,
                updated_at
            FROM spec_requirements
            WHERE id = ?",
        )
        .bind(requirement_id)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .ok_or_else(|| "Requirement not found".to_string())
    }

    #[allow(dead_code)]
    async fn load_scenarios(&self, requirement_id: &str) -> Result<Vec<SpecScenario>, String> {
        sqlx::query_as::<_, SpecScenario>(
            "SELECT 
                id,
                requirement_id,
                name,
                when_clause,
                then_clause,
                and_clauses,
                position,
                created_at
            FROM spec_scenarios
            WHERE requirement_id = ?
            ORDER BY position",
        )
        .bind(requirement_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))
    }

    async fn find_test_files(
        &self,
        _capability: &SpecCapability,
        _project_root: &str,
    ) -> Result<Vec<String>, String> {
        // TODO: Implement test file discovery based on capability name
        // For now, return common test file patterns

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
