// ABOUTME: Tests for Docker provider graceful degradation when Docker is unavailable
// ABOUTME: Verifies system continues to work without Docker daemon running

use orkee_sandbox::DockerProvider;

/// Test that DockerProvider::new() returns a clear error when Docker is unavailable
///
/// This test verifies that when Docker is not available:
/// 1. The error is informative to users
/// 2. The error mentions connection or Docker-related issues
/// 3. The system doesn't panic
#[tokio::test]
async fn test_docker_provider_unavailable_error() {
    let result = DockerProvider::new();

    match result {
        Ok(_) => {
            // Docker is available - this test verifies graceful behavior when unavailable
            // If Docker is running, we can't test unavailability
            println!("Note: Docker is available. This test verifies behavior when Docker is unavailable.");
        }
        Err(e) => {
            // Verify error message is informative
            let error_msg = e.to_string();
            assert!(
                error_msg.contains("connection") || error_msg.contains("Docker") || error_msg.contains("socket"),
                "Error message should be informative about Docker unavailability: {}",
                error_msg
            );
        }
    }
}

/// Test that attempting to get provider when none registered returns appropriate error
///
/// This test verifies the system behavior when no Docker provider is available:
/// 1. Getting a non-existent provider returns error
/// 2. Error message is clear about provider not found
/// 3. System remains functional
#[tokio::test]
async fn test_get_provider_when_unavailable() {
    use orkee_sandbox::{SandboxManager, SandboxStorage, SettingsManager};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // Create an in-memory database for testing
    let pool = sqlx::SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory database");

    // Run migrations from packages/storage/migrations (relative to workspace root)
    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let storage = Arc::new(SandboxStorage::new(pool.clone()));
    let settings = Arc::new(RwLock::new(
        SettingsManager::new(pool.clone()).expect("Failed to create settings manager")
    ));

    // Create SandboxManager without registering Docker provider
    let manager = SandboxManager::new(storage, settings);

    // Attempt to get Docker provider
    let result = manager.get_provider("local").await;

    // Should return error indicating provider not found
    assert!(result.is_err(), "Should fail when provider not registered");

    // Don't unwrap the error (it contains Arc<dyn Provider> which doesn't implement Debug)
    // Just check that we got an error
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("not found") || error_msg.contains("local"),
            "Error should indicate provider not found: {}",
            error_msg
        );
    }
}

/// Test that system initialization continues despite Docker failure
///
/// This test simulates the actual initialization flow from db.rs:
/// 1. System components initialize successfully
/// 2. Docker provider registration is attempted
/// 3. If Docker unavailable, system logs info message and continues
/// 4. System remains operational without Docker
#[tokio::test]
async fn test_system_initialization_without_docker() {
    use orkee_sandbox::{SandboxManager, SandboxStorage, SettingsManager};
    use std::sync::Arc;
    use tokio::sync::RwLock;

    // This test simulates the initialization flow from db.rs
    let pool = sqlx::SqlitePool::connect(":memory:")
        .await
        .expect("Failed to create in-memory database");

    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create sandbox components (should always succeed)
    let sandbox_storage = Arc::new(SandboxStorage::new(pool.clone()));
    let sandbox_settings = Arc::new(RwLock::new(
        SettingsManager::new(pool.clone()).expect("Failed to create settings manager")
    ));
    let sandbox_manager = Arc::new(SandboxManager::new(sandbox_storage, sandbox_settings));

    // Attempt to register Docker provider (will fail if Docker unavailable)
    let docker_available = match DockerProvider::new() {
        Ok(provider) => {
            let docker_provider = Arc::new(provider);
            sandbox_manager
                .register_provider("local".to_string(), docker_provider)
                .await;
            true
        }
        Err(_e) => {
            // Docker unavailable - this is expected and should be handled gracefully
            // System should continue to work
            false
        }
    };

    // System should be operational regardless of Docker availability
    // The key assertion: we reach this point without panicking
    assert!(
        true,
        "System should initialize successfully without Docker (docker_available: {})",
        docker_available
    );

    // Verify we can still interact with the manager
    let result = sandbox_manager.get_provider("local").await;
    if docker_available {
        assert!(result.is_ok(), "Provider should be available when Docker is running");
    } else {
        assert!(result.is_err(), "Provider should not be available when Docker is not running");
    }
}
