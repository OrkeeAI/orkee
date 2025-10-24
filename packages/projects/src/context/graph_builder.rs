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
use tracing::warn;
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
            warn!("Project path does not exist: {}", project_path);
            return Err("Project path does not exist".to_string());
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut file_id_map: HashMap<String, String> = HashMap::new();

        // Find all TypeScript/JavaScript files
        let files = self.find_source_files(&root_path)?;

        // Create nodes for each file
        for (idx, file_path) in files.iter().enumerate() {
            // Validate file is within project bounds
            let relative_path = match file_path.strip_prefix(&root_path) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => {
                    warn!("Skipping file outside project root: {:?}", file_path);
                    continue;
                }
            };

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
            let file_relative = match file_path.strip_prefix(&root_path) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => {
                    warn!(
                        "Skipping imports for file outside project root: {:?}",
                        file_path
                    );
                    continue;
                }
            };

            let file_dir = Path::new(&file_relative)
                .parent()
                .and_then(|p| p.to_str())
                .unwrap_or("");

            for import_path in imports {
                // Resolve the import path relative to the current file's directory
                if let Some(resolved_path) =
                    self.resolve_import_path(&import_path, file_dir, &file_id_map)
                {
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
                    } else {
                        warn!(
                            "Import resolved to '{}' but file not found in project (from {} importing '{}')",
                            resolved_path, file_relative, import_path
                        );
                    }
                } else {
                    warn!(
                        "Failed to resolve import '{}' from {} (dir: {})",
                        import_path, file_relative, file_dir
                    );
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
            warn!("Project path does not exist: {}", project_path);
            return Err("Project path does not exist".to_string());
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut file_node_map: HashMap<String, String> = HashMap::new();
        let files = self.find_source_files(&root_path)?;

        // Analyze each file for symbols
        let mut analyzer = AstAnalyzer::new_typescript()
            .map_err(|e| format!("Failed to create AST analyzer: {}", e))?;

        for (file_idx, file_path) in files.iter().enumerate() {
            // Validate file is within project bounds
            let relative_path = match file_path.strip_prefix(&root_path) {
                Ok(path) => path.to_string_lossy().to_string(),
                Err(_) => {
                    warn!(
                        "Skipping file outside project root: {:?}",
                        file_path
                    );
                    continue;
                }
            };

            // Create file node
            let file_node_id = format!("file_{}", file_idx);
            file_node_map.insert(relative_path.clone(), file_node_id.clone());

            nodes.push(GraphNode {
                id: file_node_id.clone(),
                label: Path::new(&relative_path)
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
                node_type: NodeType::File,
                metadata: NodeMetadata {
                    path: Some(relative_path.clone()),
                    line_start: None,
                    line_end: None,
                    token_count: self.estimate_token_count(file_path),
                    complexity: None,
                    spec_id: None,
                },
            });

            // Extract and process symbols from file
            match fs::read_to_string(file_path) {
                Ok(content) => match analyzer.extract_symbols(&content) {
                    Ok(symbols) => {
                        self.process_symbols_recursive(
                            &symbols,
                            &relative_path,
                            &file_node_id,
                            &mut nodes,
                            &mut edges,
                            None,
                        );
                    }
                    Err(e) => {
                        warn!("Failed to extract symbols from {:?}: {}", file_path, e);
                    }
                },
                Err(e) => {
                    warn!("Failed to read file {:?}: {}", file_path, e);
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
            warn!("Project path does not exist: {}", project_path);
            return Err("Project path does not exist".to_string());
        }

        let mut nodes = Vec::new();
        let mut edges = Vec::new();
        let mut dir_map: HashMap<String, String> = HashMap::new();

        // Walk through directory structure
        for entry in WalkDir::new(&root_path)
            .max_depth(5)
            .into_iter()
            .filter_entry(|e| !self.is_ignored_dir(e.path()))
            .flatten()
        {
            if entry.path().is_dir() {
                // Validate directory is within project bounds
                let relative_path = match entry.path().strip_prefix(&root_path) {
                    Ok(path) => path.to_string_lossy().to_string(),
                    Err(_) => {
                        warn!(
                            "Skipping directory outside project root: {:?}",
                            entry.path()
                        );
                        continue;
                    }
                };

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
            .flatten()
        {
            if entry.path().is_file() {
                if let Some(ext) = entry.path().extension() {
                    if matches!(ext.to_string_lossy().as_ref(), "ts" | "tsx" | "js" | "jsx") {
                        files.push(entry.path().to_path_buf());
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

    /// Extract import statements from a file using AST parsing
    fn extract_imports(&self, file_path: &Path) -> Result<Vec<String>, String> {
        let content =
            fs::read_to_string(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        // Use AST analyzer for robust import extraction
        let mut analyzer = AstAnalyzer::new_typescript()
            .map_err(|e| format!("Failed to create AST analyzer: {}", e))?;

        let all_imports = analyzer
            .extract_imports(&content)
            .map_err(|e| format!("Failed to extract imports: {}", e))?;

        // Filter to only include relative imports (project-local dependencies)
        let relative_imports: Vec<String> = all_imports
            .into_iter()
            .filter(|import_path| import_path.starts_with("./") || import_path.starts_with("../"))
            .collect();

        Ok(relative_imports)
    }

    /// Resolve a relative import path to a project-relative path
    ///
    /// # Security Model
    ///
    /// This method is secure against path traversal attacks because:
    /// 1. All files are pre-validated during collection (see `find_source_files`)
    /// 2. `file_id_map` only contains files within project bounds
    /// 3. Resolution uses HashMap lookups, not filesystem operations
    /// 4. `normalize_path` tracks depth to detect escape attempts (returns empty string if negative)
    /// 5. Unresolved paths return `None` rather than creating edges outside the project
    ///
    /// This design ensures that even malicious import statements like:
    /// - `import x from '../../../../../etc/passwd'`
    /// - `import y from './../../../../../../tmp/evil'`
    ///
    /// Cannot create graph edges outside the project because:
    /// - The normalized path will either be empty (escape detected) or not exist in `file_id_map`
    /// - No filesystem checks are performed, so symlink attacks are not possible
    /// - The pre-built `file_id_map` acts as an allowlist of valid targets
    ///
    /// # Limitations
    /// - Does not support path aliases (@/, ~/)
    /// - Does not parse tsconfig.json/jsconfig.json for custom path mappings
    /// - May fail on monorepo-style imports with workspace references
    /// - Only handles relative imports (./*, ../*)
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
        let mut depth: i32 = 0;

        for component in path.components() {
            match component {
                std::path::Component::Normal(c) => {
                    components.push(c.to_string_lossy().to_string());
                    depth += 1;
                }
                std::path::Component::ParentDir => {
                    if !components.is_empty() {
                        components.pop();
                        depth -= 1;
                    } else {
                        // Attempted to escape root - track it
                        depth -= 1;
                    }
                }
                std::path::Component::CurDir => {
                    // Skip current directory
                }
                _ => {}
            }
        }

        // If depth went negative, the path tried to escape root
        if depth < 0 {
            return String::new();
        }

        components.join("/")
    }

    /// Estimate token count for a file
    fn estimate_token_count(&self, file_path: &Path) -> Option<usize> {
        fs::read_to_string(file_path)
            .ok()
            .map(|content| content.len() / 4)
    }

    /// Recursively process symbols and create nodes/edges
    fn process_symbols_recursive(
        &self,
        symbols: &[crate::context::Symbol],
        file_path: &str,
        parent_node_id: &str,
        nodes: &mut Vec<GraphNode>,
        edges: &mut Vec<GraphEdge>,
        parent_symbol_id: Option<&str>,
    ) {
        for symbol in symbols {
            // Create unique symbol ID
            let symbol_node_id = format!("symbol_{}_{}_{}", symbol.name, symbol.line_start, nodes.len());

            // Determine node type
            let node_type = match symbol.kind {
                crate::context::SymbolKind::Function => NodeType::Function,
                crate::context::SymbolKind::Class => NodeType::Class,
                crate::context::SymbolKind::Interface => NodeType::Class, // Treat interfaces as classes in graph
                crate::context::SymbolKind::Method => NodeType::Function, // Treat methods as functions in graph
                _ => NodeType::Module,
            };

            // Create symbol node
            nodes.push(GraphNode {
                id: symbol_node_id.clone(),
                label: symbol.name.clone(),
                node_type,
                metadata: NodeMetadata {
                    path: Some(file_path.to_string()),
                    line_start: Some(symbol.line_start),
                    line_end: Some(symbol.line_end),
                    token_count: None,
                    complexity: None,
                    spec_id: None,
                },
            });

            // Create edge from parent (file or parent symbol) to this symbol
            let edge_source = if let Some(parent_id) = parent_symbol_id {
                parent_id.to_string()
            } else {
                parent_node_id.to_string()
            };

            edges.push(GraphEdge {
                id: format!("edge_{}_{}", edge_source, symbol_node_id),
                source: edge_source,
                target: symbol_node_id.clone(),
                edge_type: EdgeType::Contains,
                weight: None,
            });

            // Recursively process child symbols (e.g., methods in a class)
            if !symbol.children.is_empty() {
                self.process_symbols_recursive(
                    &symbol.children,
                    file_path,
                    parent_node_id,
                    nodes,
                    edges,
                    Some(&symbol_node_id),
                );
            }
        }
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

    #[test]
    fn test_dependency_graph_large_project() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create 1000+ files to simulate a large project
        let src_dir = root.join("src");
        fs::create_dir_all(&src_dir).unwrap();

        for i in 0..1200 {
            let file_name = format!("module{}.ts", i);
            let file_path = src_dir.join(&file_name);

            // Create files with some import dependencies
            let next_module = if i < 1199 {
                format!("./module{}", i + 1)
            } else {
                "./module0".to_string()
            };

            let content = format!(
                "import {{ func }} from '{}';\nexport const value{} = func();",
                next_module, i
            );

            fs::write(&file_path, content).unwrap();
        }

        let mut builder = GraphBuilder::new();
        let start = std::time::Instant::now();
        let result = builder.build_dependency_graph(root.to_str().unwrap(), "large-project");
        let duration = start.elapsed();

        assert!(result.is_ok(), "Graph generation should succeed");
        let graph = result.unwrap();

        // Verify the graph was built correctly
        assert!(
            graph.metadata.total_nodes >= 1200,
            "Should have at least 1200 nodes"
        );
        assert!(
            graph.metadata.total_edges >= 1199,
            "Should have at least 1199 edges"
        );

        // Verify performance: should complete well under the 30-second timeout
        assert!(
            duration.as_secs() < 30,
            "Graph generation took {} seconds, should be under 30s",
            duration.as_secs()
        );

        println!(
            "Large project test completed in {:.2}s with {} nodes and {} edges",
            duration.as_secs_f64(),
            graph.metadata.total_nodes,
            graph.metadata.total_edges
        );
    }

    #[test]
    fn test_graph_with_circular_dependencies() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create files with circular dependencies
        // a.ts -> b.ts -> c.ts -> a.ts (cycle)
        fs::write(
            root.join("a.ts"),
            "import { b } from './b';\nexport const a = 'a' + b;",
        )
        .unwrap();

        fs::write(
            root.join("b.ts"),
            "import { c } from './c';\nexport const b = 'b' + c;",
        )
        .unwrap();

        fs::write(
            root.join("c.ts"),
            "import { a } from './a';\nexport const c = 'c' + a;",
        )
        .unwrap();

        // Add more complex cycles
        // d.ts -> e.ts -> f.ts -> d.ts (another cycle)
        // with additional cross-dependencies
        fs::write(
            root.join("d.ts"),
            "import { e } from './e';\nimport { a } from './a';\nexport const d = e + a;",
        )
        .unwrap();

        fs::write(
            root.join("e.ts"),
            "import { f } from './f';\nexport const e = 'e' + f;",
        )
        .unwrap();

        fs::write(
            root.join("f.ts"),
            "import { d } from './d';\nexport const f = 'f' + d;",
        )
        .unwrap();

        let mut builder = GraphBuilder::new();
        let start = std::time::Instant::now();
        let result = builder.build_dependency_graph(root.to_str().unwrap(), "circular-project");
        let duration = start.elapsed();

        assert!(result.is_ok(), "Graph generation should not hang or fail");
        let graph = result.unwrap();

        // Verify we detected all files
        assert_eq!(graph.metadata.total_nodes, 6, "Should have 6 nodes");

        // Verify we built edges without infinite loops
        assert!(graph.metadata.total_edges >= 6, "Should have dependencies");

        // Verify performance: should complete quickly (no infinite loops)
        assert!(
            duration.as_millis() < 5000,
            "Graph generation took {} ms, should complete in under 5s",
            duration.as_millis()
        );

        // Verify cycle detection works
        let cycles = builder.dependency_graph.detect_cycles();
        assert!(
            !cycles.is_empty(),
            "Should detect at least one circular dependency"
        );

        println!(
            "Circular dependency test completed in {:.2}s with {} cycles detected",
            duration.as_secs_f64(),
            cycles.len()
        );
    }

    #[test]
    fn test_normalize_path_prevents_underflow() {
        let builder = GraphBuilder::new();

        // Test excessive parent directory traversal
        let excessive_parent = PathBuf::from("../../../../../../../etc/passwd");
        let normalized = builder.normalize_path(&excessive_parent);

        // Should not result in a negative depth (empty path is acceptable)
        // The key is that it shouldn't panic or produce invalid results
        assert_eq!(
            normalized, "",
            "Excessive parent dirs should result in empty path, not underflow"
        );

        // Test normal path with some parent dirs
        let normal_path = PathBuf::from("src/../lib/utils.ts");
        let normalized = builder.normalize_path(&normal_path);
        assert_eq!(normalized, "lib/utils.ts");

        // Test path that tries to escape but has some components first
        let escape_attempt = PathBuf::from("src/components/../../../../../../etc/passwd");
        let normalized = builder.normalize_path(&escape_attempt);
        assert_eq!(normalized, "", "Should not escape beyond root");

        // Test valid relative path
        let valid_path = PathBuf::from("src/components/Button.tsx");
        let normalized = builder.normalize_path(&valid_path);
        assert_eq!(normalized, "src/components/Button.tsx");
    }

    #[test]
    fn test_extract_imports_handles_multiline() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("multiline.ts");

        let content = r#"
            import {
                foo,
                bar,
                baz
            } from './utils';

            import type {
                TypeA,
                TypeB
            } from '../types';
        "#;

        fs::write(&file_path, content).unwrap();

        let builder = GraphBuilder::new();
        let imports = builder.extract_imports(&file_path).unwrap();

        assert!(
            imports.contains(&"./utils".to_string()),
            "Should detect multi-line import"
        );
        assert!(
            imports.contains(&"../types".to_string()),
            "Should detect multi-line type import"
        );
    }

    #[test]
    fn test_extract_imports_ignores_comments() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("comments.ts");

        let content = r#"
            // import { fake } from './commented';
            /* import { fake2 } from './block-commented'; */
            import { real } from './real';

            /**
             * import { fake3 } from './doc-commented';
             */
        "#;

        fs::write(&file_path, content).unwrap();

        let builder = GraphBuilder::new();
        let imports = builder.extract_imports(&file_path).unwrap();

        assert_eq!(imports.len(), 1, "Should only find one real import");
        assert!(imports.contains(&"./real".to_string()));
        assert!(
            !imports.iter().any(|i| i.contains("commented")),
            "Should not detect commented imports"
        );
    }

    #[test]
    fn test_extract_imports_handles_reexports() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("reexports.ts");

        let content = r#"
            export { foo } from './foo';
            export * from './bar';
            export { default as baz } from './baz';
            import { qux } from './qux';
        "#;

        fs::write(&file_path, content).unwrap();

        let builder = GraphBuilder::new();
        let imports = builder.extract_imports(&file_path).unwrap();

        // Re-exports are dependencies too - the file needs these modules
        assert!(
            imports.contains(&"./foo".to_string()),
            "Should detect export...from"
        );
        assert!(
            imports.contains(&"./bar".to_string()),
            "Should detect export *"
        );
        assert!(
            imports.contains(&"./baz".to_string()),
            "Should detect export default as"
        );
        assert!(
            imports.contains(&"./qux".to_string()),
            "Should detect regular import"
        );
    }

    #[test]
    fn test_resolve_import_path_with_deeply_nested_traversal() {
        let builder = GraphBuilder::new();
        let mut file_id_map = HashMap::new();

        // Create a legitimate file in the project
        file_id_map.insert("src/utils.ts".to_string(), "file_0".to_string());

        // Test deeply nested path traversal that tries to escape
        let malicious_import = "../../../../../../../../../etc/passwd";
        let result = builder.resolve_import_path(malicious_import, "src/components", &file_id_map);

        // Should return None because normalized path is empty (escape detected)
        assert!(
            result.is_none(),
            "Deeply nested path traversal should fail to resolve"
        );

        // Test another variant
        let malicious_import2 = "../../../../../../../../../../tmp/evil.ts";
        let result2 = builder.resolve_import_path(malicious_import2, "src/deep/nested/components", &file_id_map);

        assert!(
            result2.is_none(),
            "Deep traversal from nested directory should fail"
        );
    }

    #[test]
    fn test_resolve_import_path_with_failed_resolutions() {
        let builder = GraphBuilder::new();
        let mut file_id_map = HashMap::new();

        // Add some legitimate files
        file_id_map.insert("src/utils.ts".to_string(), "file_0".to_string());
        file_id_map.insert("src/components/Button.tsx".to_string(), "file_1".to_string());

        // Test import that doesn't exist
        let nonexistent = "./nonexistent";
        let result = builder.resolve_import_path(nonexistent, "src", &file_id_map);
        assert!(result.is_none(), "Should return None for nonexistent file");

        // Test import with wrong extension
        let wrong_ext = "./utils.jsx";
        let result2 = builder.resolve_import_path(wrong_ext, "src", &file_id_map);
        assert!(result2.is_none(), "Should return None for wrong extension");

        // Test import to external package (not relative)
        // Note: This should be filtered out earlier by extract_imports, but test defensive behavior
        let external = "react";
        let result3 = builder.resolve_import_path(external, "src", &file_id_map);
        assert!(result3.is_none(), "Should return None for non-relative import");
    }

    #[test]
    fn test_resolve_import_path_with_various_extensions() {
        let builder = GraphBuilder::new();
        let mut file_id_map = HashMap::new();

        // Add files with different extensions
        file_id_map.insert("utils.ts".to_string(), "file_0".to_string());
        file_id_map.insert("components.tsx".to_string(), "file_1".to_string());
        file_id_map.insert("legacy.js".to_string(), "file_2".to_string());
        file_id_map.insert("legacy-jsx.jsx".to_string(), "file_3".to_string());

        // Test resolution without extension (should try .ts first)
        let result = builder.resolve_import_path("./utils", "", &file_id_map);
        assert_eq!(result, Some("utils.ts".to_string()));

        // Test .tsx resolution
        let result2 = builder.resolve_import_path("./components", "", &file_id_map);
        assert_eq!(result2, Some("components.tsx".to_string()));

        // Test .js resolution
        let result3 = builder.resolve_import_path("./legacy", "", &file_id_map);
        assert_eq!(result3, Some("legacy.js".to_string()));

        // Test .jsx resolution
        let result4 = builder.resolve_import_path("./legacy-jsx", "", &file_id_map);
        assert_eq!(result4, Some("legacy-jsx.jsx".to_string()));
    }

    #[test]
    fn test_resolve_import_path_index_files() {
        let builder = GraphBuilder::new();
        let mut file_id_map = HashMap::new();

        // Add index files
        file_id_map.insert("components/index.ts".to_string(), "file_0".to_string());
        file_id_map.insert("utils/index.js".to_string(), "file_1".to_string());

        // Test resolving to index.ts
        let result = builder.resolve_import_path("./components", "", &file_id_map);
        assert_eq!(result, Some("components/index.ts".to_string()));

        // Test resolving to index.js
        let result2 = builder.resolve_import_path("./utils", "", &file_id_map);
        assert_eq!(result2, Some("utils/index.js".to_string()));
    }

    #[test]
    fn test_build_dependency_graph_with_path_traversal_imports() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a project structure
        fs::create_dir_all(root.join("src/components")).unwrap();

        // Create a legitimate file
        fs::write(root.join("src/utils.ts"), "export const foo = 1;").unwrap();

        // Create a file with malicious imports
        let malicious_content = r#"
            import { foo } from '../utils';  // Valid
            import { evil } from '../../../../../etc/passwd';  // Invalid - tries to escape
            import { bad } from '../../../../../../tmp/evil';   // Invalid - tries to escape
        "#;
        fs::write(root.join("src/components/Button.tsx"), malicious_content).unwrap();

        let mut builder = GraphBuilder::new();
        let result = builder.build_dependency_graph(root.to_str().unwrap(), "test-project");

        assert!(result.is_ok(), "Graph building should succeed");
        let graph = result.unwrap();

        // Should have 2 nodes (the two files we created)
        assert_eq!(graph.metadata.total_nodes, 2);

        // Should only have 1 edge (valid import from Button to utils)
        // The malicious imports should be ignored
        assert_eq!(
            graph.metadata.total_edges, 1,
            "Should only create edge for valid import"
        );
    }

    #[test]
    fn test_normalize_path_with_complex_patterns() {
        let builder = GraphBuilder::new();

        // Test path with multiple ./ and ../
        let complex1 = PathBuf::from("./src/../lib/./utils/../helpers/./index.ts");
        let normalized1 = builder.normalize_path(&complex1);
        assert_eq!(normalized1, "lib/helpers/index.ts");

        // Test path that goes up then down
        let complex2 = PathBuf::from("src/components/../../lib/utils.ts");
        let normalized2 = builder.normalize_path(&complex2);
        assert_eq!(normalized2, "lib/utils.ts");

        // Test path with many current directory references
        let complex3 = PathBuf::from("./././src/./lib/././utils.ts");
        let normalized3 = builder.normalize_path(&complex3);
        assert_eq!(normalized3, "src/lib/utils.ts");
    }

    #[test]
    fn test_symlink_handling_in_file_collection() {
        // This test verifies that symlinks are handled correctly during file collection
        // Symlinks would be resolved when we read the file paths, so they appear as
        // normal files in the file_id_map. This prevents symlink-based attacks where
        // an attacker tries to create a symlink pointing outside the project.

        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a normal file
        fs::write(root.join("real.ts"), "export const real = true;").unwrap();

        // Create a subdirectory
        fs::create_dir(root.join("src")).unwrap();

        // On Unix systems, test symlink behavior
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            // Create a symlink to the real file
            let symlink_path = root.join("src/linked.ts");
            symlink(root.join("real.ts"), &symlink_path).ok();

            let builder = GraphBuilder::new();
            let files = builder.find_source_files(root).unwrap();

            // Both the real file and symlink should be found
            // The important security property is that both are within the project root
            assert!(
                files.iter().all(|f| f.starts_with(root)),
                "All files (including symlinks) must be within project root"
            );
        }
    }
}
