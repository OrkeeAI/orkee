// ABOUTME: HTTP handlers for graph API endpoints providing code visualization data.
// ABOUTME: Generates dependency, symbol, module, and spec-mapping graphs for projects.

use axum::{
    extract::{Path, Query, State},
    response::Json,
};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::{
    context::{graph_builder::GraphBuilder, graph_types::CodeGraph},
    db::DbState,
    manager::get_project as manager_get_project,
};

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

    // Generate real graph using GraphBuilder
    let mut builder = GraphBuilder::new();
    match builder.build_dependency_graph(&project.project_root, &project_id) {
        Ok(graph) => {
            info!(
                "Generated dependency graph with {} nodes and {} edges",
                graph.metadata.total_nodes, graph.metadata.total_edges
            );
            Json(GraphResponse::success(graph))
        }
        Err(e) => Json(GraphResponse::error(format!(
            "Failed to generate dependency graph: {}",
            e
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

    // Generate real graph using GraphBuilder
    let mut builder = GraphBuilder::new();
    match builder.build_symbol_graph(&project.project_root, &project_id) {
        Ok(graph) => {
            info!(
                "Generated symbol graph with {} nodes and {} edges",
                graph.metadata.total_nodes, graph.metadata.total_edges
            );
            Json(GraphResponse::success(graph))
        }
        Err(e) => Json(GraphResponse::error(format!(
            "Failed to generate symbol graph: {}",
            e
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

    // Generate real graph using GraphBuilder
    let builder = GraphBuilder::new();
    match builder.build_module_graph(&project.project_root, &project_id) {
        Ok(graph) => {
            info!(
                "Generated module graph with {} nodes and {} edges",
                graph.metadata.total_nodes, graph.metadata.total_edges
            );
            Json(GraphResponse::success(graph))
        }
        Err(e) => Json(GraphResponse::error(format!(
            "Failed to generate module graph: {}",
            e
        ))),
    }
}

/// Get spec-mapping graph for a project (placeholder for future implementation)
pub async fn get_spec_mapping_graph(
    Path(project_id): Path<String>,
    Query(_params): Query<GraphQuery>,
    State(_db): State<DbState>,
) -> Json<GraphResponse> {
    info!(
        "Generating spec-mapping graph for project: {}",
        project_id
    );

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
}
