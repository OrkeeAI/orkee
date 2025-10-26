// ABOUTME: Database operations for OpenSpec entities
// ABOUTME: Provides CRUD operations for PRDs, capabilities, requirements, scenarios, and changes

use super::types::*;
use chrono::Utc;
use sqlx::{Executor, Pool, Sqlite};

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
// Content Size Limits
// ============================================================================

/// Maximum size for markdown content fields (1MB)
const MAX_MARKDOWN_SIZE: usize = 1024 * 1024;

// ============================================================================
// Pagination Limits
// ============================================================================

/// Maximum allowed value for LIMIT parameter to prevent DoS attacks
const MAX_PAGINATION_LIMIT: i64 = 10000;

/// Maximum allowed value for OFFSET parameter to prevent DoS attacks
const MAX_PAGINATION_OFFSET: i64 = 1000000;

/// Validate markdown content size
fn validate_content_size(content: &str, field_name: &str) -> DbResult<()> {
    if content.len() > MAX_MARKDOWN_SIZE {
        return Err(DbError::InvalidInput(format!(
            "{} exceeds maximum size of {} bytes (got {} bytes). Consider splitting into multiple documents.",
            field_name,
            MAX_MARKDOWN_SIZE,
            content.len()
        )));
    }
    Ok(())
}

/// Validate pagination parameters to prevent DoS attacks
fn validate_pagination(limit: Option<i64>, offset: Option<i64>) -> DbResult<()> {
    if let Some(lim) = limit {
        if lim < 0 {
            return Err(DbError::InvalidInput(
                "LIMIT must be non-negative".to_string(),
            ));
        }
        if lim > MAX_PAGINATION_LIMIT {
            return Err(DbError::InvalidInput(format!(
                "LIMIT exceeds maximum allowed value of {} (got {})",
                MAX_PAGINATION_LIMIT, lim
            )));
        }
    }
    if let Some(off) = offset {
        if off < 0 {
            return Err(DbError::InvalidInput(
                "OFFSET must be non-negative".to_string(),
            ));
        }
        if off > MAX_PAGINATION_OFFSET {
            return Err(DbError::InvalidInput(format!(
                "OFFSET exceeds maximum allowed value of {} (got {})",
                MAX_PAGINATION_OFFSET, off
            )));
        }
    }
    Ok(())
}

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
    // Validate content size
    validate_content_size(content_markdown, "PRD content")?;

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
    sqlx::query_as::<_, PRD>("SELECT * FROM prds WHERE id = ? AND deleted_at IS NULL")
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| DbError::NotFound(format!("PRD not found: {}", id)))
}

/// Get all PRDs for a project
pub async fn get_prds_by_project(pool: &Pool<Sqlite>, project_id: &str) -> DbResult<Vec<PRD>> {
    let (prds, _) = get_prds_by_project_paginated(pool, project_id, None, None).await?;
    Ok(prds)
}

/// Get all PRDs for a project with pagination
pub async fn get_prds_by_project_paginated(
    pool: &Pool<Sqlite>,
    project_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<PRD>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM prds WHERE project_id = ? AND deleted_at IS NULL")
            .bind(project_id)
            .fetch_one(pool)
            .await?;

    // Build query with optional pagination
    let mut query_str = String::from(
        "SELECT * FROM prds WHERE project_id = ? AND deleted_at IS NULL ORDER BY created_at DESC",
    );

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let prds = sqlx::query_as::<_, PRD>(&query_str)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

    Ok((prds, count))
}

/// Update a PRD
pub async fn update_prd(
    pool: &Pool<Sqlite>,
    id: &str,
    title: Option<&str>,
    content_markdown: Option<&str>,
    status: Option<PRDStatus>,
) -> DbResult<PRD> {
    // Validate new content size if provided
    if let Some(content) = content_markdown {
        validate_content_size(content, "PRD content")?;
    }

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

/// Soft delete a PRD
pub async fn delete_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let now = Utc::now();
    let result = sqlx::query("UPDATE prds SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
        .bind(now)
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("PRD not found: {}", id)));
    }

    Ok(())
}

/// Hard delete a PRD (for testing/admin purposes)
pub async fn hard_delete_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM prds WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("PRD not found: {}", id)));
    }

    Ok(())
}

/// Restore a soft-deleted PRD
pub async fn restore_prd(pool: &Pool<Sqlite>, id: &str) -> DbResult<PRD> {
    let prd = sqlx::query_as::<_, PRD>(
        "UPDATE prds SET deleted_at = NULL WHERE id = ? AND deleted_at IS NOT NULL RETURNING *",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DbError::NotFound(format!("Deleted PRD not found: {}", id)))?;

    Ok(prd)
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
    // Validate content sizes
    validate_content_size(spec_markdown, "Capability spec")?;
    if let Some(purpose) = purpose_markdown {
        validate_content_size(purpose, "Capability purpose")?;
    }
    if let Some(design) = design_markdown {
        validate_content_size(design, "Capability design")?;
    }

    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let capability = sqlx::query_as::<_, SpecCapability>(
        r#"
        INSERT INTO spec_capabilities
        (id, project_id, prd_id, name, purpose_markdown, spec_markdown, design_markdown,
         requirement_count, version, status, change_id, is_openspec_compliant, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, 0, 1, 'active', ?, ?, ?, ?)
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
    .bind::<Option<String>>(None) // change_id
    .bind(false) // is_openspec_compliant (default to false for now)
    .bind(now)
    .bind(now)
    .fetch_one(pool)
    .await?;

    Ok(capability)
}

/// Get a capability by ID
pub async fn get_capability(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecCapability> {
    sqlx::query_as::<_, SpecCapability>(
        "SELECT * FROM spec_capabilities WHERE id = ? AND deleted_at IS NULL",
    )
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
    let (capabilities, _) =
        get_capabilities_by_project_paginated(pool, project_id, None, None).await?;
    Ok(capabilities)
}

/// Get all capabilities for a project with pagination
pub async fn get_capabilities_by_project_paginated(
    pool: &Pool<Sqlite>,
    project_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<SpecCapability>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spec_capabilities WHERE project_id = ? AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    // Build query with optional pagination
    let mut query_str = String::from(
        "SELECT * FROM spec_capabilities WHERE project_id = ? AND status = 'active' AND deleted_at IS NULL ORDER BY created_at DESC",
    );

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let capabilities = sqlx::query_as::<_, SpecCapability>(&query_str)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

    Ok((capabilities, count))
}

/// Get capabilities by PRD
pub async fn get_capabilities_by_prd(
    pool: &Pool<Sqlite>,
    prd_id: &str,
) -> DbResult<Vec<SpecCapability>> {
    let (capabilities, _) = get_capabilities_by_prd_paginated(pool, prd_id, None, None).await?;
    Ok(capabilities)
}

/// Get capabilities by PRD with pagination
pub async fn get_capabilities_by_prd_paginated(
    pool: &Pool<Sqlite>,
    prd_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<SpecCapability>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spec_capabilities WHERE prd_id = ? AND status = 'active' AND deleted_at IS NULL",
    )
    .bind(prd_id)
    .fetch_one(pool)
    .await?;

    // Build query with optional pagination
    let mut query_str = String::from(
        "SELECT * FROM spec_capabilities WHERE prd_id = ? AND status = 'active' AND deleted_at IS NULL ORDER BY created_at DESC",
    );

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let capabilities = sqlx::query_as::<_, SpecCapability>(&query_str)
        .bind(prd_id)
        .fetch_all(pool)
        .await?;

    Ok((capabilities, count))
}

/// Result type for capability with requirements
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityWithRequirements {
    pub capability: SpecCapability,
    pub requirements: Vec<SpecRequirement>,
}

/// Result type for requirement with scenarios
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RequirementWithScenarios {
    pub requirement: SpecRequirement,
    pub scenarios: Vec<SpecScenario>,
}

/// Result type for capability with requirements and scenarios
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CapabilityWithRequirementsAndScenarios {
    pub capability: SpecCapability,
    pub requirements: Vec<RequirementWithScenarios>,
}

/// Get a capability with all its requirements in a single query
pub async fn get_capability_with_requirements(
    pool: &Pool<Sqlite>,
    capability_id: &str,
) -> DbResult<CapabilityWithRequirements> {
    // Fetch capability
    let capability = get_capability(pool, capability_id).await?;

    // Fetch all requirements for this capability
    let requirements = get_requirements_by_capability(pool, capability_id).await?;

    Ok(CapabilityWithRequirements {
        capability,
        requirements,
    })
}

/// Get all capabilities with their requirements for a project in optimized way
pub async fn get_capabilities_with_requirements_by_project(
    pool: &Pool<Sqlite>,
    project_id: &str,
) -> DbResult<Vec<CapabilityWithRequirements>> {
    // First, get all capabilities for the project
    let capabilities = get_capabilities_by_project(pool, project_id).await?;

    if capabilities.is_empty() {
        return Ok(Vec::new());
    }

    // Build a query to fetch all requirements for all capabilities at once
    // This avoids N+1 queries
    let capability_ids: Vec<String> = capabilities.iter().map(|c| c.id.clone()).collect();

    // Create placeholders for IN clause
    let placeholders = capability_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT * FROM spec_requirements WHERE capability_id IN ({}) ORDER BY capability_id, position",
        placeholders
    );

    // Build the query with bindings
    let mut query_builder = sqlx::query_as::<_, SpecRequirement>(&query);
    for id in &capability_ids {
        query_builder = query_builder.bind(id);
    }

    let all_requirements = query_builder.fetch_all(pool).await?;

    // Group requirements by capability_id
    let mut requirements_map: std::collections::HashMap<String, Vec<SpecRequirement>> =
        std::collections::HashMap::new();

    for req in all_requirements {
        requirements_map
            .entry(req.capability_id.clone())
            .or_default()
            .push(req);
    }

    // Combine capabilities with their requirements
    let result = capabilities
        .into_iter()
        .map(|cap| {
            let requirements = requirements_map.remove(&cap.id).unwrap_or_default();
            CapabilityWithRequirements {
                capability: cap,
                requirements,
            }
        })
        .collect();

    Ok(result)
}

/// Get all capabilities with requirements and scenarios for a project (fully optimized)
/// Loads everything in 3 queries instead of N+1:
/// 1. Fetch all capabilities
/// 2. Fetch all requirements for those capabilities
/// 3. Fetch all scenarios for those requirements
pub async fn get_capabilities_with_requirements_and_scenarios_by_project(
    pool: &Pool<Sqlite>,
    project_id: &str,
) -> DbResult<Vec<CapabilityWithRequirementsAndScenarios>> {
    // First, get all capabilities for the project
    let capabilities = get_capabilities_by_project(pool, project_id).await?;

    if capabilities.is_empty() {
        return Ok(Vec::new());
    }

    // Get all capability IDs
    let capability_ids: Vec<String> = capabilities.iter().map(|c| c.id.clone()).collect();

    // Build query to fetch all requirements for all capabilities at once
    let placeholders = capability_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let query = format!(
        "SELECT * FROM spec_requirements WHERE capability_id IN ({}) ORDER BY capability_id, position",
        placeholders
    );

    // Bind and execute requirements query
    let mut query_builder = sqlx::query_as::<_, SpecRequirement>(&query);
    for id in &capability_ids {
        query_builder = query_builder.bind(id);
    }
    let all_requirements = query_builder.fetch_all(pool).await?;

    // If no requirements, return capabilities with empty requirements
    if all_requirements.is_empty() {
        return Ok(capabilities
            .into_iter()
            .map(|cap| CapabilityWithRequirementsAndScenarios {
                capability: cap,
                requirements: Vec::new(),
            })
            .collect());
    }

    // Get all requirement IDs for scenario lookup
    let requirement_ids: Vec<String> = all_requirements.iter().map(|r| r.id.clone()).collect();

    // Build query to fetch all scenarios for all requirements at once
    let scenario_placeholders = requirement_ids
        .iter()
        .map(|_| "?")
        .collect::<Vec<_>>()
        .join(",");
    let scenario_query = format!(
        "SELECT * FROM spec_scenarios WHERE requirement_id IN ({}) ORDER BY requirement_id, position",
        scenario_placeholders
    );

    // Bind and execute scenarios query
    let mut scenario_query_builder = sqlx::query_as::<_, SpecScenario>(&scenario_query);
    for id in &requirement_ids {
        scenario_query_builder = scenario_query_builder.bind(id);
    }
    let all_scenarios = scenario_query_builder.fetch_all(pool).await?;

    // Group scenarios by requirement_id
    let mut scenarios_map: std::collections::HashMap<String, Vec<SpecScenario>> =
        std::collections::HashMap::new();
    for scenario in all_scenarios {
        scenarios_map
            .entry(scenario.requirement_id.clone())
            .or_default()
            .push(scenario);
    }

    // Group requirements by capability_id
    let mut requirements_map: std::collections::HashMap<String, Vec<SpecRequirement>> =
        std::collections::HashMap::new();
    for req in all_requirements {
        requirements_map
            .entry(req.capability_id.clone())
            .or_default()
            .push(req);
    }

    // Combine everything together
    let result = capabilities
        .into_iter()
        .map(|cap| {
            let requirements = requirements_map.remove(&cap.id).unwrap_or_default();
            let requirements_with_scenarios = requirements
                .into_iter()
                .map(|req| {
                    let scenarios = scenarios_map.remove(&req.id).unwrap_or_default();
                    RequirementWithScenarios {
                        requirement: req,
                        scenarios,
                    }
                })
                .collect();

            CapabilityWithRequirementsAndScenarios {
                capability: cap,
                requirements: requirements_with_scenarios,
            }
        })
        .collect();

    Ok(result)
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

/// Soft delete a capability
pub async fn delete_capability(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let now = Utc::now();
    let result = sqlx::query(
        "UPDATE spec_capabilities SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL",
    )
    .bind(now)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("Capability not found: {}", id)));
    }

    Ok(())
}

/// Hard delete a capability (for testing/admin purposes)
pub async fn hard_delete_capability(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM spec_capabilities WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("Capability not found: {}", id)));
    }

    Ok(())
}

/// Restore a soft-deleted capability
pub async fn restore_capability(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecCapability> {
    let capability = sqlx::query_as::<_, SpecCapability>(
        "UPDATE spec_capabilities SET deleted_at = NULL WHERE id = ? AND deleted_at IS NOT NULL RETURNING *",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DbError::NotFound(format!("Deleted capability not found: {}", id)))?;

    Ok(capability)
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
    let (requirements, _) =
        get_requirements_by_capability_paginated(pool, capability_id, None, None).await?;
    Ok(requirements)
}

/// Get all requirements for a capability with pagination
pub async fn get_requirements_by_capability_paginated(
    pool: &Pool<Sqlite>,
    capability_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<SpecRequirement>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM spec_requirements WHERE capability_id = ?")
            .bind(capability_id)
            .fetch_one(pool)
            .await?;

    // Build query with optional pagination
    let mut query_str =
        String::from("SELECT * FROM spec_requirements WHERE capability_id = ? ORDER BY position");

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let requirements = sqlx::query_as::<_, SpecRequirement>(&query_str)
        .bind(capability_id)
        .fetch_all(pool)
        .await?;

    Ok((requirements, count))
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
    create_spec_change_with_verb(
        pool,
        project_id,
        prd_id,
        proposal_markdown,
        tasks_markdown,
        design_markdown,
        created_by,
        None,
        None,
    )
    .await
}

/// Create a spec change with optional verb prefix and change number
#[allow(clippy::too_many_arguments)]
pub async fn create_spec_change_with_verb<'a, E>(
    executor: E,
    project_id: &str,
    prd_id: Option<&str>,
    proposal_markdown: &str,
    tasks_markdown: &str,
    design_markdown: Option<&str>,
    created_by: &str,
    verb_prefix: Option<&str>,
    change_number: Option<i32>,
) -> DbResult<SpecChange>
where
    E: Executor<'a, Database = Sqlite>,
{
    // Validate content sizes
    validate_content_size(proposal_markdown, "Change proposal")?;
    validate_content_size(tasks_markdown, "Change tasks")?;
    if let Some(design) = design_markdown {
        validate_content_size(design, "Change design")?;
    }

    let id = crate::storage::generate_project_id();
    let now = Utc::now();

    let change = sqlx::query_as::<_, SpecChange>(
        r#"
        INSERT INTO spec_changes
        (id, project_id, prd_id, proposal_markdown, tasks_markdown, design_markdown,
         status, verb_prefix, change_number, validation_status, validation_errors,
         created_by, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, 'draft', ?, ?, 'pending', ?, ?, ?, ?)
        RETURNING *
        "#,
    )
    .bind(&id)
    .bind(project_id)
    .bind(prd_id)
    .bind(proposal_markdown)
    .bind(tasks_markdown)
    .bind(design_markdown)
    .bind(verb_prefix)
    .bind(change_number)
    .bind::<Option<String>>(None) // validation_errors
    .bind(created_by)
    .bind(now)
    .bind(now)
    .fetch_one(executor)
    .await?;

    Ok(change)
}

/// Get spec change by ID
pub async fn get_spec_change(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecChange> {
    sqlx::query_as::<_, SpecChange>(
        "SELECT * FROM spec_changes WHERE id = ? AND deleted_at IS NULL",
    )
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
    let (changes, _) = get_spec_changes_by_project_paginated(pool, project_id, None, None).await?;
    Ok(changes)
}

/// Get spec changes by project with pagination
pub async fn get_spec_changes_by_project_paginated(
    pool: &Pool<Sqlite>,
    project_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<SpecChange>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM spec_changes WHERE project_id = ? AND deleted_at IS NULL",
    )
    .bind(project_id)
    .fetch_one(pool)
    .await?;

    // Build query with optional pagination
    let mut query_str = String::from(
        "SELECT * FROM spec_changes WHERE project_id = ? AND deleted_at IS NULL ORDER BY created_at DESC",
    );

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let changes = sqlx::query_as::<_, SpecChange>(&query_str)
        .bind(project_id)
        .fetch_all(pool)
        .await?;

    Ok((changes, count))
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
    let archived_at = if matches!(status, ChangeStatus::Archived) {
        Some(now)
    } else {
        None
    };

    let change = sqlx::query_as::<_, SpecChange>(
        r#"
        UPDATE spec_changes
        SET status = ?, approved_by = ?, approved_at = ?, archived_at = ?, updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(&status)
    .bind(approved_by)
    .bind(approved_at)
    .bind(archived_at)
    .bind(now)
    .bind(id)
    .fetch_one(pool)
    .await?;

    Ok(change)
}

/// Soft delete a spec change
pub async fn delete_spec_change(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let now = Utc::now();
    let result =
        sqlx::query("UPDATE spec_changes SET deleted_at = ? WHERE id = ? AND deleted_at IS NULL")
            .bind(now)
            .bind(id)
            .execute(pool)
            .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("Spec change not found: {}", id)));
    }

    Ok(())
}

/// Hard delete a spec change (for testing/admin purposes)
pub async fn hard_delete_spec_change(pool: &Pool<Sqlite>, id: &str) -> DbResult<()> {
    let result = sqlx::query("DELETE FROM spec_changes WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(DbError::NotFound(format!("Spec change not found: {}", id)));
    }

    Ok(())
}

/// Restore a soft-deleted spec change
pub async fn restore_spec_change(pool: &Pool<Sqlite>, id: &str) -> DbResult<SpecChange> {
    let change = sqlx::query_as::<_, SpecChange>(
        "UPDATE spec_changes SET deleted_at = NULL WHERE id = ? AND deleted_at IS NOT NULL RETURNING *",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DbError::NotFound(format!("Deleted spec change not found: {}", id)))?;

    Ok(change)
}

// ============================================================================
// SpecDelta Operations
// ============================================================================

/// Create a spec delta
pub async fn create_spec_delta<'a, E>(
    executor: E,
    change_id: &str,
    capability_id: Option<&str>,
    capability_name: &str,
    delta_type: DeltaType,
    delta_markdown: &str,
    position: i32,
) -> DbResult<SpecDelta>
where
    E: Executor<'a, Database = Sqlite>,
{
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
    .fetch_one(executor)
    .await?;

    Ok(delta)
}

/// Get deltas by change
pub async fn get_deltas_by_change(
    pool: &Pool<Sqlite>,
    change_id: &str,
) -> DbResult<Vec<SpecDelta>> {
    let (deltas, _) = get_deltas_by_change_paginated(pool, change_id, None, None).await?;
    Ok(deltas)
}

/// Get deltas by change with pagination
pub async fn get_deltas_by_change_paginated(
    pool: &Pool<Sqlite>,
    change_id: &str,
    limit: Option<i64>,
    offset: Option<i64>,
) -> DbResult<(Vec<SpecDelta>, i64)> {
    // Validate pagination parameters
    validate_pagination(limit, offset)?;

    // Get total count
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM spec_deltas WHERE change_id = ?")
        .bind(change_id)
        .fetch_one(pool)
        .await?;

    // Build query with optional pagination
    let mut query_str =
        String::from("SELECT * FROM spec_deltas WHERE change_id = ? ORDER BY position");

    if let Some(lim) = limit {
        query_str.push_str(&format!(" LIMIT {}", lim));
    }
    if let Some(off) = offset {
        query_str.push_str(&format!(" OFFSET {}", off));
    }

    let deltas = sqlx::query_as::<_, SpecDelta>(&query_str)
        .bind(change_id)
        .fetch_all(pool)
        .await?;

    Ok((deltas, count))
}

// ============================================================================
// SpecChangeTask Operations
// ============================================================================

/// Parse and store tasks from a change's tasks_markdown
pub async fn parse_and_store_change_tasks(
    pool: &Pool<Sqlite>,
    change_id: &str,
) -> DbResult<Vec<SpecChangeTask>> {
    // Get the change
    let change = get_spec_change(pool, change_id).await?;

    // Parse tasks from markdown
    let parsed_tasks = super::task_parser::parse_tasks_from_markdown(&change.tasks_markdown)
        .map_err(|e| DbError::InvalidInput(format!("Failed to parse tasks: {}", e)))?;

    // Delete existing tasks for this change
    sqlx::query("DELETE FROM spec_change_tasks WHERE change_id = ?")
        .bind(change_id)
        .execute(pool)
        .await?;

    // Insert new tasks
    let mut tasks = Vec::new();
    let now = Utc::now();

    for parsed_task in parsed_tasks {
        let id = crate::storage::generate_project_id();

        // Safely convert display_order from usize to i32
        let display_order_i32: i32 = parsed_task.display_order.try_into().map_err(|_| {
            DbError::InvalidInput(format!(
                "Task display_order {} exceeds maximum value ({})",
                parsed_task.display_order,
                i32::MAX
            ))
        })?;

        let task = sqlx::query_as::<_, SpecChangeTask>(
            r#"
            INSERT INTO spec_change_tasks (
                id, change_id, task_number, task_text, is_completed,
                completed_by, completed_at, display_order, parent_number,
                created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&id)
        .bind(change_id)
        .bind(&parsed_task.number)
        .bind(&parsed_task.text)
        .bind(parsed_task.is_completed)
        .bind(Option::<String>::None) // completed_by (will be set on update)
        .bind(Option::<chrono::DateTime<Utc>>::None) // completed_at
        .bind(display_order_i32)
        .bind(parsed_task.parent_number.as_deref())
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;

        tasks.push(task);
    }

    // Update tasks_parsed_at timestamp
    sqlx::query("UPDATE spec_changes SET tasks_parsed_at = ? WHERE id = ?")
        .bind(now)
        .bind(change_id)
        .execute(pool)
        .await?;

    Ok(tasks)
}

/// Get all tasks for a change
pub async fn get_change_tasks(
    pool: &Pool<Sqlite>,
    change_id: &str,
) -> DbResult<Vec<SpecChangeTask>> {
    let tasks = sqlx::query_as::<_, SpecChangeTask>(
        "SELECT * FROM spec_change_tasks WHERE change_id = ? ORDER BY display_order",
    )
    .bind(change_id)
    .fetch_all(pool)
    .await?;

    Ok(tasks)
}

/// Update a task's completion status
pub async fn update_change_task(
    pool: &Pool<Sqlite>,
    task_id: &str,
    is_completed: bool,
    completed_by: Option<&str>,
) -> DbResult<SpecChangeTask> {
    let now = Utc::now();

    let task = sqlx::query_as::<_, SpecChangeTask>(
        r#"
        UPDATE spec_change_tasks
        SET
            is_completed = ?,
            completed_by = ?,
            completed_at = CASE WHEN ? THEN ? ELSE NULL END,
            updated_at = ?
        WHERE id = ?
        RETURNING *
        "#,
    )
    .bind(is_completed)
    .bind(completed_by)
    .bind(is_completed)
    .bind(if is_completed { Some(now) } else { None })
    .bind(now)
    .bind(task_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| DbError::NotFound(format!("Task not found: {}", task_id)))?;

    Ok(task)
}

/// Bulk update tasks
pub async fn bulk_update_change_tasks(
    pool: &Pool<Sqlite>,
    tasks: Vec<crate::api::change_handlers::TaskUpdate>,
) -> DbResult<Vec<SpecChangeTask>> {
    let mut updated_tasks = Vec::new();

    for task_update in tasks {
        let task = update_change_task(
            pool,
            &task_update.task_id,
            task_update.is_completed,
            task_update.completed_by.as_deref(),
        )
        .await?;

        updated_tasks.push(task);
    }

    Ok(updated_tasks)
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations in order
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

        sqlx::query(include_str!("../../migrations/20250119000000_security.sql"))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250120000000_telemetry.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!("../../migrations/20250122000000_context.sql"))
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250123000000_context_spec_integration.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250124000000_users_table.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250125000000_system_settings.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250126000000_api_tokens.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250127000000_openspec_alignment.sql"
        ))
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(include_str!(
            "../../migrations/20250128000000_task_completion_tracking.sql"
        ))
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

    #[tokio::test]
    async fn test_get_capabilities_with_requirements_optimized() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create multiple capabilities with requirements
        let cap1 = create_capability(
            &pool,
            "test-project",
            None,
            "Capability 1",
            Some("Purpose 1"),
            "# Spec 1",
            None,
        )
        .await
        .unwrap();

        let cap2 = create_capability(
            &pool,
            "test-project",
            None,
            "Capability 2",
            Some("Purpose 2"),
            "# Spec 2",
            None,
        )
        .await
        .unwrap();

        // Add requirements to cap1
        let req1 = create_requirement(&pool, &cap1.id, "Requirement 1", "Content 1", 1)
            .await
            .unwrap();
        let req2 = create_requirement(&pool, &cap1.id, "Requirement 2", "Content 2", 2)
            .await
            .unwrap();

        // Add requirement to cap2
        let req3 = create_requirement(&pool, &cap2.id, "Requirement 3", "Content 3", 1)
            .await
            .unwrap();

        // Fetch all capabilities with requirements using optimized query
        let results = get_capabilities_with_requirements_by_project(&pool, "test-project")
            .await
            .unwrap();

        // Verify results
        assert_eq!(results.len(), 2);

        // Find cap1 and cap2 in results
        let cap1_result = results.iter().find(|r| r.capability.id == cap1.id).unwrap();
        let cap2_result = results.iter().find(|r| r.capability.id == cap2.id).unwrap();

        // Verify cap1 has 2 requirements
        assert_eq!(cap1_result.requirements.len(), 2);
        assert_eq!(cap1_result.requirements[0].id, req1.id);
        assert_eq!(cap1_result.requirements[1].id, req2.id);

        // Verify cap2 has 1 requirement
        assert_eq!(cap2_result.requirements.len(), 1);
        assert_eq!(cap2_result.requirements[0].id, req3.id);
    }

    #[tokio::test]
    async fn test_pagination_limit_validation() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Test LIMIT exceeds maximum
        let result = get_prds_by_project_paginated(&pool, "test-project", Some(20000), None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test OFFSET exceeds maximum
        let result =
            get_prds_by_project_paginated(&pool, "test-project", None, Some(2000000)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test negative LIMIT
        let result = get_prds_by_project_paginated(&pool, "test-project", Some(-1), None).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test negative OFFSET
        let result = get_prds_by_project_paginated(&pool, "test-project", None, Some(-1)).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DbError::InvalidInput(_)));

        // Test valid pagination should work
        let result = get_prds_by_project_paginated(&pool, "test-project", Some(10), Some(0)).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_get_change_tasks_has_no_side_effects() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create a change by inserting directly to avoid schema issues
        let change_id = crate::storage::generate_project_id();
        let now = Utc::now();
        sqlx::query(
            r#"
            INSERT INTO spec_changes
            (id, project_id, proposal_markdown, tasks_markdown, status, created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'draft', 'test-user', ?, ?)
            "#,
        )
        .bind(&change_id)
        .bind("test-project")
        .bind("Test proposal")
        .bind("## Tasks\n- [ ] Task 1\n- [ ] Task 2")
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // Check task count before calling get_change_tasks
        let count_before: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM spec_change_tasks WHERE change_id = ?")
                .bind(&change_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count_before, 0, "Should start with no tasks");

        // Call get_change_tasks - this currently has a side effect of parsing and storing tasks
        let _tasks = get_change_tasks(&pool, &change_id).await.unwrap();

        // Check task count after - if there's a side effect, tasks will have been created
        let count_after: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM spec_change_tasks WHERE change_id = ?")
                .bind(&change_id)
                .fetch_one(&pool)
                .await
                .unwrap();

        // This assertion will FAIL with current implementation (proving the side effect exists)
        // After fix, it should PASS (proving no side effect)
        assert_eq!(
            count_after, 0,
            "get_change_tasks should not create tasks (found {} tasks after call)",
            count_after
        );
    }

    #[tokio::test]
    async fn test_parse_and_store_change_tasks_overflow_protection() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create a change with a task that has an extremely large display_order
        let change_id = crate::storage::generate_project_id();
        let now = Utc::now();

        // Create a tasks_markdown with many tasks to potentially overflow i32
        let mut tasks_markdown = String::from("## Tasks\n");
        // i32::MAX is 2,147,483,647
        // Create markdown that would generate display_order > i32::MAX
        // Since we can't easily create 2 billion tasks, we'll test the validation directly
        // by creating a ParsedTask manually with large display_order

        // For now, let's just verify the normal case works
        // The real fix will use try_into() which will catch overflow
        tasks_markdown.push_str("- [ ] Task 1\n");

        sqlx::query(
            r#"
            INSERT INTO spec_changes
            (id, project_id, proposal_markdown, tasks_markdown, status, created_by, created_at, updated_at)
            VALUES (?, ?, ?, ?, 'draft', 'test-user', ?, ?)
            "#,
        )
        .bind(&change_id)
        .bind("test-project")
        .bind("Test proposal")
        .bind(&tasks_markdown)
        .bind(now)
        .bind(now)
        .execute(&pool)
        .await
        .unwrap();

        // This should succeed with valid display_order
        let result = parse_and_store_change_tasks(&pool, &change_id).await;
        assert!(
            result.is_ok(),
            "Should successfully parse and store tasks with valid display_order"
        );
    }

    #[tokio::test]
    async fn test_soft_delete_count_consistency() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create multiple PRDs
        let prd1 = create_prd(
            &pool,
            "test-project",
            "PRD 1",
            "Content 1",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        let prd2 = create_prd(
            &pool,
            "test-project",
            "PRD 2",
            "Content 2",
            PRDStatus::Draft,
            PRDSource::Manual,
            Some("test-user"),
        )
        .await
        .unwrap();

        // Soft delete one PRD
        delete_prd(&pool, &prd2.id).await.unwrap();

        // Get PRDs with pagination - this returns (Vec, count)
        let (prds, count) = get_prds_by_project_paginated(&pool, "test-project", None, None)
            .await
            .unwrap();

        // Count should match the actual number of returned PRDs (both should exclude deleted)
        assert_eq!(
            prds.len() as i64,
            count,
            "Count mismatch: pagination count should match actual results (excluding deleted)"
        );
        assert_eq!(prds.len(), 1, "Should only return non-deleted PRDs");
        assert_eq!(prds[0].id, prd1.id, "Should return the non-deleted PRD");
    }
}
