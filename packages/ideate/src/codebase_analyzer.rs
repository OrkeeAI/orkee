// ABOUTME: Codebase context analyzer for PRD generation
// ABOUTME: Scans project files to identify patterns, frameworks, and reusable components

use crate::error::{IdeateError, Result};
use crate::types::IdeateSession;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{error, info};

/// A code pattern detected in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub pattern_type: PatternType,
    pub description: String,
    pub examples: Vec<String>,
}

/// Type of code pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    Database,
    Api,
    Frontend,
    Testing,
    Authentication,
    Architecture,
}

/// Similar feature found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimilarFeature {
    pub name: String,
    pub description: String,
    pub files: Vec<String>,
    pub similarity_score: f32,
}

/// Reusable component found in the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReusableComponent {
    pub name: String,
    pub component_type: String,
    pub file_path: String,
    pub description: String,
}

/// Architecture style detected
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ArchitectureStyle {
    Monolithic,
    Microservices,
    Layered,
    EventDriven,
    Serverless,
    Unknown,
}

/// Context about the codebase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodebaseContext {
    pub patterns: Vec<Pattern>,
    pub similar_features: Vec<SimilarFeature>,
    pub reusable_components: Vec<ReusableComponent>,
    pub architecture_style: ArchitectureStyle,
    pub tech_stack: TechStack,
    pub file_structure: FileStructure,
}

/// Detected technology stack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechStack {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub databases: Vec<String>,
    pub tools: Vec<String>,
}

/// File structure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileStructure {
    pub total_files: usize,
    pub code_files: usize,
    pub test_files: usize,
    pub config_files: usize,
}

impl Default for CodebaseContext {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
            similar_features: Vec::new(),
            reusable_components: Vec::new(),
            architecture_style: ArchitectureStyle::Unknown,
            tech_stack: TechStack {
                languages: Vec::new(),
                frameworks: Vec::new(),
                databases: Vec::new(),
                tools: Vec::new(),
            },
            file_structure: FileStructure {
                total_files: 0,
                code_files: 0,
                test_files: 0,
                config_files: 0,
            },
        }
    }
}

/// Codebase analyzer
pub struct CodebaseAnalyzer {
    project_path: PathBuf,
}

impl CodebaseAnalyzer {
    pub fn new(project_path: PathBuf) -> Self {
        Self { project_path }
    }

    /// Analyze the codebase for a session
    pub async fn analyze_for_session(&self, session: &IdeateSession) -> Result<CodebaseContext> {
        info!(
            "Analyzing codebase at {:?} for session: {}",
            self.project_path, session.id
        );

        let mut context = CodebaseContext::default();

        // 1. Scan for existing patterns
        context.patterns = self.identify_patterns().await?;

        // 2. Find similar features
        context.similar_features = self
            .find_similar_features(&session.initial_description)
            .await?;

        // 3. Identify reusable components
        context.reusable_components = self.find_reusable_components().await?;

        // 4. Detect architecture style
        context.architecture_style = self.detect_architecture().await?;

        // 5. Detect tech stack
        context.tech_stack = self.detect_tech_stack().await?;

        // 6. Analyze file structure
        context.file_structure = self.analyze_file_structure().await?;

        Ok(context)
    }

    /// Identify code patterns in the project
    async fn identify_patterns(&self) -> Result<Vec<Pattern>> {
        info!("Identifying patterns in project");

        let mut patterns = Vec::new();

        // Check for database patterns (SQLx, Diesel, etc.)
        if self.has_file("Cargo.toml").await? {
            let cargo_content = self.read_file("Cargo.toml").await?;
            if cargo_content.contains("sqlx") {
                patterns.push(Pattern {
                    name: "SQLx Database Access".to_string(),
                    pattern_type: PatternType::Database,
                    description: "Runtime SQL queries using SQLx".to_string(),
                    examples: vec!["sqlx::query()".to_string()],
                });
            }
        }

        // Check for API patterns (Axum, Actix, etc.)
        if self.has_file("Cargo.toml").await? {
            let cargo_content = self.read_file("Cargo.toml").await?;
            if cargo_content.contains("axum") {
                patterns.push(Pattern {
                    name: "Axum REST API".to_string(),
                    pattern_type: PatternType::Api,
                    description: "RESTful API using Axum framework".to_string(),
                    examples: vec!["Router::new()".to_string()],
                });
            }
        }

        // Check for frontend patterns (React, Vue, etc.)
        if self.has_file("package.json").await? {
            let package_content = self.read_file("package.json").await?;
            if package_content.contains("react") {
                patterns.push(Pattern {
                    name: "React Frontend".to_string(),
                    pattern_type: PatternType::Frontend,
                    description: "React-based user interface".to_string(),
                    examples: vec!["React components".to_string()],
                });
            }
        }

        Ok(patterns)
    }

    /// Find similar features in the codebase
    async fn find_similar_features(&self, description: &str) -> Result<Vec<SimilarFeature>> {
        info!("Finding similar features for: {}", description);

        // This is a simplified implementation
        // In a real implementation, this would use semantic search or embeddings
        let mut features = Vec::new();

        // For now, just check for common keywords
        let description_lower = description.to_lowercase();

        // Example: if description contains "authentication", find auth-related code
        if description_lower.contains("auth") {
            features.push(SimilarFeature {
                name: "Authentication System".to_string(),
                description: "Existing authentication implementation".to_string(),
                files: vec!["src/auth.rs".to_string()],
                similarity_score: 0.8,
            });
        }

        Ok(features)
    }

    /// Find reusable components
    async fn find_reusable_components(&self) -> Result<Vec<ReusableComponent>> {
        info!("Finding reusable components");

        let mut components = Vec::new();

        // Look for common utility files
        if self.has_file("src/utils/mod.rs").await? {
            components.push(ReusableComponent {
                name: "Utility Functions".to_string(),
                component_type: "Module".to_string(),
                file_path: "src/utils/mod.rs".to_string(),
                description: "Shared utility functions".to_string(),
            });
        }

        Ok(components)
    }

    /// Detect architecture style
    async fn detect_architecture(&self) -> Result<ArchitectureStyle> {
        info!("Detecting architecture style");

        // Check for common architecture patterns
        if self.has_file("Cargo.toml").await? {
            let cargo_content = self.read_file("Cargo.toml").await?;

            // Check for microservices indicators
            if cargo_content.contains("workspace") {
                return Ok(ArchitectureStyle::Microservices);
            }

            // Check for layered architecture indicators
            if self.has_file("src/api").await?
                && self.has_file("src/domain").await?
                && self.has_file("src/infrastructure").await?
            {
                return Ok(ArchitectureStyle::Layered);
            }
        }

        Ok(ArchitectureStyle::Monolithic)
    }

    /// Detect technology stack
    async fn detect_tech_stack(&self) -> Result<TechStack> {
        info!("Detecting tech stack");

        let mut tech_stack = TechStack {
            languages: Vec::new(),
            frameworks: Vec::new(),
            databases: Vec::new(),
            tools: Vec::new(),
        };

        // Detect languages
        if self.has_file("Cargo.toml").await? {
            tech_stack.languages.push("Rust".to_string());
        }
        if self.has_file("package.json").await? {
            tech_stack
                .languages
                .push("TypeScript/JavaScript".to_string());
        }

        // Detect frameworks from Cargo.toml
        if self.has_file("Cargo.toml").await? {
            let cargo_content = self.read_file("Cargo.toml").await?;
            if cargo_content.contains("axum") {
                tech_stack.frameworks.push("Axum".to_string());
            }
            if cargo_content.contains("tokio") {
                tech_stack.frameworks.push("Tokio".to_string());
            }
        }

        // Detect frameworks from package.json
        if self.has_file("package.json").await? {
            let package_content = self.read_file("package.json").await?;
            if package_content.contains("react") {
                tech_stack.frameworks.push("React".to_string());
            }
            if package_content.contains("vite") {
                tech_stack.frameworks.push("Vite".to_string());
            }
        }

        // Detect databases
        if self.has_file("Cargo.toml").await? {
            let cargo_content = self.read_file("Cargo.toml").await?;
            if cargo_content.contains("sqlx") && cargo_content.contains("sqlite") {
                tech_stack.databases.push("SQLite".to_string());
            }
        }

        Ok(tech_stack)
    }

    /// Analyze file structure
    async fn analyze_file_structure(&self) -> Result<FileStructure> {
        info!("Analyzing file structure");

        // This is a simplified implementation
        // In a real implementation, this would walk the directory tree
        Ok(FileStructure {
            total_files: 0,
            code_files: 0,
            test_files: 0,
            config_files: 0,
        })
    }

    /// Check if a file exists in the project
    async fn has_file(&self, relative_path: &str) -> Result<bool> {
        let full_path = self.project_path.join(relative_path);
        Ok(full_path.exists())
    }

    /// Read a file from the project
    async fn read_file(&self, relative_path: &str) -> Result<String> {
        let full_path = self.project_path.join(relative_path);
        std::fs::read_to_string(&full_path).map_err(|e| {
            error!("Failed to read file {:?}: {}", full_path, e);
            IdeateError::InvalidInput(format!("Failed to read file: {}", e))
        })
    }
}
