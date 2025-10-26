// ABOUTME: Helper functions for building OpenSpec changes from PRD analysis
// ABOUTME: Generates change IDs, builds markdown content, and determines change metadata

use super::db::{create_spec_change_with_verb, create_spec_delta, DbError};
use super::types::{DeltaType, SpecChange};
use crate::api::ai_handlers::{PRDAnalysisData, SpecCapability, TaskSuggestion};
use sqlx::{Pool, Sqlite};

/// Generate a unique change ID for a project
pub async fn generate_change_id(
    pool: &Pool<Sqlite>,
    project_id: &str,
    verb: &str,
) -> Result<String, DbError> {
    // Get the next change number for this verb
    let next_number: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(MAX(change_number), 0) + 1
        FROM spec_changes
        WHERE project_id = ? AND verb_prefix = ? AND deleted_at IS NULL
        "#,
    )
    .bind(project_id)
    .bind(verb)
    .fetch_one(pool)
    .await?;

    Ok(format!("{}-{}", verb, next_number))
}

/// Determine the verb prefix from PRD analysis
pub fn determine_verb_from_analysis(analysis: &PRDAnalysisData) -> String {
    // Look at the summary and capabilities to determine the primary action
    let summary_lower = analysis.summary.to_lowercase();

    // Check in order of precedence (more specific to less specific)
    if summary_lower.contains("update") || summary_lower.contains("modify") {
        "update".to_string()
    } else if summary_lower.contains("fix")
        || summary_lower.contains("bug")
        || summary_lower.contains("issue")
    {
        "fix".to_string()
    } else if summary_lower.contains("remove") || summary_lower.contains("delete") {
        "remove".to_string()
    } else if summary_lower.contains("refactor") || summary_lower.contains("improve") {
        "refactor".to_string()
    } else if summary_lower.contains("add")
        || summary_lower.contains("new")
        || summary_lower.contains("create")
    {
        "add".to_string()
    } else {
        // Default to "add" for new features
        "add".to_string()
    }
}

/// Build proposal markdown from analysis
pub fn build_proposal_markdown(analysis: &PRDAnalysisData) -> String {
    let mut markdown = String::from("## Why\n");
    markdown.push_str(&analysis.summary);
    markdown.push_str("\n\n## What Changes\n");

    for capability in &analysis.capabilities {
        markdown.push_str(&format!(
            "- **{}**: {}\n",
            capability.name, capability.purpose
        ));
    }

    markdown.push_str("\n## Impact\n");
    markdown.push_str(&format!(
        "- Affected specs: {}\n",
        analysis
            .capabilities
            .iter()
            .map(|c| c.name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    ));

    let total_complexity: u32 = analysis
        .capabilities
        .iter()
        .flat_map(|c| &c.requirements)
        .count() as u32;
    markdown.push_str(&format!(
        "- Complexity: {} requirements across {} capabilities\n",
        total_complexity,
        analysis.capabilities.len()
    ));

    if let Some(deps) = &analysis.dependencies {
        markdown.push_str(&format!("- Dependencies: {}\n", deps.join(", ")));
    } else {
        markdown.push_str("- Dependencies: None\n");
    }

    markdown
}

/// Build tasks markdown from suggested tasks
pub fn build_tasks_markdown(tasks: &[TaskSuggestion]) -> String {
    let mut markdown = String::from("## Implementation Tasks\n\n");

    for (i, task) in tasks.iter().enumerate() {
        markdown.push_str(&format!(
            "{}. **{}** (Priority: {}, Complexity: {})\n",
            i + 1,
            task.title,
            task.priority,
            task.complexity
        ));
        markdown.push_str(&format!("   {}\n", task.description));
        if let Some(hours) = task.estimated_hours {
            markdown.push_str(&format!("   Estimated: {} hours\n", hours));
        }
        markdown.push('\n');
    }

    markdown
}

/// Build design markdown if needed
pub fn build_design_markdown(analysis: &PRDAnalysisData) -> String {
    let mut markdown = String::from("## Technical Design\n\n");

    if let Some(considerations) = &analysis.technical_considerations {
        markdown.push_str("### Technical Considerations\n\n");
        for consideration in considerations {
            markdown.push_str(&format!("- {}\n", consideration));
        }
        markdown.push('\n');
    }

    markdown.push_str("### Architecture\n\n");
    markdown.push_str("[Architecture details to be added]\n\n");

    markdown.push_str("### Implementation Approach\n\n");
    markdown.push_str("[Implementation approach to be added]\n");

    markdown
}

/// Determine if a design document is needed based on analysis complexity
pub fn needs_design_doc(analysis: &PRDAnalysisData) -> bool {
    // Design doc needed if:
    // 1. Technical considerations are present
    // 2. Multiple capabilities (complex change)
    // 3. Dependencies exist

    if analysis.technical_considerations.is_some() {
        return true;
    }

    if analysis.capabilities.len() > 2 {
        return true;
    }

    if let Some(deps) = &analysis.dependencies {
        if !deps.is_empty() {
            return true;
        }
    }

    // Check total requirement count
    let total_requirements: usize = analysis
        .capabilities
        .iter()
        .map(|c| c.requirements.len())
        .sum();

    if total_requirements > 5 {
        return true;
    }

    false
}

/// Build capability delta markdown in OpenSpec format
pub fn build_capability_delta_markdown(capability: &SpecCapability) -> String {
    let mut markdown = String::from("## ADDED Requirements\n\n");

    for req in &capability.requirements {
        markdown.push_str(&format!("### Requirement: {}\n", req.name));
        markdown.push_str(&format!("{}\n\n", req.content));

        for scenario in &req.scenarios {
            markdown.push_str(&format!("#### Scenario: {}\n", scenario.name));
            markdown.push_str(&format!("- **WHEN** {}\n", scenario.when));
            markdown.push_str(&format!("- **THEN** {}\n", scenario.then));

            if let Some(and_clauses) = &scenario.and {
                for clause in and_clauses {
                    markdown.push_str(&format!("- **AND** {}\n", clause));
                }
            }
            markdown.push('\n');
        }
    }

    markdown
}

/// Calculate overall complexity from analysis
pub fn calculate_overall_complexity(analysis: &PRDAnalysisData) -> String {
    let total_requirements: usize = analysis
        .capabilities
        .iter()
        .map(|c| c.requirements.len())
        .sum();

    let total_scenarios: usize = analysis
        .capabilities
        .iter()
        .flat_map(|c| &c.requirements)
        .map(|r| r.scenarios.len())
        .sum();

    if total_scenarios > 20 {
        "Very High".to_string()
    } else if total_scenarios > 10 {
        "High".to_string()
    } else if total_scenarios > 5 || total_requirements > 3 {
        "Medium".to_string()
    } else {
        "Low".to_string()
    }
}

/// Create a change from PRD analysis with atomic ID generation
/// Uses a transaction with retry logic to prevent race conditions
pub async fn create_change_from_analysis(
    pool: &Pool<Sqlite>,
    project_id: &str,
    prd_id: &str,
    analysis: &PRDAnalysisData,
    user_id: &str,
) -> Result<SpecChange, DbError> {
    const MAX_RETRIES: u32 = 5;
    const INITIAL_BACKOFF_MS: u64 = 10;

    let verb = determine_verb_from_analysis(analysis);
    let proposal_markdown = build_proposal_markdown(analysis);
    let tasks_markdown = build_tasks_markdown(&analysis.suggested_tasks);
    let design_markdown = if needs_design_doc(analysis) {
        Some(build_design_markdown(analysis))
    } else {
        None
    };

    for attempt in 0..MAX_RETRIES {
        // Start a transaction
        let mut tx = pool.begin().await?;

        // Generate the next change number within the transaction
        let next_number: i32 = sqlx::query_scalar(
            r#"
            SELECT COALESCE(MAX(change_number), 0) + 1
            FROM spec_changes
            WHERE project_id = ? AND verb_prefix = ? AND deleted_at IS NULL
            "#,
        )
        .bind(project_id)
        .bind(&verb)
        .fetch_one(&mut *tx)
        .await?;

        // Create the change with verb and number atomically
        let change_result = create_spec_change_with_verb(
            &mut *tx,
            project_id,
            Some(prd_id),
            &proposal_markdown,
            &tasks_markdown,
            design_markdown.as_deref(),
            user_id,
            Some(&verb),
            Some(next_number),
        )
        .await;

        // Handle the result
        match change_result {
            Ok(change) => {
                // Create deltas for each capability
                for (position, capability) in analysis.capabilities.iter().enumerate() {
                    let delta_markdown = build_capability_delta_markdown(capability);

                    create_spec_delta(
                        &mut *tx,
                        &change.id,
                        None, // No existing capability yet
                        &capability.name,
                        DeltaType::Added,
                        &delta_markdown,
                        position as i32,
                    )
                    .await?;
                }

                // Commit the transaction
                match tx.commit().await {
                    Ok(_) => {
                        // Fetch the created change from the main pool
                        return super::db::get_spec_change(pool, &change.id).await;
                    }
                    Err(e) => {
                        // Check if this is a unique constraint violation
                        if is_unique_constraint_error_sqlx(&e) && attempt < MAX_RETRIES - 1 {
                            // Calculate exponential backoff and retry
                            let backoff_ms = INITIAL_BACKOFF_MS * 2_u64.pow(attempt);
                            tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms))
                                .await;
                            continue;
                        }
                        return Err(DbError::from(e));
                    }
                }
            }
            Err(e) => {
                // Rollback the transaction
                let _ = tx.rollback().await;

                // Check if this is a unique constraint violation
                if is_unique_constraint_error(&e) && attempt < MAX_RETRIES - 1 {
                    // Calculate exponential backoff and retry
                    let backoff_ms = INITIAL_BACKOFF_MS * 2_u64.pow(attempt);
                    tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                    continue;
                }
                // Not a constraint violation or out of retries, return the error
                return Err(e);
            }
        }
    }

    unreachable!("Retry loop should always return or continue")
}

/// Check if a sqlx error is a unique constraint violation or deadlock
fn is_unique_constraint_error_sqlx(e: &sqlx::Error) -> bool {
    let error_str = e.to_string();
    error_str.contains("UNIQUE constraint failed") || error_str.contains("database is deadlocked")
}

/// Check if a DbError is a unique constraint violation
fn is_unique_constraint_error(e: &DbError) -> bool {
    match e {
        DbError::SqlxError(sqlx_err) => is_unique_constraint_error_sqlx(sqlx_err),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_determine_verb_from_analysis() {
        let analysis = PRDAnalysisData {
            summary: "Add new user authentication feature".to_string(),
            capabilities: vec![],
            suggested_tasks: vec![],
            dependencies: None,
            technical_considerations: None,
        };

        assert_eq!(determine_verb_from_analysis(&analysis), "add");
    }

    #[test]
    fn test_determine_verb_fix() {
        let analysis = PRDAnalysisData {
            summary: "Fix bug in payment processing".to_string(),
            capabilities: vec![],
            suggested_tasks: vec![],
            dependencies: None,
            technical_considerations: None,
        };

        assert_eq!(determine_verb_from_analysis(&analysis), "fix");
    }

    #[test]
    fn test_determine_verb_update() {
        let analysis = PRDAnalysisData {
            summary: "Update existing dashboard with new metrics".to_string(),
            capabilities: vec![],
            suggested_tasks: vec![],
            dependencies: None,
            technical_considerations: None,
        };

        assert_eq!(determine_verb_from_analysis(&analysis), "update");
    }

    #[test]
    fn test_needs_design_doc_with_technical_considerations() {
        let analysis = PRDAnalysisData {
            summary: "Simple feature".to_string(),
            capabilities: vec![],
            suggested_tasks: vec![],
            dependencies: None,
            technical_considerations: Some(vec!["Database migration required".to_string()]),
        };

        assert!(needs_design_doc(&analysis));
    }

    #[test]
    fn test_needs_design_doc_simple_change() {
        let analysis = PRDAnalysisData {
            summary: "Simple feature".to_string(),
            capabilities: vec![],
            suggested_tasks: vec![],
            dependencies: None,
            technical_considerations: None,
        };

        assert!(!needs_design_doc(&analysis));
    }

    #[test]
    fn test_calculate_overall_complexity() {
        use crate::api::ai_handlers::{SpecRequirement, SpecScenario};

        let capability = SpecCapability {
            id: "test".to_string(),
            name: "Test".to_string(),
            purpose: "Test purpose".to_string(),
            requirements: vec![SpecRequirement {
                name: "Req1".to_string(),
                content: "Content".to_string(),
                scenarios: vec![
                    SpecScenario {
                        name: "S1".to_string(),
                        when: "when".to_string(),
                        then: "then".to_string(),
                        and: None,
                    },
                    SpecScenario {
                        name: "S2".to_string(),
                        when: "when".to_string(),
                        then: "then".to_string(),
                        and: None,
                    },
                ],
            }],
        };

        let analysis = PRDAnalysisData {
            summary: "Test".to_string(),
            capabilities: vec![capability],
            suggested_tasks: vec![],
            dependencies: None,
            technical_considerations: None,
        };

        assert_eq!(calculate_overall_complexity(&analysis), "Low");
    }
}
