// ABOUTME: Integration layer between OpenSpec and task system
// ABOUTME: Provides bidirectional sync, task generation from specs, and validation

use super::db as openspec_db;
use super::types::*;
use super::validator;
use sqlx::{Pool, Sqlite};

#[derive(Debug, thiserror::Error)]
pub enum IntegrationError {
    #[error("Database error: {0}")]
    DbError(#[from] openspec_db::DbError),

    #[error("Validation error: {0}")]
    ValidationError(#[from] validator::ValidationError),

    #[error("Task not found: {0}")]
    TaskNotFound(String),

    #[error("Requirement not found: {0}")]
    RequirementNotFound(String),

    #[error("Invalid integration state: {0}")]
    InvalidState(String),
}

pub type IntegrationResult<T> = Result<T, IntegrationError>;

/// Link a task to a spec requirement
pub async fn link_task_to_requirement(
    pool: &Pool<Sqlite>,
    task_id: &str,
    requirement_id: &str,
) -> IntegrationResult<()> {
    // Verify requirement exists
    openspec_db::get_requirement(pool, requirement_id).await?;

    // Create link in task_spec_links table
    sqlx::query(
        "INSERT INTO task_spec_links (task_id, requirement_id) VALUES (?, ?)
         ON CONFLICT (task_id, requirement_id) DO NOTHING",
    )
    .bind(task_id)
    .bind(requirement_id)
    .execute(pool)
    .await
    .map_err(openspec_db::DbError::SqlxError)?;

    Ok(())
}

/// Get all requirements linked to a task
pub async fn get_task_requirements(
    pool: &Pool<Sqlite>,
    task_id: &str,
) -> IntegrationResult<Vec<SpecRequirement>> {
    let requirement_ids: Vec<(String,)> =
        sqlx::query_as("SELECT requirement_id FROM task_spec_links WHERE task_id = ?")
            .bind(task_id)
            .fetch_all(pool)
            .await
            .map_err(openspec_db::DbError::SqlxError)?;

    let mut requirements = Vec::new();
    for (req_id,) in requirement_ids {
        let req = openspec_db::get_requirement(pool, &req_id).await?;
        requirements.push(req);
    }

    Ok(requirements)
}

/// Get all tasks linked to a requirement
pub async fn get_requirement_tasks(
    pool: &Pool<Sqlite>,
    requirement_id: &str,
) -> IntegrationResult<Vec<String>> {
    let task_ids: Vec<(String,)> =
        sqlx::query_as("SELECT task_id FROM task_spec_links WHERE requirement_id = ?")
            .bind(requirement_id)
            .fetch_all(pool)
            .await
            .map_err(openspec_db::DbError::SqlxError)?;

    Ok(task_ids.into_iter().map(|(id,)| id).collect())
}

/// Generate tasks from a spec requirement
pub async fn generate_tasks_from_requirement(
    pool: &Pool<Sqlite>,
    requirement_id: &str,
    project_id: &str,
    tag_id: &str,
) -> IntegrationResult<Vec<String>> {
    let requirement = openspec_db::get_requirement(pool, requirement_id).await?;
    let scenarios = openspec_db::get_scenarios_by_requirement(pool, requirement_id).await?;

    let mut task_ids = Vec::new();

    // Create main task for the requirement
    let main_task_title = format!("Implement: {}", requirement.name);
    let main_task_description = format!(
        "{}\n\nScenarios to validate:\n{}",
        requirement.content_markdown,
        scenarios
            .iter()
            .map(|s| format!("- {}: {} → {}", s.name, s.when_clause, s.then_clause))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let main_task_id = create_task(
        pool,
        project_id,
        tag_id,
        &main_task_title,
        &main_task_description,
        true, // spec_driven
        Some(requirement_id),
    )
    .await?;

    task_ids.push(main_task_id.clone());

    // Link task to requirement
    link_task_to_requirement(pool, &main_task_id, requirement_id).await?;

    Ok(task_ids)
}

/// Generate tasks from all requirements in a capability
pub async fn generate_tasks_from_capability(
    pool: &Pool<Sqlite>,
    capability_id: &str,
    project_id: &str,
    tag_id: &str,
) -> IntegrationResult<Vec<String>> {
    let requirements = openspec_db::get_requirements_by_capability(pool, capability_id).await?;

    let mut all_task_ids = Vec::new();

    for requirement in requirements {
        let task_ids =
            generate_tasks_from_requirement(pool, &requirement.id, project_id, tag_id).await?;
        all_task_ids.extend(task_ids);
    }

    Ok(all_task_ids)
}

/// Validate task completion against spec scenarios
pub async fn validate_task_completion(
    pool: &Pool<Sqlite>,
    task_id: &str,
) -> IntegrationResult<TaskValidationResult> {
    let requirements = get_task_requirements(pool, task_id).await?;

    if requirements.is_empty() {
        return Ok(TaskValidationResult {
            is_valid: true,
            total_scenarios: 0,
            validated_scenarios: 0,
            pending_scenarios: Vec::new(),
            notes: Some("No spec requirements linked to this task".to_string()),
        });
    }

    let mut total_scenarios = 0;
    let mut pending_scenarios = Vec::new();
    let requirement_count = requirements.len();

    for requirement in requirements {
        let scenarios = openspec_db::get_scenarios_by_requirement(pool, &requirement.id).await?;

        for scenario in scenarios {
            total_scenarios += 1;
            pending_scenarios.push(ScenarioValidation {
                requirement_name: requirement.name.clone(),
                scenario_name: scenario.name.clone(),
                when_clause: scenario.when_clause.clone(),
                then_clause: scenario.then_clause.clone(),
                status: TaskValidationStatus::Pending,
            });
        }
    }

    Ok(TaskValidationResult {
        is_valid: false,
        total_scenarios,
        validated_scenarios: 0,
        pending_scenarios,
        notes: Some(format!(
            "Task has {} scenarios to validate across {} requirements",
            total_scenarios, requirement_count
        )),
    })
}

/// Update task validation status
pub async fn update_task_validation(
    pool: &Pool<Sqlite>,
    task_id: &str,
    validation_result: &TaskValidationResult,
) -> IntegrationResult<()> {
    let status = if validation_result.is_valid {
        "passed"
    } else {
        "pending"
    };

    let result_json = serde_json::to_string(validation_result)
        .map_err(|e| IntegrationError::InvalidState(e.to_string()))?;

    sqlx::query(
        "UPDATE tasks SET spec_validation_status = ?, spec_validation_result = ? WHERE id = ?",
    )
    .bind(status)
    .bind(&result_json)
    .bind(task_id)
    .execute(pool)
    .await
    .map_err(openspec_db::DbError::SqlxError)?;

    Ok(())
}

/// Sync spec changes to related tasks
pub async fn sync_spec_changes_to_tasks(
    pool: &Pool<Sqlite>,
    change_id: &str,
) -> IntegrationResult<SyncChangesResult> {
    let deltas = openspec_db::get_deltas_by_change(pool, change_id).await?;

    let mut affected_tasks = 0;
    let mut task_ids = Vec::new();

    for delta in deltas {
        if let Some(capability_id) = &delta.capability_id {
            let requirements =
                openspec_db::get_requirements_by_capability(pool, capability_id).await?;

            for requirement in requirements {
                let tasks = get_requirement_tasks(pool, &requirement.id).await?;

                for task_id in tasks {
                    if !task_ids.contains(&task_id) {
                        // Mark task for re-validation
                        sqlx::query(
                            "UPDATE tasks SET spec_validation_status = 'needs_revalidation' WHERE id = ?",
                        )
                        .bind(&task_id)
                        .execute(pool)
                        .await
                        .map_err(openspec_db::DbError::SqlxError)?;

                        task_ids.push(task_id);
                        affected_tasks += 1;
                    }
                }
            }
        }
    }

    Ok(SyncChangesResult {
        affected_tasks,
        task_ids,
        change_id: change_id.to_string(),
    })
}

// Helper function to create a task (simplified - would use actual task creation API)
async fn create_task(
    pool: &Pool<Sqlite>,
    project_id: &str,
    _tag_id: &str,
    title: &str,
    description: &str,
    _spec_driven: bool,
    _from_prd_id: Option<&str>,
) -> IntegrationResult<String> {
    let task_id = orkee_core::generate_project_id();

    sqlx::query(
        r#"
        INSERT INTO tasks
        (id, project_id, title, description, status, priority, created_at, updated_at)
        VALUES (?, ?, ?, ?, 'pending', 'medium', datetime('now'), datetime('now'))
        "#,
    )
    .bind(&task_id)
    .bind(project_id)
    .bind(title)
    .bind(description)
    .execute(pool)
    .await
    .map_err(openspec_db::DbError::SqlxError)?;

    Ok(task_id)
}

// Integration result types

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskValidationResult {
    pub is_valid: bool,
    pub total_scenarios: usize,
    pub validated_scenarios: usize,
    pub pending_scenarios: Vec<ScenarioValidation>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScenarioValidation {
    pub requirement_name: String,
    pub scenario_name: String,
    pub when_clause: String,
    pub then_clause: String,
    pub status: TaskValidationStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TaskValidationStatus {
    Pending,
    Passed,
    Failed,
}

#[derive(Debug, Clone)]
pub struct SyncChangesResult {
    pub affected_tasks: usize,
    pub task_ids: Vec<String>,
    pub change_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> Pool<Sqlite> {
        let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

        // Run migrations
        sqlx::migrate!("../storage/migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

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
    async fn test_link_task_to_requirement() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create a capability and requirement
        let capability = openspec_db::create_capability(
            &pool,
            "test-project",
            None,
            "Test Cap",
            None,
            "spec",
            None,
        )
        .await
        .unwrap();

        let requirement =
            openspec_db::create_requirement(&pool, &capability.id, "Test Req", "content", 1)
                .await
                .unwrap();

        // Create a task
        let task_id = create_task(
            &pool,
            "test-project",
            "tag-1",
            "Test Task",
            "desc",
            false,
            None,
        )
        .await
        .unwrap();

        // Link them
        link_task_to_requirement(&pool, &task_id, &requirement.id)
            .await
            .unwrap();

        // Verify link
        let tasks = get_requirement_tasks(&pool, &requirement.id).await.unwrap();
        assert_eq!(tasks.len(), 1);
        assert_eq!(tasks[0], task_id);
    }

    #[tokio::test]
    async fn test_generate_tasks_from_requirement() {
        let pool = setup_test_db().await;
        create_test_project(&pool, "test-project").await;

        // Create test data
        let capability = openspec_db::create_capability(
            &pool,
            "test-project",
            None,
            "Test Cap",
            None,
            "spec",
            None,
        )
        .await
        .unwrap();

        let requirement = openspec_db::create_requirement(
            &pool,
            &capability.id,
            "User Login",
            "Users should be able to log in",
            1,
        )
        .await
        .unwrap();

        // Add scenarios
        openspec_db::create_scenario(
            &pool,
            &requirement.id,
            "Valid credentials",
            "user enters valid email and password",
            "user is logged in",
            Some(vec!["session is created".to_string()]),
            1,
        )
        .await
        .unwrap();

        // Generate tasks
        let task_ids =
            generate_tasks_from_requirement(&pool, &requirement.id, "test-project", "tag-1")
                .await
                .unwrap();

        assert_eq!(task_ids.len(), 1);

        // Verify task was linked
        let linked_tasks = get_requirement_tasks(&pool, &requirement.id).await.unwrap();
        assert_eq!(linked_tasks.len(), 1);
    }
}
