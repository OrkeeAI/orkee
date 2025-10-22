use crate::context::ast_analyzer::{AstAnalyzer, Symbol, SymbolKind};
use crate::openspec::types::{SpecCapability, SpecRequirement, SpecScenario};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Builder for creating spec-aware context
pub struct SpecContextBuilder {
    analyzer: Option<AstAnalyzer>,
}

/// Validation result for a spec requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub requirement_id: String,
    pub requirement_name: String,
    pub status: ValidationStatus,
    pub details: Vec<String>,
    pub code_references: Vec<CodeReference>,
}

/// Status of a validation check
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ValidationStatus {
    Passed,
    Failed,
    Warning,
    Unknown,
}

/// Reference to implementing code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeReference {
    pub file_path: String,
    pub line_number: usize,
    pub symbol_name: String,
    pub snippet: Option<String>,
}

/// AST to Spec mapping for database storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstSpecMapping {
    pub id: Option<String>,
    pub project_id: String,
    pub file_path: String,
    pub symbol_name: String,
    pub symbol_type: String,
    pub line_number: Option<i32>,
    pub requirement_id: Option<String>,
    pub confidence: f64,
    pub verified: bool,
}

impl SpecContextBuilder {
    /// Create a new spec context builder
    pub fn new() -> Self {
        Self { analyzer: None }
    }

    /// Initialize analyzer for a specific language
    fn ensure_analyzer(&mut self, file_ext: &str) -> Result<(), String> {
        if self.analyzer.is_none() {
            self.analyzer = Some(AstAnalyzer::from_extension(file_ext)?);
        }
        Ok(())
    }

    /// Generate context for a specific capability
    pub async fn build_capability_context(
        &mut self,
        capability: &SpecCapability,
        requirements: &[SpecRequirement],
        project_root: &str,
    ) -> Result<String, String> {
        let mut context = String::new();

        // 1. Add capability header
        context.push_str(&format!("# Capability: {}\n\n", capability.name));
        
        if let Some(purpose) = &capability.purpose_markdown {
            context.push_str(&format!("## Purpose\n{}\n\n", purpose));
        }

        // 2. Add spec content
        context.push_str(&format!("## Specification\n{}\n\n", capability.spec_markdown));

        // 3. Add requirements
        context.push_str("## Requirements\n\n");
        for req in requirements {
            context.push_str(&format!(
                "### {}\n{}\n\n",
                req.name, req.content_markdown
            ));
        }

        // 4. Find and add implementing code
        context.push_str("## Implementation References\n\n");
        let implementations = self
            .find_implementations(capability, requirements, project_root)
            .await;

        if implementations.is_empty() {
            context.push_str("*No implementations found. Code may need to be written.*\n\n");
        } else {
            for (file, symbols) in implementations {
                context.push_str(&format!("### File: `{}`\n", file));
                for symbol in symbols {
                    context.push_str(&format!(
                        "- `{}` (lines {}-{})\n",
                        symbol.name, symbol.line_start, symbol.line_end
                    ));
                }
                context.push_str("\n");
            }
        }

        Ok(context)
    }

    /// Find code that implements a spec requirement
    /// This is a simple implementation that matches by name similarity
    /// In production, this could use embeddings or ML for better matching
    pub async fn find_implementations(
        &mut self,
        capability: &SpecCapability,
        requirements: &[SpecRequirement],
        project_root: &str,
    ) -> HashMap<String, Vec<Symbol>> {
        let mut implementations: HashMap<String, Vec<Symbol>> = HashMap::new();

        // Extract key terms from capability and requirements
        let search_terms = self.extract_search_terms(capability, requirements);

        // Search for matching symbols in the codebase
        // This is a placeholder implementation
        // TODO: Implement actual file traversal and symbol matching
        
        implementations
    }

    /// Extract search terms from capability and requirements
    fn extract_search_terms(
        &self,
        capability: &SpecCapability,
        requirements: &[SpecRequirement],
    ) -> Vec<String> {
        let mut terms = Vec::new();

        // Extract words from capability name
        terms.extend(
            capability
                .name
                .split_whitespace()
                .map(|s| s.to_lowercase()),
        );

        // Extract words from requirement names
        for req in requirements {
            terms.extend(req.name.split_whitespace().map(|s| s.to_lowercase()));
        }

        // Remove common words and deduplicate
        terms.retain(|t| t.len() > 3 && !is_common_word(t));
        terms.sort();
        terms.dedup();

        terms
    }

    /// Validate that code matches spec scenarios
    pub async fn validate_spec_scenarios(
        &mut self,
        requirement: &SpecRequirement,
        scenarios: &[SpecScenario],
        code_files: &[String],
    ) -> Vec<ValidationResult> {
        let mut results = Vec::new();

        // For each scenario, check if there's implementing code
        for scenario in scenarios {
            let result = ValidationResult {
                requirement_id: requirement.id.clone(),
                requirement_name: requirement.name.clone(),
                status: ValidationStatus::Unknown,
                details: vec![format!(
                    "Scenario: {} | When: {} | Then: {}",
                    scenario.name, scenario.when_clause, scenario.then_clause
                )],
                code_references: Vec::new(),
            };

            results.push(result);
        }

        results
    }

    /// Analyze a file and extract symbols
    pub fn analyze_file(&mut self, file_path: &str, content: &str) -> Result<Vec<Symbol>, String> {
        let ext = Path::new(file_path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("ts");

        self.ensure_analyzer(ext)?;

        if let Some(analyzer) = &mut self.analyzer {
            analyzer.extract_symbols(content)
        } else {
            Err("Failed to initialize analyzer".to_string())
        }
    }

    /// Create AST to spec mapping
    pub fn create_mapping(
        &self,
        project_id: &str,
        file_path: &str,
        symbol: &Symbol,
        requirement_id: Option<String>,
        confidence: f64,
    ) -> AstSpecMapping {
        AstSpecMapping {
            id: None,
            project_id: project_id.to_string(),
            file_path: file_path.to_string(),
            symbol_name: symbol.name.clone(),
            symbol_type: format!("{:?}", symbol.kind).to_lowercase(),
            line_number: Some(symbol.line_start as i32),
            requirement_id,
            confidence,
            verified: false,
        }
    }
}

impl Default for SpecContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to check if a word is too common to be useful for search
fn is_common_word(word: &str) -> bool {
    matches!(
        word,
        "the" | "and" | "for" | "with" | "from" | "this" | "that" | "will" | "should" | "must"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_search_terms() {
        let builder = SpecContextBuilder::new();
        let capability = SpecCapability {
            id: "test".to_string(),
            project_id: "proj".to_string(),
            prd_id: None,
            name: "User Authentication System".to_string(),
            purpose_markdown: None,
            spec_markdown: "".to_string(),
            design_markdown: None,
            requirement_count: 0,
            version: 1,
            status: crate::openspec::types::CapabilityStatus::Active,
            deleted_at: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let requirements = vec![SpecRequirement {
            id: "req1".to_string(),
            capability_id: "test".to_string(),
            name: "Login Validation".to_string(),
            content_markdown: "".to_string(),
            position: 1,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        }];

        let terms = builder.extract_search_terms(&capability, &requirements);
        
        // Should extract meaningful terms
        assert!(terms.contains(&"user".to_string()));
        assert!(terms.contains(&"authentication".to_string()));
        assert!(terms.contains(&"login".to_string()));
        assert!(terms.contains(&"validation".to_string()));
    }
}
