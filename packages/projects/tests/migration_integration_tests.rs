// ABOUTME: Integration tests for database migrations
// ABOUTME: Verifies schema creation, seed data, indexes, and constraints

use sqlx::{Pool, Row, Sqlite};

/// Helper to create a fresh in-memory database with migrations applied
async fn setup_migrated_db() -> Pool<Sqlite> {
    let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Migration should succeed");

    pool
}

#[tokio::test]
async fn test_initial_schema_migration_succeeds() {
    let pool = setup_migrated_db().await;

    // Verify we can query the database
    let result: i32 = sqlx::query_scalar("SELECT 1")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(result, 1, "Database should be queryable after migration");
}

#[tokio::test]
async fn test_all_core_tables_created() {
    let pool = setup_migrated_db().await;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // Core tables that must exist
    // Note: agents table removed - now loaded from config/agents.json
    let required_tables = vec![
        "_sqlx_migrations",
        "agent_executions",
        "ai_usage_logs",
        "api_tokens",
        "ast_spec_mappings",
        "context_configurations",
        "context_snapshots",
        "context_templates",
        "context_usage_patterns",
        "encryption_settings",
        "password_attempts",
        "pr_reviews",
        "prd_spec_sync_history",
        "prds",
        "projects",
        "projects_fts",
        "spec_capabilities",
        "spec_capabilities_history",
        "spec_change_tasks",
        "spec_changes",
        "spec_deltas",
        "spec_materializations",
        "spec_requirements",
        "spec_scenarios",
        "storage_metadata",
        "sync_snapshots",
        "sync_state",
        "system_settings",
        "tags",
        "task_spec_links",
        "tasks",
        "tasks_fts",
        "telemetry_events",
        "telemetry_settings",
        "telemetry_stats",
        "user_agents",
        "users",
    ];

    for required_table in &required_tables {
        assert!(
            tables.contains(&required_table.to_string()),
            "Missing required table: {}",
            required_table
        );
    }

    // Verify we have at least the required tables (allows for additional tables)
    assert!(
        tables.len() >= required_tables.len(),
        "Expected at least {} tables, got {}",
        required_tables.len(),
        tables.len()
    );
}

#[tokio::test]
async fn test_seed_data_default_user_created() {
    let pool = setup_migrated_db().await;

    let user_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(user_count, 1, "Should have exactly 1 default user");

    // Verify default user details
    let (id, email, name): (String, String, String) =
        sqlx::query_as("SELECT id, email, name FROM users WHERE id = 'default-user'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(
        id, "default-user",
        "Default user ID should be 'default-user'"
    );
    assert_eq!(
        email, "user@localhost",
        "Default user email should be 'user@localhost'"
    );
    assert_eq!(
        name, "Default User",
        "Default user name should be 'Default User'"
    );
}

#[tokio::test]
async fn test_agents_loaded_from_json() {
    // Agents are now loaded from config/agents.json via ModelRegistry
    // This test verifies the registry can be initialized and contains expected agents
    use orkee_projects::models::REGISTRY;

    let agent = REGISTRY.get_agent("claude-code");
    assert!(agent.is_some(), "Should find claude-code agent");
    assert_eq!(agent.unwrap().name, "Claude Code");

    let agent = REGISTRY.get_agent("aider");
    assert!(agent.is_some(), "Should find aider agent");
    assert_eq!(agent.unwrap().name, "Aider");

    // Verify we have at least 2 agents
    let agents = REGISTRY.list_agents();
    assert!(
        agents.len() >= 2,
        "Should have at least 2 agents from JSON config"
    );
}

#[tokio::test]
async fn test_storage_metadata_seeded() {
    let pool = setup_migrated_db().await;

    // Verify storage_metadata has required keys
    let keys: Vec<String> = sqlx::query_scalar("SELECT key FROM storage_metadata ORDER BY key")
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(
        keys.contains(&"created_at".to_string()),
        "storage_metadata should have 'created_at' key"
    );
    assert!(
        keys.contains(&"storage_type".to_string()),
        "storage_metadata should have 'storage_type' key"
    );

    // Verify schema_version is NOT present (managed by SQLx)
    assert!(
        !keys.contains(&"schema_version".to_string()),
        "storage_metadata should NOT have 'schema_version' (managed by SQLx)"
    );

    // Verify storage_type value
    let storage_type: String =
        sqlx::query_scalar("SELECT value FROM storage_metadata WHERE key = 'storage_type'")
            .fetch_one(&pool)
            .await
            .unwrap();

    assert_eq!(storage_type, "sqlite", "storage_type should be 'sqlite'");
}

#[tokio::test]
async fn test_foreign_key_constraints_enabled() {
    let pool = setup_migrated_db().await;

    let foreign_keys_enabled: i64 = sqlx::query_scalar("PRAGMA foreign_keys")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert_eq!(
        foreign_keys_enabled, 1,
        "Foreign keys should be enabled (PRAGMA foreign_keys = ON)"
    );
}

#[tokio::test]
async fn test_critical_indexes_created() {
    let pool = setup_migrated_db().await;

    // Get all indexes
    let indexes: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%' ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // Critical indexes for performance
    let critical_indexes = vec![
        "idx_projects_name",
        "idx_tasks_project_id",
        "idx_tasks_status",
        "idx_tasks_created_by_user_id",
        "idx_tasks_change_id",
        "idx_tasks_from_prd_id",
        "idx_tasks_user_status",
        "idx_spec_changes_project",
        "idx_spec_changes_created_at",
        // idx_spec_capabilities_project removed - redundant with idx_spec_capabilities_project_status
        "idx_spec_capabilities_project_status",
        "idx_prds_project",
        "idx_ai_usage_logs_project",
    ];

    for critical_index in &critical_indexes {
        assert!(
            indexes.contains(&critical_index.to_string()),
            "Missing critical index: {}",
            critical_index
        );
    }
}

#[tokio::test]
async fn test_tasks_foreign_keys_configured() {
    let pool = setup_migrated_db().await;

    // Get foreign key definitions for tasks table
    // PRAGMA foreign_key_list returns: id, seq, table, from, to, on_update, on_delete, match
    let fk_rows: Vec<(i64, i64, String, String, String, String, String, String)> =
        sqlx::query_as("PRAGMA foreign_key_list(tasks)")
            .fetch_all(&pool)
            .await
            .unwrap();

    // Verify critical foreign keys exist
    let fk_tables: Vec<String> = fk_rows
        .iter()
        .map(|(_, _, table, _, _, _, _, _)| table.clone())
        .collect();

    assert!(
        fk_tables.contains(&"projects".to_string()),
        "tasks should have FK to projects"
    );
    assert!(
        fk_tables.contains(&"users".to_string()),
        "tasks should have FK to users (created_by_user_id)"
    );
    // Note: FK to agents removed - agent_id fields now reference config/agents.json (no FK enforcement)
    assert!(
        fk_tables.contains(&"spec_changes".to_string()),
        "tasks should have FK to spec_changes"
    );
    assert!(
        fk_tables.contains(&"prds".to_string()),
        "tasks should have FK to prds"
    );

    // Verify ON DELETE behavior for created_by_user_id
    let user_fk = fk_rows
        .iter()
        .find(|(_, _, table, from, _, _, _, _)| table == "users" && from == "created_by_user_id");

    assert!(
        user_fk.is_some(),
        "Should have FK from created_by_user_id to users"
    );

    // Check ON DELETE RESTRICT is set (column 7 is on_delete)
    let (_, _, _, _, _, _, on_delete, _) = user_fk.unwrap();
    assert_eq!(
        on_delete, "RESTRICT",
        "created_by_user_id should have ON DELETE RESTRICT"
    );
}

#[tokio::test]
async fn test_spec_changes_foreign_keys_configured() {
    let pool = setup_migrated_db().await;

    // Get foreign key definitions for spec_changes table
    // PRAGMA foreign_key_list returns: id, seq, table, from, to, on_update, on_delete, match
    let fk_rows: Vec<(i64, i64, String, String, String, String, String, String)> =
        sqlx::query_as("PRAGMA foreign_key_list(spec_changes)")
            .fetch_all(&pool)
            .await
            .unwrap();

    // Verify foreign keys
    let fk_tables: Vec<String> = fk_rows
        .iter()
        .map(|(_, _, table, _, _, _, _, _)| table.clone())
        .collect();

    assert!(
        fk_tables.contains(&"projects".to_string()),
        "spec_changes should have FK to projects"
    );
    assert!(
        fk_tables.contains(&"prds".to_string()),
        "spec_changes should have FK to prds"
    );
}

#[tokio::test]
async fn test_boolean_fields_use_boolean_type() {
    let pool = setup_migrated_db().await;

    // Check that boolean fields are defined as BOOLEAN (not INTEGER)
    let schema = sqlx::query("SELECT sql FROM sqlite_master WHERE name = 'users'")
        .fetch_one(&pool)
        .await
        .unwrap();

    let sql: String = schema.get("sql");

    assert!(
        sql.contains("ai_gateway_enabled BOOLEAN"),
        "users.ai_gateway_enabled should be BOOLEAN type"
    );
}

#[tokio::test]
async fn test_sqlx_migrations_table_tracks_version() {
    let pool = setup_migrated_db().await;

    // Verify _sqlx_migrations table exists and has entries
    let migration_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM _sqlx_migrations")
        .fetch_one(&pool)
        .await
        .unwrap();

    assert!(
        migration_count >= 1,
        "Should have at least 1 migration recorded"
    );

    // Verify initial schema migration is recorded
    let exists: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM _sqlx_migrations WHERE description = 'initial schema')",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert!(
        exists,
        "_sqlx_migrations should contain 'initial schema' migration"
    );
}

#[tokio::test]
async fn test_fts_tables_created_for_search() {
    let pool = setup_migrated_db().await;

    // Verify FTS tables exist for full-text search
    let fts_tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE '%_fts%' ORDER BY name",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(
        fts_tables.iter().any(|t| t.starts_with("projects_fts")),
        "Should have projects_fts tables for search"
    );
    assert!(
        fts_tables.iter().any(|t| t.starts_with("tasks_fts")),
        "Should have tasks_fts tables for search"
    );
}

#[tokio::test]
async fn test_encryption_settings_table_initialized() {
    let pool = setup_migrated_db().await;

    // encryption_settings table should exist but be empty (populated on first use)
    let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM encryption_settings")
        .fetch_one(&pool)
        .await
        .unwrap();

    // Table exists and is queryable (may be empty or have default row)
    assert!(count >= 0, "encryption_settings table should be queryable");
}

#[tokio::test]
async fn test_telemetry_tables_exist() {
    let pool = setup_migrated_db().await;

    // Verify telemetry tables exist
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name LIKE 'telemetry_%' ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(
        tables.contains(&"telemetry_events".to_string()),
        "Should have telemetry_events table"
    );
    assert!(
        tables.contains(&"telemetry_settings".to_string()),
        "Should have telemetry_settings table"
    );
    assert!(
        tables.contains(&"telemetry_stats".to_string()),
        "Should have telemetry_stats table"
    );
}

#[tokio::test]
async fn test_no_orphaned_indexes() {
    let pool = setup_migrated_db().await;

    // Get all indexes
    let indexes: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, tbl_name FROM sqlite_master WHERE type='index' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // Get all tables
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    // Verify every index points to an existing table
    for (index_name, table_name) in indexes {
        assert!(
            tables.contains(&table_name),
            "Index '{}' references non-existent table '{}'",
            index_name,
            table_name
        );
    }
}
