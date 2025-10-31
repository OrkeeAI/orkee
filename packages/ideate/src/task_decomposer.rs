// ABOUTME: Task decomposition service for breaking down Epics into actionable tasks
// ABOUTME: Handles dependency detection, parallel group assignment, and size estimation

use crate::epic::{ConflictAnalysis, DependencyGraph, GraphEdge, GraphNode, TaskConflict, WorkAnalysis, WorkStream};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use ::storage::StorageError as StoreError;
use tasks::types::{SizeEstimate, Task, TaskCreateInput, TaskPriority, TaskStatus, TaskType};

/// Input for task decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecomposeEpicInput {
    pub epic_id: String,
    pub task_categories: Vec<TaskCategory>,
}

/// Task category with tasks to generate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCategory {
    pub name: String,
    pub description: String,
    pub tasks: Vec<TaskTemplate>,
}

/// Template for generating a task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskTemplate {
    pub title: String,
    pub description: Option<String>,
    pub technical_details: Option<String>,
    pub size_estimate: Option<SizeEstimate>,
    pub effort_hours: Option<i32>,
    pub depends_on_titles: Option<Vec<String>>, // Task titles this depends on
    pub acceptance_criteria: Option<String>,
    pub test_strategy: Option<String>,
}

/// Result of task decomposition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecompositionResult {
    pub tasks: Vec<Task>,
    pub dependency_graph: DependencyGraph,
    pub parallel_groups: Vec<ParallelGroup>,
    pub conflicts: Vec<TaskConflict>,
}

/// Group of tasks that can be executed in parallel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParallelGroup {
    pub id: String,
    pub name: String,
    pub task_ids: Vec<String>,
}

/// Task decomposer service
pub struct TaskDecomposer {
    pool: SqlitePool,
}

impl TaskDecomposer {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Decompose an epic into tasks
    pub async fn decompose_epic(
        &self,
        project_id: &str,
        user_id: &str,
        input: DecomposeEpicInput,
    ) -> Result<DecompositionResult, StoreError> {
        // 1. Validate epic exists
        let epic = self.get_epic(&input.epic_id).await?;

        if epic.project_id != project_id {
            return Err(StoreError::Database(
                "Epic does not belong to this project".to_string(),
            ));
        }

        // 2. Generate tasks per category
        let mut all_tasks = Vec::new();
        let mut title_to_id_map = std::collections::HashMap::new();

        for category in &input.task_categories {
            for task_template in &category.tasks {
                let task_id = nanoid::nanoid!();
                title_to_id_map.insert(task_template.title.clone(), task_id.clone());

                let task_input = TaskCreateInput {
                    title: task_template.title.clone(),
                    description: task_template.description.clone(),
                    status: Some(TaskStatus::Pending),
                    priority: Some(TaskPriority::Medium),
                    assigned_agent_id: None,
                    parent_id: None,
                    position: None,
                    dependencies: None, // Will be set after all tasks are created
                    due_date: None,
                    estimated_hours: task_template.effort_hours.map(|h| h as f64),
                    complexity_score: None,
                    details: task_template.technical_details.clone(),
                    test_strategy: task_template.test_strategy.clone(),
                    acceptance_criteria: task_template.acceptance_criteria.clone(),
                    prompt: None,
                    context: None,
                    tag_id: None,
                    tags: None,
                    category: Some(category.name.clone()),
                    epic_id: Some(input.epic_id.clone()),
                    parallel_group: None, // Will be assigned later
                    depends_on: None,     // Will be set later
                    conflicts_with: None,
                    task_type: Some(TaskType::Task),
                    size_estimate: task_template.size_estimate.clone(),
                    technical_details: task_template.technical_details.clone(),
                    effort_hours: task_template.effort_hours,
                    can_parallel: Some(false), // Will be determined later
                };

                let task = self.create_task(project_id, user_id, task_input).await?;
                all_tasks.push((task_template.clone(), task));
            }
        }

        // 3. Detect dependencies and build dependency graph
        let dependency_graph = self.build_dependency_graph(&all_tasks, &title_to_id_map)?;

        // 4. Update tasks with dependencies
        for (template, task) in &all_tasks {
            if let Some(dep_titles) = &template.depends_on_titles {
                let dep_ids: Vec<String> = dep_titles
                    .iter()
                    .filter_map(|title| title_to_id_map.get(title).cloned())
                    .collect();

                if !dep_ids.is_empty() {
                    self.update_task_dependencies(&task.id, dep_ids).await?;
                }
            }
        }

        // 5. Assign parallel groups
        let parallel_groups = self.assign_parallel_groups(&all_tasks, &dependency_graph).await?;

        // 6. Detect conflicts
        let conflicts = self.detect_conflicts(&all_tasks)?;

        // 7. Reload tasks with updated dependencies
        let mut final_tasks = Vec::new();
        for (_, task) in all_tasks {
            let updated_task = self.get_task(&task.id).await?;
            final_tasks.push(updated_task);
        }

        Ok(DecompositionResult {
            tasks: final_tasks,
            dependency_graph,
            parallel_groups,
            conflicts,
        })
    }

    /// Build dependency graph from tasks
    fn build_dependency_graph(
        &self,
        tasks: &[(TaskTemplate, Task)],
        title_to_id: &std::collections::HashMap<String, String>,
    ) -> Result<DependencyGraph, StoreError> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for (_, task) in tasks {
            nodes.push(GraphNode {
                id: task.id.clone(),
                label: task.title.clone(),
            });
        }

        for (template, task) in tasks {
            if let Some(dep_titles) = &template.depends_on_titles {
                for dep_title in dep_titles {
                    if let Some(dep_id) = title_to_id.get(dep_title) {
                        edges.push(GraphEdge {
                            from: dep_id.clone(),
                            to: task.id.clone(),
                            edge_type: Some("dependency".to_string()),
                        });
                    }
                }
            }
        }

        Ok(DependencyGraph { nodes, edges })
    }

    /// Assign parallel groups to tasks that can run concurrently
    async fn assign_parallel_groups(
        &self,
        tasks: &[(TaskTemplate, Task)],
        dependency_graph: &DependencyGraph,
    ) -> Result<Vec<ParallelGroup>, StoreError> {
        let mut parallel_groups = Vec::new();
        let mut task_to_group: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        // Build dependency map
        let mut dependencies: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for edge in &dependency_graph.edges {
            dependencies
                .entry(edge.to.clone())
                .or_insert_with(Vec::new)
                .push(edge.from.clone());
        }

        // Topological sorting to assign levels (parallel groups)
        let mut level = 0;
        let mut processed: std::collections::HashSet<String> = std::collections::HashSet::new();

        loop {
            let mut current_level_tasks = Vec::new();

            for (_, task) in tasks {
                if processed.contains(&task.id) {
                    continue;
                }

                // Check if all dependencies are processed
                let deps = dependencies.get(&task.id);
                let can_process = deps.map_or(true, |d| d.iter().all(|dep| processed.contains(dep)));

                if can_process {
                    current_level_tasks.push(task.id.clone());
                }
            }

            if current_level_tasks.is_empty() {
                break;
            }

            let group_id = format!("group_{}", level);
            let group_name = format!("Parallel Group {}", level + 1);

            for task_id in &current_level_tasks {
                task_to_group.insert(task_id.clone(), level);
                processed.insert(task_id.clone());

                // Update task with parallel group
                self.update_task_parallel_group(task_id, &group_id, true)
                    .await?;
            }

            parallel_groups.push(ParallelGroup {
                id: group_id,
                name: group_name,
                task_ids: current_level_tasks,
            });

            level += 1;
        }

        Ok(parallel_groups)
    }

    /// Detect conflicts between tasks (e.g., tasks that modify the same files)
    fn detect_conflicts(
        &self,
        tasks: &[(TaskTemplate, Task)],
    ) -> Result<Vec<TaskConflict>, StoreError> {
        let mut conflicts = Vec::new();

        // For now, we'll use a simple heuristic: tasks in the same category might conflict
        // In a real implementation, this would analyze file patterns, dependencies, etc.
        let mut category_tasks: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        for (_, task) in tasks {
            if let Some(category) = &task.category {
                category_tasks
                    .entry(category.clone())
                    .or_insert_with(Vec::new)
                    .push(task.id.clone());
            }
        }

        // Mark tasks in the same category as potential conflicts
        for (category, task_ids) in category_tasks {
            if task_ids.len() > 1 {
                for i in 0..task_ids.len() {
                    for j in (i + 1)..task_ids.len() {
                        conflicts.push(TaskConflict {
                            task1: task_ids[i].clone(),
                            task2: task_ids[j].clone(),
                            reason: format!("Both tasks in category '{}'", category),
                        });
                    }
                }
            }
        }

        Ok(conflicts)
    }

    // Database helper methods

    async fn get_epic(&self, epic_id: &str) -> Result<crate::epic::Epic, StoreError> {
        let row = sqlx::query("SELECT * FROM epics WHERE id = ?")
            .bind(epic_id)
            .fetch_one(&self.pool)
            .await
            .map_err(StoreError::Sqlx)?;

        crate::epic_manager::EpicManager::new(self.pool.clone())
            .row_to_epic(&row)
            .map_err(|e| StoreError::Database(e.to_string()))
    }

    async fn create_task(
        &self,
        project_id: &str,
        user_id: &str,
        input: TaskCreateInput,
    ) -> Result<Task, StoreError> {
        let storage = tasks::storage::TaskStorage::new(self.pool.clone());
        storage.create_task(project_id, user_id, input).await
    }

    async fn get_task(&self, task_id: &str) -> Result<Task, StoreError> {
        let storage = tasks::storage::TaskStorage::new(self.pool.clone());
        storage.get_task(task_id).await
    }

    async fn update_task_dependencies(
        &self,
        task_id: &str,
        depends_on: Vec<String>,
    ) -> Result<(), StoreError> {
        let depends_on_json = serde_json::to_string(&depends_on)?;

        sqlx::query("UPDATE tasks SET depends_on = ?, updated_at = ? WHERE id = ?")
            .bind(depends_on_json)
            .bind(Utc::now())
            .bind(task_id)
            .execute(&self.pool)
            .await
            .map_err(StoreError::Sqlx)?;

        Ok(())
    }

    async fn update_task_parallel_group(
        &self,
        task_id: &str,
        parallel_group: &str,
        can_parallel: bool,
    ) -> Result<(), StoreError> {
        sqlx::query(
            "UPDATE tasks SET parallel_group = ?, can_parallel = ?, updated_at = ? WHERE id = ?",
        )
        .bind(parallel_group)
        .bind(can_parallel)
        .bind(Utc::now())
        .bind(task_id)
        .execute(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        Ok(())
    }

    /// Analyze work streams for parallel execution
    pub async fn analyze_work_streams(
        &self,
        epic_id: &str,
    ) -> Result<WorkAnalysis, StoreError> {
        // Get all tasks for the epic
        let tasks = self.get_epic_tasks(epic_id).await?;

        // Identify work streams based on task categories
        let streams = self.identify_work_streams(&tasks)?;

        // Build dependency graph
        let dependency_graph = self.build_task_dependency_graph(&tasks)?;

        // Detect conflicts
        let conflict_analysis = Some(ConflictAnalysis {
            conflicts: self.detect_task_conflicts(&tasks)?,
        });

        // Generate parallelization strategy
        let parallelization_strategy = self.generate_parallelization_strategy(&streams)?;

        // Calculate confidence score
        let confidence_score = self.calculate_confidence_score(&tasks, &streams)?;

        // Create work analysis record
        let analysis_id = nanoid::nanoid!();
        let now = Utc::now();

        let analysis = WorkAnalysis {
            id: analysis_id.clone(),
            epic_id: epic_id.to_string(),
            parallel_streams: streams.clone(),
            file_patterns: None,
            dependency_graph: dependency_graph.clone(),
            conflict_analysis,
            parallelization_strategy: Some(parallelization_strategy),
            analyzed_at: now,
            is_current: true,
            analysis_version: 1,
            confidence_score: Some(confidence_score),
        };

        // Save to database
        self.save_work_analysis(&analysis).await?;

        Ok(analysis)
    }

    fn identify_work_streams(&self, tasks: &[Task]) -> Result<Vec<WorkStream>, StoreError> {
        let mut streams = Vec::new();
        let mut category_tasks: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();

        // Group tasks by category
        for task in tasks {
            if let Some(category) = &task.category {
                category_tasks
                    .entry(category.clone())
                    .or_insert_with(Vec::new)
                    .push(task.id.clone());
            }
        }

        // Create work streams from categories
        for (category, task_ids) in category_tasks {
            streams.push(WorkStream {
                name: category.clone(),
                description: format!("Tasks in {} category", category),
                tasks: task_ids,
                file_patterns: None,
            });
        }

        Ok(streams)
    }

    fn build_task_dependency_graph(&self, tasks: &[Task]) -> Result<DependencyGraph, StoreError> {
        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        for task in tasks {
            nodes.push(GraphNode {
                id: task.id.clone(),
                label: task.title.clone(),
            });

            if let Some(depends_on) = &task.depends_on {
                for dep_id in depends_on {
                    edges.push(GraphEdge {
                        from: dep_id.clone(),
                        to: task.id.clone(),
                        edge_type: Some("dependency".to_string()),
                    });
                }
            }
        }

        Ok(DependencyGraph { nodes, edges })
    }

    fn detect_task_conflicts(&self, tasks: &[Task]) -> Result<Vec<TaskConflict>, StoreError> {
        let mut conflicts = Vec::new();

        // Check for conflicts specified in tasks
        for task in tasks {
            if let Some(conflicts_with) = &task.conflicts_with {
                for conflict_id in conflicts_with {
                    conflicts.push(TaskConflict {
                        task1: task.id.clone(),
                        task2: conflict_id.clone(),
                        reason: "Explicit conflict marker".to_string(),
                    });
                }
            }
        }

        Ok(conflicts)
    }

    fn generate_parallelization_strategy(
        &self,
        streams: &[WorkStream],
    ) -> Result<String, StoreError> {
        let mut strategy = String::from("Parallelization Strategy:\n\n");

        for (i, stream) in streams.iter().enumerate() {
            strategy.push_str(&format!(
                "{}. {} ({}): {} tasks\n",
                i + 1,
                stream.name,
                stream.description,
                stream.tasks.len()
            ));
        }

        strategy.push_str("\nRecommendations:\n");
        strategy.push_str("- Execute work streams in parallel where possible\n");
        strategy.push_str("- Monitor for file conflicts between concurrent tasks\n");
        strategy.push_str("- Consider task dependencies when scheduling\n");

        Ok(strategy)
    }

    fn calculate_confidence_score(
        &self,
        tasks: &[Task],
        streams: &[WorkStream],
    ) -> Result<f64, StoreError> {
        // Simple confidence calculation based on:
        // - Number of tasks with dependencies defined
        // - Number of tasks with size estimates
        // - Distribution across work streams

        let total_tasks = tasks.len() as f64;
        if total_tasks == 0.0 {
            return Ok(0.0);
        }

        let tasks_with_deps = tasks
            .iter()
            .filter(|t| t.depends_on.is_some() && !t.depends_on.as_ref().unwrap().is_empty())
            .count() as f64;

        let tasks_with_estimates = tasks
            .iter()
            .filter(|t| t.size_estimate.is_some())
            .count() as f64;

        let stream_distribution = if streams.len() > 1 {
            1.0
        } else {
            0.5
        };

        let confidence = ((tasks_with_deps / total_tasks * 0.4)
            + (tasks_with_estimates / total_tasks * 0.4)
            + (stream_distribution * 0.2))
            .min(1.0);

        Ok(confidence)
    }

    async fn get_epic_tasks(&self, epic_id: &str) -> Result<Vec<Task>, StoreError> {
        let rows = sqlx::query("SELECT * FROM tasks WHERE epic_id = ? ORDER BY position, created_at")
            .bind(epic_id)
            .fetch_all(&self.pool)
            .await
            .map_err(StoreError::Sqlx)?;

        let _storage = tasks::storage::TaskStorage::new(self.pool.clone());
        rows.iter()
            .map(|row| storage::row_to_task_result(row))
            .collect::<Result<Vec<Task>, StoreError>>()
    }

    async fn save_work_analysis(&self, analysis: &WorkAnalysis) -> Result<(), StoreError> {
        // Mark previous analyses as not current
        sqlx::query("UPDATE work_analysis SET is_current = FALSE WHERE epic_id = ?")
            .bind(&analysis.epic_id)
            .execute(&self.pool)
            .await
            .map_err(StoreError::Sqlx)?;

        // Insert new analysis
        sqlx::query(
            r#"
            INSERT INTO work_analysis (
                id, epic_id, parallel_streams, file_patterns, dependency_graph,
                conflict_analysis, parallelization_strategy, analyzed_at,
                is_current, analysis_version, confidence_score
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&analysis.id)
        .bind(&analysis.epic_id)
        .bind(serde_json::to_string(&analysis.parallel_streams).unwrap())
        .bind(
            analysis
                .file_patterns
                .as_ref()
                .map(|fp| serde_json::to_string(fp).unwrap()),
        )
        .bind(serde_json::to_string(&analysis.dependency_graph).unwrap())
        .bind(
            analysis
                .conflict_analysis
                .as_ref()
                .map(|ca| serde_json::to_string(ca).unwrap()),
        )
        .bind(&analysis.parallelization_strategy)
        .bind(analysis.analyzed_at)
        .bind(analysis.is_current)
        .bind(analysis.analysis_version)
        .bind(analysis.confidence_score)
        .execute(&self.pool)
        .await
        .map_err(StoreError::Sqlx)?;

        Ok(())
    }
}

// Helper function to convert row to Task (needed since TaskStorage methods are private)
mod storage {
    use super::*;
    use sqlx::Row;

    pub fn row_to_task_result(row: &sqlx::sqlite::SqliteRow) -> Result<Task, StoreError> {
        Ok(Task {
            id: row.try_get("id")?,
            project_id: row.try_get("project_id")?,
            title: row.try_get("title")?,
            description: row.try_get("description")?,
            status: row.try_get("status")?,
            priority: row.try_get("priority")?,
            created_by_user_id: row.try_get("created_by_user_id")?,
            assigned_agent_id: row.try_get("assigned_agent_id")?,
            reviewed_by_agent_id: row.try_get("reviewed_by_agent_id")?,
            parent_id: row.try_get("parent_id")?,
            position: row.try_get("position")?,
            subtasks: None,
            dependencies: row
                .try_get::<Option<String>, _>("dependencies")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            blockers: row
                .try_get::<Option<String>, _>("blockers")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            due_date: row.try_get("due_date")?,
            estimated_hours: row.try_get("estimated_hours")?,
            actual_hours: row.try_get("actual_hours")?,
            complexity_score: row.try_get("complexity_score")?,
            details: row.try_get("details")?,
            test_strategy: row.try_get("test_strategy")?,
            acceptance_criteria: row.try_get("acceptance_criteria")?,
            prompt: row.try_get("prompt")?,
            context: row.try_get("context")?,
            output_format: row.try_get("output_format")?,
            validation_rules: row
                .try_get::<Option<String>, _>("validation_rules")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            started_at: row.try_get("started_at")?,
            completed_at: row.try_get("completed_at")?,
            execution_log: row
                .try_get::<Option<String>, _>("execution_log")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            error_log: row
                .try_get::<Option<String>, _>("error_log")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            retry_count: row.try_get("retry_count")?,
            tags: row
                .try_get::<Option<String>, _>("tags")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            category: row.try_get("category")?,
            metadata: row
                .try_get::<Option<String>, _>("metadata")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
            epic_id: row.try_get("epic_id")?,
            github_issue_number: row.try_get("github_issue_number")?,
            github_issue_url: row.try_get("github_issue_url")?,
            parallel_group: row.try_get("parallel_group")?,
            depends_on: row
                .try_get::<Option<String>, _>("depends_on")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            conflicts_with: row
                .try_get::<Option<String>, _>("conflicts_with")?
                .and_then(|s| serde_json::from_str(&s).ok()),
            task_type: row.try_get("task_type")?,
            size_estimate: row.try_get("size_estimate")?,
            technical_details: row.try_get("technical_details")?,
            effort_hours: row.try_get("effort_hours")?,
            can_parallel: row.try_get("can_parallel")?,
        })
    }
}
