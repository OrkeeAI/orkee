// ABOUTME: OpenSpec materialization service for exporting database contents to filesystem
// ABOUTME: Supports creating OpenSpec directory structure and materializing specs, changes, and project metadata

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Sqlite};
use std::path::Path;
use tokio::fs;

use crate::openspec::db as openspec_db;
use crate::openspec::db::DbError;
use crate::openspec::parser::{parse_spec_markdown, ParseError};
use crate::openspec::sync::MergeStrategy;
use crate::openspec::types::{ChangeStatus, SpecCapability, ValidationStatus};

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

    #[error("Parse error: {0}")]
    Parse(#[from] ParseError),

    #[error("Import conflict: {0}")]
    ImportConflict(String),

    #[error("Invalid OpenSpec structure: {0}")]
    InvalidStructure(String),
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
    /// ```text
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
        self.materialize_project_md(project_id, &openspec_path)
            .await?;

        // Create AGENTS.md stub
        self.create_agents_stub(&openspec_path).await?;

        // Materialize active specs
        self.materialize_specs(project_id, &openspec_path).await?;

        // Materialize active changes
        self.materialize_changes(project_id, &openspec_path, false)
            .await?;

        // Materialize archived changes
        self.materialize_changes(project_id, &openspec_path, true)
            .await?;

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
        // Query project name and description directly from the pool
        let (name, description): (String, Option<String>) =
            sqlx::query_as("SELECT name, description FROM projects WHERE id = ?")
                .bind(project_id)
                .fetch_optional(&self.pool)
                .await?
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
            name,
            description.unwrap_or_default()
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
            self.materialize_capability(&capability, openspec_path)
                .await?;
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

    /// Import OpenSpec structure from filesystem to database
    ///
    /// Reads the OpenSpec directory structure and imports specs and changes.
    /// By default, uses PreferLocal strategy for conflicts (keeps database data).
    pub async fn import_from_path(
        &self,
        project_id: &str,
        base_path: &Path,
        strategy: MergeStrategy,
    ) -> MaterializerResult<ImportReport> {
        let openspec_path = base_path.join("openspec");

        // Verify structure exists
        if !openspec_path.exists() {
            return Err(MaterializerError::InvalidStructure(
                "openspec directory not found".to_string(),
            ));
        }

        let mut report = ImportReport::default();

        // Import specs
        let specs_path = openspec_path.join("specs");
        if specs_path.exists() {
            let spec_report = self.import_specs(project_id, &specs_path, strategy).await?;
            report.capabilities_imported = spec_report.capabilities_imported;
            report.capabilities_skipped = spec_report.capabilities_skipped;
            report.requirements_imported = spec_report.requirements_imported;
        }

        // Import active changes
        let changes_path = openspec_path.join("changes");
        if changes_path.exists() {
            let change_report = self
                .import_changes(project_id, &changes_path, false)
                .await?;
            report.changes_imported += change_report;
        }

        // Import archived changes
        let archive_path = openspec_path.join("archive");
        if archive_path.exists() {
            let change_report = self.import_changes(project_id, &archive_path, true).await?;
            report.changes_imported += change_report;
        }

        Ok(report)
    }

    /// Import specs from specs/ directory
    async fn import_specs(
        &self,
        project_id: &str,
        specs_path: &Path,
        strategy: MergeStrategy,
    ) -> MaterializerResult<ImportReport> {
        let mut report = ImportReport::default();

        // Get existing capabilities
        let existing_capabilities =
            openspec_db::get_capabilities_by_project(&self.pool, project_id).await?;

        // Read all spec directories
        let mut entries = fs::read_dir(specs_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let capability_name = entry.file_name().to_string_lossy().to_string();
            let capability_dir = entry.path();

            // Read spec.md
            let spec_md_path = capability_dir.join("spec.md");
            if !spec_md_path.exists() {
                continue;
            }

            let spec_content = fs::read_to_string(&spec_md_path).await?;

            // Read design.md if present
            let design_md_path = capability_dir.join("design.md");
            let design_content = if design_md_path.exists() {
                Some(fs::read_to_string(&design_md_path).await?)
            } else {
                None
            };

            // Parse spec content
            let parsed = parse_spec_markdown(&spec_content)?;

            // Check if capability already exists
            let existing = existing_capabilities
                .iter()
                .find(|c| c.name == capability_name);

            match (existing, strategy) {
                (Some(_existing_cap), MergeStrategy::PreferLocal) => {
                    // Skip import, keep database version
                    report.capabilities_skipped += 1;
                    continue;
                }
                (Some(existing_cap), MergeStrategy::PreferRemote) => {
                    // Update existing capability
                    openspec_db::update_capability(
                        &self.pool,
                        &existing_cap.id,
                        Some(&spec_content),
                        None, // purpose_markdown (keep existing)
                        design_content.as_deref(),
                        None, // status (keep existing)
                    )
                    .await?;
                    report.capabilities_imported += 1;
                }
                (Some(_), MergeStrategy::Manual) => {
                    return Err(MaterializerError::ImportConflict(format!(
                        "Capability '{}' exists in database. Manual resolution required.",
                        capability_name
                    )));
                }
                (None, _) => {
                    // Create new capability
                    let purpose = parsed.capabilities.first().map(|c| c.purpose.clone());

                    let capability = openspec_db::create_capability(
                        &self.pool,
                        project_id,
                        None,
                        &capability_name,
                        purpose.as_deref(),
                        &spec_content,
                        design_content.as_deref(),
                    )
                    .await?;

                    // Parse and create requirements/scenarios
                    if let Some(parsed_cap) = parsed.capabilities.first() {
                        for (idx, requirement) in parsed_cap.requirements.iter().enumerate() {
                            let req = openspec_db::create_requirement(
                                &self.pool,
                                &capability.id,
                                &requirement.name,
                                &requirement.description,
                                idx as i32,
                            )
                            .await?;

                            report.requirements_imported += 1;

                            for (scenario_idx, scenario) in requirement.scenarios.iter().enumerate()
                            {
                                openspec_db::create_scenario(
                                    &self.pool,
                                    &req.id,
                                    &scenario.name,
                                    &scenario.when,
                                    &scenario.then,
                                    Some(scenario.and.clone()),
                                    scenario_idx as i32,
                                )
                                .await?;
                            }
                        }
                    }

                    report.capabilities_imported += 1;
                }
                (Some(_existing_cap), MergeStrategy::ThreeWayMerge) => {
                    // Not implemented for now, fall back to PreferLocal
                    report.capabilities_skipped += 1;
                    continue;
                }
            }
        }

        Ok(report)
    }

    /// Import changes from changes/ or archive/ directory
    async fn import_changes(
        &self,
        project_id: &str,
        changes_path: &Path,
        archived: bool,
    ) -> MaterializerResult<usize> {
        let mut imported_count = 0;

        let mut entries = fs::read_dir(changes_path).await?;
        while let Some(entry) = entries.next_entry().await? {
            if !entry.file_type().await?.is_dir() {
                continue;
            }

            let change_id = entry.file_name().to_string_lossy().to_string();
            let change_dir = entry.path();

            // Read required files
            let proposal_path = change_dir.join("proposal.md");
            let tasks_path = change_dir.join("tasks.md");

            if !proposal_path.exists() || !tasks_path.exists() {
                continue;
            }

            let proposal_markdown = fs::read_to_string(&proposal_path).await?;
            let tasks_markdown = fs::read_to_string(&tasks_path).await?;

            // Read optional design.md
            let design_path = change_dir.join("design.md");
            let design_markdown = if design_path.exists() {
                Some(fs::read_to_string(&design_path).await?)
            } else {
                None
            };

            // Check if change already exists
            let existing = openspec_db::get_spec_change(&self.pool, &change_id).await;

            if existing.is_ok() {
                // Skip if already exists
                continue;
            }

            // Create change
            let status = if archived {
                ChangeStatus::Archived
            } else {
                ChangeStatus::Draft
            };

            sqlx::query(
                r#"
                INSERT INTO spec_changes
                (id, project_id, proposal_markdown, tasks_markdown, design_markdown, status, validation_status, created_by, created_at, updated_at, archived_at)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(&change_id)
            .bind(project_id)
            .bind(&proposal_markdown)
            .bind(&tasks_markdown)
            .bind(design_markdown.as_deref())
            .bind(status)
            .bind(ValidationStatus::Pending)
            .bind("imported")
            .bind(Utc::now())
            .bind(Utc::now())
            .bind(if archived { Some(Utc::now()) } else { None })
            .execute(&self.pool)
            .await?;

            imported_count += 1;
        }

        Ok(imported_count)
    }

    /// Initialize OpenSpec structure in a sandbox environment
    ///
    /// This is a convenience method for setting up OpenSpec in a temporary
    /// workspace (like for AI agents or build environments).
    /// It materializes the full structure without tracking the materialization.
    pub async fn initialize_sandbox(
        &self,
        project_id: &str,
        sandbox_path: &Path,
    ) -> MaterializerResult<()> {
        // Create openspec directory
        let openspec_path = sandbox_path.join("openspec");
        fs::create_dir_all(&openspec_path).await?;
        fs::create_dir_all(openspec_path.join("specs")).await?;
        fs::create_dir_all(openspec_path.join("changes")).await?;
        fs::create_dir_all(openspec_path.join("archive")).await?;

        // Create project.md
        self.materialize_project_md(project_id, &openspec_path)
            .await?;

        // Create AGENTS.md
        self.create_agents_stub(&openspec_path).await?;

        // Materialize specs
        self.materialize_specs(project_id, &openspec_path).await?;

        // Materialize active changes only (not archived)
        self.materialize_changes(project_id, &openspec_path, false)
            .await?;

        // Don't track materialization for sandboxes

        Ok(())
    }

    /// Clean up OpenSpec structure from a path
    ///
    /// Removes the openspec directory and all its contents.
    /// Useful for cleaning up after sandbox operations.
    pub async fn cleanup_sandbox(sandbox_path: &Path) -> MaterializerResult<()> {
        let openspec_path = sandbox_path.join("openspec");

        if openspec_path.exists() {
            fs::remove_dir_all(&openspec_path).await?;
        }

        Ok(())
    }
}

/// Report of import operation
#[derive(Debug, Clone, Default)]
pub struct ImportReport {
    pub capabilities_imported: usize,
    pub capabilities_skipped: usize,
    pub requirements_imported: usize,
    pub changes_imported: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::generate_project_id;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_materializer_creation() {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();
        let _materializer = OpenSpecMaterializer::new(pool);
        assert!(true, "Materializer created successfully");
    }

    #[tokio::test]
    async fn test_export_import_roundtrip() {
        // Create in-memory database
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Create test project
        let project_id = generate_project_id();
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&project_id)
        .bind("Test Project")
        .bind("/test/path")
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&pool)
        .await
        .unwrap();

        // Create test capability
        let spec_markdown = r#"## Test Capability

Test purpose.

### Test Requirement

Test description.

#### Scenario: Test scenario
- **WHEN** something happens
- **THEN** something results
- **AND** additional result
"#;

        let capability = openspec_db::create_capability(
            &pool,
            &project_id,
            None,
            "test-capability",
            Some("Test purpose"),
            spec_markdown,
            None,
        )
        .await
        .unwrap();

        // Create temporary directory for export
        let temp_dir = TempDir::new().unwrap();
        let export_path = temp_dir.path();

        // Export
        let materializer = OpenSpecMaterializer::new(pool.clone());
        materializer
            .materialize_to_path(&project_id, export_path)
            .await
            .unwrap();

        // Verify exported files exist
        assert!(export_path.join("openspec").exists());
        assert!(export_path.join("openspec/project.md").exists());
        assert!(export_path.join("openspec/AGENTS.md").exists());
        assert!(export_path.join("openspec/specs").exists());
        assert!(export_path
            .join("openspec/specs/test-capability/spec.md")
            .exists());

        // Delete the capability from database
        sqlx::query("DELETE FROM spec_capabilities WHERE id = ?")
            .bind(&capability.id)
            .execute(&pool)
            .await
            .unwrap();

        // Verify it's gone
        let caps = openspec_db::get_capabilities_by_project(&pool, &project_id)
            .await
            .unwrap();
        assert_eq!(caps.len(), 0);

        // Import back
        let report = materializer
            .import_from_path(&project_id, export_path, MergeStrategy::PreferRemote)
            .await
            .unwrap();

        // Verify import report
        assert_eq!(report.capabilities_imported, 1);
        assert_eq!(report.capabilities_skipped, 0);

        // Verify capability is back in database
        let caps = openspec_db::get_capabilities_by_project(&pool, &project_id)
            .await
            .unwrap();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].name, "test-capability");
    }

    #[tokio::test]
    async fn test_import_conflict_prefer_local() {
        // Create in-memory database
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Create test project
        let project_id = generate_project_id();
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&project_id)
        .bind("Test Project")
        .bind("/test/path")
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&pool)
        .await
        .unwrap();

        // Create test capability in database
        openspec_db::create_capability(
            &pool,
            &project_id,
            None,
            "test-capability",
            Some("Database version"),
            "Database content",
            None,
        )
        .await
        .unwrap();

        // Create temporary directory with different content
        let temp_dir = TempDir::new().unwrap();
        let import_path = temp_dir.path();
        let openspec_path = import_path.join("openspec");
        let spec_path = openspec_path.join("specs/test-capability");

        fs::create_dir_all(&spec_path).await.unwrap();
        fs::write(
            spec_path.join("spec.md"),
            "## Test Capability\n\nFile system version",
        )
        .await
        .unwrap();

        // Import with PreferLocal strategy
        let materializer = OpenSpecMaterializer::new(pool.clone());
        let report = materializer
            .import_from_path(&project_id, import_path, MergeStrategy::PreferLocal)
            .await
            .unwrap();

        // Should skip import
        assert_eq!(report.capabilities_skipped, 1);
        assert_eq!(report.capabilities_imported, 0);

        // Verify database version is unchanged
        let caps = openspec_db::get_capabilities_by_project(&pool, &project_id)
            .await
            .unwrap();
        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0].spec_markdown, "Database content");
    }

    #[tokio::test]
    async fn test_sandbox_initialization() {
        // Create in-memory database
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        // Create test project
        let project_id = generate_project_id();
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(&project_id)
        .bind("Test Project")
        .bind("/test/path")
        .bind(Utc::now())
        .bind(Utc::now())
        .execute(&pool)
        .await
        .unwrap();

        // Create test capability
        openspec_db::create_capability(
            &pool,
            &project_id,
            None,
            "sandbox-test",
            Some("Sandbox test"),
            "## Sandbox Test\n\nTest capability",
            None,
        )
        .await
        .unwrap();

        // Create temporary sandbox directory
        let temp_dir = TempDir::new().unwrap();
        let sandbox_path = temp_dir.path();

        // Initialize sandbox
        let materializer = OpenSpecMaterializer::new(pool.clone());
        materializer
            .initialize_sandbox(&project_id, sandbox_path)
            .await
            .unwrap();

        // Verify structure created
        assert!(sandbox_path.join("openspec").exists());
        assert!(sandbox_path.join("openspec/project.md").exists());
        assert!(sandbox_path.join("openspec/AGENTS.md").exists());
        assert!(sandbox_path
            .join("openspec/specs/sandbox-test/spec.md")
            .exists());

        // Clean up sandbox
        OpenSpecMaterializer::cleanup_sandbox(sandbox_path)
            .await
            .unwrap();

        // Verify cleanup
        assert!(!sandbox_path.join("openspec").exists());
    }
}
