use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

/// Represents a dependency graph for code files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyGraph {
    /// Map from file path to its dependencies (files it imports)
    edges: HashMap<String, HashSet<String>>,
    /// Map from file path to symbols it exports
    exports: HashMap<String, Vec<ExportedSymbol>>,
    /// Map from file path to symbols it imports
    imports: HashMap<String, Vec<ImportedSymbol>>,
}

/// Represents an exported symbol from a file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedSymbol {
    pub name: String,
    pub kind: String,
    pub line: usize,
}

/// Represents an imported symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportedSymbol {
    pub name: String,
    pub from: String,
    pub line: usize,
}

/// Result of dependency analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAnalysis {
    pub file: String,
    pub direct_dependencies: Vec<String>,
    pub transitive_dependencies: Vec<String>,
    pub dependents: Vec<String>,
    pub depth: usize,
}

impl DependencyGraph {
    /// Create a new empty dependency graph
    pub fn new() -> Self {
        Self {
            edges: HashMap::new(),
            exports: HashMap::new(),
            imports: HashMap::new(),
        }
    }

    /// Add a dependency edge from one file to another
    pub fn add_edge(&mut self, from: String, to: String) {
        self.edges
            .entry(from)
            .or_default()
            .insert(to);
    }

    /// Add an exported symbol
    pub fn add_export(&mut self, file: String, symbol: ExportedSymbol) {
        self.exports
            .entry(file)
            .or_default()
            .push(symbol);
    }

    /// Add an imported symbol
    pub fn add_import(&mut self, file: String, symbol: ImportedSymbol) {
        self.imports
            .entry(file)
            .or_default()
            .push(symbol);
    }

    /// Get all direct dependencies of a file
    pub fn get_direct_dependencies(&self, file: &str) -> Vec<String> {
        self.edges
            .get(file)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Get all dependencies up to a certain depth using BFS
    pub fn get_dependencies(&self, file: &str, depth: usize) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut queue = VecDeque::new();

        queue.push_back((file.to_string(), 0));

        while let Some((current, current_depth)) = queue.pop_front() {
            if current_depth > depth || visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());
            if current != file {
                result.push(current.clone());
            }

            if let Some(deps) = self.edges.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        queue.push_back((dep.clone(), current_depth + 1));
                    }
                }
            }
        }

        result
    }

    /// Get all transitive dependencies (unlimited depth)
    pub fn get_all_dependencies(&self, file: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![file.to_string()];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());
            if current != file {
                result.push(current.clone());
            }

            if let Some(deps) = self.edges.get(&current) {
                for dep in deps {
                    if !visited.contains(dep) {
                        stack.push(dep.clone());
                    }
                }
            }
        }

        result
    }

    /// Find all files that depend on this file (reverse dependencies)
    pub fn get_dependents(&self, file: &str) -> Vec<String> {
        let mut dependents = Vec::new();

        for (source, targets) in &self.edges {
            if targets.contains(file) {
                dependents.push(source.clone());
            }
        }

        dependents
    }

    /// Get all dependents transitively
    pub fn get_all_dependents(&self, file: &str) -> Vec<String> {
        let mut result = Vec::new();
        let mut visited = HashSet::new();
        let mut stack = vec![file.to_string()];

        while let Some(current) = stack.pop() {
            if visited.contains(&current) {
                continue;
            }

            visited.insert(current.clone());
            if current != file {
                result.push(current.clone());
            }

            let dependents = self.get_dependents(&current);
            for dep in dependents {
                if !visited.contains(&dep) {
                    stack.push(dep);
                }
            }
        }

        result
    }

    /// Get exported symbols from a file
    pub fn get_exports(&self, file: &str) -> Vec<ExportedSymbol> {
        self.exports.get(file).cloned().unwrap_or_default()
    }

    /// Get imported symbols in a file
    pub fn get_imports(&self, file: &str) -> Vec<ImportedSymbol> {
        self.imports.get(file).cloned().unwrap_or_default()
    }

    /// Perform a complete dependency analysis for a file
    pub fn analyze(&self, file: &str) -> DependencyAnalysis {
        let direct = self.get_direct_dependencies(file);
        let transitive = self.get_all_dependencies(file);
        let dependents = self.get_all_dependents(file);
        let depth = self.calculate_depth(file);

        DependencyAnalysis {
            file: file.to_string(),
            direct_dependencies: direct,
            transitive_dependencies: transitive,
            dependents,
            depth,
        }
    }

    /// Calculate the maximum depth from this file to any leaf
    fn calculate_depth(&self, file: &str) -> usize {
        let mut visited = HashSet::new();

        fn dfs(
            graph: &DependencyGraph,
            node: &str,
            visited: &mut HashSet<String>,
            current_depth: usize,
        ) -> usize {
            if visited.contains(node) {
                return current_depth;
            }

            visited.insert(node.to_string());

            let mut max = current_depth;
            if let Some(deps) = graph.edges.get(node) {
                for dep in deps {
                    let depth = dfs(graph, dep, visited, current_depth + 1);
                    max = max.max(depth);
                }
            }

            visited.remove(node);
            max
        }

        dfs(self, file, &mut visited, 0)
    }

    /// Detect circular dependencies
    pub fn detect_cycles(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();

        for node in self.edges.keys() {
            if !visited.contains(node) {
                self.detect_cycles_util(
                    node,
                    &mut visited,
                    &mut rec_stack,
                    &mut Vec::new(),
                    &mut cycles,
                );
            }
        }

        cycles
    }

    fn detect_cycles_util(
        &self,
        node: &str,
        visited: &mut HashSet<String>,
        rec_stack: &mut HashSet<String>,
        path: &mut Vec<String>,
        cycles: &mut Vec<Vec<String>>,
    ) {
        visited.insert(node.to_string());
        rec_stack.insert(node.to_string());
        path.push(node.to_string());

        if let Some(neighbors) = self.edges.get(node) {
            for neighbor in neighbors {
                if !visited.contains(neighbor) {
                    self.detect_cycles_util(neighbor, visited, rec_stack, path, cycles);
                } else if rec_stack.contains(neighbor) {
                    // Found a cycle
                    let cycle_start = path.iter().position(|x| x == neighbor).unwrap();
                    let cycle: Vec<String> = path[cycle_start..].to_vec();
                    cycles.push(cycle);
                }
            }
        }

        path.pop();
        rec_stack.remove(node);
    }

    /// Get all files in the graph
    pub fn get_all_files(&self) -> Vec<String> {
        let mut files: HashSet<String> = self.edges.keys().cloned().collect();

        // Also include files that are only dependencies (no outgoing edges)
        for deps in self.edges.values() {
            files.extend(deps.iter().cloned());
        }

        files.into_iter().collect()
    }

    /// Get statistics about the dependency graph
    pub fn get_stats(&self) -> DependencyStats {
        let total_files = self.get_all_files().len();
        let total_edges = self.edges.values().map(|deps| deps.len()).sum();
        let cycles = self.detect_cycles();

        let mut max_dependencies = 0;
        let mut max_dependents = 0;
        let mut avg_dependencies = 0.0;

        for file in self.edges.keys() {
            let deps = self.get_direct_dependencies(file);
            let dependents = self.get_dependents(file);

            max_dependencies = max_dependencies.max(deps.len());
            max_dependents = max_dependents.max(dependents.len());
        }

        if !self.edges.is_empty() {
            avg_dependencies = total_edges as f64 / self.edges.len() as f64;
        }

        DependencyStats {
            total_files,
            total_edges,
            max_dependencies,
            max_dependents,
            avg_dependencies,
            circular_dependencies: cycles.len(),
        }
    }

    /// Find all files that have no dependencies (entry points)
    pub fn find_entry_points(&self) -> Vec<String> {
        let all_files: HashSet<String> = self.get_all_files().into_iter().collect();
        let mut has_incoming = HashSet::new();

        for deps in self.edges.values() {
            has_incoming.extend(deps.iter().cloned());
        }

        all_files.difference(&has_incoming).cloned().collect()
    }

    /// Find all files that are not dependencies of any other file (leaf nodes)
    pub fn find_leaf_nodes(&self) -> Vec<String> {
        self.edges
            .keys()
            .filter(|file| {
                self.edges
                    .get(*file)
                    .map(|deps| deps.is_empty())
                    .unwrap_or(true)
            })
            .cloned()
            .collect()
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the dependency graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyStats {
    pub total_files: usize,
    pub total_edges: usize,
    pub max_dependencies: usize,
    pub max_dependents: usize,
    pub avg_dependencies: f64,
    pub circular_dependencies: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_edge() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a.ts".to_string(), "b.ts".to_string());

        let deps = graph.get_direct_dependencies("a.ts");
        assert_eq!(deps.len(), 1);
        assert!(deps.contains(&"b.ts".to_string()));
    }

    #[test]
    fn test_transitive_dependencies() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a.ts".to_string(), "b.ts".to_string());
        graph.add_edge("b.ts".to_string(), "c.ts".to_string());

        let deps = graph.get_all_dependencies("a.ts");
        assert_eq!(deps.len(), 2);
        assert!(deps.contains(&"b.ts".to_string()));
        assert!(deps.contains(&"c.ts".to_string()));
    }

    #[test]
    fn test_dependents() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a.ts".to_string(), "c.ts".to_string());
        graph.add_edge("b.ts".to_string(), "c.ts".to_string());

        let dependents = graph.get_dependents("c.ts");
        assert_eq!(dependents.len(), 2);
        assert!(dependents.contains(&"a.ts".to_string()));
        assert!(dependents.contains(&"b.ts".to_string()));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("a.ts".to_string(), "b.ts".to_string());
        graph.add_edge("b.ts".to_string(), "c.ts".to_string());
        graph.add_edge("c.ts".to_string(), "a.ts".to_string());

        let cycles = graph.detect_cycles();
        assert!(!cycles.is_empty());
    }

    #[test]
    fn test_entry_points() {
        let mut graph = DependencyGraph::new();
        graph.add_edge("main.ts".to_string(), "utils.ts".to_string());
        graph.add_edge("utils.ts".to_string(), "helpers.ts".to_string());

        let entry_points = graph.find_entry_points();
        assert!(entry_points.contains(&"main.ts".to_string()));
        assert!(!entry_points.contains(&"utils.ts".to_string()));
    }
}
