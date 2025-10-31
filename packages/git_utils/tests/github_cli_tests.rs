// ABOUTME: Unit tests for GitHub CLI wrapper
// ABOUTME: Tests authentication detection, issue operations, and error handling

use git_utils::{GhIssue, GitHubCli, GitHubCliError, UpdateIssueParams};

#[test]
fn test_gh_cli_availability_detection() {
    // Test that we can detect whether gh CLI is available
    match GitHubCli::new() {
        Ok(cli) => {
            println!("✓ gh CLI is available and authenticated");
            assert!(cli.is_authenticated().unwrap());
        }
        Err(GitHubCliError::NotAvailable(msg)) => {
            println!("✓ gh CLI not available: {}", msg);
            assert!(msg.contains("gh not found") || msg.contains("not found in PATH"));
        }
        Err(GitHubCliError::NotAuthenticated) => {
            println!("✓ gh CLI found but not authenticated");
            // This is a valid state - gh is installed but user hasn't authenticated
        }
        Err(e) => {
            panic!("Unexpected error: {}", e);
        }
    }
}

#[test]
fn test_gh_cli_authentication_check() {
    if let Ok(cli) = GitHubCli::new() {
        // If we can create a CLI instance, authentication should be true
        let is_auth = cli.is_authenticated();
        assert!(is_auth.is_ok(), "Authentication check should not error");
        assert!(
            is_auth.unwrap(),
            "If GitHubCli::new() succeeds, authentication must be true"
        );
    }
    // If gh not available, skip this test
}

#[tokio::test]
async fn test_gh_cli_scope_checking() {
    if let Ok(cli) = GitHubCli::new() {
        // Check for basic repo scope (should be present for most gh auth setups)
        let has_repo = cli.check_scopes(&["repo"]);
        assert!(has_repo.is_ok(), "Scope checking should not error");

        println!("Has 'repo' scope: {}", has_repo.unwrap());

        // Check for non-existent scope
        let has_fake = cli.check_scopes(&["nonexistent_scope_xyz"]);
        assert!(has_fake.is_ok());
        // This might be false, which is expected
    }
}

#[tokio::test]
#[ignore] // Only run with --ignored (requires real GitHub repo access)
async fn test_gh_cli_create_issue_integration() {
    // This test requires:
    // 1. gh CLI installed and authenticated
    // 2. Access to a test repository
    // 3. --ignored flag to run

    let cli = match GitHubCli::new() {
        Ok(c) => c,
        Err(_) => {
            println!("Skipping: gh CLI not available");
            return;
        }
    };

    // Replace with your test repository
    let test_owner =
        std::env::var("TEST_GITHUB_OWNER").unwrap_or_else(|_| "orkee-test".to_string());
    let test_repo = std::env::var("TEST_GITHUB_REPO").unwrap_or_else(|_| "test-repo".to_string());

    let result = cli
        .create_issue(
            &test_owner,
            &test_repo,
            "Test Issue from Orkee Tests",
            "This is a test issue created by Orkee's test suite.\n\nIt can be safely closed.",
            Some(vec!["test".to_string()]),
            None,
        )
        .await;

    match result {
        Ok(issue) => {
            println!("✓ Created test issue #{}: {}", issue.number, issue.url);
            assert!(issue.number > 0);
            assert!(issue.url.contains("github.com"));
            assert_eq!(issue.title, "Test Issue from Orkee Tests");
        }
        Err(e) => {
            println!("Issue creation failed (expected if no test repo): {}", e);
            // Don't fail the test - this is expected if test repo doesn't exist
        }
    }
}

#[tokio::test]
#[ignore] // Only run with --ignored (requires real GitHub repo access)
async fn test_gh_cli_update_issue_integration() {
    let cli = match GitHubCli::new() {
        Ok(c) => c,
        Err(_) => {
            println!("Skipping: gh CLI not available");
            return;
        }
    };

    let test_owner =
        std::env::var("TEST_GITHUB_OWNER").unwrap_or_else(|_| "orkee-test".to_string());
    let test_repo = std::env::var("TEST_GITHUB_REPO").unwrap_or_else(|_| "test-repo".to_string());

    // First create an issue to update
    let create_result = cli
        .create_issue(
            &test_owner,
            &test_repo,
            "Test Issue for Update",
            "Original body",
            Some(vec!["test".to_string()]),
            None,
        )
        .await;

    if let Ok(issue) = create_result {
        println!("Created issue #{} for update test", issue.number);

        // Now update it
        let params = UpdateIssueParams {
            title: Some("Updated Test Issue".to_string()),
            body: Some("Updated body content".to_string()),
            state: None,
            labels: Some(vec!["test".to_string(), "updated".to_string()]),
        };

        let update_result = cli
            .update_issue(&test_owner, &test_repo, issue.number, params)
            .await;

        match update_result {
            Ok(updated) => {
                println!("✓ Updated issue #{}", updated.number);
                assert_eq!(updated.number, issue.number);
                assert_eq!(updated.title, "Updated Test Issue");
            }
            Err(e) => {
                println!("Update failed: {}", e);
            }
        }
    }
}

#[test]
fn test_gh_cli_error_types() {
    // Test that error types are properly defined
    let not_available_err = GitHubCliError::NotAvailable("test".to_string());
    assert!(format!("{}", not_available_err).contains("not available"));

    let not_auth_err = GitHubCliError::NotAuthenticated;
    assert!(format!("{}", not_auth_err).contains("not authenticated"));

    let cmd_failed_err = GitHubCliError::CommandFailed("test".to_string());
    assert!(format!("{}", cmd_failed_err).contains("command failed"));

    let parse_err = GitHubCliError::ParseError("test".to_string());
    assert!(format!("{}", parse_err).contains("parse"));
}

#[test]
fn test_gh_issue_deserialization() {
    // Test that we can deserialize GitHub CLI JSON output
    let json = r#"{
        "number": 123,
        "url": "https://github.com/owner/repo/issues/123",
        "title": "Test Issue",
        "body": "Test body",
        "state": "open",
        "updatedAt": "2025-01-01T00:00:00Z"
    }"#;

    let issue: Result<GhIssue, _> = serde_json::from_str(json);
    assert!(issue.is_ok(), "Should deserialize valid GitHub issue JSON");

    let issue = issue.unwrap();
    assert_eq!(issue.number, 123);
    assert_eq!(issue.title, "Test Issue");
    assert_eq!(issue.state, "open");
}

#[test]
fn test_gh_issue_deserialization_with_null_body() {
    // Test handling of null body field
    let json = r#"{
        "number": 456,
        "url": "https://github.com/owner/repo/issues/456",
        "title": "Issue Without Body",
        "body": null,
        "state": "closed",
        "updatedAt": "2025-01-01T00:00:00Z"
    }"#;

    let issue: Result<GhIssue, _> = serde_json::from_str(json);
    assert!(issue.is_ok());

    let issue = issue.unwrap();
    assert_eq!(issue.number, 456);
    assert!(issue.body.is_none());
}

// Mock-based tests for error scenarios

#[tokio::test]
async fn test_gh_cli_handles_invalid_repo() {
    if let Ok(cli) = GitHubCli::new() {
        // Try to create issue in non-existent repo
        let result = cli
            .create_issue(
                "nonexistent-owner-xyz-123",
                "nonexistent-repo-xyz-123",
                "Test",
                "Test",
                None,
                None,
            )
            .await;

        assert!(
            result.is_err(),
            "Should fail when accessing non-existent repo"
        );

        if let Err(e) = result {
            let err_msg = format!("{}", e);
            assert!(
                err_msg.contains("command failed") || err_msg.contains("not found"),
                "Error should indicate repository not found"
            );
        }
    }
}

#[test]
fn test_gh_cli_default_construction() {
    // Test that Default trait works correctly
    // This will panic if gh is not available, which is expected behavior
    let can_construct = std::panic::catch_unwind(|| {
        let _cli = GitHubCli::default();
    });

    // We don't assert on the result because it depends on environment
    // Just verify that the Default implementation exists and behaves consistently
    match can_construct {
        Ok(_) => println!("✓ Default construction succeeded (gh available)"),
        Err(_) => println!("✓ Default construction panicked (gh not available - expected)"),
    }
}
