// ABOUTME: Integration tests for preview server storage initialization and migration
// ABOUTME: Tests database schema setup and JSON-to-SQLite migration flows

use chrono::Utc;
use orkee_preview::storage::{PreviewServerEntry, PreviewServerStorage};
use orkee_preview::types::{DevServerStatus, ServerSource};
use orkee_storage::{sqlite::SqliteStorage, ProjectStorage, StorageConfig, StorageProvider};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use uuid::Uuid;

/// Helper to set up a test database with projects table
async fn setup_test_storage() -> (SqliteStorage, TempDir) {
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("test.db");

    let config = StorageConfig {
        provider: StorageProvider::Sqlite {
            path: db_path.clone(),
        },
        max_connections: 5,
        busy_timeout_seconds: 30,
        enable_wal: false, // WAL doesn't work well with temporary files
        enable_fts: true,
    };

    let storage = SqliteStorage::new(config).await.unwrap();
    storage.initialize().await.unwrap();

    (storage, temp_dir)
}

/// Helper to insert a test project
async fn insert_test_project(storage: &SqliteStorage, project_id: &str, project_name: &str) {
    sqlx::query(
        "INSERT INTO projects (id, name, project_root, created_at, updated_at) VALUES (?, ?, ?, ?, ?)"
    )
    .bind(project_id)
    .bind(project_name)
    .bind(format!("/home/test/{}", project_id))
    .bind(Utc::now().to_rfc3339())
    .bind(Utc::now().to_rfc3339())
    .execute(storage.pool())
    .await
    .unwrap();
}

#[tokio::test]
async fn test_preview_storage_initialization() {
    // Set up storage and initialize
    let (storage, _temp_dir) = setup_test_storage().await;

    // Verify preview_servers table exists and has correct schema
    let table_exists: bool = sqlx::query_scalar(
        "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='preview_servers'",
    )
    .fetch_one(storage.pool())
    .await
    .unwrap();

    assert!(
        table_exists,
        "preview_servers table should exist after initialization"
    );

    // Verify we can create a PreviewServerStorage instance
    let preview_storage = PreviewServerStorage::new(&storage).await;
    assert!(
        preview_storage.is_ok(),
        "Should be able to create PreviewServerStorage"
    );
}

#[tokio::test]
async fn test_basic_crud_operations() {
    let (storage, _temp_dir) = setup_test_storage().await;
    insert_test_project(&storage, "test-project", "Test Project").await;

    let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

    // Create a test entry
    let entry = PreviewServerEntry {
        id: Uuid::new_v4().to_string(),
        project_id: "test-project".to_string(),
        project_name: Some("Test Project".to_string()),
        port: 3000,
        preview_url: Some("http://localhost:3000".to_string()),
        pid: Some(12345),
        status: DevServerStatus::Running,
        source: ServerSource::Orkee,
        project_root: PathBuf::from("/home/test/project"),
        matched_project_id: None,
        framework_name: Some("Next.js".to_string()),
        actual_command: Some("npm run dev".to_string()),
        started_at: Utc::now(),
        last_seen: Utc::now(),
        api_port: 4001,
    };

    // Test INSERT
    preview_storage
        .insert(&entry)
        .await
        .expect("Should insert server entry");

    // Test GET
    let retrieved = preview_storage
        .get(&entry.id)
        .await
        .expect("Should retrieve server")
        .expect("Server should exist");
    assert_eq!(retrieved.project_id, "test-project");
    assert_eq!(retrieved.port, 3000);

    // Test GET ALL
    let all_servers = preview_storage
        .get_all()
        .await
        .expect("Should get all servers");
    assert_eq!(all_servers.len(), 1);

    // Test GET BY PROJECT
    let project_servers = preview_storage
        .get_by_project("test-project")
        .await
        .expect("Should get servers by project");
    assert_eq!(project_servers.len(), 1);

    // Test GET BY PORT
    let port_server = preview_storage
        .get_by_port(3000)
        .await
        .expect("Should get server by port")
        .expect("Server should exist on port 3000");
    assert_eq!(port_server.id, entry.id);

    // Test DELETE
    preview_storage
        .delete(&entry.id)
        .await
        .expect("Should delete server");

    let deleted = preview_storage
        .get(&entry.id)
        .await
        .expect("Query should succeed");
    assert!(deleted.is_none(), "Server should be deleted");
}

#[tokio::test]
async fn test_foreign_key_constraint() {
    let (storage, _temp_dir) = setup_test_storage().await;
    let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

    // Try to insert a server with non-existent project_id
    let entry = PreviewServerEntry {
        id: Uuid::new_v4().to_string(),
        project_id: "non-existent-project".to_string(),
        project_name: Some("Test Project".to_string()),
        port: 3000,
        preview_url: Some("http://localhost:3000".to_string()),
        pid: Some(12345),
        status: DevServerStatus::Running,
        source: ServerSource::Orkee,
        project_root: PathBuf::from("/home/test/project"),
        matched_project_id: None,
        framework_name: Some("Next.js".to_string()),
        actual_command: Some("npm run dev".to_string()),
        started_at: Utc::now(),
        last_seen: Utc::now(),
        api_port: 4001,
    };

    let result = preview_storage.insert(&entry).await;
    assert!(
        result.is_err(),
        "Should fail to insert with non-existent project_id"
    );

    // Verify error message mentions FK constraint
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("non-existent-project") && error_msg.contains("may not exist"),
        "Error should mention the missing project_id. Got: {}",
        error_msg
    );
}

#[tokio::test]
async fn test_migration_from_json_success() {
    let (storage, temp_dir) = setup_test_storage().await;
    insert_test_project(&storage, "migrated-project", "Migrated Project").await;

    let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

    // Create a JSON file with test data
    let json_path = temp_dir.path().join("server-registry.json");
    let json_content = serde_json::json!({
        "server-1": {
            "id": "server-1",
            "project_id": "migrated-project",
            "project_name": "Migrated Project",
            "port": 3000,
            "preview_url": "http://localhost:3000",
            "pid": 12345,
            "status": "running",
            "source": "orkee",
            "project_root": "/home/test/migrated",
            "matched_project_id": null,
            "framework_name": "Next.js",
            "actual_command": "npm run dev",
            "started_at": Utc::now().to_rfc3339(),
            "last_seen": Utc::now().to_rfc3339(),
            "api_port": 4001
        }
    });
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&json_content).unwrap(),
    )
    .unwrap();

    // Run migration
    let result = preview_storage.migrate_from_json(&json_path).await;
    assert!(
        result.is_ok(),
        "Migration should succeed: {:?}",
        result.err()
    );

    // Verify data was migrated
    let migrated = preview_storage
        .get("server-1")
        .await
        .expect("Should query migrated server")
        .expect("Server should exist after migration");
    assert_eq!(migrated.project_id, "migrated-project");
    assert_eq!(migrated.port, 3000);

    // Verify JSON file was renamed
    assert!(!json_path.exists(), "Original JSON should be renamed");
    let backup_path = json_path.with_extension("json.migrated");
    assert!(backup_path.exists(), "Backup file should exist");
}

#[tokio::test]
async fn test_migration_from_json_partial_failure() {
    let (storage, temp_dir) = setup_test_storage().await;
    insert_test_project(&storage, "valid-project", "Valid Project").await;

    let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

    // Create a JSON file with both valid and invalid entries
    let json_path = temp_dir.path().join("server-registry.json");
    let json_content = serde_json::json!({
        "server-1": {
            "id": "server-1",
            "project_id": "valid-project",
            "project_name": "Valid Project",
            "port": 3000,
            "preview_url": "http://localhost:3000",
            "pid": 12345,
            "status": "running",
            "source": "orkee",
            "project_root": "/home/test/valid",
            "matched_project_id": null,
            "framework_name": "Next.js",
            "actual_command": "npm run dev",
            "started_at": Utc::now().to_rfc3339(),
            "last_seen": Utc::now().to_rfc3339(),
            "api_port": 4001
        },
        "server-2": {
            "id": "server-2",
            "project_id": "invalid-project",  // This project doesn't exist
            "project_name": "Invalid Project",
            "port": 3001,
            "preview_url": "http://localhost:3001",
            "pid": 12346,
            "status": "running",
            "source": "orkee",
            "project_root": "/home/test/invalid",
            "matched_project_id": null,
            "framework_name": "Vite",
            "actual_command": "npm run dev",
            "started_at": Utc::now().to_rfc3339(),
            "last_seen": Utc::now().to_rfc3339(),
            "api_port": 4001
        }
    });
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&json_content).unwrap(),
    )
    .unwrap();

    // Run migration - should fail
    let result = preview_storage.migrate_from_json(&json_path).await;
    assert!(
        result.is_err(),
        "Migration should fail when some entries are invalid"
    );

    // Verify error message indicates partial failure
    let error_msg = result.unwrap_err().to_string();
    assert!(
        error_msg.contains("partially failed") && error_msg.contains("1/2"),
        "Error should indicate 1 out of 2 migrated. Got: {}",
        error_msg
    );

    // Verify valid entry was still migrated
    let migrated = preview_storage.get("server-1").await.unwrap();
    assert!(
        migrated.is_some(),
        "Valid entry should be migrated even on partial failure"
    );

    // Verify invalid entry was not migrated
    let not_migrated = preview_storage.get("server-2").await.unwrap();
    assert!(
        not_migrated.is_none(),
        "Invalid entry should not be migrated"
    );

    // Verify JSON file was NOT renamed (preserved for recovery)
    assert!(
        json_path.exists(),
        "Original JSON should be preserved on partial failure"
    );
    let backup_path = json_path.with_extension("json.migrated");
    assert!(
        !backup_path.exists(),
        "Backup file should not exist on partial failure"
    );
}

#[tokio::test]
async fn test_migration_no_json_file() {
    let (storage, temp_dir) = setup_test_storage().await;
    let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

    let json_path = temp_dir.path().join("nonexistent.json");

    // Migration should succeed silently when no JSON file exists
    let result = preview_storage.migrate_from_json(&json_path).await;
    assert!(
        result.is_ok(),
        "Migration should succeed when JSON doesn't exist"
    );
}

#[tokio::test]
async fn test_stale_server_cleanup() {
    let (storage, _temp_dir) = setup_test_storage().await;
    insert_test_project(&storage, "test-project", "Test Project").await;

    let preview_storage = PreviewServerStorage::new(&storage).await.unwrap();

    // Insert two servers with different last_seen timestamps
    let old_time = Utc::now() - chrono::Duration::hours(2);
    let recent_time = Utc::now();

    let old_server = PreviewServerEntry {
        id: "old-server".to_string(),
        project_id: "test-project".to_string(),
        project_name: Some("Test Project".to_string()),
        port: 3000,
        preview_url: Some("http://localhost:3000".to_string()),
        pid: Some(12345),
        status: DevServerStatus::Stopped,
        source: ServerSource::Orkee,
        project_root: PathBuf::from("/home/test/project"),
        matched_project_id: None,
        framework_name: Some("Next.js".to_string()),
        actual_command: Some("npm run dev".to_string()),
        started_at: old_time,
        last_seen: old_time,
        api_port: 4001,
    };

    let recent_server = PreviewServerEntry {
        id: "recent-server".to_string(),
        project_id: "test-project".to_string(),
        project_name: Some("Test Project".to_string()),
        port: 3001,
        preview_url: Some("http://localhost:3001".to_string()),
        pid: Some(12346),
        status: DevServerStatus::Running,
        source: ServerSource::Orkee,
        project_root: PathBuf::from("/home/test/project"),
        matched_project_id: None,
        framework_name: Some("Vite".to_string()),
        actual_command: Some("bun run dev".to_string()),
        started_at: recent_time,
        last_seen: recent_time,
        api_port: 4001,
    };

    preview_storage.insert(&old_server).await.unwrap();
    preview_storage.insert(&recent_server).await.unwrap();

    // Delete servers older than 1 hour
    let cutoff = Utc::now() - chrono::Duration::hours(1);
    let deleted = preview_storage
        .delete_stale(cutoff)
        .await
        .expect("Should delete stale servers");

    assert_eq!(deleted, 1, "Should delete exactly 1 stale server");

    // Verify old server is gone
    let old = preview_storage.get("old-server").await.unwrap();
    assert!(old.is_none(), "Old server should be deleted");

    // Verify recent server is still there
    let recent = preview_storage.get("recent-server").await.unwrap();
    assert!(recent.is_some(), "Recent server should still exist");
}
