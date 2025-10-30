// ABOUTME: Build order optimization using graph algorithms
// ABOUTME: Computes optimal feature build sequences, identifies parallel work, and detects circular dependencies

use crate::dependency_analyzer::{DependencyStrength, FeatureDependency};
use crate::error::{IdeateError, Result};
use crate::types::IdeateFeature;
use chrono::Utc;
use petgraph::algo::{is_cyclic_directed, toposort};
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::{info, warn};

/// Type alias for dependency graph tuple
type DependencyGraph = (
    DiGraph<String, ()>,
    HashMap<String, NodeIndex>,
    HashMap<NodeIndex, String>,
);

/// Optimization strategy for build order
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OptimizationStrategy {
    /// Minimize total time (maximize parallelism)
    Fastest,
    /// Balance between speed and risk
    Balanced,
    /// Minimize risk (more sequential, less parallelism)
    Safest,
}

/// Result of build order optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOrderResult {
    pub session_id: String,
    pub build_sequence: Vec<String>, // Feature IDs in optimal order
    pub parallel_groups: Vec<Vec<String>>, // Groups of features that can be built in parallel
    pub critical_path: Vec<String>,  // Feature IDs on the critical path
    pub estimated_phases: usize,
    pub optimization_strategy: OptimizationStrategy,
    pub computed_at: chrono::DateTime<Utc>,
    pub is_valid: bool,
}

/// Circular dependency detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircularDependency {
    pub id: String,
    pub session_id: String,
    pub cycle_path: Vec<String>, // Feature IDs forming the cycle
    pub severity: CircularDependencySeverity,
    pub detected_at: chrono::DateTime<Utc>,
    pub resolved: bool,
    pub resolution_note: Option<String>,
}

/// Severity level for circular dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CircularDependencySeverity {
    Warning,
    Error,
    Critical,
}

/// Build order optimizer
pub struct BuildOptimizer {
    db: SqlitePool,
}

impl BuildOptimizer {
    pub fn new(db: SqlitePool) -> Self {
        Self { db }
    }

    /// Optimize build order for a session
    pub async fn optimize(
        &self,
        session_id: &str,
        strategy: OptimizationStrategy,
    ) -> Result<BuildOrderResult> {
        info!(
            "Optimizing build order for session: {} (strategy: {:?})",
            session_id, strategy
        );

        // Get features and dependencies
        let features = self.get_features(session_id).await?;
        let dependencies = self.get_dependencies(session_id).await?;

        if features.is_empty() {
            return Ok(BuildOrderResult {
                session_id: session_id.to_string(),
                build_sequence: vec![],
                parallel_groups: vec![],
                critical_path: vec![],
                estimated_phases: 0,
                optimization_strategy: strategy,
                computed_at: Utc::now(),
                is_valid: true,
            });
        }

        // Build dependency graph
        let (graph, node_map, reverse_map) = self.build_graph(&features, &dependencies)?;

        // Check for circular dependencies
        let cycles = self.detect_cycles(&graph, &reverse_map)?;
        if !cycles.is_empty() {
            warn!("Circular dependencies detected: {} cycles", cycles.len());
            // Store cycles in database
            for cycle in &cycles {
                self.store_circular_dependency(session_id, cycle).await?;
            }
            return Err(IdeateError::ValidationError(format!(
                "Circular dependencies detected: {} cycles found",
                cycles.len()
            )));
        }

        // Compute topological sort
        let topo_order = self.topological_sort(&graph, &reverse_map)?;

        // Identify parallel groups
        let parallel_groups =
            self.identify_parallel_groups(&graph, &topo_order, &reverse_map, strategy)?;

        // Compute critical path
        let critical_path = self.compute_critical_path(&graph, &node_map, &reverse_map)?;

        let result = BuildOrderResult {
            session_id: session_id.to_string(),
            build_sequence: topo_order,
            parallel_groups: parallel_groups.clone(),
            critical_path,
            estimated_phases: parallel_groups.len(),
            optimization_strategy: strategy,
            computed_at: Utc::now(),
            is_valid: true,
        };

        // Store result in database
        self.store_build_order(&result).await?;

        info!(
            "Build order optimization complete: {} features in {} phases",
            result.build_sequence.len(),
            result.estimated_phases
        );

        Ok(result)
    }

    /// Build dependency graph using petgraph
    fn build_graph(
        &self,
        features: &[IdeateFeature],
        dependencies: &[FeatureDependency],
    ) -> Result<DependencyGraph> {
        let mut graph = DiGraph::new();
        let mut node_map = HashMap::new();
        let mut reverse_map = HashMap::new();

        // Add nodes for all features
        for feature in features {
            let node_idx = graph.add_node(feature.id.clone());
            node_map.insert(feature.id.clone(), node_idx);
            reverse_map.insert(node_idx, feature.id.clone());
        }

        // Add edges for dependencies (only required and recommended)
        for dep in dependencies {
            if dep.strength == DependencyStrength::Optional {
                continue; // Skip optional dependencies for build order
            }

            if let (Some(&from_idx), Some(&to_idx)) = (
                node_map.get(&dep.from_feature_id),
                node_map.get(&dep.to_feature_id),
            ) {
                // Edge goes from dependency TO dependent (reverse of logical direction)
                graph.add_edge(to_idx, from_idx, ());
            }
        }

        Ok((graph, node_map, reverse_map))
    }

    /// Detect circular dependencies
    fn detect_cycles(
        &self,
        graph: &DiGraph<String, ()>,
        reverse_map: &HashMap<NodeIndex, String>,
    ) -> Result<Vec<Vec<String>>> {
        if !is_cyclic_directed(graph) {
            return Ok(vec![]);
        }

        // Find all cycles using DFS
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = Vec::new();

        for node in graph.node_indices() {
            if !visited.contains(&node) {
                Self::find_cycles_dfs(
                    graph,
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut cycles,
                    reverse_map,
                )?;
            }
        }

        Ok(cycles)
    }

    /// DFS helper for cycle detection
    fn find_cycles_dfs(
        graph: &DiGraph<String, ()>,
        node: NodeIndex,
        visited: &mut HashSet<NodeIndex>,
        rec_stack: &mut Vec<NodeIndex>,
        cycles: &mut Vec<Vec<String>>,
        reverse_map: &HashMap<NodeIndex, String>,
    ) -> Result<()> {
        visited.insert(node);
        rec_stack.push(node);

        for neighbor in graph.neighbors(node) {
            if !visited.contains(&neighbor) {
                Self::find_cycles_dfs(graph, neighbor, visited, rec_stack, cycles, reverse_map)?;
            } else if rec_stack.contains(&neighbor) {
                // Found a cycle
                let cycle_start = rec_stack
                    .iter()
                    .position(|&n| n == neighbor)
                    .ok_or_else(|| {
                        IdeateError::ValidationError(
                            "Cycle detection failed: neighbor not found in recursion stack"
                                .to_string(),
                        )
                    })?;
                let cycle: Vec<String> = rec_stack[cycle_start..]
                    .iter()
                    .filter_map(|idx| reverse_map.get(idx).cloned())
                    .collect();
                cycles.push(cycle);
            }
        }

        rec_stack.pop();
        Ok(())
    }

    /// Perform topological sort
    fn topological_sort(
        &self,
        graph: &DiGraph<String, ()>,
        reverse_map: &HashMap<NodeIndex, String>,
    ) -> Result<Vec<String>> {
        let sorted = toposort(graph, None).map_err(|_| {
            IdeateError::ValidationError(
                "Circular dependency detected during topological sort".to_string(),
            )
        })?;

        let feature_ids: Vec<String> = sorted
            .into_iter()
            .filter_map(|idx| reverse_map.get(&idx).cloned())
            .collect();

        Ok(feature_ids)
    }

    /// Identify groups of features that can be built in parallel
    fn identify_parallel_groups(
        &self,
        graph: &DiGraph<String, ()>,
        topo_order: &[String],
        reverse_map: &HashMap<NodeIndex, String>,
        strategy: OptimizationStrategy,
    ) -> Result<Vec<Vec<String>>> {
        let node_map: HashMap<String, NodeIndex> = reverse_map
            .iter()
            .map(|(idx, id)| (id.clone(), *idx))
            .collect();

        let mut groups = Vec::new();
        let mut completed = HashSet::new();
        let mut remaining: VecDeque<String> = topo_order.iter().cloned().collect();

        while !remaining.is_empty() {
            let mut current_group = Vec::new();

            // Find features with no incomplete dependencies
            let candidates: Vec<String> = remaining
                .iter()
                .filter(|feature_id| {
                    if let Some(&node_idx) = node_map.get(*feature_id) {
                        // Check if all dependencies are completed
                        let deps_complete = graph
                            .neighbors_directed(node_idx, Direction::Incoming)
                            .all(|dep_idx| {
                                reverse_map
                                    .get(&dep_idx)
                                    .map(|id| completed.contains(id))
                                    .unwrap_or(false)
                            });
                        deps_complete
                    } else {
                        false
                    }
                })
                .cloned()
                .collect();

            if candidates.is_empty() {
                break; // Should not happen if topological sort is correct
            }

            // Apply strategy to determine group size
            let group_size = match strategy {
                OptimizationStrategy::Fastest => candidates.len(), // All candidates in parallel
                OptimizationStrategy::Balanced => (candidates.len() / 2).max(1), // Half in parallel
                OptimizationStrategy::Safest => 1,                 // One at a time
            };

            // Take candidates for this group
            for feature_id in candidates.into_iter().take(group_size) {
                current_group.push(feature_id.clone());
                completed.insert(feature_id.clone());
                remaining.retain(|id| id != &feature_id);
            }

            if !current_group.is_empty() {
                groups.push(current_group);
            }
        }

        Ok(groups)
    }

    /// Compute critical path (longest path through the graph)
    fn compute_critical_path(
        &self,
        graph: &DiGraph<String, ()>,
        _node_map: &HashMap<String, NodeIndex>,
        reverse_map: &HashMap<NodeIndex, String>,
    ) -> Result<Vec<String>> {
        if graph.node_count() == 0 {
            return Ok(vec![]);
        }

        // Compute longest path from each node
        let mut longest_path = HashMap::new();
        let mut predecessor = HashMap::new();

        // Initialize all nodes to 0
        for node in graph.node_indices() {
            longest_path.insert(node, 0);
        }

        // Topological sort for processing order
        let sorted = toposort(graph, None).map_err(|_| {
            IdeateError::ValidationError("Cannot compute critical path with cycles".to_string())
        })?;

        // Compute longest paths
        for &node in &sorted {
            let node_dist = *longest_path.get(&node).unwrap_or(&0);
            for neighbor in graph.neighbors(node) {
                let new_dist = node_dist + 1;
                let current_dist = *longest_path.get(&neighbor).unwrap_or(&0);
                if new_dist > current_dist {
                    longest_path.insert(neighbor, new_dist);
                    predecessor.insert(neighbor, node);
                }
            }
        }

        // Find node with maximum distance (end of critical path)
        let end_node = longest_path
            .iter()
            .max_by_key(|(_, &dist)| dist)
            .map(|(&node, _)| node);

        if let Some(mut node) = end_node {
            // Reconstruct path
            let mut path = vec![node];
            while let Some(&pred) = predecessor.get(&node) {
                path.push(pred);
                node = pred;
            }
            path.reverse();

            // Convert to feature IDs
            let feature_path: Vec<String> = path
                .into_iter()
                .filter_map(|idx| reverse_map.get(&idx).cloned())
                .collect();

            Ok(feature_path)
        } else {
            Ok(vec![])
        }
    }

    /// Get features for a session
    async fn get_features(&self, session_id: &str) -> Result<Vec<IdeateFeature>> {
        let rows = sqlx::query(
            "SELECT id, session_id, feature_name, what_it_does, why_important, how_it_works,
                    depends_on, enables, build_phase, is_visible, created_at
             FROM ideate_features
             WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        let features = rows
            .into_iter()
            .map(|row| IdeateFeature {
                id: row.get("id"),
                session_id: row.get("session_id"),
                feature_name: row.get("feature_name"),
                what_it_does: row.get("what_it_does"),
                why_important: row.get("why_important"),
                how_it_works: row.get("how_it_works"),
                depends_on: row
                    .get::<Option<String>, _>("depends_on")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                enables: row
                    .get::<Option<String>, _>("enables")
                    .and_then(|s| serde_json::from_str(&s).ok()),
                build_phase: row.get("build_phase"),
                is_visible: row.get::<i32, _>("is_visible") != 0,
                created_at: row.get("created_at"),
            })
            .collect();

        Ok(features)
    }

    /// Get dependencies for a session
    async fn get_dependencies(&self, session_id: &str) -> Result<Vec<FeatureDependency>> {
        let rows = sqlx::query(
            "SELECT id, session_id, from_feature_id, to_feature_id, dependency_type, strength, reason, auto_detected
             FROM feature_dependencies
             WHERE session_id = $1",
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        let dependencies = rows
            .into_iter()
            .map(|row| {
                let strength_str: String = row.get("strength");
                let strength = match strength_str.as_str() {
                    "required" => DependencyStrength::Required,
                    "recommended" => DependencyStrength::Recommended,
                    "optional" => DependencyStrength::Optional,
                    _ => DependencyStrength::Required,
                };

                FeatureDependency {
                    id: row.get("id"),
                    session_id: row.get("session_id"),
                    from_feature_id: row.get("from_feature_id"),
                    to_feature_id: row.get("to_feature_id"),
                    dependency_type: row.get("dependency_type"),
                    strength,
                    reason: row.get("reason"),
                    auto_detected: row.get::<i32, _>("auto_detected") != 0,
                }
            })
            .collect();

        Ok(dependencies)
    }

    /// Store build order result
    async fn store_build_order(&self, result: &BuildOrderResult) -> Result<()> {
        let id = nanoid::nanoid!(8);
        let build_sequence_json = serde_json::to_string(&result.build_sequence)?;
        let parallel_groups_json = serde_json::to_string(&result.parallel_groups)?;
        let critical_path_json = serde_json::to_string(&result.critical_path)?;

        let strategy_str = match result.optimization_strategy {
            OptimizationStrategy::Fastest => "fastest",
            OptimizationStrategy::Balanced => "balanced",
            OptimizationStrategy::Safest => "safest",
        };

        // Invalidate old optimizations
        sqlx::query("UPDATE build_order_optimization SET is_valid = 0 WHERE session_id = $1")
            .bind(&result.session_id)
            .execute(&self.db)
            .await?;

        // Insert new optimization
        sqlx::query(
            "INSERT INTO build_order_optimization
             (id, session_id, build_sequence, parallel_groups, critical_path, estimated_phases, optimization_strategy, computed_at, is_valid, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
        )
        .bind(&id)
        .bind(&result.session_id)
        .bind(&build_sequence_json)
        .bind(&parallel_groups_json)
        .bind(&critical_path_json)
        .bind(result.estimated_phases as i32)
        .bind(strategy_str)
        .bind(result.computed_at)
        .bind(result.is_valid)
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Store circular dependency
    async fn store_circular_dependency(&self, session_id: &str, cycle: &[String]) -> Result<()> {
        let id = nanoid::nanoid!(8);
        let cycle_json = serde_json::to_string(cycle)?;

        // Determine severity based on cycle length
        let severity = if cycle.len() == 2 {
            "error"
        } else if cycle.len() > 4 {
            "critical"
        } else {
            "warning"
        };

        sqlx::query(
            "INSERT INTO circular_dependencies
             (id, session_id, cycle_path, severity, detected_at, resolved, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)
             ON CONFLICT DO NOTHING",
        )
        .bind(&id)
        .bind(session_id)
        .bind(&cycle_json)
        .bind(severity)
        .bind(Utc::now())
        .bind(0) // not resolved
        .bind(Utc::now())
        .execute(&self.db)
        .await?;

        Ok(())
    }

    /// Get latest valid build order
    pub async fn get_build_order(&self, session_id: &str) -> Result<Option<BuildOrderResult>> {
        let row = sqlx::query(
            "SELECT id, session_id, build_sequence, parallel_groups, critical_path, estimated_phases, optimization_strategy, computed_at, is_valid
             FROM build_order_optimization
             WHERE session_id = $1 AND is_valid = 1
             ORDER BY computed_at DESC
             LIMIT 1",
        )
        .bind(session_id)
        .fetch_optional(&self.db)
        .await?;

        if let Some(row) = row {
            let build_sequence: Vec<String> =
                serde_json::from_str(&row.get::<String, _>("build_sequence"))?;
            let parallel_groups: Vec<Vec<String>> =
                serde_json::from_str(&row.get::<String, _>("parallel_groups"))?;
            let critical_path: Vec<String> =
                serde_json::from_str(&row.get::<String, _>("critical_path"))?;

            let strategy_str: String = row.get("optimization_strategy");
            let strategy = match strategy_str.as_str() {
                "fastest" => OptimizationStrategy::Fastest,
                "balanced" => OptimizationStrategy::Balanced,
                "safest" => OptimizationStrategy::Safest,
                _ => OptimizationStrategy::Balanced,
            };

            return Ok(Some(BuildOrderResult {
                session_id: row.get("session_id"),
                build_sequence,
                parallel_groups,
                critical_path,
                estimated_phases: row.get::<i32, _>("estimated_phases") as usize,
                optimization_strategy: strategy,
                computed_at: row.get("computed_at"),
                is_valid: row.get::<i32, _>("is_valid") != 0,
            }));
        }

        Ok(None)
    }

    /// Get circular dependencies for a session
    pub async fn get_circular_dependencies(
        &self,
        session_id: &str,
    ) -> Result<Vec<CircularDependency>> {
        let rows = sqlx::query(
            "SELECT id, session_id, cycle_path, severity, detected_at, resolved, resolution_note
             FROM circular_dependencies
             WHERE session_id = $1 AND resolved = 0
             ORDER BY detected_at DESC",
        )
        .bind(session_id)
        .fetch_all(&self.db)
        .await?;

        let cycles = rows
            .into_iter()
            .map(|row| {
                let cycle_path: Vec<String> =
                    serde_json::from_str(&row.get::<String, _>("cycle_path")).unwrap_or_default();
                let severity_str: String = row.get("severity");
                let severity = match severity_str.as_str() {
                    "warning" => CircularDependencySeverity::Warning,
                    "error" => CircularDependencySeverity::Error,
                    "critical" => CircularDependencySeverity::Critical,
                    _ => CircularDependencySeverity::Error,
                };

                CircularDependency {
                    id: row.get("id"),
                    session_id: row.get("session_id"),
                    cycle_path,
                    severity,
                    detected_at: row.get("detected_at"),
                    resolved: row.get::<i32, _>("resolved") != 0,
                    resolution_note: row.get("resolution_note"),
                }
            })
            .collect();

        Ok(cycles)
    }
}
