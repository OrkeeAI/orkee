// ABOUTME: Tests for JSON serialization/deserialization of complex types
// ABOUTME: Verifies round-trip consistency, field name mapping, and edge cases

use chrono::Utc;
use orkee_core::types::{
    GitRepositoryInfo, ManualSubtask, ManualTask, Priority, ProjectStatus, TaskSource, TaskStatus,
};
use serde_json;
use std::collections::HashMap;

// ==============================================================================
// GIT REPOSITORY INFO SERIALIZATION TESTS
// ==============================================================================

#[test]
fn test_git_repository_info_round_trip() {
    let original = GitRepositoryInfo {
        owner: "OrkeeAI".to_string(),
        repo: "orkee".to_string(),
        url: "https://github.com/OrkeeAI/orkee".to_string(),
        branch: Some("main".to_string()),
    };

    // Serialize to JSON
    let json = serde_json::to_string(&original).expect("Should serialize to JSON");

    // Deserialize back to Rust
    let deserialized: GitRepositoryInfo =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    // Verify round-trip identity
    assert_eq!(original.owner, deserialized.owner);
    assert_eq!(original.repo, deserialized.repo);
    assert_eq!(original.url, deserialized.url);
    assert_eq!(original.branch, deserialized.branch);
}

#[test]
fn test_git_repository_info_with_null_branch() {
    let git_info = GitRepositoryInfo {
        owner: "test".to_string(),
        repo: "repo".to_string(),
        url: "https://github.com/test/repo".to_string(),
        branch: None,
    };

    let json = serde_json::to_string(&git_info).expect("Should serialize");

    // Verify null branch serializes as JSON null
    assert!(
        json.contains("\"branch\":null"),
        "branch should be JSON null"
    );

    // Verify deserialization handles null
    let deserialized: GitRepositoryInfo =
        serde_json::from_str(&json).expect("Should deserialize with null branch");
    assert_eq!(deserialized.branch, None);
}

#[test]
fn test_git_repository_info_field_names_are_snake_case() {
    // GitRepositoryInfo doesn't use serde(rename), so fields should be snake_case in JSON
    let git_info = GitRepositoryInfo {
        owner: "test".to_string(),
        repo: "repo".to_string(),
        url: "https://github.com/test/repo".to_string(),
        branch: Some("main".to_string()),
    };

    let json = serde_json::to_string(&git_info).expect("Should serialize");

    // Verify field names are snake_case (not camelCase)
    assert!(json.contains("\"owner\""), "Should have 'owner' field");
    assert!(json.contains("\"repo\""), "Should have 'repo' field");
    assert!(json.contains("\"url\""), "Should have 'url' field");
    assert!(json.contains("\"branch\""), "Should have 'branch' field");
}

// ==============================================================================
// MANUAL TASK SERIALIZATION TESTS
// ==============================================================================

#[test]
fn test_manual_task_round_trip_with_all_fields() {
    let now = Utc::now();
    let subtask = ManualSubtask {
        id: 1,
        title: "Subtask 1".to_string(),
        description: "First subtask".to_string(),
        dependencies: vec![],
        details: Some("Additional details".to_string()),
        status: TaskStatus::Pending,
        test_strategy: Some("Unit tests".to_string()),
    };

    let task = ManualTask {
        id: 100,
        title: "Main Task".to_string(),
        description: "Task description".to_string(),
        details: Some("Task details".to_string()),
        test_strategy: Some("Integration tests".to_string()),
        priority: Priority::High,
        dependencies: vec![99],
        status: TaskStatus::InProgress,
        subtasks: vec![subtask],
        created_at: now,
        updated_at: now,
    };

    // Serialize to JSON
    let json = serde_json::to_string(&task).expect("Should serialize to JSON");

    // Deserialize back
    let deserialized: ManualTask =
        serde_json::from_str(&json).expect("Should deserialize from JSON");

    // Verify all fields
    assert_eq!(task.id, deserialized.id);
    assert_eq!(task.title, deserialized.title);
    assert_eq!(task.description, deserialized.description);
    assert_eq!(task.details, deserialized.details);
    assert_eq!(task.test_strategy, deserialized.test_strategy);
    assert_eq!(task.dependencies, deserialized.dependencies);
    assert_eq!(task.subtasks.len(), deserialized.subtasks.len());
    assert_eq!(task.created_at, deserialized.created_at);
    assert_eq!(task.updated_at, deserialized.updated_at);
}

#[test]
fn test_manual_task_camel_case_field_names() {
    let now = Utc::now();
    let task = ManualTask {
        id: 1,
        title: "Test".to_string(),
        description: "Description".to_string(),
        details: None,
        test_strategy: Some("Strategy".to_string()),
        priority: Priority::Medium,
        dependencies: vec![],
        status: TaskStatus::Pending,
        subtasks: vec![],
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_string(&task).expect("Should serialize");

    // Verify #[serde(rename)] attributes work correctly
    assert!(
        json.contains("\"testStrategy\""),
        "test_strategy should serialize as testStrategy"
    );
    assert!(
        json.contains("\"createdAt\""),
        "created_at should serialize as createdAt"
    );
    assert!(
        json.contains("\"updatedAt\""),
        "updated_at should serialize as updatedAt"
    );

    // Verify no snake_case versions exist
    assert!(
        !json.contains("\"test_strategy\""),
        "Should not have snake_case test_strategy"
    );
    assert!(
        !json.contains("\"created_at\""),
        "Should not have snake_case created_at"
    );
    assert!(
        !json.contains("\"updated_at\""),
        "Should not have snake_case updated_at"
    );
}

#[test]
fn test_manual_subtask_camel_case_field_names() {
    let subtask = ManualSubtask {
        id: 1,
        title: "Subtask".to_string(),
        description: "Description".to_string(),
        dependencies: vec![2, 3],
        details: Some("Details".to_string()),
        status: TaskStatus::Done,
        test_strategy: Some("Test plan".to_string()),
    };

    let json = serde_json::to_string(&subtask).expect("Should serialize");

    // Verify testStrategy uses camelCase
    assert!(
        json.contains("\"testStrategy\""),
        "test_strategy should serialize as testStrategy"
    );
    assert!(
        !json.contains("\"test_strategy\""),
        "Should not have snake_case"
    );
}

#[test]
fn test_manual_task_with_empty_arrays() {
    let now = Utc::now();
    let task = ManualTask {
        id: 1,
        title: "Empty Task".to_string(),
        description: "Task with empty arrays".to_string(),
        details: None,
        test_strategy: None,
        priority: Priority::Low,
        dependencies: vec![], // Empty dependencies
        status: TaskStatus::Pending,
        subtasks: vec![], // Empty subtasks
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_string(&task).expect("Should serialize");

    // Verify empty arrays serialize as JSON arrays (not null)
    assert!(
        json.contains("\"dependencies\":[]"),
        "Empty dependencies should be []"
    );
    assert!(
        json.contains("\"subtasks\":[]"),
        "Empty subtasks should be []"
    );

    // Verify deserialization handles empty arrays
    let deserialized: ManualTask =
        serde_json::from_str(&json).expect("Should deserialize with empty arrays");
    assert_eq!(deserialized.dependencies.len(), 0);
    assert_eq!(deserialized.subtasks.len(), 0);
}

#[test]
fn test_manual_task_with_all_null_optionals() {
    let now = Utc::now();
    let task = ManualTask {
        id: 1,
        title: "Minimal Task".to_string(),
        description: "Task with all optional fields null".to_string(),
        details: None,
        test_strategy: None,
        priority: Priority::Medium,
        dependencies: vec![],
        status: TaskStatus::Pending,
        subtasks: vec![],
        created_at: now,
        updated_at: now,
    };

    let json = serde_json::to_string(&task).expect("Should serialize");

    // Verify optional fields serialize as JSON null
    assert!(
        json.contains("\"details\":null"),
        "details should be JSON null"
    );
    assert!(
        json.contains("\"testStrategy\":null"),
        "testStrategy should be JSON null"
    );

    // Verify deserialization handles null optionals
    let deserialized: ManualTask =
        serde_json::from_str(&json).expect("Should deserialize with null optionals");
    assert_eq!(deserialized.details, None);
    assert_eq!(deserialized.test_strategy, None);
}

#[test]
fn test_manual_subtask_with_nested_dependencies() {
    let subtask = ManualSubtask {
        id: 5,
        title: "Dependent Subtask".to_string(),
        description: "Has multiple dependencies".to_string(),
        dependencies: vec![1, 2, 3, 4], // Multiple dependencies
        details: None,
        status: TaskStatus::Pending,
        test_strategy: None,
    };

    let json = serde_json::to_string(&subtask).expect("Should serialize");
    let deserialized: ManualSubtask = serde_json::from_str(&json).expect("Should deserialize");

    assert_eq!(deserialized.dependencies.len(), 4);
    assert_eq!(deserialized.dependencies, vec![1, 2, 3, 4]);
}

// ==============================================================================
// ENUM SERIALIZATION TESTS
// ==============================================================================

#[test]
fn test_project_status_kebab_case_serialization() {
    // ProjectStatus uses #[serde(rename_all = "kebab-case")]
    let test_cases = vec![
        (ProjectStatus::Planning, "\"planning\""),
        (ProjectStatus::Building, "\"building\""),
        (ProjectStatus::Review, "\"review\""),
        (ProjectStatus::Launched, "\"launched\""),
        (ProjectStatus::OnHold, "\"on-hold\""), // kebab-case
        (ProjectStatus::Archived, "\"archived\""),
    ];

    for (status, expected_json) in test_cases {
        let json = serde_json::to_string(&status).expect("Should serialize");
        assert_eq!(
            json, expected_json,
            "ProjectStatus::{:?} should serialize as {}",
            status, expected_json
        );

        // Verify round-trip
        let deserialized: ProjectStatus = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(status, deserialized);
    }
}

#[test]
fn test_priority_lowercase_serialization() {
    // Priority uses #[serde(rename_all = "lowercase")]
    let test_cases = vec![
        (Priority::High, "\"high\""),
        (Priority::Medium, "\"medium\""),
        (Priority::Low, "\"low\""),
    ];

    for (priority, expected_json) in test_cases {
        let json = serde_json::to_string(&priority).expect("Should serialize");
        assert_eq!(
            json, expected_json,
            "Priority::{:?} should serialize as {}",
            priority, expected_json
        );

        // Verify round-trip
        let deserialized: Priority = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(priority, deserialized);
    }
}

#[test]
fn test_task_source_lowercase_serialization() {
    // TaskSource uses #[serde(rename_all = "lowercase")]
    let test_cases = vec![
        (TaskSource::Taskmaster, "\"taskmaster\""),
        (TaskSource::Manual, "\"manual\""),
    ];

    for (source, expected_json) in test_cases {
        let json = serde_json::to_string(&source).expect("Should serialize");
        assert_eq!(
            json, expected_json,
            "TaskSource::{:?} should serialize as {}",
            source, expected_json
        );

        // Verify round-trip
        let deserialized: TaskSource = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(source, deserialized);
    }
}

#[test]
fn test_task_status_kebab_case_serialization() {
    // TaskStatus uses #[serde(rename_all = "kebab-case")]
    let test_cases = vec![
        (TaskStatus::Pending, "\"pending\""),
        (TaskStatus::Done, "\"done\""),
        (TaskStatus::InProgress, "\"in-progress\""), // kebab-case
        (TaskStatus::Review, "\"review\""),
        (TaskStatus::Deferred, "\"deferred\""),
        (TaskStatus::Cancelled, "\"cancelled\""),
    ];

    for (status, expected_json) in test_cases {
        let json = serde_json::to_string(&status).expect("Should serialize");
        assert_eq!(
            json, expected_json,
            "TaskStatus::{:?} should serialize as {}",
            status, expected_json
        );

        // Verify round-trip
        let deserialized: TaskStatus = serde_json::from_str(&json).expect("Should deserialize");
        assert_eq!(status, deserialized);
    }
}

#[test]
fn test_enum_defaults() {
    // Verify #[default] attributes work correctly
    assert_eq!(ProjectStatus::default(), ProjectStatus::Planning);
    assert_eq!(Priority::default(), Priority::Medium);
    assert_eq!(TaskStatus::default(), TaskStatus::Pending);

    // Verify #[serde(default)] works in deserialization
    let json_task = r#"{
        "id": 1,
        "title": "Test",
        "description": "Test",
        "dependencies": [],
        "subtasks": [],
        "createdAt": "2024-01-01T00:00:00Z",
        "updatedAt": "2024-01-01T00:00:00Z"
    }"#;

    let task: ManualTask = serde_json::from_str(json_task).expect("Should deserialize");
    assert_eq!(
        task.priority,
        Priority::Medium,
        "Should use default priority"
    );
    assert_eq!(
        task.status,
        TaskStatus::Pending,
        "Should use default status"
    );
}

// ==============================================================================
// DATABASE JSON VALIDATION TESTS
// ==============================================================================
// Tests interaction with SQLite json_valid() CHECK constraints

#[test]
fn test_tags_array_produces_valid_json() {
    let tags: Vec<String> = vec![
        "rust".to_string(),
        "database".to_string(),
        "testing".to_string(),
    ];

    let json = serde_json::to_string(&tags).expect("Should serialize");

    // Verify it's valid JSON (SQLite json_valid() would accept this)
    let parsed: Vec<String> = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(tags, parsed);
}

#[test]
fn test_empty_tags_array_produces_valid_json() {
    let tags: Vec<String> = vec![];

    let json = serde_json::to_string(&tags).expect("Should serialize");
    assert_eq!(json, "[]");

    // Verify it's valid JSON
    let parsed: Vec<String> = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(tags, parsed);
}

#[test]
fn test_mcp_servers_hashmap_produces_valid_json() {
    let mut mcp_servers: HashMap<String, bool> = HashMap::new();
    mcp_servers.insert("filesystem".to_string(), true);
    mcp_servers.insert("brave-search".to_string(), false);

    let json = serde_json::to_string(&mcp_servers).expect("Should serialize");

    // Verify it's valid JSON (SQLite json_valid() would accept this)
    let parsed: HashMap<String, bool> = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(mcp_servers, parsed);
}

#[test]
fn test_empty_mcp_servers_hashmap_produces_valid_json() {
    let mcp_servers: HashMap<String, bool> = HashMap::new();

    let json = serde_json::to_string(&mcp_servers).expect("Should serialize");
    assert_eq!(json, "{}");

    // Verify it's valid JSON
    let parsed: HashMap<String, bool> = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(mcp_servers, parsed);
}

#[test]
fn test_manual_tasks_array_produces_valid_json() {
    let now = Utc::now();
    let tasks = vec![
        ManualTask {
            id: 1,
            title: "Task 1".to_string(),
            description: "First task".to_string(),
            details: None,
            test_strategy: None,
            priority: Priority::High,
            dependencies: vec![],
            status: TaskStatus::Pending,
            subtasks: vec![],
            created_at: now,
            updated_at: now,
        },
        ManualTask {
            id: 2,
            title: "Task 2".to_string(),
            description: "Second task".to_string(),
            details: Some("Details".to_string()),
            test_strategy: Some("Test plan".to_string()),
            priority: Priority::Low,
            dependencies: vec![1],
            status: TaskStatus::Done,
            subtasks: vec![],
            created_at: now,
            updated_at: now,
        },
    ];

    let json = serde_json::to_string(&tasks).expect("Should serialize");

    // Verify it's valid JSON (SQLite json_valid() would accept this)
    let parsed: Vec<ManualTask> = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(tasks.len(), parsed.len());
}

#[test]
fn test_git_repository_produces_valid_json() {
    let git_info = GitRepositoryInfo {
        owner: "OrkeeAI".to_string(),
        repo: "orkee".to_string(),
        url: "https://github.com/OrkeeAI/orkee".to_string(),
        branch: Some("main".to_string()),
    };

    let json = serde_json::to_string(&git_info).expect("Should serialize");

    // Verify it's valid JSON (SQLite json_valid() would accept this)
    let parsed: GitRepositoryInfo = serde_json::from_str(&json).expect("Should be valid JSON");
    assert_eq!(git_info, parsed);
}

// ==============================================================================
// EDGE CASE: DEEPLY NESTED STRUCTURES
// ==============================================================================

#[test]
fn test_manual_task_with_multiple_subtasks_and_dependencies() {
    let now = Utc::now();

    // Create a complex nested structure
    let subtasks = vec![
        ManualSubtask {
            id: 1,
            title: "Subtask 1".to_string(),
            description: "First".to_string(),
            dependencies: vec![],
            details: Some("Details 1".to_string()),
            status: TaskStatus::Done,
            test_strategy: Some("Test 1".to_string()),
        },
        ManualSubtask {
            id: 2,
            title: "Subtask 2".to_string(),
            description: "Second".to_string(),
            dependencies: vec![1], // Depends on subtask 1
            details: Some("Details 2".to_string()),
            status: TaskStatus::InProgress,
            test_strategy: Some("Test 2".to_string()),
        },
        ManualSubtask {
            id: 3,
            title: "Subtask 3".to_string(),
            description: "Third".to_string(),
            dependencies: vec![1, 2], // Depends on subtasks 1 and 2
            details: None,
            status: TaskStatus::Pending,
            test_strategy: None,
        },
    ];

    let task = ManualTask {
        id: 100,
        title: "Complex Task".to_string(),
        description: "Task with nested subtasks".to_string(),
        details: Some("Complex task details".to_string()),
        test_strategy: Some("Integration test strategy".to_string()),
        priority: Priority::High,
        dependencies: vec![99, 98], // Depends on other tasks
        status: TaskStatus::InProgress,
        subtasks,
        created_at: now,
        updated_at: now,
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&task).expect("Should serialize complex structure");
    let deserialized: ManualTask =
        serde_json::from_str(&json).expect("Should deserialize complex structure");

    // Verify all nested data is preserved
    assert_eq!(task.id, deserialized.id);
    assert_eq!(task.subtasks.len(), deserialized.subtasks.len());
    assert_eq!(task.dependencies, deserialized.dependencies);

    // Verify subtask dependencies
    assert_eq!(deserialized.subtasks[0].dependencies, Vec::<u32>::new());
    assert_eq!(deserialized.subtasks[1].dependencies, vec![1]);
    assert_eq!(deserialized.subtasks[2].dependencies, vec![1, 2]);

    // Verify nested statuses
    assert_eq!(deserialized.subtasks[0].status, TaskStatus::Done);
    assert_eq!(deserialized.subtasks[1].status, TaskStatus::InProgress);
    assert_eq!(deserialized.subtasks[2].status, TaskStatus::Pending);
}
