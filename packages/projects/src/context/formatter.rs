use crate::context::ast_analyzer::{Symbol, SymbolKind};
use crate::context::language_support::{LanguageConfig, LANGUAGE_CONFIGS, estimate_tokens, remove_comments};
use std::collections::HashMap;

pub struct ContextFormatter {
    include_imports: bool,
    include_comments: bool,
    max_line_length: usize,
    group_by_language: bool,
}

impl ContextFormatter {
    pub fn new() -> Self {
        Self {
            include_imports: true,
            include_comments: false,
            max_line_length: 120,
            group_by_language: true,
        }
    }

    pub fn with_config(
        include_imports: bool,
        include_comments: bool,
        max_line_length: usize,
    ) -> Self {
        Self {
            include_imports,
            include_comments,
            max_line_length,
            group_by_language: true,
        }
    }

    /// Format multiple files into a cohesive context string
    pub fn format_context(&self, files: Vec<ParsedFileInfo>) -> FormattedContext {
        let mut output = String::new();
        let mut total_tokens = 0;
        let mut file_count = 0;

        if self.group_by_language {
            // Group files by language
            let mut by_language: HashMap<String, Vec<ParsedFileInfo>> = HashMap::new();
            
            for file in files {
                by_language
                    .entry(file.language.clone())
                    .or_insert_with(Vec::new)
                    .push(file);
            }

            // Format each language group
            for (language, files) in by_language {
                let config = LANGUAGE_CONFIGS.get(&language);
                let lang_name = config.map(|c| c.name.as_str()).unwrap_or(&language);
                
                output.push_str(&format!("\n## {} Files\n\n", lang_name));

                for file in files {
                    let formatted = self.format_file(&file);
                    output.push_str(&formatted);
                    total_tokens += estimate_tokens(&formatted, &file.language);
                    file_count += 1;
                }
            }
        } else {
            // Format files in order
            for file in files {
                let formatted = self.format_file(&file);
                output.push_str(&formatted);
                total_tokens += estimate_tokens(&formatted, &file.language);
                file_count += 1;
            }
        }

        FormattedContext {
            content: output,
            total_tokens,
            file_count,
        }
    }

    fn format_file(&self, file: &ParsedFileInfo) -> String {
        let mut output = String::new();

        // File header
        output.push_str(&format!("\n### File: `{}`\n\n", file.path));
        
        let config = LANGUAGE_CONFIGS.get(&file.language);
        if let Some(cfg) = config {
            output.push_str(&format!("**Language**: {}\n\n", cfg.name));
        }

        // Symbol summary
        if !file.symbols.is_empty() {
            output.push_str("**Symbols**:\n\n");
            for symbol in &file.symbols {
                let icon = self.get_symbol_icon(&symbol.kind);
                output.push_str(&format!(
                    "- {} **`{}`** (lines {}-{})\n",
                    icon,
                    symbol.name,
                    symbol.line_start,
                    symbol.line_end
                ));
            }
            output.push('\n');
        }

        // Dependencies/Imports
        if self.include_imports && !file.imports.is_empty() {
            output.push_str("**Dependencies**:\n\n");
            for import in &file.imports {
                let truncated = if import.len() > self.max_line_length {
                    format!("{}...", &import[..self.max_line_length])
                } else {
                    import.clone()
                };
                output.push_str(&format!("- `{}`\n", truncated));
            }
            output.push('\n');
        }

        // Content preview or full content
        if let Some(content) = &file.content {
            let display_content = if self.include_comments {
                content.clone()
            } else {
                remove_comments(content, &file.language)
            };

            output.push_str("```");
            output.push_str(&file.language);
            output.push('\n');
            output.push_str(&display_content);
            if !display_content.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```\n\n");
        }

        output.push_str("---\n\n");
        output
    }

    fn get_symbol_icon(&self, kind: &SymbolKind) -> &str {
        match kind {
            SymbolKind::Function => "ƒ",
            SymbolKind::Method => "ƒ",
            SymbolKind::Class => "C",
            SymbolKind::Interface => "I",
            SymbolKind::Struct => "S",
            SymbolKind::Enum => "E",
            SymbolKind::Trait => "T",
            SymbolKind::Variable => "v",
            SymbolKind::Field => "v",
            SymbolKind::Unknown => "?",
        }
    }
}

#[derive(Clone)]
pub struct ParsedFileInfo {
    pub path: String,
    pub language: String,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<String>,
    pub content: Option<String>,
}

pub struct FormattedContext {
    pub content: String,
    pub total_tokens: usize,
    pub file_count: usize,
}

impl Default for ContextFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// Create a summary context (symbols only, no full content)
pub fn create_summary_context(files: Vec<ParsedFileInfo>) -> FormattedContext {
    let formatter = ContextFormatter {
        include_imports: true,
        include_comments: false,
        max_line_length: 120,
        group_by_language: true,
    };

    // Strip content from files for summary
    let summary_files: Vec<ParsedFileInfo> = files.into_iter().map(|mut f| {
        f.content = None;
        f
    }).collect();

    formatter.format_context(summary_files)
}

/// Create a detailed context (with full file content)
pub fn create_detailed_context(files: Vec<ParsedFileInfo>, include_comments: bool) -> FormattedContext {
    let formatter = ContextFormatter {
        include_imports: true,
        include_comments,
        max_line_length: 120,
        group_by_language: true,
    };

    formatter.format_context(files)
}

/// Optimize context to fit within token limit
pub fn optimize_context_for_tokens(
    files: Vec<ParsedFileInfo>,
    max_tokens: usize,
) -> FormattedContext {
    let mut current_files = files;
    let mut formatter = ContextFormatter::new();

    // Try without comments first
    formatter.include_comments = false;
    let mut result = formatter.format_context(current_files.clone());

    if result.total_tokens <= max_tokens {
        return result;
    }

    // Try summary only (no content)
    let summary_files: Vec<ParsedFileInfo> = current_files.into_iter().map(|mut f| {
        f.content = None;
        f
    }).collect();

    result = formatter.format_context(summary_files.clone());

    if result.total_tokens <= max_tokens {
        return result;
    }

    // Last resort: truncate files
    let available_per_file = max_tokens / summary_files.len().max(1);
    let truncated_files: Vec<ParsedFileInfo> = summary_files.into_iter().map(|mut f| {
        if f.symbols.len() > available_per_file / 10 {
            f.symbols.truncate(available_per_file / 10);
        }
        f
    }).collect();

    formatter.format_context(truncated_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file() {
        let formatter = ContextFormatter::new();
        
        let file = ParsedFileInfo {
            path: "test.ts".to_string(),
            language: "typescript".to_string(),
            symbols: vec![
                Symbol {
                    name: "testFunction".to_string(),
                    kind: SymbolKind::Function,
                    line_start: 1,
                    line_end: 5,
                    children: vec![],
                    doc_comment: None,
                }
            ],
            imports: vec!["import { foo } from './bar'".to_string()],
            content: Some("function testFunction() {\n  return true;\n}".to_string()),
        };

        let formatted = formatter.format_file(&file);
        
        assert!(formatted.contains("test.ts"));
        assert!(formatted.contains("testFunction"));
        assert!(formatted.contains("TypeScript"));
    }

    #[test]
    fn test_summary_context() {
        let files = vec![
            ParsedFileInfo {
                path: "a.ts".to_string(),
                language: "typescript".to_string(),
                symbols: vec![],
                imports: vec![],
                content: Some("const x = 10;".to_string()),
            }
        ];

        let result = create_summary_context(files);
        
        // Summary should not include content
        assert!(!result.content.contains("const x = 10"));
        assert!(result.content.contains("a.ts"));
    }

    #[test]
    fn test_context_grouping() {
        let formatter = ContextFormatter::new();
        
        let files = vec![
            ParsedFileInfo {
                path: "a.ts".to_string(),
                language: "typescript".to_string(),
                symbols: vec![],
                imports: vec![],
                content: None,
            },
            ParsedFileInfo {
                path: "b.rs".to_string(),
                language: "rust".to_string(),
                symbols: vec![],
                imports: vec![],
                content: None,
            },
            ParsedFileInfo {
                path: "c.ts".to_string(),
                language: "typescript".to_string(),
                symbols: vec![],
                imports: vec![],
                content: None,
            },
        ];

        let result = formatter.format_context(files);
        
        // Should group by language
        assert!(result.content.contains("## TypeScript Files"));
        assert!(result.content.contains("## Rust Files"));
        assert_eq!(result.file_count, 3);
    }
}
