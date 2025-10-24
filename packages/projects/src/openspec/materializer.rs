// ABOUTME: OpenSpec materialization service for exporting database contents to filesystem
// ABOUTME: Supports creating OpenSpec directory structure and materializing specs, changes, and project metadata

use std::path::Path;
use sqlx::{Pool, Sqlite};
use tokio::fs;
use sha2::{Sha256, Digest};
use chrono::Utc;

use crate::openspec::db as openspec_db;
use crate::openspec::types::SpecCapability;
use crate::openspec::db::DbError;
use crate::manager::get_project;

/// Errors that can occur during materialization
#[derive(Debug, thiserror::Error)]
pub enum MaterializerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] DbError),

    #[error("SQLx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),
}

pub type MaterializerResult<T> = Result<T, MaterializerError>;

/// Service for materializing OpenSpec data to filesystem
pub struct OpenSpecMaterializer {
    pool: Pool<Sqlite>,
}

impl OpenSpecMaterializer {
    /// Create a new materializer with the given database pool
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    /// Materialize OpenSpec structure for a project to disk
    ///
    /// Creates the following structure:
    /// ```
    /// openspec/
    ///   ├── project.md
    ///   ├── AGENTS.md
    ///   ├── specs/
    ///   │   └── [capability]/
    ///   │       ├── spec.md
    ///   │       └── design.md (optional)
    ///   ├── changes/
    ///   │   └── [change-id]/
    ///   │       ├── proposal.md
    ///   │       ├── tasks.md
    ///   │       └── design.md (optional)
    ///   └── archive/
    ///       └── [archived-change-id]/
    /// ```
    pub async fn materialize_to_path(
        &self,
        project_id: &str,
        base_path: &Path,
    ) -> MaterializerResult<()> {
        // Create base directory structure
        let openspec_path = base_path.join("openspec");
        fs::create_dir_all(&openspec_path).await?;
        fs::create_dir_all(openspec_path.join("specs")).await?;
        fs::create_dir_all(openspec_path.join("changes")).await?;
        fs::create_dir_all(openspec_path.join("archive")).await?;

        // Create project.md from project metadata
        self.materialize_project_md(project_id, &openspec_path).await?;

        // Create AGENTS.md stub
        self.create_agents_stub(&openspec_path).await?;

        // Materialize active specs
        self.materialize_specs(project_id, &openspec_path).await?;

        // Materialize active changes
        self.materialize_changes(project_id, &openspec_path, false).await?;

        // Materialize archived changes
        self.materialize_changes(project_id, &openspec_path, true).await?;

        // Track materialization
        self.track_materialization(project_id, base_path).await?;

        Ok(())
    }

    /// Create project.md with project context
    async fn materialize_project_md(
        &self,
        project_id: &str,
        openspec_path: &Path,
    ) -> MaterializerResult<()> {
        let project = get_project(project_id)
            .await
            .map_err(|e| MaterializerError::Database(DbError::InvalidInput(format!("Failed to get project: {}", e))))?
            .ok_or_else(|| MaterializerError::ProjectNotFound(project_id.to_string()))?;

        let content = format!(
            r#"# {} Context

## Purpose
{}

## Tech Stack
- TypeScript, React, Rust
- SQLite, Orkee

## Project Conventions
[Project-specific conventions]

## Domain Context
[Domain knowledge]
"#,
            project.name,
            project.description.unwrap_or_default()
        );

        fs::write(openspec_path.join("project.md"), content).await?;
        Ok(())
    }

    /// Create AGENTS.md stub pointing to OpenSpec documentation
    async fn create_agents_stub(&self, openspec_path: &Path) -> MaterializerResult<()> {
        let content = r#"# OpenSpec Instructions

See https://github.com/Fission-AI/OpenSpec/blob/main/openspec/AGENTS.md for full instructions.

This project uses OpenSpec for spec-driven development.
"#;
        fs::write(openspec_path.join("AGENTS.md"), content).await?;
        Ok(())
    }

    /// Materialize all active specs to specs/ directory
    async fn materialize_specs(
        &self,
        project_id: &str,
        openspec_path: &Path,
    ) -> MaterializerResult<()> {
        let capabilities = openspec_db::get_capabilities_by_project(&self.pool, project_id).await?;

        for capability in capabilities {
            self.materialize_capability(&capability, openspec_path).await?;
        }

        Ok(())
    }

    /// Materialize a single capability
    async fn materialize_capability(
        &self,
        capability: &SpecCapability,
        openspec_path: &Path,
    ) -> MaterializerResult<()> {
        let cap_dir = openspec_path.join("specs").join(&capability.name);
        fs::create_dir_all(&cap_dir).await?;

        // Write spec.md
        fs::write(cap_dir.join("spec.md"), &capability.spec_markdown).await?;

        // Write design.md if present
        if let Some(design) = &capability.design_markdown {
            fs::write(cap_dir.join("design.md"), design).await?;
        }

        Ok(())
    }

    /// Materialize changes (active or archived)
    async fn materialize_changes(
        &self,
        project_id: &str,
        openspec_path: &Path,
        archived: bool,
    ) -> MaterializerResult<()> {
        let changes = openspec_db::get_spec_changes_by_project(&self.pool, project_id).await?;

        let target_dir = if archived {
            openspec_path.join("archive")
        } else {
            openspec_path.join("changes")
        };

        for change in changes {
            let is_archived = change.status == crate::openspec::types::ChangeStatus::Archived;
            if is_archived != archived {
                continue;
            }

            let change_dir = target_dir.join(&change.id);
            fs::create_dir_all(&change_dir).await?;

            // Write proposal.md
            fs::write(change_dir.join("proposal.md"), &change.proposal_markdown).await?;

            // Write tasks.md
            fs::write(change_dir.join("tasks.md"), &change.tasks_markdown).await?;

            // Write design.md if present
            if let Some(design) = &change.design_markdown {
                fs::write(change_dir.join("design.md"), design).await?;
            }
        }

        Ok(())
    }

    /// Track materialization in database
    async fn track_materialization(
        &self,
        project_id: &str,
        base_path: &Path,
    ) -> MaterializerResult<()> {
        let path_str = base_path.to_string_lossy().to_string();

        // Calculate hash of materialized structure (simplified - just use timestamp for now)
        let mut hasher = Sha256::new();
        hasher.update(format!("{}-{}", project_id, Utc::now().timestamp()).as_bytes());
        let hash = format!("{:x}", hasher.finalize());

        let id = crate::storage::generate_project_id();
        let now = Utc::now();

        sqlx::query(
            r#"
            INSERT INTO spec_materializations
            (id, project_id, path, materialized_at, sha256_hash)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&id)
        .bind(project_id)
        .bind(&path_str)
        .bind(now)
        .bind(&hash)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_materializer_creation() {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();
        let materializer = OpenSpecMaterializer::new(pool);
        assert!(true, "Materializer created successfully");
    }
}
