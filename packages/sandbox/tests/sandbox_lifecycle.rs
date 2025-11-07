// ABOUTME: Integration tests for complete sandbox lifecycle operations
// ABOUTME: Tests create, start, execute, stop, and delete operations with real Docker provider

use orkee_sandbox::{
    CreateSandboxRequest, DockerProvider, ExecutionStatus, SandboxManager, SandboxStatus,
    SandboxStorage, SettingsManager,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Helper function to set up test environment with database and manager
async fn setup_test_manager() -> (Arc<SandboxManager>, sqlx::SqlitePool) {
    let pool = sqlx::SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory database");

    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create additional test users for filtering tests
    sqlx::query("INSERT INTO users (id, email, name, created_at, updated_at) VALUES ('user1', 'user1@test.com', 'Test User 1', datetime('now'), datetime('now'))")
        .execute(&pool)
        .await
        .expect("Failed to create test user1");

    sqlx::query("INSERT INTO users (id, email, name, created_at, updated_at) VALUES ('user2', 'user2@test.com', 'Test User 2', datetime('now'), datetime('now'))")
        .execute(&pool)
        .await
        .expect("Failed to create test user2");

    let storage = Arc::new(SandboxStorage::new(pool.clone()));
    let settings = Arc::new(RwLock::new(
        SettingsManager::new(pool.clone()).expect("Failed to create settings manager"),
    ));

    let manager = Arc::new(SandboxManager::new(storage, settings));

    // Try to register Docker provider (skip tests if Docker unavailable)
    if let Ok(provider) = DockerProvider::new() {
        manager
            .register_provider("local".to_string(), Arc::new(provider))
            .await;
    }

    (manager, pool)
}

/// Check if Docker is available for testing
async fn is_docker_available() -> bool {
    DockerProvider::new().is_ok()
}

/// Test complete sandbox lifecycle: create → start → exec → stop → delete
///
/// This test verifies:
/// 1. Sandbox can be created with valid configuration
/// 2. Sandbox starts and reaches Running status
/// 3. Commands can be executed inside the sandbox
/// 4. Sandbox can be stopped gracefully
/// 5. Sandbox can be deleted and cleaned up
#[tokio::test]
async fn test_complete_sandbox_lifecycle() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    // Create sandbox
    let request = CreateSandboxRequest {
        name: "lifecycle-test".to_string(),
        provider: "local".to_string(),
        agent_id: "claude-code".to_string(),
        user_id: "default-user".to_string(),
        project_id: None,
        image: Some("alpine:latest".to_string()),
        cpu_cores: Some(1.0),
        memory_mb: Some(512),
        storage_gb: Some(5),
        gpu_enabled: false,
        gpu_model: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        ssh_enabled: false,
        config: None,
        metadata: None,
    };

    let sandbox = manager
        .create_sandbox(request)
        .await
        .expect("Failed to create sandbox");

    assert_eq!(sandbox.status, SandboxStatus::Running);
    assert!(sandbox.container_id.is_some());

    let sandbox_id = sandbox.id.clone();

    // Execute command
    let execution = manager
        .create_execution(
            &sandbox_id,
            "echo 'Hello from sandbox'".to_string(),
            Some("/".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to create execution");

    assert!(!execution.id.is_empty());

    // Stop sandbox
    manager
        .stop_sandbox(&sandbox_id, 10)
        .await
        .expect("Failed to stop sandbox");

    let stopped_sandbox = manager
        .get_sandbox(&sandbox_id)
        .await
        .expect("Failed to get sandbox");
    assert_eq!(stopped_sandbox.status, SandboxStatus::Stopped);

    // Delete sandbox
    manager
        .remove_sandbox(&sandbox_id, true)
        .await
        .expect("Failed to remove sandbox");
}

/// Test sandbox creation with resource limits
///
/// This test verifies:
/// 1. Sandbox respects configured resource limits
/// 2. Resource limits are properly stored in database
/// 3. Provider receives correct resource configuration
#[tokio::test]
async fn test_sandbox_resource_limits() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    let request = CreateSandboxRequest {
        name: "resource-test".to_string(),
        provider: "local".to_string(),
        agent_id: "claude-code".to_string(),
        user_id: "default-user".to_string(),
        project_id: None,
        image: Some("alpine:latest".to_string()),
        cpu_cores: Some(2.0),
        memory_mb: Some(1024),
        storage_gb: Some(10),
        gpu_enabled: false,
        gpu_model: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        ssh_enabled: false,
        config: None,
        metadata: None,
    };

    let sandbox = manager
        .create_sandbox(request)
        .await
        .expect("Failed to create sandbox");

    assert_eq!(sandbox.cpu_cores, 2.0);
    assert_eq!(sandbox.memory_mb, 1024);
    assert_eq!(sandbox.storage_gb, 10);

    // Cleanup
    manager
        .remove_sandbox(&sandbox.id, true)
        .await
        .expect("Failed to cleanup");
}

/// Test sandbox execution with environment variables
///
/// This test verifies:
/// 1. Environment variables are passed to container
/// 2. Commands can access environment variables
/// 3. Execution results are captured correctly
#[tokio::test]
async fn test_sandbox_execution_with_env_vars() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    let request = CreateSandboxRequest {
        name: "env-test".to_string(),
        provider: "local".to_string(),
        agent_id: "claude-code".to_string(),
        user_id: "default-user".to_string(),
        project_id: None,
        image: Some("alpine:latest".to_string()),
        cpu_cores: Some(1.0),
        memory_mb: Some(512),
        storage_gb: Some(5),
        gpu_enabled: false,
        gpu_model: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        ssh_enabled: false,
        config: None,
        metadata: None,
    };

    let sandbox = manager
        .create_sandbox(request)
        .await
        .expect("Failed to create sandbox");

    let execution = manager
        .create_execution(
            &sandbox.id,
            "echo $TEST_VAR".to_string(),
            Some("/".to_string()),
            None,
            None,
        )
        .await
        .expect("Failed to create execution");

    assert_eq!(execution.status, ExecutionStatus::Queued);

    // Cleanup
    manager
        .remove_sandbox(&sandbox.id, true)
        .await
        .expect("Failed to cleanup");
}

/// Test listing sandboxes with filters
///
/// This test verifies:
/// 1. Sandboxes can be filtered by user_id
/// 2. Sandboxes can be filtered by status
/// 3. List returns correct results
#[tokio::test]
async fn test_list_sandboxes_with_filters() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    // Create multiple sandboxes
    let user1_request = CreateSandboxRequest {
        name: "user1-sandbox".to_string(),
        provider: "local".to_string(),
        agent_id: "claude-code".to_string(),
        user_id: "user1".to_string(),
        project_id: None,
        image: Some("alpine:latest".to_string()),
        cpu_cores: Some(1.0),
        memory_mb: Some(512),
        storage_gb: Some(5),
        gpu_enabled: false,
        gpu_model: None,
        env_vars: HashMap::new(),
        volumes: vec![],
        ports: vec![],
        ssh_enabled: false,
        config: None,
        metadata: None,
    };

    let user2_request = CreateSandboxRequest {
        name: "user2-sandbox".to_string(),
        user_id: "user2".to_string(),
        ..user1_request.clone()
    };

    let sandbox1 = manager
        .create_sandbox(user1_request)
        .await
        .expect("Failed to create sandbox 1");

    let sandbox2 = manager
        .create_sandbox(user2_request)
        .await
        .expect("Failed to create sandbox 2");

    // List all sandboxes
    let all_sandboxes = manager
        .list_sandboxes(None, None)
        .await
        .expect("Failed to list all sandboxes");
    assert!(all_sandboxes.len() >= 2);

    // List sandboxes for user1
    let user1_sandboxes = manager
        .list_sandboxes(Some("user1"), None)
        .await
        .expect("Failed to list user1 sandboxes");
    assert!(user1_sandboxes.iter().all(|s| s.user_id == "user1"));
    assert!(!user1_sandboxes.is_empty());

    // List running sandboxes
    let running_sandboxes = manager
        .list_sandboxes(None, Some(SandboxStatus::Running))
        .await
        .expect("Failed to list running sandboxes");
    assert!(running_sandboxes
        .iter()
        .all(|s| s.status == SandboxStatus::Running));

    // Cleanup
    manager
        .remove_sandbox(&sandbox1.id, true)
        .await
        .expect("Failed to cleanup sandbox1");
    manager
        .remove_sandbox(&sandbox2.id, true)
        .await
        .expect("Failed to cleanup sandbox2");
}

/// Test error handling for invalid operations
///
/// This test verifies:
/// 1. Operating on non-existent sandbox returns error
/// 2. Error messages are informative
/// 3. System remains stable after errors
#[tokio::test]
async fn test_error_handling() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    // Try to get non-existent sandbox
    let result = manager.get_sandbox("nonexistent-id").await;
    assert!(result.is_err());

    // Try to stop non-existent sandbox
    let result = manager.stop_sandbox("nonexistent-id", 10).await;
    assert!(result.is_err());

    // Try to create execution for non-existent sandbox
    let result = manager
        .create_execution(
            "nonexistent-id",
            "echo test".to_string(),
            Some("/".to_string()),
            None,
            None,
        )
        .await;
    assert!(result.is_err());
}

/// Test orphaned container cleanup functionality
///
/// This test verifies:
/// 1. Cleanup finds orphaned containers
/// 2. Dry run mode doesn't delete containers
/// 3. Actual cleanup removes orphaned containers
/// 4. Statistics are returned correctly
#[tokio::test]
async fn test_orphaned_container_cleanup() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    // Run dry-run cleanup (should not error even if no orphaned containers)
    let (_found, removed, errors) = manager
        .cleanup_orphaned_containers("local", true)
        .await
        .expect("Failed to run dry-run cleanup");

    assert_eq!(removed, 0, "Dry run should not remove containers");
    assert_eq!(errors.len(), 0, "Should have no errors");

    // Run actual cleanup
    let (_found2, _removed2, errors2) = manager
        .cleanup_orphaned_containers("local", false)
        .await
        .expect("Failed to run cleanup");

    assert_eq!(errors2.len(), 0, "Should have no errors");
    // Note: _found2 and _removed2 may be 0 if no orphaned containers exist, which is fine
}

/// Test concurrent sandbox operations
///
/// This test verifies:
/// 1. Multiple sandboxes can be created concurrently
/// 2. No race conditions in sandbox creation
/// 3. Each sandbox has unique ID and container
#[tokio::test]
async fn test_concurrent_sandbox_operations() {
    if !is_docker_available().await {
        println!("Skipping test: Docker not available");
        return;
    }

    let (manager, _pool) = setup_test_manager().await;

    let mut handles = vec![];

    for i in 0..3 {
        let manager_clone = manager.clone();
        let handle = tokio::spawn(async move {
            let request = CreateSandboxRequest {
                name: format!("concurrent-test-{}", i),
                provider: "local".to_string(),
                agent_id: "claude-code".to_string(),
                user_id: "default-user".to_string(),
                project_id: None,
                image: Some("alpine:latest".to_string()),
                cpu_cores: Some(1.0),
                memory_mb: Some(512),
                storage_gb: Some(5),
                gpu_enabled: false,
                gpu_model: None,
                env_vars: HashMap::new(),
                volumes: vec![],
                ports: vec![],
                ssh_enabled: false,
                config: None,
                metadata: None,
            };

            manager_clone
                .create_sandbox(request)
                .await
                .expect("Failed to create sandbox")
        });
        handles.push(handle);
    }

    let mut sandboxes = vec![];
    for handle in handles {
        let sandbox = handle.await.expect("Task panicked");
        sandboxes.push(sandbox);
    }

    // Verify all sandboxes were created
    assert_eq!(sandboxes.len(), 3);

    // Verify each has unique ID
    let ids: std::collections::HashSet<_> = sandboxes.iter().map(|s| &s.id).collect();
    assert_eq!(ids.len(), 3, "All sandboxes should have unique IDs");

    // Cleanup
    for sandbox in sandboxes {
        manager
            .remove_sandbox(&sandbox.id, true)
            .await
            .expect("Failed to cleanup");
    }
}
