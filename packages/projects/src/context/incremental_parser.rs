use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::SystemTime;
use sha2::{Sha256, Digest};
use tree_sitter::{Parser, Tree};
use crate::context::ast_analyzer::{Symbol, SymbolKind};

/// Parser with incremental caching based on file content SHA256 hashes
pub struct IncrementalParser {
    cache: HashMap<String, ParsedFile>,
    parsers: HashMap<String, Parser>,
}

#[derive(Clone)]
pub struct ParsedFile {
    pub content_hash: String,
    pub symbols: Vec<Symbol>,
    pub last_modified: SystemTime,
    pub dependencies: Vec<String>,
}

impl IncrementalParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Initialize TypeScript parser
        let mut ts_parser = Parser::new();
        ts_parser.set_language(tree_sitter_typescript::language_typescript()).unwrap();
        parsers.insert("typescript".to_string(), ts_parser);

        // Initialize JavaScript parser
        let mut js_parser = Parser::new();
        js_parser.set_language(tree_sitter_javascript::language()).unwrap();
        parsers.insert("javascript".to_string(), js_parser);

        // Initialize Rust parser
        let mut rust_parser = Parser::new();
        rust_parser.set_language(tree_sitter_rust::language()).unwrap();
        parsers.insert("rust".to_string(), rust_parser);

        // Initialize Python parser
        let mut python_parser = Parser::new();
        python_parser.set_language(tree_sitter_python::language()).unwrap();
        parsers.insert("python".to_string(), python_parser);

        Self {
            cache: HashMap::new(),
            parsers,
        }
    }

    /// Parse a file with caching - returns cached result if content unchanged
    pub fn parse_file(&mut self, path: &PathBuf) -> Result<ParsedFile, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;

        // Calculate content hash for cache lookup
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let hash = format!("{:x}", hasher.finalize());

        // Check cache
        let path_str = path.to_string_lossy().to_string();
        if let Some(cached) = self.cache.get(&path_str) {
            if cached.content_hash == hash {
                tracing::debug!("Cache hit for: {}", path_str);
                return Ok(cached.clone());
            }
        }

        tracing::debug!("Cache miss for: {}, parsing...", path_str);

        // Determine language from extension
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let language = match extension {
            "ts" | "tsx" => "typescript",
            "js" | "jsx" => "javascript",
            "rs" => "rust",
            "py" => "python",
            _ => return Err(format!("Unsupported file extension: {}", extension)),
        };

        // Parse with appropriate parser
        let parser = self.parsers.get_mut(language)
            .ok_or_else(|| format!("No parser for language: {}", language))?;

        let tree = parser.parse(&content, None)
            .ok_or_else(|| "Failed to parse file".to_string())?;

        // Extract symbols and dependencies
        let symbols = extract_symbols(&tree, &content, language)?;
        let dependencies = extract_dependencies(&tree, &content, language);

        // Store in cache
        let parsed = ParsedFile {
            content_hash: hash,
            symbols,
            last_modified: SystemTime::now(),
            dependencies,
        };

        self.cache.insert(path_str.clone(), parsed.clone());
        Ok(parsed)
    }

    /// Invalidate cache entries older than the specified duration
    pub fn invalidate_stale_entries(&mut self, max_age_secs: u64) {
        let now = SystemTime::now();
        self.cache.retain(|_, file| {
            if let Ok(elapsed) = now.duration_since(file.last_modified) {
                elapsed.as_secs() < max_age_secs
            } else {
                false
            }
        });
    }

    /// Clear all cache entries
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        CacheStats {
            total_entries: self.cache.len(),
            total_symbols: self.cache.values().map(|f| f.symbols.len()).sum(),
        }
    }
}

pub struct CacheStats {
    pub total_entries: usize,
    pub total_symbols: usize,
}

fn extract_symbols(tree: &Tree, content: &str, language: &str) -> Result<Vec<Symbol>, String> {
    let root = tree.root_node();
    let mut symbols = Vec::new();

    let mut cursor = root.walk();
    extract_symbols_recursive(&mut cursor, content, language, &mut symbols);

    Ok(symbols)
}

fn extract_symbols_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &str,
    language: &str,
    symbols: &mut Vec<Symbol>,
) {
    let node = cursor.node();

    // Language-specific symbol extraction
    match language {
        "typescript" | "javascript" => {
            match node.kind() {
                "function_declaration" | "function" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Function,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                "class_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Class,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                "interface_declaration" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Interface,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                _ => {}
            }
        }
        "rust" => {
            match node.kind() {
                "function_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Function,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                "struct_item" | "enum_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Struct,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                "trait_item" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Trait,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                _ => {}
            }
        }
        "python" => {
            match node.kind() {
                "function_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Function,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                "class_definition" => {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let name = &source[name_node.byte_range()];
                        symbols.push(Symbol {
                            name: name.to_string(),
                            kind: SymbolKind::Class,
                            line_start: node.start_position().row + 1,
                            line_end: node.end_position().row + 1,
                            children: vec![],
                        });
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }

    // Recurse into children
    if cursor.goto_first_child() {
        loop {
            extract_symbols_recursive(cursor, source, language, symbols);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

fn extract_dependencies(tree: &Tree, content: &str, language: &str) -> Vec<String> {
    let root = tree.root_node();
    let mut dependencies = Vec::new();

    let mut cursor = root.walk();
    extract_dependencies_recursive(&mut cursor, content, language, &mut dependencies);

    dependencies
}

fn extract_dependencies_recursive(
    cursor: &mut tree_sitter::TreeCursor,
    source: &str,
    language: &str,
    dependencies: &mut Vec<String>,
) {
    let node = cursor.node();

    match language {
        "typescript" | "javascript" => {
            if node.kind() == "import_statement" {
                let text = &source[node.byte_range()];
                dependencies.push(text.to_string());
            }
        }
        "rust" => {
            if node.kind() == "use_declaration" {
                let text = &source[node.byte_range()];
                dependencies.push(text.to_string());
            }
        }
        "python" => {
            if node.kind() == "import_statement" || node.kind() == "import_from_statement" {
                let text = &source[node.byte_range()];
                dependencies.push(text.to_string());
            }
        }
        _ => {}
    }

    if cursor.goto_first_child() {
        loop {
            extract_dependencies_recursive(cursor, source, language, dependencies);
            if !cursor.goto_next_sibling() {
                break;
            }
        }
        cursor.goto_parent();
    }
}

impl Default for IncrementalParser {
    fn default() -> Self {
        Self::new()
    }
}
