// ABOUTME: HTTP handlers for graph API endpoints providing code visualization data.
// ABOUTME: Generates dependency, symbol, and module graphs for projects.

use axum::{
    extract::{Path, State},
    response::Json,
};
use serde::Serialize;
use std::time::Duration;
use tokio::time::timeout;
use tracing::info;

use context::{graph_builder::GraphBuilder, graph_types::CodeGraph};
use orkee_projects::{get_project as manager_get_project, DbState};

// Timeout configuration
const DEFAULT_GRAPH_GENERATION_TIMEOUT_SECS: u64 = 30;

/// Get graph generation timeout from environment or use default
fn get_graph_timeout() -> u64 {
    std::env::var("ORKEE_GRAPH_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_GRAPH_GENERATION_TIMEOUT_SECS)
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
    let timeout_secs = get_graph_timeout();

    let result = timeout(
        Duration::from_secs(timeout_secs),
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
            "Graph generation timed out after {} seconds. Large projects may need more time. Try: (1) excluding node_modules with .gitignore, (2) using path filters, or (3) increasing timeout with ORKEE_GRAPH_TIMEOUT_SECS environment variable.",
            timeout_secs
        ))),
    }
}

/// Get symbol graph for a project
pub async fn get_symbol_graph(
    Path(project_id): Path<String>,
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
    let timeout_secs = get_graph_timeout();

    let result = timeout(
        Duration::from_secs(timeout_secs),
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
            "Graph generation timed out after {} seconds. Large projects may need more time. Try: (1) excluding node_modules with .gitignore, (2) using path filters, or (3) increasing timeout with ORKEE_GRAPH_TIMEOUT_SECS environment variable.",
            timeout_secs
        ))),
    }
}

/// Get module graph for a project
pub async fn get_module_graph(
    Path(project_id): Path<String>,
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
    let timeout_secs = get_graph_timeout();

    let result = timeout(
        Duration::from_secs(timeout_secs),
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
            "Graph generation timed out after {} seconds. Large projects may need more time. Try: (1) excluding node_modules with .gitignore, (2) using path filters, or (3) increasing timeout with ORKEE_GRAPH_TIMEOUT_SECS environment variable.",
            timeout_secs
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_response_success() {
        use chrono::Utc;
        use context::graph_types::{CodeGraph, GraphMetadata};

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
}
