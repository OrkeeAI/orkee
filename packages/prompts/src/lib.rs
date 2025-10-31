// ABOUTME: Rust implementation of centralized prompt management
// ABOUTME: Provides type-safe prompt loading and parameter substitution from JSON files

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum PromptError {
    #[error("Prompt not found: {0}")]
    NotFound(String),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Failed to read prompt file: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Failed to parse prompt JSON: {0}")]
    ParseError(#[from] serde_json::Error),

    #[error("Invalid prompt format: {0}")]
    InvalidFormat(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptMetadata {
    pub version: String,
    #[serde(rename = "lastModified")]
    pub last_modified: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    pub id: String,
    pub name: String,
    pub category: String,
    pub template: String,
    pub parameters: Vec<String>,
    #[serde(rename = "outputSchema", skip_serializing_if = "Option::is_none")]
    pub output_schema: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<PromptMetadata>,
}

pub struct PromptManager {
    prompts_dir: PathBuf,
    cache: HashMap<String, Prompt>,
}

impl PromptManager {
    /// Create a new PromptManager
    ///
    /// If prompts_dir is None, it will try to find the prompts directory relative to the binary
    pub fn new(prompts_dir: Option<PathBuf>) -> Result<Self, PromptError> {
        let prompts_dir = match prompts_dir {
            Some(dir) => dir,
            None => {
                // Try to find prompts directory relative to the binary
                let exe_dir = std::env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()));

                if let Some(dir) = exe_dir {
                    // Look for prompts directory in common locations
                    let candidates = vec![
                        dir.join("prompts"),
                        dir.join("../prompts"),
                        dir.join("../../prompts"),
                        dir.join("../../../packages/prompts"),
                    ];

                    candidates.into_iter()
                        .find(|p| p.exists())
                        .unwrap_or_else(|| PathBuf::from("./packages/prompts"))
                } else {
                    PathBuf::from("./packages/prompts")
                }
            }
        };

        Ok(Self {
            prompts_dir,
            cache: HashMap::new(),
        })
    }

    /// Get a prompt by ID with parameter substitution
    pub fn get_prompt(&mut self, prompt_id: &str, parameters: &[(&str, &str)]) -> Result<String, PromptError> {
        let prompt = self.load_prompt(prompt_id)?;

        // Always validate required parameters, even if empty list provided
        self.substitute_parameters(&prompt.template, parameters, &prompt.parameters)
    }

    /// Get a system prompt by category
    pub fn get_system_prompt(&mut self, category: &str) -> Result<String, PromptError> {
        let path = self.prompts_dir.join("system").join(format!("{}.json", category));
        let prompt = self.load_prompt_from_path(&path)?;
        Ok(prompt.template)
    }

    /// Get prompt metadata without substitution
    pub fn get_prompt_metadata(&mut self, prompt_id: &str) -> Result<Prompt, PromptError> {
        self.load_prompt(prompt_id)
    }

    /// Clear the prompt cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// List all prompts in a category
    pub fn list_prompts(&self, category: &str) -> Result<Vec<String>, PromptError> {
        let category_dir = self.prompts_dir.join(category);

        if !category_dir.exists() {
            return Ok(Vec::new());
        }

        let mut prompts = Vec::new();
        for entry in fs::read_dir(category_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    prompts.push(stem.to_string());
                }
            }
        }

        Ok(prompts)
    }

    /// Substitute parameters in a template
    fn substitute_parameters(
        &self,
        template: &str,
        parameters: &[(&str, &str)],
        required_params: &[String],
    ) -> Result<String, PromptError> {
        // Check all required parameters are provided
        let param_map: HashMap<&str, &str> = parameters.iter().copied().collect();

        for required in required_params {
            if !param_map.contains_key(required.as_str()) {
                return Err(PromptError::MissingParameter(required.clone()));
            }
        }

        // Replace {{parameter}} with values
        let mut result = template.to_string();
        for (key, value) in parameters {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        Ok(result)
    }

    /// Load a prompt from disk with caching
    fn load_prompt(&mut self, prompt_id: &str) -> Result<Prompt, PromptError> {
        // Check cache first
        if let Some(prompt) = self.cache.get(prompt_id) {
            return Ok(prompt.clone());
        }

        // Try to find the prompt in standard locations
        let possible_paths = vec![
            self.prompts_dir.join("prd").join(format!("{}.json", prompt_id)),
            self.prompts_dir.join("research").join(format!("{}.json", prompt_id)),
            self.prompts_dir.join("system").join(format!("{}.json", prompt_id)),
        ];

        for path in possible_paths {
            if let Ok(prompt) = self.load_prompt_from_path(&path) {
                self.cache.insert(prompt_id.to_string(), prompt.clone());
                return Ok(prompt);
            }
        }

        Err(PromptError::NotFound(prompt_id.to_string()))
    }

    /// Load a prompt from a specific file path
    fn load_prompt_from_path(&self, path: &Path) -> Result<Prompt, PromptError> {
        let content = fs::read_to_string(path)?;
        let prompt: Prompt = serde_json::from_str(&content)?;

        // Basic validation
        if prompt.id.is_empty() || prompt.template.is_empty() || prompt.category.is_empty() {
            return Err(PromptError::InvalidFormat(
                format!("Invalid prompt format in {}", path.display())
            ));
        }

        Ok(prompt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_prompts_dir() -> PathBuf {
        // This assumes tests are run from the workspace root
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }

    #[test]
    fn test_load_system_prompt() {
        let mut manager = PromptManager::new(Some(get_test_prompts_dir())).unwrap();
        let prompt = manager.get_system_prompt("prd").unwrap();
        assert!(prompt.contains("product manager"));
        assert!(prompt.contains("Product Requirement Documents"));
    }

    #[test]
    fn test_load_prompt_with_parameters() {
        let mut manager = PromptManager::new(Some(get_test_prompts_dir())).unwrap();
        let prompt = manager.get_prompt("overview", &[("description", "A mobile app")]).unwrap();
        assert!(prompt.contains("A mobile app"));
        assert!(prompt.contains("problemStatement"));
    }

    #[test]
    fn test_missing_parameter_error() {
        let mut manager = PromptManager::new(Some(get_test_prompts_dir())).unwrap();
        let result = manager.get_prompt("overview", &[]);
        assert!(matches!(result, Err(PromptError::MissingParameter(_))));
    }

    #[test]
    fn test_prompt_not_found() {
        let mut manager = PromptManager::new(Some(get_test_prompts_dir())).unwrap();
        let result = manager.get_prompt("nonexistent", &[]);
        assert!(matches!(result, Err(PromptError::NotFound(_))));
    }

    #[test]
    fn test_list_prompts() {
        let manager = PromptManager::new(Some(get_test_prompts_dir())).unwrap();
        let prompts = manager.list_prompts("prd").unwrap();
        assert!(prompts.contains(&"overview".to_string()));
        assert!(prompts.contains(&"features".to_string()));
    }

    #[test]
    fn test_cache() {
        let mut manager = PromptManager::new(Some(get_test_prompts_dir())).unwrap();

        // Load once
        manager.get_prompt("overview", &[("description", "Test")]).unwrap();

        // Should be cached now
        let metadata = manager.get_prompt_metadata("overview").unwrap();
        assert_eq!(metadata.id, "overview");

        // Clear cache
        manager.clear_cache();

        // Should still work after cache clear
        let prompt = manager.get_prompt("overview", &[("description", "Test2")]).unwrap();
        assert!(prompt.contains("Test2"));
    }
}
