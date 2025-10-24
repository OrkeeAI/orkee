// ABOUTME: HTTP handlers for graph API endpoints providing code visualization data.
// ABOUTME: Generates dependency, symbol, module, and spec-mapping graphs for projects.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

use crate::{
    context::{graph_builder::GraphBuilder, graph_types::CodeGraph},
    db::DbState,
    manager::get_project as manager_get_project,
};

// Timeout configuration
const GRAPH_GENERATION_TIMEOUT_SECS: u64 = 30;

// Validation constants
const MAX_DEPTH: usize = 10;
const DEFAULT_DEPTH: usize = 3;
const MAX_FILTER_LENGTH: usize = 256;
const ALLOWED_LAYOUTS: &[&str] = &["hierarchical", "force", "circular", "grid"];
const DEFAULT_LAYOUT: &str = "hierarchical";

/// Query parameters for graph requests
#[derive(Debug, Deserialize)]
pub struct GraphQuery {
    #[serde(default)]
    pub max_depth: Option<usize>,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub layout: Option<String>,
}

/// Validated query parameters with guaranteed safe values
#[derive(Debug)]
pub struct ValidatedGraphQuery {
    pub max_depth: usize,
    pub filter: Option<String>,
    pub layout: String,
}

impl GraphQuery {
    /// Validate and normalize query parameters
    pub fn validate(self) -> ValidatedGraphQuery {
        let max_depth = self.max_depth.unwrap_or(DEFAULT_DEPTH).min(MAX_DEPTH);

        let filter = self.filter.and_then(|f| {
            if f.len() <= MAX_FILTER_LENGTH {
                Some(f)
            } else {
                None
            }
        });

        let layout = self
            .layout
            .and_then(|l| {
                if ALLOWED_LAYOUTS.contains(&l.as_str()) {
                    Some(l)
                } else {
                    None
                }
            })
            .unwrap_or_else(|| DEFAULT_LAYOUT.to_string());

        ValidatedGraphQuery {
            max_depth,
            filter,
            layout,
        }
    }

    /// Validate filter parameter (for testing)
    #[cfg(test)]
    fn validate_filter(&self) -> Result<(), String> {
        if let Some(filter) = &self.filter {
            if filter.len() > MAX_FILTER_LENGTH {
                return Err(format!(
                    "Filter string too long (max {} characters)",
                    MAX_FILTER_LENGTH
                ));
            }
        }
        Ok(())
    }

    /// Validate layout parameter (for testing)
    #[cfg(test)]
    fn validate_layout(&self) -> Result<String, String> {
        if let Some(layout) = &self.layout {
            if !ALLOWED_LAYOUTS.contains(&layout.as_str()) {
                return Err(format!(
                    "Invalid layout '{}'. Allowed: {}",
                    layout,
                    ALLOWED_LAYOUTS.join(", ")
                ));
            }
            Ok(layout.clone())
        } else {
            Ok(DEFAULT_LAYOUT.to_string())
        }
    }
}

/// Response format for graph API
#[derive(Debug, Serialize)]
pub struct GraphResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<CodeGraph>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl GraphResponse {
    fn success(data: CodeGraph) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Get dependency graph for a project
pub async fn get_dependency_graph(
    Path(project_id): Path<String>,
    Query(_params): Query<GraphQuery>,
    State(_db): State<DbState>,
) -> Json<GraphResponse> {
    info!("Generating dependency graph for project: {}", project_id);

    // Fetch project from database
    let project = match manager_get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Json(GraphResponse::error(format!(
                "Project not found: {}",
                project_id
            )))
        }
        Err(e) => {
            return Json(GraphResponse::error(format!(
                "Failed to fetch project: {}",
                e
            )))
        }
    };

    // Generate real graph using GraphBuilder with timeout protection
    let project_root = project.project_root.clone();
    let project_id_clone = project_id.clone();

    let result = timeout(
        Duration::from_secs(GRAPH_GENERATION_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            let mut builder = GraphBuilder::new();
            builder.build_dependency_graph(&project_root, &project_id_clone)
        }),
    )
    .await;

    match result {
        Ok(Ok(Ok(graph))) => {
            info!(
                "Generated dependency graph with {} nodes and {} edges",
                graph.metadata.total_nodes, graph.metadata.total_edges
            );
            Json(GraphResponse::success(graph))
        }
        Ok(Ok(Err(e))) => Json(GraphResponse::error(format!(
            "Failed to generate dependency graph: {}",
            e
        ))),
        Ok(Err(e)) => Json(GraphResponse::error(format!(
            "Graph generation task failed: {}",
            e
        ))),
        Err(_) => Json(GraphResponse::error(format!(
            "Graph generation timed out after {} seconds",
            GRAPH_GENERATION_TIMEOUT_SECS
        ))),
    }
}

/// Get symbol graph for a project
pub async fn get_symbol_graph(
    Path(project_id): Path<String>,
    Query(_params): Query<GraphQuery>,
    State(_db): State<DbState>,
) -> Json<GraphResponse> {
    info!("Generating symbol graph for project: {}", project_id);

    // Fetch project from database
    let project = match manager_get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Json(GraphResponse::error(format!(
                "Project not found: {}",
                project_id
            )))
        }
        Err(e) => {
            return Json(GraphResponse::error(format!(
                "Failed to fetch project: {}",
                e
            )))
        }
    };

    // Generate real graph using GraphBuilder with timeout protection
    let project_root = project.project_root.clone();
    let project_id_clone = project_id.clone();

    let result = timeout(
        Duration::from_secs(GRAPH_GENERATION_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            let mut builder = GraphBuilder::new();
            builder.build_symbol_graph(&project_root, &project_id_clone)
        }),
    )
    .await;

    match result {
        Ok(Ok(Ok(graph))) => {
            info!(
                "Generated symbol graph with {} nodes and {} edges",
                graph.metadata.total_nodes, graph.metadata.total_edges
            );
            Json(GraphResponse::success(graph))
        }
        Ok(Ok(Err(e))) => Json(GraphResponse::error(format!(
            "Failed to generate symbol graph: {}",
            e
        ))),
        Ok(Err(e)) => Json(GraphResponse::error(format!(
            "Graph generation task failed: {}",
            e
        ))),
        Err(_) => Json(GraphResponse::error(format!(
            "Graph generation timed out after {} seconds",
            GRAPH_GENERATION_TIMEOUT_SECS
        ))),
    }
}

/// Get module graph for a project
pub async fn get_module_graph(
    Path(project_id): Path<String>,
    Query(_params): Query<GraphQuery>,
    State(_db): State<DbState>,
) -> Json<GraphResponse> {
    info!("Generating module graph for project: {}", project_id);

    // Fetch project from database
    let project = match manager_get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Json(GraphResponse::error(format!(
                "Project not found: {}",
                project_id
            )))
        }
        Err(e) => {
            return Json(GraphResponse::error(format!(
                "Failed to fetch project: {}",
                e
            )))
        }
    };

    // Generate real graph using GraphBuilder with timeout protection
    let project_root = project.project_root.clone();
    let project_id_clone = project_id.clone();

    let result = timeout(
        Duration::from_secs(GRAPH_GENERATION_TIMEOUT_SECS),
        tokio::task::spawn_blocking(move || {
            let builder = GraphBuilder::new();
            builder.build_module_graph(&project_root, &project_id_clone)
        }),
    )
    .await;

    match result {
        Ok(Ok(Ok(graph))) => {
            info!(
                "Generated module graph with {} nodes and {} edges",
                graph.metadata.total_nodes, graph.metadata.total_edges
            );
            Json(GraphResponse::success(graph))
        }
        Ok(Ok(Err(e))) => Json(GraphResponse::error(format!(
            "Failed to generate module graph: {}",
            e
        ))),
        Ok(Err(e)) => Json(GraphResponse::error(format!(
            "Graph generation task failed: {}",
            e
        ))),
        Err(_) => Json(GraphResponse::error(format!(
            "Graph generation timed out after {} seconds",
            GRAPH_GENERATION_TIMEOUT_SECS
        ))),
    }
}

/// Get spec-mapping graph for a project (placeholder for future implementation)
pub async fn get_spec_mapping_graph(
    Path(project_id): Path<String>,
    Query(_params): Query<GraphQuery>,
    State(_db): State<DbState>,
) -> Json<GraphResponse> {
    info!("Generating spec-mapping graph for project: {}", project_id);

    // Placeholder: This will be implemented when OpenSpec integration is ready
    Json(GraphResponse::error(
        "Spec-mapping graph not yet implemented".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_response_success() {
        use crate::context::graph_types::{CodeGraph, GraphMetadata};
        use chrono::Utc;

        let graph = CodeGraph {
            nodes: vec![],
            edges: vec![],
            metadata: GraphMetadata {
                total_nodes: 0,
                total_edges: 0,
                graph_type: "test".to_string(),
                generated_at: Utc::now(),
                project_id: "test-project".to_string(),
            },
        };

        let response = GraphResponse::success(graph);
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_graph_response_error() {
        let response = GraphResponse::error("Test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("Test error".to_string()));
    }

    #[test]
    fn test_graph_query_deserialization() {
        let json = r#"{"max_depth": 5, "filter": "*.ts"}"#;
        let query: GraphQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.max_depth, Some(5));
        assert_eq!(query.filter, Some("*.ts".to_string()));
    }

    #[test]
    fn test_graph_query_validation_max_depth() {
        // Valid depth
        let query = GraphQuery {
            max_depth: Some(5),
            filter: None,
            layout: None,
        };
        let validated = query.validate();
        assert_eq!(validated.max_depth, 5);

        // Exceeds maximum
        let query = GraphQuery {
            max_depth: Some(15),
            filter: None,
            layout: None,
        };
        let validated = query.validate();
        assert_eq!(validated.max_depth, 10); // Should be capped at MAX_DEPTH

        // None defaults to DEFAULT_DEPTH
        let query = GraphQuery {
            max_depth: None,
            filter: None,
            layout: None,
        };
        let validated = query.validate();
        assert_eq!(validated.max_depth, 3);
    }

    #[test]
    fn test_graph_query_validation_filter() {
        // Valid filter
        let query = GraphQuery {
            max_depth: None,
            filter: Some("*.ts".to_string()),
            layout: None,
        };
        let result = query.validate_filter();
        assert!(result.is_ok());

        // Filter too long
        let long_filter = "a".repeat(300);
        let query = GraphQuery {
            max_depth: None,
            filter: Some(long_filter),
            layout: None,
        };
        let result = query.validate_filter();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Filter string too long"));
    }

    #[test]
    fn test_graph_query_validation_layout() {
        // Valid layout
        let query = GraphQuery {
            max_depth: None,
            filter: None,
            layout: Some("hierarchical".to_string()),
        };
        let result = query.validate_layout();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hierarchical");

        // Invalid layout
        let query = GraphQuery {
            max_depth: None,
            filter: None,
            layout: Some("invalid".to_string()),
        };
        let result = query.validate_layout();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid layout"));

        // None defaults to hierarchical
        let query = GraphQuery {
            max_depth: None,
            filter: None,
            layout: None,
        };
        let result = query.validate_layout();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hierarchical");
    }
}
