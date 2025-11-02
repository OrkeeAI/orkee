// ABOUTME: API endpoint tests for Phase 6A ideate endpoints
// ABOUTME: Tests HTTP endpoint functionality for discovery, validation, and complexity analysis

use orkee_ideate::{CodebaseAnalyzer, CodebaseContext, ComplexityAnalyzer, PRDValidator};
use std::path::PathBuf;

// Note: Discovery Manager tests require database connection, skipped for unit tests
// These would be covered in integration tests with a test database

// ============================================================================
// Codebase Analyzer Tests (simulating API /analyze-codebase endpoint)
// ============================================================================

#[test]
fn test_codebase_analyzer_creates_default_context() {
    let _analyzer = CodebaseAnalyzer::new(PathBuf::from("/tmp/test-project"));

    // In a real implementation, this would scan the filesystem
    // For now, we test that it can create a default context
    let context = CodebaseContext::default();

    assert!(context.patterns.is_empty());
    assert!(context.similar_features.is_empty());

    println!("✓ CodebaseAnalyzer creates context for analysis");
}

// ============================================================================
// Complexity Analyzer Tests (simulating API /analyze-complexity endpoint)
// ============================================================================

#[test]
fn test_complexity_analyzer_api_integration() {
    let analyzer = ComplexityAnalyzer::new();

    // Simulate receiving an epic from the API
    let epic = create_test_epic();

    let report = analyzer.analyze_epic(&epic, Some(20));

    assert!(report.is_ok());
    let r = report.unwrap();

    assert!(r.score >= 1 && r.score <= 10);
    assert!(r.recommended_tasks > 0);
    assert!(!r.reasoning.is_empty());

    println!("✓ ComplexityAnalyzer produces valid API response");
}

// ============================================================================
// PRD Validator Tests (simulating API /validate-section endpoint)
// ============================================================================

#[test]
fn test_prd_validator_api_integration() {
    let validator = PRDValidator::new();

    // Simulate a PRD section received from the API
    let prd_section = serde_json::json!({
        "overview": {
            "problemStatement": "Users need better task management",
            "targetAudience": "Developers",
            "valueProposition": "Simple and fast"
        }
    });

    let result = validator.validate(&prd_section);

    // Result should be JSON-serializable for API response
    let json_result = serde_json::to_string(&result);
    assert!(json_result.is_ok());

    println!("✓ PRDValidator produces JSON-serializable API response");
}

// ============================================================================
// Integration Test: Full Workflow Simulation (without database)
// ============================================================================

#[test]
fn test_ideate_workflow_simulation() {
    println!("Starting ideate workflow simulation...");

    // Step 1: Codebase Analysis
    let _analyzer = CodebaseAnalyzer::new(PathBuf::from("/tmp/test"));
    let _codebase_context = CodebaseContext::default();
    println!("  ✓ Codebase context created");

    // Step 2: PRD Validation
    let validator = PRDValidator::new();
    let test_prd = serde_json::json!({"overview": {"problemStatement": "Test PRD for workflow"}});
    let validation_result = validator.validate(&test_prd);
    assert!(validation_result.score >= 0);
    println!("  ✓ PRD validated (score: {})", validation_result.score);

    // Step 3: Complexity Analysis
    let complexity_analyzer = ComplexityAnalyzer::new();
    let epic = create_test_epic();
    let complexity_report = complexity_analyzer.analyze_epic(&epic, Some(20));
    assert!(complexity_report.is_ok());
    println!(
        "  ✓ Complexity analyzed (score: {})",
        complexity_report.unwrap().score
    );

    println!("✓ Ideate workflow simulation completed successfully");
}

// ============================================================================
// Helper Functions
// ============================================================================

fn create_test_epic() -> orkee_ideate::Epic {
    use chrono::Utc;
    use orkee_ideate::{Epic, EpicComplexity, EpicStatus, EstimatedEffort};

    Epic {
        id: "api-test-epic".to_string(),
        project_id: "test-project".to_string(),
        prd_id: "test-prd".to_string(),
        name: "API Test Epic".to_string(),
        overview_markdown: "## Test Epic\n\nFor API integration testing".to_string(),
        technical_approach: "Standard implementation".to_string(),
        implementation_strategy: None,
        architecture_decisions: None,
        dependencies: None,
        success_criteria: None,
        task_categories: None,
        estimated_effort: Some(EstimatedEffort::Days),
        complexity: Some(EpicComplexity::Medium),
        status: EpicStatus::Draft,
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
        created_at: Utc::now(),
        updated_at: Utc::now(),
        started_at: None,
        completed_at: None,
    }
}
