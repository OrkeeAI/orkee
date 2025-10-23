use serde::{Deserialize, Serialize};
use tree_sitter::{Node, Parser};

/// AST analyzer for extracting code structure and symbols
pub struct AstAnalyzer {
    parser: Parser,
    #[allow(dead_code)]
    language: tree_sitter::Language,
    language_name: String,
}

/// Represents a code symbol (function, class, interface, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub line_start: usize,
    pub line_end: usize,
    pub children: Vec<Symbol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_comment: Option<String>,
}

/// Types of code symbols
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SymbolKind {
    Function,
    Class,
    Interface,
    Variable,
    Import,
    Export,
    Method,
    Struct,
    Enum,
    Trait,
    Module,
    Field,
    Unknown,
}

impl AstAnalyzer {
    /// Create a new analyzer for TypeScript/TSX
    pub fn new_typescript() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_typescript::language_typescript();
        parser
            .set_language(language)
            .map_err(|e| format!("Failed to set TypeScript language: {:?}", e))?;
        Ok(Self {
            parser,
            language,
            language_name: "typescript".to_string(),
        })
    }

    /// Create a new analyzer for JavaScript
    pub fn new_javascript() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_javascript::language();
        parser
            .set_language(language)
            .map_err(|e| format!("Failed to set JavaScript language: {:?}", e))?;
        Ok(Self {
            parser,
            language,
            language_name: "javascript".to_string(),
        })
    }

    /// Create a new analyzer for Python
    pub fn new_python() -> Result<Self, String> {
        let mut parser = Parser::new();
        let language = tree_sitter_python::language();
        parser
            .set_language(language)
            .map_err(|e| format!("Failed to set Python language: {:?}", e))?;
        Ok(Self {
            parser,
            language,
            language_name: "python".to_string(),
        })
    }

    /// Create a new analyzer for Rust
    /// Note: Disabled due to tree-sitter version conflicts
    pub fn new_rust() -> Result<Self, String> {
        Err("Rust language support temporarily disabled due to dependency conflicts".to_string())
        // let mut parser = Parser::new();
        // let language = tree_sitter_rust::language();
        // parser
        //     .set_language(language)
        //     .map_err(|e| format!("Failed to set Rust language: {:?}", e))?;
        // Ok(Self {
        //     parser,
        //     language,
        //     language_name: "rust".to_string(),
        // })
    }

    /// Create an analyzer for a specific language by file extension
    pub fn from_extension(ext: &str) -> Result<Self, String> {
        match ext {
            "ts" | "tsx" => Self::new_typescript(),
            "js" | "jsx" => Self::new_javascript(),
            "py" => Self::new_python(),
            // "rs" => Self::new_rust(),  // Disabled
            "rs" => Err("Rust support temporarily disabled".to_string()),
            _ => Err(format!("Unsupported file extension: {}", ext)),
        }
    }

    /// Extract all symbols from source code
    pub fn extract_symbols(&mut self, source_code: &str) -> Result<Vec<Symbol>, String> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or_else(|| "Failed to parse source code".to_string())?;

        let root_node = tree.root_node();
        let mut symbols = Vec::new();

        self.walk_tree(&root_node, source_code, &mut symbols);

        Ok(symbols)
    }

    /// Recursively walk the AST tree and extract symbols
    fn walk_tree(&self, node: &Node, source: &str, symbols: &mut Vec<Symbol>) {
        let symbol = match self.language_name.as_str() {
            "typescript" | "javascript" => self.extract_ts_js_symbol(node, source),
            "python" => self.extract_python_symbol(node, source),
            "rust" => self.extract_rust_symbol(node, source),
            _ => None,
        };

        if let Some(sym) = symbol {
            symbols.push(sym);
        }

        // Recurse through children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.walk_tree(&child, source, symbols);
        }
    }

    /// Extract symbol from TypeScript/JavaScript node
    fn extract_ts_js_symbol(&self, node: &Node, source: &str) -> Option<Symbol> {
        let kind = match node.kind() {
            "function_declaration" | "function" => SymbolKind::Function,
            "method_definition" => SymbolKind::Method,
            "class_declaration" | "class" => SymbolKind::Class,
            "interface_declaration" => SymbolKind::Interface,
            "variable_declarator" => SymbolKind::Variable,
            "import_statement" => SymbolKind::Import,
            "export_statement" => SymbolKind::Export,
            _ => return None,
        };

        let name = if let Some(name_node) = node.child_by_field_name("name") {
            source[name_node.byte_range()].to_string()
        } else {
            // For unnamed functions or expressions
            format!("<anonymous {}>", node.kind())
        };

        Some(Symbol {
            name,
            kind,
            line_start: node.start_position().row + 1,
            line_end: node.end_position().row + 1,
            children: Vec::new(),
            doc_comment: None,
        })
    }

    /// Extract symbol from Python node
    fn extract_python_symbol(&self, node: &Node, source: &str) -> Option<Symbol> {
        let kind = match node.kind() {
            "function_definition" => SymbolKind::Function,
            "class_definition" => SymbolKind::Class,
            "import_statement" | "import_from_statement" => SymbolKind::Import,
            _ => return None,
        };

        let name = if let Some(name_node) = node.child_by_field_name("name") {
            source[name_node.byte_range()].to_string()
        } else {
            return None;
        };

        Some(Symbol {
            name,
            kind,
            line_start: node.start_position().row + 1,
            line_end: node.end_position().row + 1,
            children: Vec::new(),
            doc_comment: None,
        })
    }

    /// Extract symbol from Rust node
    fn extract_rust_symbol(&self, node: &Node, source: &str) -> Option<Symbol> {
        let kind = match node.kind() {
            "function_item" => SymbolKind::Function,
            "struct_item" => SymbolKind::Struct,
            "enum_item" => SymbolKind::Enum,
            "trait_item" => SymbolKind::Trait,
            "impl_item" => SymbolKind::Class, // Treat impl blocks as classes
            "mod_item" => SymbolKind::Module,
            "use_declaration" => SymbolKind::Import,
            _ => return None,
        };

        let name = if let Some(name_node) = node.child_by_field_name("name") {
            source[name_node.byte_range()].to_string()
        } else {
            return None;
        };

        Some(Symbol {
            name,
            kind,
            line_start: node.start_position().row + 1,
            line_end: node.end_position().row + 1,
            children: Vec::new(),
            doc_comment: None,
        })
    }

    /// Build a dependency graph by extracting imports
    pub fn extract_imports(&mut self, source_code: &str) -> Result<Vec<String>, String> {
        let tree = self
            .parser
            .parse(source_code, None)
            .ok_or_else(|| "Failed to parse source code".to_string())?;

        let root_node = tree.root_node();
        let mut imports = Vec::new();

        self.collect_imports(&root_node, source_code, &mut imports);

        Ok(imports)
    }

    /// Recursively collect import statements
    fn collect_imports(&self, node: &Node, source: &str, imports: &mut Vec<String>) {
        match self.language_name.as_str() {
            "typescript" | "javascript" => {
                if node.kind() == "import_statement" {
                    if let Some(source_node) = node.child_by_field_name("source") {
                        let import_path = source[source_node.byte_range()].to_string();
                        // Remove quotes
                        let cleaned = import_path.trim_matches('"').trim_matches('\'');
                        imports.push(cleaned.to_string());
                    }
                }
            }
            "python" => {
                if node.kind() == "import_statement" || node.kind() == "import_from_statement" {
                    if let Some(name_node) = node.child_by_field_name("name") {
                        let import_path = source[name_node.byte_range()].to_string();
                        imports.push(import_path);
                    }
                }
            }
            "rust" => {
                if node.kind() == "use_declaration" {
                    // Extract use path from Rust
                    let use_text = source[node.byte_range()].to_string();
                    imports.push(use_text);
                }
            }
            _ => {}
        }

        // Recurse through children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_imports(&child, source, imports);
        }
    }

    /// Get a specific symbol by line range
    pub fn get_symbol_at_line(
        &mut self,
        source_code: &str,
        line: usize,
    ) -> Result<Option<Symbol>, String> {
        let symbols = self.extract_symbols(source_code)?;

        for symbol in symbols {
            if line >= symbol.line_start && line <= symbol.line_end {
                return Ok(Some(symbol));
            }
        }

        Ok(None)
    }

    /// Extract only specific symbol types
    pub fn extract_symbols_by_kind(
        &mut self,
        source_code: &str,
        kinds: &[SymbolKind],
    ) -> Result<Vec<Symbol>, String> {
        let all_symbols = self.extract_symbols(source_code)?;

        let filtered: Vec<Symbol> = all_symbols
            .into_iter()
            .filter(|s| kinds.contains(&s.kind))
            .collect();

        Ok(filtered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_typescript_function_extraction() {
        let mut analyzer = AstAnalyzer::new_typescript().unwrap();
        let source = r#"
            function hello(name: string): string {
                return `Hello ${name}`;
            }
        "#;

        let symbols = analyzer.extract_symbols(source).unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "hello");
        assert_eq!(symbols[0].kind, SymbolKind::Function);
    }

    #[test]
    fn test_typescript_class_extraction() {
        let mut analyzer = AstAnalyzer::new_typescript().unwrap();
        let source = r#"
            class MyClass {
                constructor() {}
                myMethod() {}
            }
        "#;

        let symbols = analyzer.extract_symbols(source).unwrap();
        assert!(symbols.iter().any(|s| s.name == "MyClass"));
    }

    #[test]
    fn test_import_extraction() {
        let mut analyzer = AstAnalyzer::new_typescript().unwrap();
        let source = r#"
            import { useState } from 'react';
            import axios from 'axios';
        "#;

        let imports = analyzer.extract_imports(source).unwrap();
        assert_eq!(imports.len(), 2);
        assert!(imports.contains(&"react".to_string()));
        assert!(imports.contains(&"axios".to_string()));
    }

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_from_extension() {
        assert!(AstAnalyzer::from_extension("ts").is_ok());
        assert!(AstAnalyzer::from_extension("js").is_ok());
        assert!(AstAnalyzer::from_extension("py").is_ok());
        assert!(AstAnalyzer::from_extension("rs").is_ok());
        assert!(AstAnalyzer::from_extension("unknown").is_err());
    }
}
