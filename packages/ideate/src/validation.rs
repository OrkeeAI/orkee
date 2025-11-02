// ABOUTME: PRD quality validation with scoring and issue detection
// ABOUTME: Validates PRD completeness, quality, and adherence to best practices

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub passed: bool,
    pub score: i32,
    pub issues: Vec<String>,
    pub suggestions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PRDSection {
    pub name: String,
    pub content: Option<String>,
}

pub struct PRDValidator;

impl PRDValidator {
    pub fn new() -> Self {
        Self
    }

    /// Validate a complete PRD and return quality score
    pub fn validate(&self, prd: &serde_json::Value) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 100;

        // Check overview section
        if let Some(overview) = prd.get("overview") {
            score -= self.validate_overview(overview, &mut issues);
        } else {
            issues.push("Missing overview section".to_string());
            score -= 20;
        }

        // Check for Non-Goals section (Phase 3 enhancement)
        if let Some(non_goals) = prd.get("nonGoals") {
            score -= self.validate_non_goals(non_goals, &mut issues);
        } else {
            issues.push("Missing Non-Goals section (prevents scope creep)".to_string());
            score -= 15;
        }

        // Check for Open Questions section (Phase 3 enhancement)
        if let Some(open_questions) = prd.get("openQuestions") {
            score -= self.validate_open_questions(open_questions, &mut issues);
        } else {
            issues.push("Missing Open Questions section (identifies unknowns)".to_string());
            score -= 10;
        }

        // Check for Success Metrics section (Phase 3 enhancement)
        if let Some(success_metrics) = prd.get("successMetrics") {
            score -= self.validate_success_metrics(success_metrics, &mut issues);
        } else {
            issues.push("Missing Success Metrics section (defines measurable goals)".to_string());
            score -= 15;
        }

        // Check features section
        if let Some(features) = prd.get("features") {
            score -= self.validate_features(features, &mut issues);
        } else {
            issues.push("Missing features section".to_string());
            score -= 20;
        }

        // Check for acceptance criteria in features
        if let Some(features) = prd.get("features").and_then(|f| f.as_array()) {
            for (idx, feature) in features.iter().enumerate() {
                if feature.get("acceptanceCriteria").is_none() {
                    issues.push(format!(
                        "Feature {} missing acceptance criteria",
                        feature
                            .get("name")
                            .and_then(|n| n.as_str())
                            .unwrap_or(&format!("#{}", idx + 1))
                    ));
                    score -= 5;
                }
            }
        }

        // Check technical section
        if let Some(technical) = prd.get("technical") {
            score -= self.validate_technical(technical, &mut issues);
        } else {
            issues.push("Missing technical section".to_string());
            score -= 15;
        }

        // Check roadmap section
        if prd.get("roadmap").is_none() {
            issues.push("Missing roadmap section".to_string());
            score -= 10;
        }

        // Check risks section
        if prd.get("risks").is_none() {
            issues.push("Missing risks section".to_string());
            score -= 10;
        }

        // Generate suggestions based on issues
        let suggestions = self.generate_suggestions(&issues);

        ValidationResult {
            passed: score >= 70,
            score: score.max(0),
            issues,
            suggestions,
        }
    }

    fn validate_overview(&self, overview: &serde_json::Value, issues: &mut Vec<String>) -> i32 {
        let mut deduction = 0;

        // Check for placeholder text
        if let Some(problem_statement) = overview.get("problemStatement").and_then(|p| p.as_str()) {
            if problem_statement.contains("TODO")
                || problem_statement.contains("[")
                || problem_statement.len() < 20
            {
                issues.push("Overview contains placeholder text or is too brief".to_string());
                deduction += 10;
            }
        } else {
            issues.push("Missing problem statement in overview".to_string());
            deduction += 10;
        }

        if overview.get("targetAudience").is_none() {
            issues.push("Missing target audience in overview".to_string());
            deduction += 5;
        }

        if overview.get("valueProposition").is_none() {
            issues.push("Missing value proposition in overview".to_string());
            deduction += 5;
        }

        deduction
    }

    fn validate_non_goals(
        &self,
        non_goals: &serde_json::Value,
        issues: &mut Vec<String>,
    ) -> i32 {
        let mut deduction = 0;

        if let Some(non_goals_array) = non_goals.as_array() {
            if non_goals_array.is_empty() {
                issues.push("Non-Goals section is empty - should list what's out of scope".to_string());
                deduction += 10;
            } else {
                // Check each non-goal has required fields
                for (idx, item) in non_goals_array.iter().enumerate() {
                    if item.get("item").is_none() {
                        issues.push(format!("Non-Goal #{} missing item description", idx + 1));
                        deduction += 2;
                    }
                    if item.get("rationale").is_none() {
                        issues.push(format!("Non-Goal #{} missing rationale", idx + 1));
                        deduction += 2;
                    }
                }
            }
        } else {
            issues.push("Non-Goals should be an array of items".to_string());
            deduction += 10;
        }

        deduction
    }

    fn validate_open_questions(
        &self,
        open_questions: &serde_json::Value,
        issues: &mut Vec<String>,
    ) -> i32 {
        let mut deduction = 0;

        if let Some(questions_array) = open_questions.as_array() {
            if questions_array.len() > 20 {
                issues.push("Too many open questions (>20) - may indicate unclear requirements".to_string());
                deduction += 5;
            }

            // Check for critical questions
            let critical_count = questions_array
                .iter()
                .filter(|q| {
                    q.get("priority")
                        .and_then(|p| p.as_str())
                        .map(|p| p == "critical")
                        .unwrap_or(false)
                })
                .count();

            if critical_count > 5 {
                issues.push(
                    "More than 5 critical open questions - may need more discovery work".to_string(),
                );
                deduction += 5;
            }
        } else {
            issues.push("Open Questions should be an array".to_string());
            deduction += 5;
        }

        deduction
    }

    fn validate_success_metrics(
        &self,
        success_metrics: &serde_json::Value,
        issues: &mut Vec<String>,
    ) -> i32 {
        let mut deduction = 0;

        // Check for primary metrics
        if let Some(primary_metrics) = success_metrics.get("primaryMetrics").and_then(|m| m.as_array()) {
            if primary_metrics.is_empty() {
                issues.push("No primary success metrics defined".to_string());
                deduction += 10;
            } else {
                // Validate each metric has quantifiable targets
                for (idx, metric) in primary_metrics.iter().enumerate() {
                    if let Some(target) = metric.get("target").and_then(|t| t.as_str()) {
                        // Check if target contains numbers
                        if !target.chars().any(|c| c.is_numeric()) {
                            issues.push(format!(
                                "Primary metric #{} lacks quantifiable target (no numbers)",
                                idx + 1
                            ));
                            deduction += 5;
                        }
                    } else {
                        issues.push(format!("Primary metric #{} missing target", idx + 1));
                        deduction += 5;
                    }

                    if metric.get("timeframe").is_none() {
                        issues.push(format!("Primary metric #{} missing timeframe", idx + 1));
                        deduction += 3;
                    }

                    if metric.get("measurementMethod").is_none() {
                        issues.push(format!(
                            "Primary metric #{} missing measurement method",
                            idx + 1
                        ));
                        deduction += 3;
                    }
                }
            }
        } else {
            issues.push("Missing primary metrics array".to_string());
            deduction += 10;
        }

        deduction
    }

    fn validate_features(
        &self,
        features: &serde_json::Value,
        issues: &mut Vec<String>,
    ) -> i32 {
        let mut deduction = 0;

        if let Some(features_array) = features.as_array() {
            if features_array.is_empty() {
                issues.push("No features defined".to_string());
                deduction += 15;
            }

            // Check for overly long feature lists
            if features_array.len() > 30 {
                issues.push(
                    "Very large number of features (>30) - consider breaking into phases".to_string(),
                );
                deduction += 5;
            }
        } else {
            issues.push("Features should be an array".to_string());
            deduction += 15;
        }

        deduction
    }

    fn validate_technical(
        &self,
        technical: &serde_json::Value,
        issues: &mut Vec<String>,
    ) -> i32 {
        let mut deduction = 0;

        if technical.get("components").is_none() {
            issues.push("Technical section missing components list".to_string());
            deduction += 5;
        }

        if technical.get("dataModels").is_none() {
            issues.push("Technical section missing data models".to_string());
            deduction += 5;
        }

        deduction
    }

    fn generate_suggestions(&self, issues: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        if issues.iter().any(|i| i.contains("Non-Goals")) {
            suggestions.push(
                "Add a Non-Goals section listing what's explicitly out of scope to prevent scope creep"
                    .to_string(),
            );
        }

        if issues.iter().any(|i| i.contains("Success Metrics")) {
            suggestions.push(
                "Define quantifiable success metrics with specific numeric targets and timeframes"
                    .to_string(),
            );
        }

        if issues.iter().any(|i| i.contains("Open Questions")) {
            suggestions.push(
                "Document open questions and unknowns that need clarification before or during implementation"
                    .to_string(),
            );
        }

        if issues.iter().any(|i| i.contains("acceptance criteria")) {
            suggestions.push(
                "Add acceptance criteria to all features to define clear completion conditions"
                    .to_string(),
            );
        }

        if issues.iter().any(|i| i.contains("placeholder")) {
            suggestions.push("Replace placeholder text with specific, actionable content".to_string());
        }

        if issues
            .iter()
            .any(|i| i.contains("critical open questions"))
        {
            suggestions.push(
                "Consider additional discovery work to resolve critical questions before implementation"
                    .to_string(),
            );
        }

        suggestions
    }

    /// Validate a single PRD section
    pub fn validate_section(&self, section_name: &str, content: &str) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 100;

        // Check for minimum content length
        if content.len() < 50 {
            issues.push(format!("Section '{}' is very brief (< 50 chars)", section_name));
            score -= 20;
        }

        // Check for placeholder text
        if content.contains("TODO") || content.contains("[TODO]") || content.contains("TBD") {
            issues.push(format!(
                "Section '{}' contains placeholder text (TODO/TBD)",
                section_name
            ));
            score -= 15;
        }

        // Check for empty brackets or parentheses
        if content.contains("[]") || content.contains("()") {
            issues.push(format!(
                "Section '{}' contains empty placeholders",
                section_name
            ));
            score -= 10;
        }

        let suggestions = if !issues.is_empty() {
            vec![format!(
                "Review and complete the '{}' section with specific, actionable content",
                section_name
            )]
        } else {
            vec![]
        };

        ValidationResult {
            passed: score >= 70,
            score: score.max(0),
            issues,
            suggestions,
        }
    }
}

impl Default for PRDValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_validate_complete_prd_with_new_sections() {
        let prd = json!({
            "overview": {
                "problemStatement": "Users need a way to track their daily tasks efficiently",
                "targetAudience": "Busy professionals and students",
                "valueProposition": "Simple, fast task management without complexity"
            },
            "nonGoals": [
                {
                    "item": "Calendar integration",
                    "rationale": "Out of MVP scope",
                    "deferredTo": "Phase 2"
                }
            ],
            "openQuestions": [
                {
                    "question": "Should we support offline mode?",
                    "category": "technical",
                    "priority": "high",
                    "blocking": "none"
                }
            ],
            "successMetrics": {
                "primaryMetrics": [
                    {
                        "metric": "Daily Active Users",
                        "target": "500 DAU within 3 months",
                        "timeframe": "3 months",
                        "measurementMethod": "Analytics tracking",
                        "rationale": "Validates product-market fit"
                    }
                ]
            },
            "features": [
                {
                    "name": "Task creation",
                    "acceptanceCriteria": ["Can add task", "Can edit task"]
                }
            ],
            "technical": {
                "components": ["API", "Database"],
                "dataModels": ["Task", "User"]
            },
            "roadmap": { "mvpScope": [] },
            "risks": { "technicalRisks": [] }
        });

        let validator = PRDValidator::new();
        let result = validator.validate(&prd);

        assert!(result.passed, "PRD should pass validation");
        assert!(result.score >= 70, "Score should be >= 70");
        assert!(
            result.issues.is_empty() || result.score >= 70,
            "Should have no issues or score >= 70"
        );
    }

    #[test]
    fn test_validate_prd_missing_non_goals() {
        let prd = json!({
            "overview": {
                "problemStatement": "Test problem",
                "targetAudience": "Test audience"
            },
            "features": [{"name": "Feature 1"}],
            "technical": {
                "components": ["API"],
                "dataModels": ["Model"]
            }
        });

        let validator = PRDValidator::new();
        let result = validator.validate(&prd);

        assert!(
            result.issues.iter().any(|i| i.contains("Non-Goals")),
            "Should flag missing Non-Goals"
        );
        assert!(result.score < 100, "Score should be reduced");
    }

    #[test]
    fn test_validate_success_metrics_without_numbers() {
        let prd = json!({
            "overview": {
                "problemStatement": "Test problem statement that is long enough to pass",
                "targetAudience": "Test audience"
            },
            "successMetrics": {
                "primaryMetrics": [
                    {
                        "metric": "User satisfaction",
                        "target": "High satisfaction",
                        "timeframe": "Soon",
                        "measurementMethod": "Survey"
                    }
                ]
            },
            "features": [{"name": "Feature"}],
            "technical": { "components": ["API"], "dataModels": ["Model"] }
        });

        let validator = PRDValidator::new();
        let result = validator.validate(&prd);

        assert!(
            result
                .issues
                .iter()
                .any(|i| i.contains("quantifiable target")),
            "Should flag non-quantifiable metrics"
        );
    }

    #[test]
    fn test_validate_section_with_placeholder() {
        let validator = PRDValidator::new();
        let result = validator.validate_section("overview", "TODO: Write overview");

        assert!(!result.passed, "Should fail validation");
        assert!(
            result.issues.iter().any(|i| i.contains("placeholder")),
            "Should detect placeholder text"
        );
    }

    #[test]
    fn test_validate_section_too_brief() {
        let validator = PRDValidator::new();
        let result = validator.validate_section("overview", "Short");

        assert!(!result.passed, "Should fail validation");
        assert!(
            result.issues.iter().any(|i| i.contains("brief")),
            "Should detect brief content"
        );
    }
}
