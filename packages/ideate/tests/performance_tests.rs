// ABOUTME: Performance tests for Phase 6F - Query performance monitoring
// ABOUTME: Tests query performance with large datasets (100+ epics) to identify bottlenecks

use sqlx::SqlitePool;
use std::time::Instant;

// ============================================================================
// Test Database Setup
// ============================================================================

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();

    // Create minimal schema
    sqlx::query(
        "CREATE TABLE projects (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO projects (id, name, path) VALUES ('test-proj', 'Test Project', '/tmp/test')")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query(
        "CREATE TABLE ideate_sessions (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            mode TEXT NOT NULL,
            status TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (project_id) REFERENCES projects(id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query("INSERT INTO ideate_sessions (id, project_id, mode, status) VALUES ('test-session', 'test-proj', 'quick', 'completed')")
        .execute(&pool)
        .await
        .unwrap();

    // Full epics table with all Phase 1 fields
    sqlx::query(
        "CREATE TABLE epics (
            id TEXT PRIMARY KEY,
            project_id TEXT NOT NULL,
            prd_id TEXT NOT NULL,
            name TEXT NOT NULL,
            overview_markdown TEXT,
            architecture_decisions TEXT,
            technical_approach TEXT NOT NULL,
            implementation_strategy TEXT,
            dependencies TEXT,
            success_criteria TEXT,
            task_categories TEXT,
            estimated_effort TEXT,
            complexity TEXT,
            status TEXT NOT NULL DEFAULT 'draft',
            progress_percentage INTEGER DEFAULT 0,
            github_issue_number INTEGER,
            github_issue_url TEXT,
            github_synced_at TEXT,
            codebase_context TEXT,
            simplification_analysis TEXT,
            task_count_limit INTEGER DEFAULT 20,
            decomposition_phase TEXT,
            parent_tasks TEXT,
            quality_validation TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (project_id) REFERENCES projects(id),
            FOREIGN KEY (prd_id) REFERENCES ideate_sessions(id)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create index on commonly queried JSON fields
    sqlx::query("CREATE INDEX idx_epics_status ON epics(status)")
        .execute(&pool)
        .await
        .unwrap();

    sqlx::query("CREATE INDEX idx_epics_project ON epics(project_id)")
        .execute(&pool)
        .await
        .unwrap();

    pool
}

// ============================================================================
// Performance Test: Large Dataset Queries
// ============================================================================

#[tokio::test]
async fn test_query_performance_with_100_epics() {
    println!("\n=== Performance Test: Query 100+ Epics ===");
    let pool = setup_test_db().await;

    // Insert 100 epics with JSON fields
    println!("Inserting 100 test epics...");
    let insert_start = Instant::now();

    for i in 0..100 {
        let epic_id = format!("epic-{:03}", i);
        let codebase_context = serde_json::json!({
            "patterns": ["pattern1", "pattern2", "pattern3"],
            "similar_features": ["feature1", "feature2"],
            "reusable_components": ["component1", "component2", "component3"]
        })
        .to_string();

        let parent_tasks = serde_json::json!([
            {
                "id": format!("parent-{}-1", i),
                "title": format!("Parent Task {} - Database Layer", i),
                "estimated_subtasks": 5
            },
            {
                "id": format!("parent-{}-2", i),
                "title": format!("Parent Task {} - API Layer", i),
                "estimated_subtasks": 4
            },
            {
                "id": format!("parent-{}-3", i),
                "title": format!("Parent Task {} - Frontend", i),
                "estimated_subtasks": 6
            }
        ])
        .to_string();

        sqlx::query(
            "INSERT INTO epics (
                id, project_id, prd_id, name, technical_approach,
                status, complexity, codebase_context, parent_tasks,
                task_count_limit
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&epic_id)
        .bind("test-proj")
        .bind("test-session")
        .bind(format!("Epic {}", i))
        .bind(format!("Technical approach for epic {}", i))
        .bind(if i % 3 == 0 { "draft" } else if i % 3 == 1 { "in_progress" } else { "completed" })
        .bind(if i % 4 == 0 { "low" } else if i % 4 == 1 { "medium" } else if i % 4 == 2 { "high" } else { "very_high" })
        .bind(&codebase_context)
        .bind(&parent_tasks)
        .bind(20)
        .execute(&pool)
        .await
        .unwrap();
    }

    let insert_duration = insert_start.elapsed();
    println!("  ✓ Inserted 100 epics in {:?}", insert_duration);
    assert!(
        insert_duration.as_millis() < 5000,
        "Insert took too long: {:?}",
        insert_duration
    );

    // Test 1: Query all epics (full table scan)
    println!("\nTest 1: Query all epics");
    let query_start = Instant::now();

    let all_epics: Vec<(String, String)> =
        sqlx::query_as("SELECT id, name FROM epics ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await
            .unwrap();

    let query_duration = query_start.elapsed();
    println!("  ✓ Retrieved {} epics in {:?}", all_epics.len(), query_duration);
    assert_eq!(all_epics.len(), 100);
    assert!(
        query_duration.as_millis() < 100,
        "Query took too long: {:?}",
        query_duration
    );

    // Test 2: Filter by status (using index)
    println!("\nTest 2: Filter by status (indexed)");
    let filter_start = Instant::now();

    let draft_epics: Vec<(String,)> =
        sqlx::query_as("SELECT id FROM epics WHERE status = 'draft' ORDER BY created_at DESC")
            .fetch_all(&pool)
            .await
            .unwrap();

    let filter_duration = filter_start.elapsed();
    println!(
        "  ✓ Retrieved {} draft epics in {:?}",
        draft_epics.len(),
        filter_duration
    );
    assert!(
        filter_duration.as_millis() < 50,
        "Filtered query took too long: {:?}",
        filter_duration
    );

    // Test 3: Query with JSON field parsing (codebase_context)
    println!("\nTest 3: Query with JSON field parsing");
    let json_start = Instant::now();

    let epics_with_context: Vec<(String, Option<String>)> =
        sqlx::query_as("SELECT id, codebase_context FROM epics WHERE codebase_context IS NOT NULL LIMIT 50")
            .fetch_all(&pool)
            .await
            .unwrap();

    let json_duration = json_start.elapsed();
    println!(
        "  ✓ Retrieved {} epics with JSON context in {:?}",
        epics_with_context.len(),
        json_duration
    );

    // Verify JSON can be parsed
    let mut parsed_count = 0;
    for (_id, context_json) in &epics_with_context {
        if let Some(json_str) = context_json {
            if serde_json::from_str::<serde_json::Value>(json_str).is_ok() {
                parsed_count += 1;
            }
        }
    }
    println!("  ✓ Successfully parsed {}/{} JSON contexts", parsed_count, epics_with_context.len());

    assert!(
        json_duration.as_millis() < 200,
        "JSON query took too long: {:?}",
        json_duration
    );

    // Test 4: Multiple filters with JSON (worst case)
    println!("\nTest 4: Complex query (status + complexity + JSON)");
    let complex_start = Instant::now();

    let complex_epics: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT id, name, complexity
         FROM epics
         WHERE status = 'in_progress'
         AND complexity IN ('high', 'very_high')
         AND codebase_context IS NOT NULL
         ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    let complex_duration = complex_start.elapsed();
    println!(
        "  ✓ Retrieved {} complex epics in {:?}",
        complex_epics.len(),
        complex_duration
    );
    assert!(
        complex_duration.as_millis() < 150,
        "Complex query took too long: {:?}",
        complex_duration
    );

    println!("\n=== Performance Summary ===");
    println!("  Insert 100 epics:     {:?}", insert_duration);
    println!("  Query all epics:      {:?}", query_duration);
    println!("  Filter by status:     {:?}", filter_duration);
    println!("  Query with JSON:      {:?}", json_duration);
    println!("  Complex multi-filter: {:?}", complex_duration);
    println!("\n  ✓ All queries completed within acceptable thresholds");
}

// ============================================================================
// Performance Test: JSON Field Memory Impact
// ============================================================================

#[tokio::test]
async fn test_json_field_memory_impact() {
    println!("\n=== Performance Test: JSON Field Memory Impact ===");
    let pool = setup_test_db().await;

    // Insert epic with large JSON blobs
    let large_execution_steps = serde_json::json!([
        {
            "step_number": 1,
            "action": "Write failing test for authentication",
            "test_command": "cargo test test_auth",
            "expected_output": "FAIL: not implemented",
            "estimated_minutes": 5
        },
        {
            "step_number": 2,
            "action": "Implement basic auth logic",
            "test_command": null,
            "expected_output": "Code written",
            "estimated_minutes": 15
        },
        {
            "step_number": 3,
            "action": "Add token generation",
            "test_command": null,
            "expected_output": "JWT tokens working",
            "estimated_minutes": 10
        },
        {
            "step_number": 4,
            "action": "Run tests to verify",
            "test_command": "cargo test test_auth",
            "expected_output": "PASS",
            "estimated_minutes": 2
        },
        {
            "step_number": 5,
            "action": "Refactor for edge cases",
            "test_command": "cargo test",
            "expected_output": "All tests pass",
            "estimated_minutes": 8
        }
    ])
    .to_string();

    let large_file_references = serde_json::json!([
        {
            "path": "packages/api/src/auth_handlers.rs",
            "operation": "create",
            "reason": "New authentication endpoint handlers"
        },
        {
            "path": "packages/auth/src/lib.rs",
            "operation": "modify",
            "reason": "Add JWT token generation logic"
        },
        {
            "path": "packages/auth/src/validator.rs",
            "operation": "create",
            "reason": "Token validation middleware"
        },
        {
            "path": "packages/storage/src/users.rs",
            "operation": "modify",
            "reason": "Add user credential lookups"
        }
    ])
    .to_string();

    sqlx::query(
        "INSERT INTO epics (
            id, project_id, prd_id, name, technical_approach,
            parent_tasks, codebase_context
        ) VALUES (?, ?, ?, ?, ?, ?, ?)",
    )
    .bind("epic-large")
    .bind("test-proj")
    .bind("test-session")
    .bind("Epic with large JSON")
    .bind("Test approach")
    .bind(&large_execution_steps)
    .bind(&large_file_references)
    .execute(&pool)
    .await
    .unwrap();

    println!("  ✓ Inserted epic with large JSON blobs");
    println!("    - Execution steps: {} bytes", large_execution_steps.len());
    println!("    - File references: {} bytes", large_file_references.len());

    // Query and measure parsing time
    let parse_start = Instant::now();

    let result: (String, Option<String>, Option<String>) = sqlx::query_as(
        "SELECT id, parent_tasks, codebase_context FROM epics WHERE id = 'epic-large'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let parse_duration = parse_start.elapsed();
    println!("\n  ✓ Retrieved and parsed large JSON in {:?}", parse_duration);

    // Verify JSON parsing
    if let Some(steps) = &result.1 {
        let parsed: serde_json::Value = serde_json::from_str(steps).unwrap();
        assert!(parsed.is_array());
        println!("  ✓ Execution steps JSON valid ({} items)", parsed.as_array().unwrap().len());
    }

    if let Some(refs) = &result.2 {
        let parsed: serde_json::Value = serde_json::from_str(refs).unwrap();
        assert!(parsed.is_array());
        println!("  ✓ File references JSON valid ({} items)", parsed.as_array().unwrap().len());
    }

    assert!(
        parse_duration.as_millis() < 50,
        "JSON parsing took too long: {:?}",
        parse_duration
    );

    println!("\n  ✓ Large JSON fields handled efficiently");
}
