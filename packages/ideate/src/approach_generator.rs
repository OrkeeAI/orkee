// ABOUTME: Alternative approach generator for technical implementation
// ABOUTME: Generates multiple implementation approaches with trade-off analysis

use crate::codebase_analyzer::CodebaseContext;
use crate::epic::Epic;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use tracing::info;

/// A technical implementation approach
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalApproach {
    pub name: String,
    pub description: String,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_days: i32,
    pub complexity: ComplexityLevel,
    pub recommended: bool,
    pub reasoning: String,
}

/// Complexity level of an approach
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplexityLevel {
    Low,
    Medium,
    High,
}

/// Generates alternative technical approaches
pub struct ApproachGenerator {
    epic: Epic,
    codebase_context: CodebaseContext,
}

impl ApproachGenerator {
    pub fn new(epic: Epic, codebase_context: CodebaseContext) -> Self {
        Self {
            epic,
            codebase_context,
        }
    }

    /// Generate 2-3 alternative approaches with trade-offs
    pub async fn generate_alternatives(&self) -> Result<Vec<TechnicalApproach>> {
        info!(
            "Generating alternative approaches for epic: {}",
            self.epic.id
        );

        let mut approaches = Vec::new();

        // Approach 1: Leverage existing system (recommended by default)
        approaches.push(self.generate_leverage_existing_approach());

        // Approach 2: Clean slate implementation
        approaches.push(self.generate_clean_slate_approach());

        // Approach 3: Hybrid approach (if applicable)
        if self.should_generate_hybrid_approach() {
            approaches.push(self.generate_hybrid_approach());
        }

        Ok(approaches)
    }

    /// Generate approach that leverages existing code
    fn generate_leverage_existing_approach(&self) -> TechnicalApproach {
        let has_similar_features = !self.codebase_context.similar_features.is_empty();
        let has_reusable_components = !self.codebase_context.reusable_components.is_empty();

        let mut pros = vec![
            "Faster implementation time".to_string(),
            "Lower risk - using proven patterns".to_string(),
            "Familiar to the team".to_string(),
        ];

        let cons = vec![
            "May inherit technical debt".to_string(),
            "Limited by existing architecture".to_string(),
        ];

        if has_similar_features {
            pros.push("Can reuse similar feature implementations".to_string());
        }

        if has_reusable_components {
            pros.push(format!(
                "Can leverage {} existing components",
                self.codebase_context.reusable_components.len()
            ));
        }

        let estimated_days = if has_similar_features { 5 } else { 8 };

        TechnicalApproach {
            name: "Extend Existing System".to_string(),
            description: "Build on top of current architecture and patterns".to_string(),
            pros,
            cons,
            estimated_days,
            complexity: ComplexityLevel::Medium,
            recommended: true,
            reasoning: "Best balance of speed and maintainability for most projects".to_string(),
        }
    }

    /// Generate clean slate approach
    fn generate_clean_slate_approach(&self) -> TechnicalApproach {
        TechnicalApproach {
            name: "Clean Slate Implementation".to_string(),
            description: "Build from scratch with modern patterns and best practices".to_string(),
            pros: vec![
                "Latest best practices".to_string(),
                "No legacy constraints".to_string(),
                "Optimized for specific requirements".to_string(),
                "Clean, maintainable codebase".to_string(),
            ],
            cons: vec![
                "Longer implementation timeline".to_string(),
                "Higher risk - unproven in this codebase".to_string(),
                "Team learning curve for new patterns".to_string(),
                "More testing required".to_string(),
            ],
            estimated_days: 15,
            complexity: ComplexityLevel::High,
            recommended: false,
            reasoning: "Only recommended if current system has fundamental limitations".to_string(),
        }
    }

    /// Generate hybrid approach
    fn generate_hybrid_approach(&self) -> TechnicalApproach {
        TechnicalApproach {
            name: "Hybrid Approach".to_string(),
            description:
                "Combine existing components with new, improved implementations where needed"
                    .to_string(),
            pros: vec![
                "Balanced risk and innovation".to_string(),
                "Reuse what works, improve what doesn't".to_string(),
                "Gradual modernization".to_string(),
                "Moderate timeline".to_string(),
            ],
            cons: vec![
                "Requires careful integration planning".to_string(),
                "May have inconsistent patterns initially".to_string(),
                "Need to manage both old and new code".to_string(),
            ],
            estimated_days: 10,
            complexity: ComplexityLevel::Medium,
            recommended: false,
            reasoning: "Good for projects with some good existing code but needing improvements"
                .to_string(),
        }
    }

    /// Determine if a hybrid approach makes sense
    fn should_generate_hybrid_approach(&self) -> bool {
        // Generate hybrid if we have some reusable components but also see room for improvement
        let has_components = !self.codebase_context.reusable_components.is_empty();
        let has_patterns = !self.codebase_context.patterns.is_empty();

        has_components || has_patterns
    }

    /// Get a summary comparison of all approaches
    pub fn compare_approaches(approaches: &[TechnicalApproach]) -> ApproachComparison {
        let fastest = approaches.iter().min_by_key(|a| a.estimated_days).cloned();

        let safest = approaches.iter().find(|a| a.recommended).cloned();

        let most_modern = approaches
            .iter()
            .find(|a| a.name.contains("Clean Slate"))
            .cloned();

        ApproachComparison {
            total_approaches: approaches.len(),
            fastest,
            safest,
            most_modern,
        }
    }
}

/// Comparison summary of approaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApproachComparison {
    pub total_approaches: usize,
    pub fastest: Option<TechnicalApproach>,
    pub safest: Option<TechnicalApproach>,
    pub most_modern: Option<TechnicalApproach>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codebase_analyzer::CodebaseContext;
    use crate::epic::{CreateEpicInput, Epic};

    #[tokio::test]
    async fn test_generate_approaches() {
        let epic = create_test_epic();
        let context = CodebaseContext::default();

        let generator = ApproachGenerator::new(epic, context);
        let approaches = generator.generate_alternatives().await.unwrap();

        // Should generate at least 2 approaches
        assert!(approaches.len() >= 2);

        // Should have exactly one recommended approach
        let recommended_count = approaches.iter().filter(|a| a.recommended).count();
        assert_eq!(recommended_count, 1);
    }

    fn create_test_epic() -> Epic {
        Epic {
            id: "test123".to_string(),
            project_id: "project123".to_string(),
            prd_id: "prd123".to_string(),
            name: "Test Epic".to_string(),
            overview_markdown: "Test description".to_string(),
            architecture_decisions: None,
            technical_approach: "Test approach".to_string(),
            implementation_strategy: None,
            dependencies: None,
            success_criteria: None,
            task_categories: None,
            estimated_effort: None,
            complexity: None,
            status: crate::epic::EpicStatus::Draft,
            progress_percentage: 0,
            github_issue_number: None,
            github_issue_url: None,
            github_synced_at: None,
            codebase_context: None,
            simplification_analysis: None,
            task_count_limit: Some(20),
            decomposition_phase: None,
            parent_tasks: None,
            quality_validation: None,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }
}
