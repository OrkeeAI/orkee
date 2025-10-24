// ABOUTME: Graph builder for generating code visualization graphs from project source files.
// ABOUTME: Leverages existing AST analyzer and dependency graph to build file, symbol, and module graphs.

use super::ast_analyzer::AstAnalyzer;
use super::dependency_graph::DependencyGraph;
use super::graph_types::{
    CodeGraph, EdgeType, GraphEdge, GraphMetadata, GraphNode, NodeMetadata, NodeType,
};
use chrono::Utc;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Builds various types of code graphs for visualization
pub struct GraphBuilder {
    dependency_graph: DependencyGraph,
}

impl GraphBuilder {
    /// Create a new GraphBuilder instance
    pub fn new() -> Self {
        Self {
            dependency_graph: DependencyGraph::new(),
        }
    }

    /// Build file dependency graph for a project
    pub fn build_dependency_graph(
        &mut self,
        project_path: &str,
        project_id: &str,
    ) -> Result<CodeGraph, String> {
        let root_path = PathBuf::from(project_path);
        if !root_path.exists() {
            return Err(format!("Project path does not exist: {}", project_path));
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut file_id_map: HashMap<String, String> = HashMap::new();

        // Find all TypeScript/JavaScript files
        let files = self.find_source_files(&root_path)?;

        // Create nodes for each file
        for (idx, file_path) in files.iter().enumerate() {
            let relative_path = file_path
                .strip_prefix(&root_path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let node_id = format!("file_{}", idx);
            file_id_map.insert(relative_path.clone(), node_id.clone());

            let metadata = NodeMetadata {
                path: Some(relative_path.clone()),
                line_start: None,
                line_end: None,
                token_count: self.estimate_token_count(file_path),
                complexity: None,
                spec_id: None,
            };

            nodes.push(GraphNode {
                id: node_id,
                label: Path::new(&relative_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                node_type: NodeType::File,
                metadata,
            });
        }

        // Analyze dependencies and create edges
        for (idx, file_path) in files.iter().enumerate() {
            let imports = self.extract_imports(file_path)?;
            let source_id = format!("file_{}", idx);

            // Get the directory of the current file (relative to project root)
            let file_relative = file_path
                .strip_prefix(&root_path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            let file_dir = Path::new(&file_relative)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");

            for import_path in imports {
                // Resolve the import path relative to the current file's directory
                if let Some(resolved_path) = self.resolve_import_path(&import_path, file_dir, &file_id_map) {
                    if let Some(target_id) = file_id_map.get(&resolved_path) {
                        let edge_id = format!("edge_{}_{}", source_id, target_id);
                        edges.push(GraphEdge {
                            id: edge_id,
                            source: source_id.clone(),
                            target: target_id.clone(),
                            edge_type: EdgeType::Import,
                            weight: Some(1.0),
                        });

                        // Also add to dependency graph
                        self.dependency_graph
                            .add_edge(source_id.clone(), target_id.clone());
                    }
                }
            }
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            graph_type: "dependencies".to_string(),
            generated_at: Utc::now(),
            project_id: project_id.to_string(),
        };

        Ok(CodeGraph {
            nodes,
            edges,
            metadata,
        })
    }

    /// Build symbol reference graph for a project
    pub fn build_symbol_graph(
        &mut self,
        project_path: &str,
        project_id: &str,
    ) -> Result<CodeGraph, String> {
        let root_path = PathBuf::from(project_path);
        if !root_path.exists() {
            return Err(format!("Project path does not exist: {}", project_path));
        }

        let mut nodes = Vec::new();
        let edges = Vec::new();
        let files = self.find_source_files(&root_path)?;

        // Analyze each file for symbols
        let mut analyzer = AstAnalyzer::new_typescript()
            .map_err(|e| format!("Failed to create AST analyzer: {}", e))?;

        for file_path in files.iter() {
            if let Ok(content) = fs::read_to_string(file_path) {
                if let Ok(symbols) = analyzer.extract_symbols(&content) {
                    for symbol in symbols {
                        let relative_path = file_path
                            .strip_prefix(&root_path)
                            .unwrap_or(file_path)
                            .to_string_lossy()
                            .to_string();

                        let node_id = format!("symbol_{}_{}", symbol.name, nodes.len());
                        let node_type = match symbol.kind {
                            crate::context::SymbolKind::Function => NodeType::Function,
                            crate::context::SymbolKind::Class => NodeType::Class,
                            _ => NodeType::Module,
                        };

                        nodes.push(GraphNode {
                            id: node_id,
                            label: symbol.name.clone(),
                            node_type,
                            metadata: NodeMetadata {
                                path: Some(relative_path),
                                line_start: Some(symbol.line_start),
                                line_end: Some(symbol.line_end),
                                token_count: None,
                                complexity: None,
                                spec_id: None,
                            },
                        });
                    }
                }
            }
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            graph_type: "symbols".to_string(),
            generated_at: Utc::now(),
            project_id: project_id.to_string(),
        };

        Ok(CodeGraph {
            nodes,
            edges,
            metadata,
        })
    }

    /// Build module hierarchy graph for a project
    pub fn build_module_graph(
        &self,
        project_path: &str,
        project_id: &str,
    ) -> Result<CodeGraph, String> {
        let root_path = PathBuf::from(project_path);
        if !root_path.exists() {
            return Err(format!("Project path does not exist: {}", project_path));
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut dir_map: HashMap<String, String> = HashMap::new();

        // Walk through directory structure
        for entry in WalkDir::new(&root_path)
            .max_depth(5)
            .into_iter()
            .filter_entry(|e| !self.is_ignored_dir(e.path()))
        {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    let relative_path = entry
                        .path()
                        .strip_prefix(&root_path)
                        .unwrap_or(entry.path())
                        .to_string_lossy()
                        .to_string();

                    if relative_path.is_empty() {
                        continue;
                    }

                    let node_id = format!("module_{}", nodes.len());
                    dir_map.insert(relative_path.clone(), node_id.clone());

                    nodes.push(GraphNode {
                        id: node_id,
                        label: entry
                            .path()
                            .file_name()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string(),
                        node_type: NodeType::Module,
                        metadata: NodeMetadata {
                            path: Some(relative_path),
                            line_start: None,
                            line_end: None,
                            token_count: None,
                            complexity: None,
                            spec_id: None,
                        },
                    });
                }
            }
        }

        // Create edges for parent-child relationships
        for (path, node_id) in &dir_map {
            if let Some(parent_path) = Path::new(path).parent() {
                let parent_str = parent_path.to_string_lossy().to_string();
                if !parent_str.is_empty() {
                    if let Some(parent_id) = dir_map.get(&parent_str) {
                        edges.push(GraphEdge {
                            id: format!("contains_{}_{}", parent_id, node_id),
                            source: parent_id.clone(),
                            target: node_id.clone(),
                            edge_type: EdgeType::Contains,
                            weight: None,
                        });
                    }
                }
            }
        }

        let metadata = GraphMetadata {
            total_nodes: nodes.len(),
            total_edges: edges.len(),
            graph_type: "modules".to_string(),
            generated_at: Utc::now(),
            project_id: project_id.to_string(),
        };

        Ok(CodeGraph {
            nodes,
            edges,
            metadata,
        })
    }

    /// Find all source files in the project
    fn find_source_files(&self, root_path: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        for entry in WalkDir::new(root_path)
            .max_depth(10)
            .into_iter()
            .filter_entry(|e| !self.is_ignored_dir(e.path()))
        {
            if let Ok(entry) = entry {
                if entry.path().is_file() {
                    if let Some(ext) = entry.path().extension() {
                        if matches!(ext.to_string_lossy().as_ref(), "ts" | "tsx" | "js" | "jsx") {
                            files.push(entry.path().to_path_buf());
                        }
                    }
                }
            }
        }

        Ok(files)
    }

    /// Check if a directory should be ignored
    fn is_ignored_dir(&self, path: &Path) -> bool {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            matches!(
                name_str.as_ref(),
                "node_modules" | ".git" | "dist" | "build" | "target" | ".next" | "out"
            )
        } else {
            false
        }
    }

    /// Extract import statements from a file (simplified)
    fn extract_imports(&self, file_path: &Path) -> Result<Vec<String>, String> {
        let content =
            fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        let mut imports = Vec::new();

        // Simple regex-based import extraction (can be improved with AST)
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("import ") || trimmed.starts_with("from ") {
                // Extract the path from import/from statements
                if let Some(start) = trimmed.find('"').or_else(|| trimmed.find('\'')) {
                    if let Some(end) = trimmed[start + 1..].find('"').or_else(|| trimmed[start + 1..].find('\'')) {
                        let import_path = &trimmed[start + 1..start + 1 + end];
                        // Only include relative imports
                        if import_path.starts_with("./") || import_path.starts_with("../") {
                            imports.push(import_path.to_string());
                        }
                    }
                }
            }
        }

        Ok(imports)
    }

    /// Resolve a relative import path to a project-relative path
    fn resolve_import_path(
        &self,
        import_path: &str,
        file_dir: &str,
        file_id_map: &HashMap<String, String>,
    ) -> Option<String> {
        // Join the file directory with the import path
        let joined = if file_dir.is_empty() {
            PathBuf::from(import_path)
        } else {
            PathBuf::from(file_dir).join(import_path)
        };

        // Normalize the path (resolve .. and .)
        let normalized = self.normalize_path(&joined);

        // Try the path as-is first
        if file_id_map.contains_key(&normalized) {
            return Some(normalized);
        }

        // If not found, try adding common extensions
        let extensions = [".js", ".jsx", ".ts", ".tsx", ".mjs", ".cjs"];
        for ext in &extensions {
            let with_ext = format!("{}{}", normalized, ext);
            if file_id_map.contains_key(&with_ext) {
                return Some(with_ext);
            }
        }

        // Try as index file
        for ext in &extensions {
            let index_path = format!("{}/index{}", normalized, ext);
            if file_id_map.contains_key(&index_path) {
                return Some(index_path);
            }
        }

        None
    }

    /// Normalize a path by resolving . and .. components
    fn normalize_path(&self, path: &Path) -> String {
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                std::path::Component::Normal(c) => {
                    components.push(c.to_string_lossy().to_string());
                }
                std::path::Component::ParentDir => {
                    components.pop();
                }
                std::path::Component::CurDir => {
                    // Skip current directory
                }
                _ => {}
            }
        }

        components.join("/")
    }

    /// Estimate token count for a file
    fn estimate_token_count(&self, file_path: &Path) -> Option<usize> {
        fs::read_to_string(file_path)
            .ok()
            .map(|content| content.len() / 4)
    }
}

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_graph_builder_new() {
        let builder = GraphBuilder::new();
        assert!(builder.dependency_graph.get_all_files().is_empty());
    }

    #[test]
    fn test_find_source_files() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create test files
        fs::write(root.join("index.ts"), "export const foo = 1;").unwrap();
        fs::write(root.join("utils.js"), "export const bar = 2;").unwrap();
        fs::write(root.join("README.md"), "# Test").unwrap();

        let builder = GraphBuilder::new();
        let files = builder.find_source_files(root).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|f| f.ends_with("index.ts")));
        assert!(files.iter().any(|f| f.ends_with("utils.js")));
    }

    #[test]
    fn test_is_ignored_dir() {
        let builder = GraphBuilder::new();

        assert!(builder.is_ignored_dir(Path::new("node_modules")));
        assert!(builder.is_ignored_dir(Path::new(".git")));
        assert!(builder.is_ignored_dir(Path::new("dist")));
        assert!(!builder.is_ignored_dir(Path::new("src")));
    }

    #[test]
    fn test_extract_imports() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");

        let content = r#"
            import { foo } from './utils';
            import type { Bar } from '../types';
            import React from 'react';
            const x = 1;
        "#;

        fs::write(&file_path, content).unwrap();

        let builder = GraphBuilder::new();
        let imports = builder.extract_imports(&file_path).unwrap();

        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&"./utils".to_string()));
        assert!(imports.contains(&"../types".to_string()));
        assert!(!imports.iter().any(|i| i.contains("react")));
    }
}
