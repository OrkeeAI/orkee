// ABOUTME: Mock graph data generator for testing frontend visualization
// ABOUTME: Provides realistic sample graphs for development and testing

use super::graph_types::*;
use chrono::Utc;

/// Generate mock dependency graph
pub fn generate_mock_dependency_graph(project_id: &str) -> CodeGraph {
    let nodes = vec![
        GraphNode {
            id: "src/main.ts".to_string(),
            label: "main.ts".to_string(),
            node_type: NodeType::File,
            metadata: NodeMetadata {
                path: Some("src/main.ts".to_string()),
                line_start: Some(1),
                line_end: Some(150),
                token_count: Some(1200),
                complexity: Some(8.5),
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/app.ts".to_string(),
            label: "app.ts".to_string(),
            node_type: NodeType::File,
            metadata: NodeMetadata {
                path: Some("src/app.ts".to_string()),
                line_start: Some(1),
                line_end: Some(200),
                token_count: Some(1800),
                complexity: Some(12.3),
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/utils/helpers.ts".to_string(),
            label: "helpers.ts".to_string(),
            node_type: NodeType::File,
            metadata: NodeMetadata {
                path: Some("src/utils/helpers.ts".to_string()),
                line_start: Some(1),
                line_end: Some(80),
                token_count: Some(600),
                complexity: Some(4.2),
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/api/client.ts".to_string(),
            label: "client.ts".to_string(),
            node_type: NodeType::File,
            metadata: NodeMetadata {
                path: Some("src/api/client.ts".to_string()),
                line_start: Some(1),
                line_end: Some(120),
                token_count: Some(950),
                complexity: Some(6.7),
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/components/Header.tsx".to_string(),
            label: "Header.tsx".to_string(),
            node_type: NodeType::File,
            metadata: NodeMetadata {
                path: Some("src/components/Header.tsx".to_string()),
                line_start: Some(1),
                line_end: Some(60),
                token_count: Some(480),
                complexity: Some(3.1),
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/services/auth.ts".to_string(),
            label: "auth.ts".to_string(),
            node_type: NodeType::File,
            metadata: NodeMetadata {
                path: Some("src/services/auth.ts".to_string()),
                line_start: Some(1),
                line_end: Some(180),
                token_count: Some(1500),
                complexity: Some(10.8),
                spec_id: None,
            },
        },
    ];

    let edges = vec![
        GraphEdge {
            id: "edge-1".to_string(),
            source: "src/main.ts".to_string(),
            target: "src/app.ts".to_string(),
            edge_type: EdgeType::Import,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "edge-2".to_string(),
            source: "src/app.ts".to_string(),
            target: "src/utils/helpers.ts".to_string(),
            edge_type: EdgeType::Import,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "edge-3".to_string(),
            source: "src/app.ts".to_string(),
            target: "src/api/client.ts".to_string(),
            edge_type: EdgeType::Import,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "edge-4".to_string(),
            source: "src/app.ts".to_string(),
            target: "src/components/Header.tsx".to_string(),
            edge_type: EdgeType::Import,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "edge-5".to_string(),
            source: "src/api/client.ts".to_string(),
            target: "src/services/auth.ts".to_string(),
            edge_type: EdgeType::Import,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "edge-6".to_string(),
            source: "src/services/auth.ts".to_string(),
            target: "src/utils/helpers.ts".to_string(),
            edge_type: EdgeType::Import,
            weight: Some(1.0),
        },
    ];

    CodeGraph {
        nodes,
        edges,
        metadata: GraphMetadata {
            total_nodes: 6,
            total_edges: 6,
            graph_type: "dependencies".to_string(),
            generated_at: Utc::now(),
            project_id: project_id.to_string(),
        },
    }
}

/// Generate mock symbol graph
pub fn generate_mock_symbol_graph(project_id: &str) -> CodeGraph {
    let nodes = vec![
        GraphNode {
            id: "App".to_string(),
            label: "App".to_string(),
            node_type: NodeType::Class,
            metadata: NodeMetadata {
                path: Some("src/app.ts".to_string()),
                line_start: Some(10),
                line_end: Some(50),
                token_count: Some(400),
                complexity: Some(5.2),
                spec_id: None,
            },
        },
        GraphNode {
            id: "initialize".to_string(),
            label: "initialize()".to_string(),
            node_type: NodeType::Function,
            metadata: NodeMetadata {
                path: Some("src/app.ts".to_string()),
                line_start: Some(15),
                line_end: Some(25),
                token_count: Some(120),
                complexity: Some(2.1),
                spec_id: None,
            },
        },
        GraphNode {
            id: "formatDate".to_string(),
            label: "formatDate()".to_string(),
            node_type: NodeType::Function,
            metadata: NodeMetadata {
                path: Some("src/utils/helpers.ts".to_string()),
                line_start: Some(5),
                line_end: Some(15),
                token_count: Some(80),
                complexity: Some(1.5),
                spec_id: None,
            },
        },
        GraphNode {
            id: "ApiClient".to_string(),
            label: "ApiClient".to_string(),
            node_type: NodeType::Class,
            metadata: NodeMetadata {
                path: Some("src/api/client.ts".to_string()),
                line_start: Some(8),
                line_end: Some(80),
                token_count: Some(650),
                complexity: Some(8.3),
                spec_id: None,
            },
        },
        GraphNode {
            id: "fetchData".to_string(),
            label: "fetchData()".to_string(),
            node_type: NodeType::Function,
            metadata: NodeMetadata {
                path: Some("src/api/client.ts".to_string()),
                line_start: Some(20),
                line_end: Some(40),
                token_count: Some(200),
                complexity: Some(3.7),
                spec_id: None,
            },
        },
    ];

    let edges = vec![
        GraphEdge {
            id: "sym-edge-1".to_string(),
            source: "App".to_string(),
            target: "initialize".to_string(),
            edge_type: EdgeType::Contains,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "sym-edge-2".to_string(),
            source: "initialize".to_string(),
            target: "formatDate".to_string(),
            edge_type: EdgeType::Reference,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "sym-edge-3".to_string(),
            source: "ApiClient".to_string(),
            target: "fetchData".to_string(),
            edge_type: EdgeType::Contains,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "sym-edge-4".to_string(),
            source: "App".to_string(),
            target: "ApiClient".to_string(),
            edge_type: EdgeType::Reference,
            weight: Some(1.0),
        },
    ];

    CodeGraph {
        nodes,
        edges,
        metadata: GraphMetadata {
            total_nodes: 5,
            total_edges: 4,
            graph_type: "symbols".to_string(),
            generated_at: Utc::now(),
            project_id: project_id.to_string(),
        },
    }
}

/// Generate mock module graph
pub fn generate_mock_module_graph(project_id: &str) -> CodeGraph {
    let nodes = vec![
        GraphNode {
            id: "src".to_string(),
            label: "src/".to_string(),
            node_type: NodeType::Module,
            metadata: NodeMetadata {
                path: Some("src".to_string()),
                line_start: None,
                line_end: None,
                token_count: Some(5000),
                complexity: None,
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/components".to_string(),
            label: "components/".to_string(),
            node_type: NodeType::Module,
            metadata: NodeMetadata {
                path: Some("src/components".to_string()),
                line_start: None,
                line_end: None,
                token_count: Some(1200),
                complexity: None,
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/api".to_string(),
            label: "api/".to_string(),
            node_type: NodeType::Module,
            metadata: NodeMetadata {
                path: Some("src/api".to_string()),
                line_start: None,
                line_end: None,
                token_count: Some(1500),
                complexity: None,
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/utils".to_string(),
            label: "utils/".to_string(),
            node_type: NodeType::Module,
            metadata: NodeMetadata {
                path: Some("src/utils".to_string()),
                line_start: None,
                line_end: None,
                token_count: Some(800),
                complexity: None,
                spec_id: None,
            },
        },
        GraphNode {
            id: "src/services".to_string(),
            label: "services/".to_string(),
            node_type: NodeType::Module,
            metadata: NodeMetadata {
                path: Some("src/services".to_string()),
                line_start: None,
                line_end: None,
                token_count: Some(1500),
                complexity: None,
                spec_id: None,
            },
        },
    ];

    let edges = vec![
        GraphEdge {
            id: "mod-edge-1".to_string(),
            source: "src".to_string(),
            target: "src/components".to_string(),
            edge_type: EdgeType::Contains,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "mod-edge-2".to_string(),
            source: "src".to_string(),
            target: "src/api".to_string(),
            edge_type: EdgeType::Contains,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "mod-edge-3".to_string(),
            source: "src".to_string(),
            target: "src/utils".to_string(),
            edge_type: EdgeType::Contains,
            weight: Some(1.0),
        },
        GraphEdge {
            id: "mod-edge-4".to_string(),
            source: "src".to_string(),
            target: "src/services".to_string(),
            edge_type: EdgeType::Contains,
            weight: Some(1.0),
        },
    ];

    CodeGraph {
        nodes,
        edges,
        metadata: GraphMetadata {
            total_nodes: 5,
            total_edges: 4,
            graph_type: "modules".to_string(),
            generated_at: Utc::now(),
            project_id: project_id.to_string(),
        },
    }
}
