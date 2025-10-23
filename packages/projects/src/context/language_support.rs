use lazy_static::lazy_static;
use std::collections::HashMap;
use tree_sitter::Parser;

#[derive(Clone)]
pub struct LanguageConfig {
    pub name: String,
    pub file_extensions: Vec<&'static str>,
    pub comment_single: String,
    pub comment_multi_start: String,
    pub comment_multi_end: String,
    pub token_multiplier: f64,
}

lazy_static! {
    pub static ref LANGUAGE_CONFIGS: HashMap<String, LanguageConfig> = {
        let mut configs = HashMap::new();

        // TypeScript/TSX
        configs.insert("typescript".to_string(), LanguageConfig {
            name: "TypeScript".to_string(),
            file_extensions: vec!["ts", "tsx"],
            comment_single: "//".to_string(),
            comment_multi_start: "/*".to_string(),
            comment_multi_end: "*/".to_string(),
            token_multiplier: 0.35,
        });

        // JavaScript/JSX
        configs.insert("javascript".to_string(), LanguageConfig {
            name: "JavaScript".to_string(),
            file_extensions: vec!["js", "jsx"],
            comment_single: "//".to_string(),
            comment_multi_start: "/*".to_string(),
            comment_multi_end: "*/".to_string(),
            token_multiplier: 0.35,
        });

        // Python
        configs.insert("python".to_string(), LanguageConfig {
            name: "Python".to_string(),
            file_extensions: vec!["py"],
            comment_single: "#".to_string(),
            comment_multi_start: "\"\"\"".to_string(),
            comment_multi_end: "\"\"\"".to_string(),
            token_multiplier: 0.30,
        });

        // Rust
        configs.insert("rust".to_string(), LanguageConfig {
            name: "Rust".to_string(),
            file_extensions: vec!["rs"],
            comment_single: "//".to_string(),
            comment_multi_start: "/*".to_string(),
            comment_multi_end: "*/".to_string(),
            token_multiplier: 0.35,
        });

        // Go
        configs.insert("go".to_string(), LanguageConfig {
            name: "Go".to_string(),
            file_extensions: vec!["go"],
            comment_single: "//".to_string(),
            comment_multi_start: "/*".to_string(),
            comment_multi_end: "*/".to_string(),
            token_multiplier: 0.30,
        });

        // Java
        configs.insert("java".to_string(), LanguageConfig {
            name: "Java".to_string(),
            file_extensions: vec!["java"],
            comment_single: "//".to_string(),
            comment_multi_start: "/*".to_string(),
            comment_multi_end: "*/".to_string(),
            token_multiplier: 0.40,
        });

        configs
    };
}

pub struct MultiLanguageParser {
    parsers: HashMap<String, Parser>,
}

impl MultiLanguageParser {
    pub fn new() -> Self {
        let mut parsers = HashMap::new();

        // Initialize TypeScript parser
        let mut ts_parser = Parser::new();
        ts_parser
            .set_language(tree_sitter_typescript::language_typescript())
            .unwrap();
        parsers.insert("typescript".to_string(), ts_parser);

        // Initialize JavaScript parser
        let mut js_parser = Parser::new();
        js_parser
            .set_language(tree_sitter_javascript::language())
            .unwrap();
        parsers.insert("javascript".to_string(), js_parser);

        // Rust parser disabled due to dependency conflicts
        // let mut rust_parser = Parser::new();
        // rust_parser
        //     .set_language(tree_sitter_rust::language())
        //     .unwrap();
        // parsers.insert("rust".to_string(), rust_parser);

        // Initialize Python parser
        let mut python_parser = Parser::new();
        python_parser
            .set_language(tree_sitter_python::language())
            .unwrap();
        parsers.insert("python".to_string(), python_parser);

        Self { parsers }
    }

    /// Detect programming language from file path
    pub fn detect_language(&self, file_path: &str) -> Option<String> {
        let extension = std::path::Path::new(file_path)
            .extension()
            .and_then(|e| e.to_str())?;

        for (lang_name, config) in LANGUAGE_CONFIGS.iter() {
            if config.file_extensions.contains(&extension) {
                return Some(lang_name.clone());
            }
        }

        None
    }

    /// Get parser for a specific language
    pub fn get_parser(&mut self, language: &str) -> Option<&mut Parser> {
        self.parsers.get_mut(language)
    }

    /// Get language configuration
    pub fn get_config(language: &str) -> Option<&LanguageConfig> {
        LANGUAGE_CONFIGS.get(language)
    }

    /// Get all supported languages
    pub fn supported_languages() -> Vec<String> {
        LANGUAGE_CONFIGS.keys().cloned().collect()
    }

    /// Check if a file extension is supported
    pub fn is_supported_extension(ext: &str) -> bool {
        LANGUAGE_CONFIGS
            .values()
            .any(|config| config.file_extensions.contains(&ext))
    }
}

/// Estimate token count for content based on language
pub fn estimate_tokens(content: &str, language: &str) -> usize {
    let config = LANGUAGE_CONFIGS.get(language);
    let multiplier = config.map(|c| c.token_multiplier).unwrap_or(0.25);

    (content.len() as f64 * multiplier) as usize
}

/// Remove comments from source code based on language
pub fn remove_comments(content: &str, language: &str) -> String {
    let config = match LANGUAGE_CONFIGS.get(language) {
        Some(c) => c,
        None => return content.to_string(),
    };

    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut in_single_line_comment = false;
    let mut in_multi_line_comment = false;
    let mut in_string = false;
    let mut string_delimiter = '\0';

    while let Some(ch) = chars.next() {
        // Handle string literals to avoid removing comments in strings
        if !in_single_line_comment && !in_multi_line_comment
            && (ch == '"' || ch == '\'') {
                if !in_string {
                    in_string = true;
                    string_delimiter = ch;
                } else if ch == string_delimiter {
                    in_string = false;
                }
                result.push(ch);
                continue;
            }

        if in_string {
            result.push(ch);
            continue;
        }

        // Handle single-line comments
        if !in_multi_line_comment
            && ch.to_string() + &chars.peek().unwrap_or(&' ').to_string() == config.comment_single
        {
            in_single_line_comment = true;
            chars.next(); // Skip next char
            continue;
        }

        if in_single_line_comment {
            if ch == '\n' {
                in_single_line_comment = false;
                result.push(ch);
            }
            continue;
        }

        // Handle multi-line comments
        if !in_single_line_comment {
            let next_chars: String = chars
                .clone()
                .take(config.comment_multi_start.len() - 1)
                .collect();
            if ch.to_string() + &next_chars == config.comment_multi_start {
                in_multi_line_comment = true;
                for _ in 0..config.comment_multi_start.len() - 1 {
                    chars.next();
                }
                continue;
            }
        }

        if in_multi_line_comment {
            let next_chars: String = chars
                .clone()
                .take(config.comment_multi_end.len() - 1)
                .collect();
            if ch.to_string() + &next_chars == config.comment_multi_end {
                in_multi_line_comment = false;
                for _ in 0..config.comment_multi_end.len() - 1 {
                    chars.next();
                }
            }
            continue;
        }

        result.push(ch);
    }

    result
}

/// Get language statistics from content
pub struct LanguageStats {
    pub language: String,
    pub line_count: usize,
    pub char_count: usize,
    pub estimated_tokens: usize,
    pub has_functions: bool,
    pub has_classes: bool,
}

pub fn analyze_language_stats(content: &str, language: &str) -> LanguageStats {
    let line_count = content.lines().count();
    let char_count = content.len();
    let estimated_tokens = estimate_tokens(content, language);

    // Simple heuristics for detecting functions and classes
    let has_functions = match language {
        "typescript" | "javascript" => {
            content.contains("function ") || content.contains("const ") && content.contains("=>")
        }
        "python" => content.contains("def "),
        "rust" => content.contains("fn "),
        "go" => content.contains("func "),
        "java" => content.contains("public ") || content.contains("private "),
        _ => false,
    };

    let has_classes = match language {
        "typescript" | "javascript" | "python" | "java" => content.contains("class "),
        "rust" => content.contains("struct ") || content.contains("impl "),
        _ => false,
    };

    LanguageStats {
        language: language.to_string(),
        line_count,
        char_count,
        estimated_tokens,
        has_functions,
        has_classes,
    }
}

impl Default for MultiLanguageParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        let parser = MultiLanguageParser::new();

        assert_eq!(
            parser.detect_language("test.ts"),
            Some("typescript".to_string())
        );
        assert_eq!(
            parser.detect_language("test.js"),
            Some("javascript".to_string())
        );
        assert_eq!(parser.detect_language("test.rs"), Some("rust".to_string()));
        assert_eq!(
            parser.detect_language("test.py"),
            Some("python".to_string())
        );
        assert_eq!(parser.detect_language("test.unknown"), None);
    }

    #[test]
    fn test_token_estimation() {
        let ts_code = "const hello = (): string => 'world';";
        let tokens = estimate_tokens(ts_code, "typescript");
        assert!(tokens > 0);
        assert!(tokens < ts_code.len()); // Should be less than character count
    }

    #[test]
    fn test_comment_removal() {
        let code = r#"
        // This is a comment
        const x = 10; // inline comment
        /* multi
           line
           comment */
        const y = 20;
        "#;

        let cleaned = remove_comments(code, "javascript");
        assert!(!cleaned.contains("This is a comment"));
        assert!(cleaned.contains("const x = 10;"));
        assert!(cleaned.contains("const y = 20;"));
    }

    #[test]
    fn test_supported_languages() {
        let languages = MultiLanguageParser::supported_languages();
        assert!(languages.contains(&"typescript".to_string()));
        assert!(languages.contains(&"javascript".to_string()));
        assert!(languages.contains(&"rust".to_string()));
        assert!(languages.contains(&"python".to_string()));
    }

    #[test]
    fn test_extension_support() {
        assert!(MultiLanguageParser::is_supported_extension("ts"));
        assert!(MultiLanguageParser::is_supported_extension("js"));
        assert!(MultiLanguageParser::is_supported_extension("rs"));
        assert!(MultiLanguageParser::is_supported_extension("py"));
        assert!(!MultiLanguageParser::is_supported_extension("unknown"));
    }
}
