// ABOUTME: Type definitions for code graph visualization data structures
// ABOUTME: Defines nodes, edges, and metadata for dependency, symbol, module, and spec-mapping graphs

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub label: String,
    pub node_type: NodeType,
    pub metadata: NodeMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    File,
    Function,
    Class,
    Module,
    Spec,
    Requirement,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetadata {
    pub path: Option<String>,
    pub line_start: Option<usize>,
    pub line_end: Option<usize>,
    pub token_count: Option<usize>,
    pub complexity: Option<f32>,
    pub spec_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub edge_type: EdgeType,
    pub weight: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    Import,
    Export,
    Reference,
    Implementation,
    Dependency,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeGraph {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub metadata: GraphMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub graph_type: String,
    pub generated_at: DateTime<Utc>,
    pub project_id: String,
}
