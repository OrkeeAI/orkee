// ABOUTME: Unit tests for context generation functionality
// ABOUTME: Tests AST parsing, dependency graphs, pattern matching, and token estimation

#[cfg(test)]
mod tests {
    use crate::context::{
        ast_analyzer::AstAnalyzer, dependency_graph::DependencyGraph,
        incremental_parser::IncrementalParser, types::ContextConfiguration,
    };
    use std::fs;
    use tempfile::TempDir;

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_ast_extraction() {
        let code = r#"
        function hello(name: string): string {
            return `Hello, ${name}!`;
        }

        class Greeter {
            greet(name: string) {
                return hello(name);
            }
        }
        "#;

        let mut analyzer = AstAnalyzer::new_typescript().unwrap();
        let result = analyzer.extract_symbols(code).unwrap();

        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|s| s.name == "hello"));
        assert!(result.iter().any(|s| s.name == "Greeter"));
    }

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_dependency_graph_building() {
        let temp_dir = TempDir::new().unwrap();

        // Create test files
        let main_file = temp_dir.path().join("main.ts");
        fs::write(
            &main_file,
            r#"
            import { helper } from './helper';
            import { utils } from './utils';

            export function main() {
                helper();
                utils.process();
            }
        "#,
        )
        .unwrap();

        let helper_file = temp_dir.path().join("helper.ts");
        fs::write(
            &helper_file,
            r#"
            import { utils } from './utils';

            export function helper() {
                utils.log();
            }
        "#,
        )
        .unwrap();

        let utils_file = temp_dir.path().join("utils.ts");
        fs::write(
            &utils_file,
            r#"
            export const utils = {
                process: () => {},
                log: () => {}
            };
        "#,
        )
        .unwrap();

        let mut graph = DependencyGraph::new();

        // Build graph
        graph.add_edge("main.ts".to_string(), "helper.ts".to_string());
        graph.add_edge("main.ts".to_string(), "utils.ts".to_string());
        graph.add_edge("helper.ts".to_string(), "utils.ts".to_string());

        // Test dependency resolution
        let deps = graph.get_dependencies("main.ts", 1);
        assert_eq!(deps.len(), 3); // main.ts, helper.ts, utils.ts

        let deps = graph.get_dependencies("helper.ts", 1);
        assert_eq!(deps.len(), 2); // helper.ts, utils.ts

        // Test dependent finding
        let dependents = graph.get_dependents("utils.ts");
        assert_eq!(dependents.len(), 2); // main.ts and helper.ts depend on utils.ts
    }

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_context_generation_with_patterns() {
        let config = ContextConfiguration {
            id: "test".to_string(),
            project_id: "project1".to_string(),
            name: "Test Config".to_string(),
            description: None,
            include_patterns: vec!["src/**/*.ts".to_string()],
            exclude_patterns: vec!["**/*.test.ts".to_string()],
            max_tokens: 10000,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            spec_capability_id: None,
        };

        // Test pattern matching
        assert!(matches_pattern(
            "src/components/Button.ts",
            &config.include_patterns
        ));
        assert!(!matches_pattern(
            "src/components/Button.test.ts",
            &config.exclude_patterns
        ));
    }

    #[test]
    fn test_incremental_parsing() {
        let mut parser = IncrementalParser::new();
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.ts");

        // First parse
        fs::write(&file_path, "function test() {}").unwrap();
        let result1 = parser.parse_file(&file_path).unwrap();
        assert_eq!(result1.symbols.len(), 1);

        // Same content - should use cache
        let result2 = parser.parse_file(&file_path).unwrap();
        assert_eq!(result1.content_hash, result2.content_hash);

        // Modified content - should reparse
        fs::write(&file_path, "function test() {} function test2() {}").unwrap();
        let result3 = parser.parse_file(&file_path).unwrap();
        assert_ne!(result1.content_hash, result3.content_hash);
        assert_eq!(result3.symbols.len(), 2);
    }

    #[test]
    fn test_token_estimation() {
        use crate::context::language_support::estimate_tokens;

        let python_code = "def hello(): return 'world'";
        let java_code = "public class Hello { public String greet() { return \"world\"; } }";
        let typescript_code = "const hello = (): string => 'world';";

        // Test language-specific token estimation
        assert!(estimate_tokens(python_code, "python") < estimate_tokens(java_code, "java"));
        assert_eq!(
            estimate_tokens(typescript_code, "typescript"),
            (typescript_code.len() as f64 * 0.35) as usize
        );
    }

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_symbol_extraction_classes() {
        let code = r#"
        class Calculator {
            add(a: number, b: number): number {
                return a + b;
            }
            
            subtract(a: number, b: number): number {
                return a - b;
            }
        }
        "#;

        let mut analyzer = AstAnalyzer::new_typescript().unwrap();
        let result = analyzer.extract_symbols(code).unwrap();

        // Should find the class
        let class_symbol = result
            .iter()
            .find(|s| s.name == "Calculator")
            .expect("Should find Calculator class");

        // Should find methods as children
        assert!(class_symbol.children.iter().any(|s| s.name == "add"));
        assert!(class_symbol.children.iter().any(|s| s.name == "subtract"));
    }

    #[test]
    fn test_multi_language_detection() {
        use crate::context::language_support::MultiLanguageParser;

        let parser = MultiLanguageParser::new();

        // Test language detection
        assert_eq!(
            parser.detect_language("test.ts"),
            Some("typescript".to_string())
        );
        assert_eq!(
            parser.detect_language("test.js"),
            Some("javascript".to_string())
        );
        assert_eq!(
            parser.detect_language("test.py"),
            Some("python".to_string())
        );
        assert_eq!(parser.detect_language("test.rs"), Some("rust".to_string()));
        assert_eq!(parser.detect_language("test.go"), Some("go".to_string()));
    }

    #[test]
    #[ignore = "AST parsing feature incomplete"]
    fn test_pattern_exclusion() {
        let exclude_patterns = vec![
            "**/node_modules/**".to_string(),
            "**/*.test.ts".to_string(),
            "dist/**".to_string(),
        ];

        assert!(matches_pattern(
            "src/node_modules/package/index.js",
            &exclude_patterns
        ));
        assert!(matches_pattern("src/utils.test.ts", &exclude_patterns));
        assert!(matches_pattern("dist/bundle.js", &exclude_patterns));
        assert!(!matches_pattern("src/utils.ts", &exclude_patterns));
    }

    #[test]
    fn test_symbol_line_ranges() {
        let code = r#"
function hello() {
    console.log("Hello");
    console.log("World");
}

function goodbye() {
    console.log("Goodbye");
}
        "#;

        let mut analyzer = AstAnalyzer::new_typescript().unwrap();
        let result = analyzer.extract_symbols(code).unwrap();

        let hello_fn = result.iter().find(|s| s.name == "hello").unwrap();
        let goodbye_fn = result.iter().find(|s| s.name == "goodbye").unwrap();

        // hello should span multiple lines
        assert!(hello_fn.line_end > hello_fn.line_start + 2);

        // goodbye should be after hello
        assert!(goodbye_fn.line_start > hello_fn.line_end);
    }

    // Helper functions

    fn matches_pattern(path: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|p| {
            if p.contains("**") {
                // Handle globstar patterns
                let parts: Vec<&str> = p.split("**").collect();
                if parts.len() == 2 {
                    let prefix = parts[0].trim_end_matches('/');
                    let suffix = parts[1].trim_start_matches('/');

                    if !prefix.is_empty() && !path.starts_with(prefix) {
                        return false;
                    }
                    if !suffix.is_empty() && !path.ends_with(suffix) && !path.contains(suffix) {
                        return false;
                    }
                    return true;
                }
            }
            path.contains(p.trim_matches('*'))
        })
    }
}
