// ABOUTME: Database operations for OpenSpec entities
// ABOUTME: Provides CRUD operations for PRDs, capabilities, requirements, scenarios, and changes

use super::types::*;
use chrono::Utc;
use sqlx::{Pool, Sqlite};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("Entity not found: {0}")]
    NotFound(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

pub type DbResult<T> = Result<T, DbError>;

// ============================================================================
// PRD Operations
// ============================================================================

/// Create a new PRD
pub async fn create_prd(
    pool: &Pool<Sqlite>,
    project_id: &str,
    title: &str,
    content_markdown: &str,
    status: PRDStatus,
    source: PRDSource,
    created_by: Option<&str>,
) -> DbResult<PRD> {
    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let prd = sqlx::query_as::<_, PRD>(
        r#"
        INSERT INTO prds (id, project_id, title, content_markdown, version, status, source, created_at, updated_at, created_by)
        VALUES (?, ?, ?, ?, 1, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(project_id)
    .bind(title)
    .bind(content_markdown)
    .bind(&status)
    .bind(&source)
    .bind(now)
    .bind(now)
    .bind(created_by)
    .fetch_one(pool)
    .await?;

    Ok(prd)
}

/// Get a PRD by ID
pub async fn get_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<PRD> {
    sqlx::query_as::<_, PRD>("SELECT * FROM prds WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("PRD not found: {}", id)))
}

/// Get all PRDs for a project
pub async fn get_prds_by_project(pool: &Pool<Sqlite>, project_id: &str) -> DbResult<Vec<PRD>> {
    Ok(
        sqlx::query_as::<_, PRD>(
            "SELECT * FROM prds WHERE project_id = ? ORDER BY created_at DESC",
        )
        .bind(project_id)
        .fetch_all(pool)
        .await?,
    )
}

/// Update a PRD
pub async fn update_prd(
    pool: &Pool<Sqlite>,
    id: &str,
    title: Option<&str>,
    content_markdown: Option<&str>,
    status: Option<PRDStatus>,
) -> DbResult<PRD> {
    // Get current PRD
    let current = get_prd(pool, id).await?;

    let new_title = title.unwrap_or(&current.title);
    let new_content = content_markdown.unwrap_or(&current.content_markdown);
    let new_status = status.unwrap_or(current.status);
    let now = Utc::now();

    let prd = sqlx::query_as::<_, PRD>(
        r#"
        UPDATE prds
        SET title = ?, content_markdown = ?, status = ?, updated_at = ?, version = version + 1
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(new_title)
    .bind(new_content)
    .bind(&new_status)
    .bind(now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(prd)
}

/// Delete a PRD
pub async fn delete_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM prds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("PRD not found: {}", id)));
    }

    Ok(())
}

// ============================================================================
// SpecCapability Operations
// ============================================================================

/// Create a new capability
pub async fn create_capability(
    pool: &Pool<Sqlite>,
    project_id: &str,
    prd_id: Option<&str>,
    name: &str,
    purpose_markdown: Option<&str>,
    spec_markdown: &str,
    design_markdown: Option<&str>,
) -> DbResult<SpecCapability> {
    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let capability = sqlx::query_as::<_, SpecCapability>(
        r#"
        INSERT INTO spec_capabilities
        (id, project_id, prd_id, name, purpose_markdown, spec_markdown, design_markdown,
         requirement_count, version, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, 1, 'active', ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(project_id)
    .bind(prd_id)
    .bind(name)
    .bind(purpose_markdown)
    .bind(spec_markdown)
    .bind(design_markdown)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(capability)
}

/// Get a capability by ID
pub async fn get_capability(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecCapability> {
    sqlx::query_as::<_, SpecCapability>("SELECT * FROM spec_capabilities WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("Capability not found: {}", id)))
}

/// Get all capabilities for a project
pub async fn get_capabilities_by_project(
    pool: &Pool<Sqlite>,
    project_id: &str,
) -> DbResult<Vec<SpecCapability>> {
    Ok(sqlx::query_as::<_, SpecCapability>(
        "SELECT * FROM spec_capabilities WHERE project_id = ? AND status = 'active' ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?)
}

/// Get capabilities by PRD
pub async fn get_capabilities_by_prd(
    pool: &Pool<Sqlite>,
    prd_id: &str,
) -> DbResult<Vec<SpecCapability>> {
    Ok(sqlx::query_as::<_, SpecCapability>(
        "SELECT * FROM spec_capabilities WHERE prd_id = ? AND status = 'active' ORDER BY created_at DESC",
    )
    .bind(prd_id)
    .fetch_all(pool)
    .await?)
}

/// Update capability
pub async fn update_capability(
    pool: &Pool<Sqlite>,
    id: &str,
    spec_markdown: Option<&str>,
    purpose_markdown: Option<&str>,
    design_markdown: Option<&str>,
    status: Option<CapabilityStatus>,
) -> DbResult<SpecCapability> {
    let current = get_capability(pool, id).await?;

    let new_spec = spec_markdown.unwrap_or(&current.spec_markdown);
    let new_purpose = purpose_markdown.or(current.purpose_markdown.as_deref());
    let new_design = design_markdown.or(current.design_markdown.as_deref());
    let new_status = status.unwrap_or(current.status);
    let now = Utc::now();

    let capability = sqlx::query_as::<_, SpecCapability>(
        r#"
        UPDATE spec_capabilities
        SET spec_markdown = ?, purpose_markdown = ?, design_markdown = ?,
            status = ?, updated_at = ?, version = version + 1
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(new_spec)
    .bind(new_purpose)
    .bind(new_design)
    .bind(&new_status)
    .bind(now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(capability)
}

/// Update capability requirement count
pub async fn update_capability_requirement_count(
    pool: &Pool<Sqlite>,
    capability_id: &str,
    count: i32,
) -> DbResult<()> {
    sqlx::query("UPDATE spec_capabilities SET requirement_count = ? WHERE id = ?")
        .bind(count)
        .bind(capability_id)
        .execute(pool)
        .await?;

    Ok(())
}

// ============================================================================
// SpecRequirement Operations
// ============================================================================

/// Create a new requirement
pub async fn create_requirement(
    pool: &Pool<Sqlite>,
    capability_id: &str,
    name: &str,
    content_markdown: &str,
    position: i32,
) -> DbResult<SpecRequirement> {
    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let requirement = sqlx::query_as::<_, SpecRequirement>(
        r#"
        INSERT INTO spec_requirements (id, capability_id, name, content_markdown, position, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(capability_id)
    .bind(name)
    .bind(content_markdown)
    .bind(position)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    // Update capability requirement count
    let count = get_requirements_count(pool, capability_id).await?;
    update_capability_requirement_count(pool, capability_id, count).await?;

    Ok(requirement)
}

/// Get requirement by ID
pub async fn get_requirement(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecRequirement> {
    sqlx::query_as::<_, SpecRequirement>("SELECT * FROM spec_requirements WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("Requirement not found: {}", id)))
}

/// Get all requirements for a capability
pub async fn get_requirements_by_capability(
    pool: &Pool<Sqlite>,
    capability_id: &str,
) -> DbResult<Vec<SpecRequirement>> {
    Ok(sqlx::query_as::<_, SpecRequirement>(
        "SELECT * FROM spec_requirements WHERE capability_id = ? ORDER BY position",
    )
    .bind(capability_id)
    .fetch_all(pool)
    .await?)
}

/// Get requirement count for a capability
async fn get_requirements_count(pool: &Pool<Sqlite>, capability_id: &str) -> DbResult<i32> {
    let count: (i32,) =
        sqlx::query_as("SELECT COUNT(*) FROM spec_requirements WHERE capability_id = ?")
            .bind(capability_id)
            .fetch_one(pool)
            .await?;

    Ok(count.0)
}

/// Update requirement
pub async fn update_requirement(
    pool: &Pool<Sqlite>,
    id: &str,
    content_markdown: Option<&str>,
    position: Option<i32>,
) -> DbResult<SpecRequirement> {
    let current = get_requirement(pool, id).await?;

    let new_content = content_markdown.unwrap_or(&current.content_markdown);
    let new_position = position.unwrap_or(current.position);
    let now = Utc::now();

    let requirement = sqlx::query_as::<_, SpecRequirement>(
        r#"
        UPDATE spec_requirements
        SET content_markdown = ?, position = ?, updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(new_content)
    .bind(new_position)
    .bind(now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(requirement)
}

/// Delete requirement
pub async fn delete_requirement(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let requirement = get_requirement(pool, id).await?;
    let capability_id = requirement.capability_id;

    let result = sqlx::query("DELETE FROM spec_requirements WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("Requirement not found: {}", id)));
    }

    // Update capability requirement count
    let count = get_requirements_count(pool, &capability_id).await?;
    update_capability_requirement_count(pool, &capability_id, count).await?;

    Ok(())
}

// ============================================================================
// SpecScenario Operations
// ============================================================================

/// Create a new scenario
pub async fn create_scenario(
    pool: &Pool<Sqlite>,
    requirement_id: &str,
    name: &str,
    when_clause: &str,
    then_clause: &str,
    and_clauses: Option<Vec<String>>,
    position: i32,
) -> DbResult<SpecScenario> {
    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let scenario = sqlx::query_as::<_, SpecScenario>(
        r#"
        INSERT INTO spec_scenarios (id, requirement_id, name, when_clause, then_clause, and_clauses, position, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(requirement_id)
    .bind(name)
    .bind(when_clause)
    .bind(then_clause)
    .bind(and_clauses.as_ref().and_then(|v| serde_json::to_string(v).ok()))
    .bind(position)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(scenario)
}

/// Get scenarios by requirement
pub async fn get_scenarios_by_requirement(
    pool: &Pool<Sqlite>,
    requirement_id: &str,
) -> DbResult<Vec<SpecScenario>> {
    Ok(sqlx::query_as::<_, SpecScenario>(
        "SELECT * FROM spec_scenarios WHERE requirement_id = ? ORDER BY position",
    )
    .bind(requirement_id)
    .fetch_all(pool)
    .await?)
}

/// Delete scenario
pub async fn delete_scenario(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM spec_scenarios WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("Scenario not found: {}", id)));
    }

    Ok(())
}

// ============================================================================
// SpecChange Operations
// ============================================================================

/// Create a spec change
pub async fn create_spec_change(
    pool: &Pool<Sqlite>,
    project_id: &str,
    prd_id: Option<&str>,
    proposal_markdown: &str,
    tasks_markdown: &str,
    design_markdown: Option<&str>,
    created_by: &str,
) -> DbResult<SpecChange> {
    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let change = sqlx::query_as::<_, SpecChange>(
        r#"
        INSERT INTO spec_changes
        (id, project_id, prd_id, proposal_markdown, tasks_markdown, design_markdown,
         status, created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, 'draft', ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(project_id)
    .bind(prd_id)
    .bind(proposal_markdown)
    .bind(tasks_markdown)
    .bind(design_markdown)
    .bind(created_by)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(change)
}

/// Get spec change by ID
pub async fn get_spec_change(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecChange> {
    sqlx::query_as::<_, SpecChange>("SELECT * FROM spec_changes WHERE id = ?")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("Spec change not found: {}", id)))
}

/// Get spec changes by project
pub async fn get_spec_changes_by_project(
    pool: &Pool<Sqlite>,
    project_id: &str,
) -> DbResult<Vec<SpecChange>> {
    Ok(sqlx::query_as::<_, SpecChange>(
        "SELECT * FROM spec_changes WHERE project_id = ? ORDER BY created_at DESC",
    )
    .bind(project_id)
    .fetch_all(pool)
    .await?)
}

/// Update spec change status
pub async fn update_spec_change_status(
    pool: &Pool<Sqlite>,
    id: &str,
    status: ChangeStatus,
    approved_by: Option<&str>,
) -> DbResult<SpecChange> {
    let now = Utc::now();
    let approved_at = if matches!(status, ChangeStatus::Approved) {
        Some(now)
    } else {
        None
    };

    let change = sqlx::query_as::<_, SpecChange>(
        r#"
        UPDATE spec_changes
        SET status = ?, approved_by = ?, approved_at = ?, updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&status)
    .bind(approved_by)
    .bind(approved_at)
    .bind(now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(change)
}

// ============================================================================
// SpecDelta Operations
// ============================================================================

/// Create a spec delta
pub async fn create_spec_delta(
    pool: &Pool<Sqlite>,
    change_id: &str,
    capability_id: Option<&str>,
    capability_name: &str,
    delta_type: DeltaType,
    delta_markdown: &str,
    position: i32,
) -> DbResult<SpecDelta> {
    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let delta = sqlx::query_as::<_, SpecDelta>(
        r#"
        INSERT INTO spec_deltas (id, change_id, capability_id, capability_name, delta_type, delta_markdown, position, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(change_id)
    .bind(capability_id)
    .bind(capability_name)
    .bind(&delta_type)
    .bind(delta_markdown)
    .bind(position)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(delta)
}

/// Get deltas by change
pub async fn get_deltas_by_change(
    pool: &Pool<Sqlite>,
    change_id: &str,
) -> DbResult<Vec<SpecDelta>> {
    Ok(sqlx::query_as::<_, SpecDelta>(
        "SELECT * FROM spec_deltas WHERE change_id = ? ORDER BY position",
    )
    .bind(change_id)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::query(include_str!("../../migrations/001_initial_schema.sql"))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250118000000_task_management.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!("../../migrations/20250120000000_openspec.sql"))
            .execute(&pool)
            .await
            .unwrap();

        pool
    }

    async fn create_test_project(pool: &Pool<Sqlite>, project_id: &str) {
        sqlx::query(
            r#"
            INSERT INTO projects (id, name, project_root, created_at, updated_at)
            VALUES (?, 'Test Project', '/test/path', datetime('now'), datetime('now'))
            "#,
        )
        .bind(project_id)
        .execute(pool)
        .await
        .unwrap();
    }

    #[tokio::test]
    async fn test_create_and_get_prd() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        let prd = create_prd(
            &pool,
            "test-project",
            "Test PRD",
            "# Test Content",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        assert_eq!(prd.title, "Test PRD");
        assert_eq!(prd.status, PRDStatus::Draft);

        let fetched = get_prd(&pool, &prd.id).await.unwrap();
        assert_eq!(fetched.id, prd.id);
    }

    #[tokio::test]
    async fn test_create_capability_and_requirements() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        let capability = create_capability(
            &pool,
            "test-project",
            None,
            "Test Capability",
            Some("Purpose"),
            "# Spec",
            None,
        )
        .await
        .unwrap();

        assert_eq!(capability.name, "Test Capability");
        assert_eq!(capability.requirement_count, 0);

        let req = create_requirement(&pool, &capability.id, "Test Req", "Content", 1)
            .await
            .unwrap();

        assert_eq!(req.name, "Test Req");

        let updated_cap = get_capability(&pool, &capability.id).await.unwrap();
        assert_eq!(updated_cap.requirement_count, 1);
    }
}
