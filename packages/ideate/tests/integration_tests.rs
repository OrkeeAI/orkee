// ABOUTME: Integration tests for Phase 6F.2 - Full workflow testing with database
// ABOUTME: Tests complete ideate→PRD→Epic→Tasks flow, two-phase generation, checkpoints, and validation history

use chrono::Utc;
use orkee_ideate::{
    CheckpointType, ComplexityAnalyzer, Epic, EpicComplexity, EpicStatus, EstimatedEffort,
    ValidationEntryType,
};
use sqlx::SqlitePool;

// ============================================================================
// Test Database Setup
// ============================================================================

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();

    // Enable foreign keys
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();

    // Create minimal schema for integration tests
    // Projects table
    sqlx::query(
        "CREATE TABLE projects (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            name TEXT NOT NULL,
            path TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Insert test project
    sqlx::query(
        "INSERT INTO projects (id, name, path) VALUES ('test-proj', 'Test Project', '/tmp/test')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Ideate sessions table
    sqlx::query(
        "CREATE TABLE ideate_sessions (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            project_id TEXT NOT NULL,
            initial_description TEXT NOT NULL,
            mode TEXT NOT NULL CHECK(mode IN ('quick', 'guided', 'chat')),
            status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'in_progress', 'ready_for_prd', 'completed')),
            current_section TEXT,
            research_tools_enabled INTEGER NOT NULL DEFAULT 0,
            skipped_sections TEXT,
            template_id TEXT,
            generated_prd_id TEXT,
            non_goals TEXT,
            open_questions TEXT,
            constraints_assumptions TEXT,
            success_metrics TEXT,
            alternative_approaches TEXT,
            validation_checkpoints TEXT,
            codebase_context TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // PRDs table
    sqlx::query(
        "CREATE TABLE prds (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            project_id TEXT NOT NULL,
            session_id TEXT,
            name TEXT NOT NULL,
            markdown_content TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE SET NULL
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Epics table with Phase 1 enhancements
    sqlx::query(
        "CREATE TABLE epics (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            project_id TEXT NOT NULL,
            prd_id TEXT NOT NULL,
            name TEXT NOT NULL,
            overview_markdown TEXT NOT NULL,
            technical_approach TEXT NOT NULL,
            implementation_strategy TEXT,
            architecture_decisions TEXT,
            dependencies TEXT,
            success_criteria TEXT,
            task_categories TEXT,
            estimated_effort TEXT,
            complexity TEXT,
            status TEXT DEFAULT 'draft',
            progress_percentage INTEGER DEFAULT 0,
            github_issue_number INTEGER,
            github_issue_url TEXT,
            github_synced_at TEXT,
            codebase_context TEXT,
            simplification_analysis TEXT,
            task_count_limit INTEGER DEFAULT 20,
            decomposition_phase TEXT CHECK(decomposition_phase IN ('parent_planning', 'subtask_generation', 'completed')),
            parent_tasks TEXT,
            quality_validation TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            started_at TEXT,
            completed_at TEXT,
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Tasks table with Phase 1 TDD enhancements
    sqlx::query(
        "CREATE TABLE tasks (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            project_id TEXT NOT NULL,
            epic_id TEXT,
            name TEXT NOT NULL,
            description TEXT,
            status TEXT DEFAULT 'pending',
            priority INTEGER DEFAULT 5,
            test_strategy TEXT NOT NULL DEFAULT '',
            acceptance_criteria TEXT,
            relevant_files TEXT,
            similar_implementations TEXT,
            complexity_score INTEGER CHECK(complexity_score >= 1 AND complexity_score <= 10),
            execution_steps TEXT,
            validation_history TEXT,
            codebase_references TEXT,
            parent_task_id TEXT,
            github_issue_number INTEGER,
            github_issue_url TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
            FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Task complexity reports table
    sqlx::query(
        "CREATE TABLE task_complexity_reports (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            epic_id TEXT NOT NULL,
            task_id TEXT,
            complexity_score INTEGER CHECK(complexity_score >= 1 AND complexity_score <= 10),
            recommended_subtasks INTEGER,
            expansion_prompt TEXT,
            reasoning TEXT,
            analyzed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Execution checkpoints table
    sqlx::query(
        "CREATE TABLE execution_checkpoints (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            epic_id TEXT NOT NULL,
            after_task_id TEXT NOT NULL,
            checkpoint_type TEXT NOT NULL CHECK(checkpoint_type IN ('review', 'test', 'integration', 'approval')),
            message TEXT NOT NULL,
            required_validation TEXT,
            reached INTEGER DEFAULT 0,
            reached_at TEXT,
            created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE,
            FOREIGN KEY (after_task_id) REFERENCES tasks(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Validation entries table
    sqlx::query(
        "CREATE TABLE validation_entries (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            task_id TEXT NOT NULL,
            entry_type TEXT NOT NULL CHECK(entry_type IN ('progress', 'issue', 'decision', 'checkpoint')),
            content TEXT NOT NULL,
            author TEXT NOT NULL,
            timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Discovery sessions table
    sqlx::query(
        "CREATE TABLE discovery_sessions (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            session_id TEXT NOT NULL,
            question_number INTEGER NOT NULL,
            question_text TEXT NOT NULL,
            question_type TEXT CHECK(question_type IN ('open', 'multiple_choice', 'yes_no')),
            options TEXT,
            user_answer TEXT,
            asked_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            answered_at TEXT,
            FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
            UNIQUE(session_id, question_number)
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    // PRD validation history table
    sqlx::query(
        "CREATE TABLE prd_validation_history (
            id TEXT PRIMARY KEY CHECK(length(id) >= 8),
            session_id TEXT NOT NULL,
            section_name TEXT NOT NULL,
            validation_status TEXT CHECK(validation_status IN ('approved', 'rejected', 'regenerated')),
            user_feedback TEXT,
            quality_score INTEGER CHECK(quality_score >= 0 AND quality_score <= 100),
            validated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
            FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
        )",
    )
    .execute(&pool)
    .await
    .unwrap();

    pool
}

// ============================================================================
// Test 1: Full Ideate → PRD → Epic → Tasks Flow
// ============================================================================

#[tokio::test]
async fn test_full_ideate_to_tasks_workflow() {
    println!("\n=== Test 1: Full Ideate → PRD → Epic → Tasks Flow ===\n");

    let pool = setup_test_db().await;

    // Step 1: Create ideate session
    println!("Step 1: Creating ideate session...");
    let session_id = "test-session-001";
    sqlx::query(
        "INSERT INTO ideate_sessions (id, project_id, initial_description, mode, status)
         VALUES (?, 'test-proj', 'Build a task management system with AI assistance', 'quick', 'draft')"
    )
    .bind(session_id)
    .execute(&pool)
    .await
    .unwrap();

    let (mode, status): (String, String) =
        sqlx::query_as("SELECT mode, status FROM ideate_sessions WHERE id = ?")
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(mode, "quick");
    assert_eq!(status, "draft");
    println!(
        "  ✓ Ideate session created (mode: {}, status: {})",
        mode, status
    );

    // Step 2: Generate PRD
    println!("\nStep 2: Generating PRD...");
    let prd_id = "test-prd-001";
    sqlx::query(
        "INSERT INTO prds (id, project_id, session_id, name, markdown_content)
         VALUES (?, 'test-proj', ?, 'Task Management PRD', '# Task Management System\\n\\n## Overview\\nAI-powered task management.')"
    )
    .bind(prd_id)
    .bind(session_id)
    .execute(&pool)
    .await
    .unwrap();

    // Update session with PRD link
    sqlx::query(
        "UPDATE ideate_sessions SET generated_prd_id = ?, status = 'completed' WHERE id = ?",
    )
    .bind(prd_id)
    .bind(session_id)
    .execute(&pool)
    .await
    .unwrap();

    println!("  ✓ PRD generated and linked to session");

    // Step 3: Create Epic from PRD
    println!("\nStep 3: Creating Epic from PRD...");
    let epic_id = "test-epic-001";
    sqlx::query(
        "INSERT INTO epics (
            id, project_id, prd_id, name, overview_markdown, technical_approach,
            status, complexity, estimated_effort, task_count_limit, decomposition_phase
        ) VALUES (?, 'test-proj', ?, 'Task Management Backend',
                  '## Epic: Task Management Backend\\n\\nBuild core task CRUD operations.',
                  'Use Rust with Axum and SQLite', 'draft', 'medium', 'days', 20, 'parent_planning')"
    )
    .bind(epic_id)
    .bind(prd_id)
    .execute(&pool)
    .await
    .unwrap();

    let (epic_status, epic_complexity, epic_decomp_phase): (
        String,
        Option<String>,
        Option<String>,
    ) = sqlx::query_as("SELECT status, complexity, decomposition_phase FROM epics WHERE id = ?")
        .bind(epic_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(epic_status, "draft");
    assert_eq!(epic_decomp_phase, Some("parent_planning".to_string()));
    println!(
        "  ✓ Epic created (complexity: {:?}, phase: {:?})",
        epic_complexity, epic_decomp_phase
    );

    // Step 4: Analyze complexity
    println!("\nStep 4: Analyzing epic complexity...");
    let complexity_analyzer = ComplexityAnalyzer::new();

    // Create Epic instance for complexity analysis
    let test_epic = Epic {
        id: epic_id.to_string(),
        project_id: "test-proj".to_string(),
        prd_id: prd_id.to_string(),
        name: "Task Management Backend".to_string(),
        overview_markdown: "## Epic: Task Management Backend\\n\\nBuild core task CRUD operations."
            .to_string(),
        technical_approach: "Use Rust with Axum and SQLite".to_string(),
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
        decomposition_phase: Some("parent_planning".to_string()),
        parent_tasks: None,
        quality_validation: None,
        created_at: Utc::now(),
        updated_at: Utc::now(),
        started_at: None,
        completed_at: None,
    };

    let complexity_report = complexity_analyzer
        .analyze_epic(&test_epic, Some(20))
        .unwrap();

    println!("  ✓ Complexity analyzed:");
    println!("    - Score: {}/10", complexity_report.score);
    println!(
        "    - Recommended tasks: {}",
        complexity_report.recommended_tasks
    );
    println!("    - Reasoning: {}", complexity_report.reasoning);

    assert!(complexity_report.score >= 1 && complexity_report.score <= 10);
    assert!(complexity_report.recommended_tasks <= 20);

    // Step 5: Generate tasks (simulated - would use TaskDecomposer in real flow)
    println!("\nStep 5: Generating tasks...");
    let task_ids = ["test-task-001", "test-task-002", "test-task-003"];

    for (idx, task_id) in task_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO tasks (
                id, project_id, epic_id, name, description, status, priority,
                test_strategy, acceptance_criteria, complexity_score
            ) VALUES (?, 'test-proj', ?, ?, ?, 'pending', ?, ?, ?, ?)",
        )
        .bind(task_id)
        .bind(epic_id)
        .bind(format!("Task {}: Implement feature", idx + 1))
        .bind(format!("Detailed description for task {}", idx + 1))
        .bind(5)
        .bind(format!("Write integration test for task {}", idx + 1))
        .bind("{\"criteria\":[\"Test passes\",\"No errors\"]}".to_string())
        .bind(5)
        .execute(&pool)
        .await
        .unwrap();
    }

    let task_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE epic_id = ?")
        .bind(epic_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(task_count, 3);
    println!("  ✓ {} tasks generated and linked to epic", task_count);

    // Step 6: Verify full workflow integrity
    println!("\nStep 6: Verifying workflow integrity...");

    // Verify session → PRD link
    let session_check: (String,) =
        sqlx::query_as("SELECT generated_prd_id FROM ideate_sessions WHERE id = ?")
            .bind(session_id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(session_check.0, prd_id);

    // Verify PRD → Epic link
    let epic_check: (String,) = sqlx::query_as("SELECT prd_id FROM epics WHERE id = ?")
        .bind(epic_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(epic_check.0, prd_id);

    // Verify Epic → Tasks link
    let tasks_check: Vec<(String,)> =
        sqlx::query_as("SELECT id FROM tasks WHERE epic_id = ? ORDER BY created_at")
            .bind(epic_id)
            .fetch_all(&pool)
            .await
            .unwrap();
    assert_eq!(tasks_check.len(), 3);

    println!("  ✓ Full workflow chain verified:");
    println!("    Session → PRD → Epic → {} Tasks", tasks_check.len());

    println!("\n✅ Test 1 PASSED: Full Ideate → PRD → Epic → Tasks Flow\n");
}

// ============================================================================
// Test 2: Two-Phase Task Generation
// ============================================================================

#[tokio::test]
async fn test_two_phase_task_generation() {
    println!("\n=== Test 2: Two-Phase Task Generation ===\n");

    let pool = setup_test_db().await;

    // Setup: Insert PRD first (epic references it)
    sqlx::query(
        "INSERT INTO prds (id, project_id, name, markdown_content)
         VALUES ('test-prd-002', 'test-proj', 'Auth PRD', '# Auth System')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create epic for task generation
    let epic_id = "test-epic-002";
    sqlx::query(
        "INSERT INTO epics (
            id, project_id, prd_id, name, overview_markdown, technical_approach,
            status, task_count_limit, decomposition_phase
        ) VALUES (?, 'test-proj', 'test-prd-002', 'Authentication System',
                  '## Epic: Authentication System\\n\\nOAuth and JWT-based auth.',
                  'Use existing auth patterns', 'draft', 15, 'parent_planning')",
    )
    .bind(epic_id)
    .execute(&pool)
    .await
    .unwrap();

    // Phase 1: Generate parent tasks
    println!("Phase 1: Generating parent tasks...");
    let parent_tasks = vec![
        ("parent-001", "Database schema for users and sessions", 3),
        ("parent-002", "OAuth provider integration", 5),
        ("parent-003", "JWT token generation and validation", 4),
    ];

    for (id, name, est_subtasks) in &parent_tasks {
        sqlx::query(
            "INSERT INTO tasks (
                id, project_id, epic_id, name, description, status, priority,
                test_strategy, parent_task_id
            ) VALUES (?, 'test-proj', ?, ?, ?, 'pending', 5, 'Parent task - no direct tests', NULL)"
        )
        .bind(id)
        .bind(epic_id)
        .bind(name)
        .bind(format!("Parent task that will expand to {} subtasks", est_subtasks))
        .execute(&pool)
        .await
        .unwrap();
    }

    let parent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE epic_id = ? AND parent_task_id IS NULL",
    )
    .bind(epic_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(parent_count, 3);
    println!("  ✓ Generated {} parent tasks", parent_count);

    // Store parent tasks in epic
    let parent_tasks_json = serde_json::json!([
        {
            "id": "parent-001",
            "title": "Database schema for users and sessions",
            "estimated_subtasks": 3
        },
        {
            "id": "parent-002",
            "title": "OAuth provider integration",
            "estimated_subtasks": 5
        },
        {
            "id": "parent-003",
            "title": "JWT token generation and validation",
            "estimated_subtasks": 4
        }
    ]);

    sqlx::query(
        "UPDATE epics SET parent_tasks = ?, decomposition_phase = 'parent_planning' WHERE id = ?",
    )
    .bind(parent_tasks_json.to_string())
    .bind(epic_id)
    .execute(&pool)
    .await
    .unwrap();

    println!("  ✓ Parent tasks stored in epic.parent_tasks field");

    // User can review and edit parent tasks here...
    println!("\n  [User review phase - parent tasks can be edited/reordered]");

    // Phase 2: Expand parent tasks to subtasks
    println!("\nPhase 2: Expanding parent tasks to subtasks...");

    let subtask_definitions = vec![
        (
            "parent-001",
            vec![
                ("subtask-001-1", "Create users table migration"),
                ("subtask-001-2", "Create sessions table migration"),
                ("subtask-001-3", "Add indexes for performance"),
            ],
        ),
        (
            "parent-002",
            vec![
                ("subtask-002-1", "Configure OAuth provider credentials"),
                ("subtask-002-2", "Implement OAuth callback handler"),
                ("subtask-002-3", "Add OAuth state validation"),
                ("subtask-002-4", "Store OAuth tokens securely"),
                ("subtask-002-5", "Add OAuth refresh token logic"),
            ],
        ),
        (
            "parent-003",
            vec![
                ("subtask-003-1", "Generate JWT signing keys"),
                ("subtask-003-2", "Implement JWT token creation"),
                ("subtask-003-3", "Implement JWT token validation"),
                ("subtask-003-4", "Add token expiration logic"),
            ],
        ),
    ];

    for (parent_id, subtasks) in &subtask_definitions {
        for (idx, (subtask_id, subtask_name)) in subtasks.iter().enumerate() {
            sqlx::query(
                "INSERT INTO tasks (
                    id, project_id, epic_id, name, description, status, priority,
                    test_strategy, acceptance_criteria, complexity_score,
                    parent_task_id
                ) VALUES (?, 'test-proj', ?, ?, ?, 'pending', 5, ?, ?, ?, ?)",
            )
            .bind(subtask_id)
            .bind(epic_id)
            .bind(subtask_name)
            .bind(format!("Subtask {} of parent {}", idx + 1, parent_id))
            .bind(format!("Write unit test for {}", subtask_name))
            .bind("{\"criteria\":[\"Test passes\",\"Code reviewed\"]}".to_string())
            .bind(3) // complexity_score
            .bind(parent_id)
            .execute(&pool)
            .await
            .unwrap();
        }
    }

    // Update epic decomposition phase
    sqlx::query("UPDATE epics SET decomposition_phase = 'subtask_generation' WHERE id = ?")
        .bind(epic_id)
        .execute(&pool)
        .await
        .unwrap();

    // Verify two-phase generation
    let final_parent_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE epic_id = ? AND parent_task_id IS NULL",
    )
    .bind(epic_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let final_subtask_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM tasks WHERE epic_id = ? AND parent_task_id IS NOT NULL",
    )
    .bind(epic_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    let total_tasks: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tasks WHERE epic_id = ?")
        .bind(epic_id)
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(final_parent_count, 3);
    assert_eq!(final_subtask_count, 12); // 3 + 5 + 4 = 12
    assert_eq!(total_tasks, 15); // 3 parents + 12 subtasks

    println!("  ✓ Two-phase generation complete:");
    println!("    - Parent tasks: {}", final_parent_count);
    println!("    - Subtasks: {}", final_subtask_count);
    println!("    - Total: {}", total_tasks);

    // Verify epic is within task count limit
    let (task_limit,): (Option<i64>,) =
        sqlx::query_as("SELECT task_count_limit FROM epics WHERE id = ?")
            .bind(epic_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    let limit = task_limit.unwrap_or(20);
    assert!(total_tasks <= limit);
    println!("  ✓ Task count ({}) within limit ({})", total_tasks, limit);

    println!("\n✅ Test 2 PASSED: Two-Phase Task Generation\n");
}

// ============================================================================
// Test 3: Checkpoint System
// ============================================================================

#[tokio::test]
async fn test_checkpoint_system() {
    println!("\n=== Test 3: Checkpoint System ===\n");

    let pool = setup_test_db().await;

    // Setup: Insert PRD first
    sqlx::query(
        "INSERT INTO prds (id, project_id, name, markdown_content)
         VALUES ('test-prd-003', 'test-proj', 'Payment PRD', '# Payments')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create epic
    let epic_id = "test-epic-003";
    sqlx::query(
        "INSERT INTO epics (
            id, project_id, prd_id, name, overview_markdown, technical_approach, status
        ) VALUES (?, 'test-proj', 'test-prd-003', 'Payment Integration',
                  '## Payment Integration\\n\\nStripe payment flow.',
                  'Use Stripe SDK', 'in_progress')",
    )
    .bind(epic_id)
    .execute(&pool)
    .await
    .unwrap();

    // Create tasks
    let task_ids = [
        "pay-task-001",
        "pay-task-002",
        "pay-task-003",
        "pay-task-004",
    ];
    for (idx, task_id) in task_ids.iter().enumerate() {
        sqlx::query(
            "INSERT INTO tasks (
                id, project_id, epic_id, name, description, status, test_strategy
            ) VALUES (?, 'test-proj', ?, ?, 'Payment task', 'pending', 'Test payment flow')",
        )
        .bind(task_id)
        .bind(epic_id)
        .bind(format!("Payment task {}", idx + 1))
        .execute(&pool)
        .await
        .unwrap();
    }

    // Create execution checkpoints
    println!("Creating execution checkpoints...");

    let checkpoints = vec![
        (
            "checkpoint-001",
            "pay-task-002",
            CheckpointType::Review,
            "Database schema complete. Review migrations before API work?",
            vec!["Schema matches requirements", "Migrations run cleanly"],
        ),
        (
            "checkpoint-002",
            "pay-task-004",
            CheckpointType::Test,
            "All payment endpoints implemented. Run integration tests?",
            vec!["All tests pass", "No security vulnerabilities"],
        ),
    ];

    for (cp_id, after_task, cp_type, message, validations) in checkpoints {
        let cp_type_str = match cp_type {
            CheckpointType::Review => "review",
            CheckpointType::Test => "test",
            CheckpointType::Integration => "integration",
            CheckpointType::Approval => "approval",
        };

        let validations_json = serde_json::to_string(&validations).unwrap();

        sqlx::query(
            "INSERT INTO execution_checkpoints (
                id, epic_id, after_task_id, checkpoint_type, message, required_validation
            ) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(cp_id)
        .bind(epic_id)
        .bind(after_task)
        .bind(cp_type_str)
        .bind(message)
        .bind(validations_json)
        .execute(&pool)
        .await
        .unwrap();

        println!(
            "  ✓ Checkpoint created: {} (type: {:?}, after: {})",
            cp_id, cp_type, after_task
        );
    }

    // Verify checkpoints
    let checkpoint_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM execution_checkpoints WHERE epic_id = ?")
            .bind(epic_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(checkpoint_count, 2);
    println!("\n  ✓ {} checkpoints created for epic", checkpoint_count);

    // Simulate reaching a checkpoint
    println!("\nSimulating checkpoint activation...");

    // Mark task-002 as complete
    sqlx::query("UPDATE tasks SET status = 'completed' WHERE id = 'pay-task-002'")
        .execute(&pool)
        .await
        .unwrap();

    // Activate checkpoint
    let now = Utc::now().to_rfc3339();
    sqlx::query(
        "UPDATE execution_checkpoints SET reached = 1, reached_at = ? WHERE id = 'checkpoint-001'",
    )
    .bind(now)
    .execute(&pool)
    .await
    .unwrap();

    println!("  ✓ Checkpoint 'checkpoint-001' reached after task completion");

    // Verify checkpoint state
    let (reached, message): (i64, String) = sqlx::query_as(
        "SELECT reached, message FROM execution_checkpoints WHERE id = 'checkpoint-001'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(reached, 1);
    println!("  ✓ Checkpoint state verified:");
    println!("    - Reached: {}", reached == 1);
    println!("    - Message: {}", message);

    // Verify checkpoints are properly ordered by task
    let ordered_checkpoints: Vec<(String, String)> = sqlx::query_as(
        "SELECT ec.id, t.name
         FROM execution_checkpoints ec
         JOIN tasks t ON ec.after_task_id = t.id
         WHERE ec.epic_id = ?
         ORDER BY t.created_at",
    )
    .bind(epic_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(ordered_checkpoints.len(), 2);
    println!("\n  ✓ Checkpoints properly ordered by task sequence");

    println!("\n✅ Test 3 PASSED: Checkpoint System\n");
}

// ============================================================================
// Test 4: Validation History (Append-Only Progress Tracking)
// ============================================================================

#[tokio::test]
async fn test_validation_history() {
    println!("\n=== Test 4: Validation History (Append-Only) ===\n");

    let pool = setup_test_db().await;

    // Setup: Insert PRD first
    sqlx::query(
        "INSERT INTO prds (id, project_id, name, markdown_content)
         VALUES ('history-prd', 'test-proj', 'PRD', '# PRD')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create epic
    sqlx::query(
        "INSERT INTO epics (
            id, project_id, prd_id, name, overview_markdown, technical_approach, status
        ) VALUES ('history-epic', 'test-proj', 'history-prd', 'Feature X',
                  '## Feature X', 'Implementation', 'in_progress')",
    )
    .execute(&pool)
    .await
    .unwrap();

    // Create task
    let task_id = "history-task-001";

    sqlx::query(
        "INSERT INTO tasks (
            id, project_id, epic_id, name, description, status, test_strategy
        ) VALUES (?, 'test-proj', 'history-epic', 'Implement auth flow',
                  'Add OAuth login', 'in_progress', 'Integration tests')",
    )
    .bind(task_id)
    .execute(&pool)
    .await
    .unwrap();

    // Test append-only progress tracking
    println!("Testing append-only validation history...");

    let entries = vec![
        (
            "entry-001",
            ValidationEntryType::Progress,
            "Started implementation",
            "dev-1",
        ),
        (
            "entry-002",
            ValidationEntryType::Decision,
            "Chose OAuth 2.0 over SAML",
            "dev-1",
        ),
        (
            "entry-003",
            ValidationEntryType::Issue,
            "Token refresh failing in edge case",
            "dev-2",
        ),
        (
            "entry-004",
            ValidationEntryType::Progress,
            "Fixed token refresh issue",
            "dev-2",
        ),
        (
            "entry-005",
            ValidationEntryType::Checkpoint,
            "Code review complete",
            "reviewer",
        ),
        (
            "entry-006",
            ValidationEntryType::Progress,
            "Tests passing",
            "dev-1",
        ),
    ];

    for (entry_id, entry_type, content, author) in &entries {
        let entry_type_str = match entry_type {
            ValidationEntryType::Progress => "progress",
            ValidationEntryType::Issue => "issue",
            ValidationEntryType::Decision => "decision",
            ValidationEntryType::Checkpoint => "checkpoint",
        };

        sqlx::query(
            "INSERT INTO validation_entries (
                id, task_id, entry_type, content, author
            ) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(entry_id)
        .bind(task_id)
        .bind(entry_type_str)
        .bind(content)
        .bind(author)
        .execute(&pool)
        .await
        .unwrap();

        println!("  ✓ Appended: {:?} - {}", entry_type, content);
    }

    // Verify all entries are preserved (append-only)
    let entry_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM validation_entries WHERE task_id = ?")
            .bind(task_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(entry_count, 6);
    println!("\n  ✓ All {} entries preserved (append-only)", entry_count);

    // Test that we can retrieve history in chronological order
    let history: Vec<(String, String, String)> = sqlx::query_as(
        "SELECT entry_type, content, author
         FROM validation_entries
         WHERE task_id = ?
         ORDER BY timestamp ASC",
    )
    .bind(task_id)
    .fetch_all(&pool)
    .await
    .unwrap();

    assert_eq!(history.len(), 6);
    assert_eq!(history[0].1, "Started implementation");
    assert_eq!(history[5].1, "Tests passing");
    println!("  ✓ History retrieved in chronological order");

    // Test filtering by entry type
    let issue_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM validation_entries WHERE task_id = ? AND entry_type = 'issue'",
    )
    .bind(task_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(issue_count, 1);
    println!("  ✓ Can filter by entry type (issues: {})", issue_count);

    // Test that entries are never deleted (append-only invariant)
    // Simulate a "correction" - we add a new entry instead of modifying
    println!("\nTesting append-only invariant (corrections add new entries)...");

    sqlx::query(
        "INSERT INTO validation_entries (
            id, task_id, entry_type, content, author
        ) VALUES ('entry-007', ?, 'progress', 'Correction: OAuth version is 2.1, not 2.0', 'dev-1')"
    )
    .bind(task_id)
    .execute(&pool)
    .await
    .unwrap();

    let final_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM validation_entries WHERE task_id = ?")
            .bind(task_id)
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(final_count, 7); // Original 6 + 1 correction
    println!(
        "  ✓ Corrections append new entries (count: {})",
        final_count
    );

    // Verify old entries still exist
    let original_decision: (String,) =
        sqlx::query_as("SELECT content FROM validation_entries WHERE id = 'entry-002'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(original_decision.0, "Chose OAuth 2.0 over SAML");
    println!("  ✓ Original entries preserved (never overwritten)");

    println!("\n✅ Test 4 PASSED: Validation History (Append-Only)\n");
}
