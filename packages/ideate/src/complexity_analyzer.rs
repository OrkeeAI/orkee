// ABOUTME: Complexity analysis for Epic task decomposition
// ABOUTME: Calculates complexity scores and recommends task counts based on Epic characteristics

use crate::epic::{Epic, EpicComplexity};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::cmp::min;

/// Complexity analysis report for an Epic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityReport {
    pub epic_id: String,
    pub score: u8, // 1-10
    pub reasoning: String,
    pub recommended_tasks: usize,
    pub expansion_strategy: String,
    pub factors: ComplexityFactors,
}

/// Factors contributing to complexity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityFactors {
    pub distributed_systems: bool,
    pub migration_work: bool,
    pub dependency_count: usize,
    pub success_criteria_count: usize,
    pub has_similar_features: bool,
    pub uses_existing_patterns: bool,
}

/// Complexity analyzer service
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    pub fn new() -> Self {
        Self
    }

    /// Analyze an Epic and generate complexity report
    pub fn analyze_epic(&self, epic: &Epic, user_limit: Option<i32>) -> Result<ComplexityReport> {
        let mut score: u8 = 5; // Start at medium complexity

        // Extract factors
        let factors = self.extract_factors(epic);

        // Increase complexity for challenging characteristics
        if factors.distributed_systems {
            score += 2;
        }
        if factors.migration_work {
            score += 2;
        }
        if factors.dependency_count > 5 {
            score += 1;
        }
        if factors.success_criteria_count > 10 {
            score += 1;
        }

        // Decrease complexity for leverage opportunities
        if factors.has_similar_features {
            score = score.saturating_sub(1);
        }
        if factors.uses_existing_patterns {
            score = score.saturating_sub(1);
        }

        // Clamp to 1-10 range
        score = score.clamp(1, 10);

        // Calculate recommended task count
        let base_tasks = self.score_to_task_count(score);
        let limit = user_limit.unwrap_or(20).max(5) as usize;
        let recommended_tasks = min(base_tasks, limit);

        // Generate reasoning
        let reasoning = self.explain_score(score, &factors);

        // Generate expansion strategy
        let expansion_strategy = self.suggest_strategy(score);

        Ok(ComplexityReport {
            epic_id: epic.id.clone(),
            score,
            reasoning,
            recommended_tasks,
            expansion_strategy,
            factors,
        })
    }

    /// Extract complexity factors from Epic
    fn extract_factors(&self, epic: &Epic) -> ComplexityFactors {
        let technical_approach_lower = epic.technical_approach.to_lowercase();

        let distributed_systems = technical_approach_lower.contains("distributed")
            || technical_approach_lower.contains("microservice")
            || technical_approach_lower.contains("event-driven");

        let migration_work = technical_approach_lower.contains("migration")
            || technical_approach_lower.contains("migrate")
            || technical_approach_lower.contains("refactor");

        let dependency_count = epic
            .dependencies
            .as_ref()
            .map(|d| d.len())
            .unwrap_or(0);

        let success_criteria_count = epic
            .success_criteria
            .as_ref()
            .map(|s| s.len())
            .unwrap_or(0);

        let has_similar_features = epic
            .codebase_context
            .as_ref()
            .and_then(|ctx| ctx.get("similar_features"))
            .and_then(|sf| sf.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        let uses_existing_patterns = epic
            .codebase_context
            .as_ref()
            .and_then(|ctx| ctx.get("patterns"))
            .and_then(|p| p.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);

        ComplexityFactors {
            distributed_systems,
            migration_work,
            dependency_count,
            success_criteria_count,
            has_similar_features,
            uses_existing_patterns,
        }
    }

    /// Map complexity score to task count
    fn score_to_task_count(&self, score: u8) -> usize {
        match score {
            1..=3 => 5,   // Simple
            4..=6 => 10,  // Medium
            7..=8 => 15,  // Complex
            9..=10 => 20, // Very Complex
            _ => 10,      // Default fallback
        }
    }

    /// Generate human-readable explanation of score
    fn explain_score(&self, score: u8, factors: &ComplexityFactors) -> String {
        let mut reasons = Vec::new();

        if factors.distributed_systems {
            reasons.push("Distributed systems architecture increases complexity".to_string());
        }
        if factors.migration_work {
            reasons.push("Migration or refactoring work adds risk".to_string());
        }
        if factors.dependency_count > 5 {
            reasons.push(format!("{} external dependencies to manage", factors.dependency_count));
        }
        if factors.success_criteria_count > 10 {
            reasons.push("High number of success criteria to satisfy".to_string());
        }
        if factors.has_similar_features {
            reasons.push("Can leverage similar existing features".to_string());
        }
        if factors.uses_existing_patterns {
            reasons.push("Existing patterns available to follow".to_string());
        }

        let level = match score {
            1..=3 => "Low",
            4..=6 => "Medium",
            7..=8 => "High",
            9..=10 => "Very High",
            _ => "Unknown",
        };

        format!(
            "Complexity Level: {} (score: {}/10). {}",
            level,
            score,
            if reasons.is_empty() {
                "Standard implementation".to_string()
            } else {
                reasons.join(". ")
            }
        )
    }

    /// Suggest decomposition strategy based on complexity
    fn suggest_strategy(&self, score: u8) -> String {
        match score {
            1..=3 => {
                "Simple implementation - Start with 3-5 parent tasks, expand to detailed subtasks as needed. Focus on clear acceptance criteria."
            }
            4..=6 => {
                "Medium complexity - Create 5-8 parent tasks representing major work streams. Use two-phase decomposition to avoid over-planning."
            }
            7..=8 => {
                "High complexity - Break into 8-12 parent tasks with clear dependencies. Consider creating checkpoints after foundational work. Look for parallelization opportunities."
            }
            9..=10 => {
                "Very high complexity - Create 10-15 parent tasks, organized by subsystem or layer. Strong focus on simplification analysis. Consider splitting into multiple Epics if task count exceeds limit."
            }
            _ => {
                "Unknown complexity - Use medium approach with 5-8 parent tasks."
            }
        }.to_string()
    }

    /// Map Epic's complexity enum to numeric score (if available)
    pub fn complexity_to_score(complexity: &EpicComplexity) -> u8 {
        match complexity {
            EpicComplexity::Low => 3,
            EpicComplexity::Medium => 6,
            EpicComplexity::High => 8,
            EpicComplexity::VeryHigh => 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;

    fn create_test_epic(technical_approach: &str) -> Epic {
        Epic {
            id: "test-epic".to_string(),
            project_id: "test-project".to_string(),
            prd_id: "test-prd".to_string(),
            name: "Test Epic".to_string(),
            overview_markdown: "Test overview".to_string(),
            architecture_decisions: None,
            technical_approach: technical_approach.to_string(),
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
            task_count_limit: None,
            decomposition_phase: None,
            parent_tasks: None,
            quality_validation: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            started_at: None,
            completed_at: None,
        }
    }

    #[test]
    fn test_simple_epic() {
        let analyzer = ComplexityAnalyzer::new();
        let epic = create_test_epic("Simple CRUD API using existing patterns");

        let report = analyzer.analyze_epic(&epic, None).unwrap();

        assert!(report.score <= 6, "Simple epic should have low-medium complexity");
        assert!(report.recommended_tasks <= 10, "Simple epic should have fewer tasks");
    }

    #[test]
    fn test_distributed_system() {
        let analyzer = ComplexityAnalyzer::new();
        let epic = create_test_epic("Distributed event-driven microservices architecture");

        let report = analyzer.analyze_epic(&epic, None).unwrap();

        assert!(report.score >= 7, "Distributed system should have high complexity");
        assert!(report.factors.distributed_systems, "Should detect distributed architecture");
    }

    #[test]
    fn test_migration_work() {
        let analyzer = ComplexityAnalyzer::new();
        let epic = create_test_epic("Migrate legacy database to new schema with refactoring");

        let report = analyzer.analyze_epic(&epic, None).unwrap();

        assert!(report.factors.migration_work, "Should detect migration work");
        assert!(report.score >= 6, "Migration should increase complexity");
    }

    #[test]
    fn test_user_limit() {
        let analyzer = ComplexityAnalyzer::new();
        let epic = create_test_epic("Very complex distributed system");

        let report_unlimited = analyzer.analyze_epic(&epic, None).unwrap();
        let report_limited = analyzer.analyze_epic(&epic, Some(8)).unwrap();

        assert!(report_limited.recommended_tasks <= 8, "Should respect user limit");
        assert!(report_limited.recommended_tasks <= report_unlimited.recommended_tasks);
    }

    #[test]
    fn test_complexity_enum_mapping() {
        assert_eq!(ComplexityAnalyzer::complexity_to_score(&EpicComplexity::Low), 3);
        assert_eq!(ComplexityAnalyzer::complexity_to_score(&EpicComplexity::Medium), 6);
        assert_eq!(ComplexityAnalyzer::complexity_to_score(&EpicComplexity::High), 8);
        assert_eq!(ComplexityAnalyzer::complexity_to_score(&EpicComplexity::VeryHigh), 10);
    }
}
