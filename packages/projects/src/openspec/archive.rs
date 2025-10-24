// ABOUTME: OpenSpec archive workflow for completed changes
// ABOUTME: Validates and applies deltas to create or update capabilities

use super::db::{
    create_capability, create_requirement, create_scenario, get_capability,
    get_deltas_by_change, get_spec_change, DbError,
};
use super::markdown_validator::OpenSpecMarkdownValidator;
use super::parser::{parse_spec_markdown, ParseError};
use super::types::{ChangeStatus, DeltaType, ParsedCapability, SpecChange, SpecDelta};
use chrono::Utc;
use sqlx::{Pool, Sqlite};

#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("Change already archived: {0}")]
    AlreadyArchived(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] DbError),

    #[error("Parse error: {0}")]
    ParseError(#[from] ParseError),

    #[error("SQL error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Invalid delta: {0}")]
    InvalidDelta(String),
}

pub type ArchiveResult<T> = Result<T, ArchiveError>;

/// Archive a completed change and optionally apply its deltas
pub async fn archive_change(
    pool: &Pool<Sqlite>,
    change_id: &str,
    apply_specs: bool,
) -> ArchiveResult<()> {
    // Get change
    let change = get_spec_change(pool, change_id).await?;

    if change.status == ChangeStatus::Archived {
        return Err(ArchiveError::AlreadyArchived(change_id.to_string()));
    }

    // Get deltas for validation
    let deltas = get_deltas_by_change(pool, change_id).await?;

    // Validate all deltas before archiving
    let validator = OpenSpecMarkdownValidator::new(true); // strict mode
    let mut all_errors = Vec::new();

    for delta in &deltas {
        let errors = validator.validate_delta_markdown(&delta.delta_markdown);
        if !errors.is_empty() {
            all_errors.extend(
                errors
                    .into_iter()
                    .map(|e| format!("Delta '{}': {}", delta.capability_name, e.message)),
            );
        }
    }

    if !all_errors.is_empty() {
        return Err(ArchiveError::ValidationFailed(all_errors.join("; ")));
    }

    // Apply deltas if requested
    if apply_specs {
        for delta in deltas {
            apply_delta(pool, &change, &delta).await?;
        }
    }

    // Update change status to archived
    sqlx::query(
        "UPDATE spec_changes
         SET status = 'archived', archived_at = ?
         WHERE id = ?",
    )
    .bind(Utc::now())
    .bind(change_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// Apply a delta to create or update a capability
async fn apply_delta(
    pool: &Pool<Sqlite>,
    change: &SpecChange,
    delta: &SpecDelta,
) -> ArchiveResult<()> {
    match delta.delta_type {
        DeltaType::Added => {
            // Parse delta markdown to extract requirements
            let parsed = parse_capability_from_delta(&delta.delta_markdown)?;

            // Create new capability
            let capability = create_capability(
                pool,
                &change.project_id,
                change.prd_id.as_deref(),
                &delta.capability_name,
                Some(&parsed.purpose),
                &delta.delta_markdown,
                None,
            )
            .await?;

            // Mark as OpenSpec compliant and associate with change
            sqlx::query(
                "UPDATE spec_capabilities
                 SET is_openspec_compliant = TRUE, change_id = ?
                 WHERE id = ?",
            )
            .bind(&change.id)
            .bind(&capability.id)
            .execute(pool)
            .await?;

            // Create requirements and scenarios
            for (req_idx, req) in parsed.requirements.iter().enumerate() {
                let req_db = create_requirement(
                    pool,
                    &capability.id,
                    &req.name,
                    &req.description,
                    req_idx as i32,
                )
                .await?;

                for (scenario_idx, scenario) in req.scenarios.iter().enumerate() {
                    create_scenario(
                        pool,
                        &req_db.id,
                        &scenario.name,
                        &scenario.when,
                        &scenario.then,
                        if scenario.and.is_empty() {
                            None
                        } else {
                            Some(scenario.and.clone())
                        },
                        scenario_idx as i32,
                    )
                    .await?;
                }
            }

            // Update requirement count
            let req_count = parsed.requirements.len() as i32;
            sqlx::query(
                "UPDATE spec_capabilities SET requirement_count = ? WHERE id = ?",
            )
            .bind(req_count)
            .bind(&capability.id)
            .execute(pool)
            .await?;
        }

        DeltaType::Modified => {
            // Update existing capability
            if let Some(cap_id) = &delta.capability_id {
                // Verify capability exists
                let _existing = get_capability(pool, cap_id).await?;

                // Update the spec markdown
                sqlx::query(
                    "UPDATE spec_capabilities
                     SET spec_markdown = ?, updated_at = ?, version = version + 1, change_id = ?
                     WHERE id = ?",
                )
                .bind(&delta.delta_markdown)
                .bind(Utc::now())
                .bind(&change.id)
                .bind(cap_id)
                .execute(pool)
                .await?;

                // Parse to get updated requirements
                let parsed = parse_capability_from_delta(&delta.delta_markdown)?;

                // Delete existing requirements and scenarios (cascading delete via FK)
                sqlx::query("DELETE FROM spec_requirements WHERE capability_id = ?")
                    .bind(cap_id)
                    .execute(pool)
                    .await?;

                // Create new requirements and scenarios
                for (req_idx, req) in parsed.requirements.iter().enumerate() {
                    let req_db = create_requirement(
                        pool,
                        cap_id,
                        &req.name,
                        &req.description,
                        req_idx as i32,
                    )
                    .await?;

                    for (scenario_idx, scenario) in req.scenarios.iter().enumerate() {
                        create_scenario(
                            pool,
                            &req_db.id,
                            &scenario.name,
                            &scenario.when,
                            &scenario.then,
                            if scenario.and.is_empty() {
                                None
                            } else {
                                Some(scenario.and.clone())
                            },
                            scenario_idx as i32,
                        )
                        .await?;
                    }
                }

                // Update requirement count
                let req_count = parsed.requirements.len() as i32;
                sqlx::query(
                    "UPDATE spec_capabilities SET requirement_count = ? WHERE id = ?",
                )
                .bind(req_count)
                .bind(cap_id)
                .execute(pool)
                .await?;
            } else {
                return Err(ArchiveError::InvalidDelta(format!(
                    "Modified delta for '{}' missing capability_id",
                    delta.capability_name
                )));
            }
        }

        DeltaType::Removed => {
            // Mark capability as deprecated
            if let Some(cap_id) = &delta.capability_id {
                sqlx::query(
                    "UPDATE spec_capabilities
                     SET status = 'deprecated', deleted_at = ?, change_id = ?
                     WHERE id = ?",
                )
                .bind(Utc::now())
                .bind(&change.id)
                .bind(cap_id)
                .execute(pool)
                .await?;
            } else {
                return Err(ArchiveError::InvalidDelta(format!(
                    "Removed delta for '{}' missing capability_id",
                    delta.capability_name
                )));
            }
        }
    }

    Ok(())
}

/// Parse a capability from delta markdown
/// Expects format like:
/// ## ADDED Requirements
/// ### Requirement: Name
/// Content
/// #### Scenario: Name
/// - **WHEN** condition
/// - **THEN** outcome
fn parse_capability_from_delta(delta_markdown: &str) -> ArchiveResult<ParsedCapability> {
    // Remove the delta operation header (## ADDED/MODIFIED/REMOVED Requirements)
    let markdown_without_header = delta_markdown
        .lines()
        .skip_while(|line| {
            line.starts_with("## ADDED")
                || line.starts_with("## MODIFIED")
                || line.starts_with("## REMOVED")
                || line.starts_with("## RENAMED")
        })
        .collect::<Vec<_>>()
        .join("\n");

    // Wrap in a capability section for the parser
    let wrapped = format!("## Capability\n\n{}", markdown_without_header);

    // Parse the spec
    let parsed_spec = parse_spec_markdown(&wrapped)?;

    if parsed_spec.capabilities.is_empty() {
        return Err(ArchiveError::ParseError(ParseError::MissingSection(
            "No capability found in delta".to_string(),
        )));
    }

    // Return the first (and should be only) capability
    Ok(parsed_spec.capabilities[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::openspec::db::{create_prd, create_spec_change, create_spec_delta, update_spec_change_status};
    use crate::openspec::types::{PRDSource, PRDStatus};

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::query(include_str!("../../migrations/001_initial_schema.sql"))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250117000000_task_management.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!("../../migrations/20250118000000_openspec.sql"))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250127000000_openspec_alignment.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        // Create test project
        sqlx::query(
            "INSERT INTO projects (id, name, project_root, description, created_at, updated_at)
             VALUES ('test-project', 'Test Project', '/tmp/test', 'Test', datetime('now'), datetime('now'))",
        )
        .execute(&pool)
        .await
        .unwrap();

        pool
    }

    #[tokio::test]
    async fn test_parse_capability_from_delta() {
        let delta_markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication using JWT tokens.

#### Scenario: Successful login
**WHEN** valid credentials are provided
**THEN** a JWT token is returned
**AND** the token expires after 24 hours
"#;

        let parsed = parse_capability_from_delta(delta_markdown).unwrap();
        assert_eq!(parsed.requirements.len(), 1);
        assert_eq!(parsed.requirements[0].name, "Requirement: User Authentication");
        assert_eq!(parsed.requirements[0].scenarios.len(), 1);
        assert_eq!(parsed.requirements[0].scenarios[0].name, "Scenario: Successful login");
    }

    #[tokio::test]
    async fn test_archive_change_already_archived() {
        let pool = setup_test_db().await;

        // Create PRD
        let prd = create_prd(
            &pool,
            "test-project",
            "Test PRD",
            "Content",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        // Create change
        let change = create_spec_change(
            &pool,
            "test-project",
            Some(&prd.id),
            "## Proposal\nTest",
            "## Tasks\nTask 1",
            None,
            "test-user",
        )
        .await
        .unwrap();

        // Archive it once
        update_spec_change_status(&pool, &change.id, ChangeStatus::Archived, None)
            .await
            .unwrap();

        // Try to archive again
        let result = archive_change(&pool, &change.id, false).await;
        assert!(matches!(result, Err(ArchiveError::AlreadyArchived(_))));
    }

    #[tokio::test]
    async fn test_archive_with_valid_delta() {
        let pool = setup_test_db().await;

        // Create PRD
        let prd = create_prd(
            &pool,
            "test-project",
            "Test PRD",
            "Content",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        // Create change
        let change = create_spec_change(
            &pool,
            "test-project",
            Some(&prd.id),
            "## Proposal\nTest",
            "## Tasks\nTask 1",
            None,
            "test-user",
        )
        .await
        .unwrap();

        // Create a valid delta
        let delta_markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        create_spec_delta(
            &pool,
            &change.id,
            None,
            "user-auth",
            DeltaType::Added,
            delta_markdown,
            0,
        )
        .await
        .unwrap();

        // Archive with apply_specs = true
        let result = archive_change(&pool, &change.id, true).await;
        assert!(result.is_ok());

        // Verify change is archived
        let updated_change = get_spec_change(&pool, &change.id).await.unwrap();
        assert_eq!(updated_change.status, ChangeStatus::Archived);

        // Verify capability was created
        let caps: Vec<String> = sqlx::query_scalar(
            "SELECT name FROM spec_capabilities WHERE project_id = 'test-project'",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(caps.len(), 1);
        assert_eq!(caps[0], "user-auth");
    }
}
