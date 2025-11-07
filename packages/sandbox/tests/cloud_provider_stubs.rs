// ABOUTME: Tests for cloud provider stub implementations
// ABOUTME: Verifies stubs return proper NotSupported errors and mark providers as unavailable

use orkee_sandbox::providers::{BeamProvider, E2BProvider, ModalProvider, Provider};

/// Test that BeamProvider stub returns proper NotSupported errors
#[tokio::test]
async fn test_beam_provider_stub_returns_not_supported() {
    let provider =
        BeamProvider::new("test-key".to_string(), None, None).expect("Failed to create provider");

    // is_available should return Ok(false)
    let available = provider.is_available().await.expect("is_available failed");
    assert!(!available, "BeamProvider stub should not be available");

    // get_info should work and indicate NotAvailable status
    let info = provider.get_info().await.expect("get_info failed");
    assert_eq!(info.name, "Beam");
    assert_eq!(info.version, "stub");
    assert!(
        matches!(
            info.status,
            orkee_sandbox::providers::ProviderStatus::NotAvailable(_)
        ),
        "BeamProvider should report NotAvailable status"
    );

    // All operational methods should return NotSupported error
    let config = orkee_sandbox::providers::ContainerConfig {
        name: "test".to_string(),
        image: "ubuntu:22.04".to_string(),
        command: Some(vec!["bash".to_string()]),
        working_dir: None,
        env_vars: Default::default(),
        volumes: vec![],
        ports: vec![],
        cpu_cores: 1.0,
        memory_mb: 512,
        storage_gb: 1,
        labels: Default::default(),
    };

    let result = provider.create_container(&config).await;
    assert!(result.is_err(), "create_container should fail");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::NotSupported(_)),
            "Should return NotSupported error"
        );
        assert!(
            e.to_string().contains("not yet implemented"),
            "Error message should be clear: {}",
            e
        );
    }

    // Test start_container
    let result = provider.start_container("test-id").await;
    assert!(result.is_err(), "start_container should fail");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::NotSupported(_)),
            "Should return NotSupported error"
        );
    }

    // Test stop_container
    let result = provider.stop_container("test-id", 10).await;
    assert!(result.is_err(), "stop_container should fail");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::NotSupported(_)),
            "Should return NotSupported error"
        );
    }

    // Test remove_container
    let result = provider.remove_container("test-id", false).await;
    assert!(result.is_err(), "remove_container should fail");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::NotSupported(_)),
            "Should return NotSupported error"
        );
    }
}

/// Test that E2BProvider stub returns proper NotSupported errors
#[tokio::test]
async fn test_e2b_provider_stub_returns_not_supported() {
    let provider =
        E2BProvider::new("test-key".to_string(), None).expect("Failed to create provider");

    // is_available should return Ok(false)
    let available = provider.is_available().await.expect("is_available failed");
    assert!(!available, "E2BProvider stub should not be available");

    // get_info should work and indicate NotAvailable status
    let info = provider.get_info().await.expect("get_info failed");
    assert_eq!(info.name, "E2B");
    assert_eq!(info.version, "stub");
    assert!(
        matches!(
            info.status,
            orkee_sandbox::providers::ProviderStatus::NotAvailable(_)
        ),
        "E2BProvider should report NotAvailable status"
    );

    // Test that operational methods return NotSupported
    let config = orkee_sandbox::providers::ContainerConfig {
        name: "test".to_string(),
        image: "ubuntu:22.04".to_string(),
        command: Some(vec!["bash".to_string()]),
        working_dir: None,
        env_vars: Default::default(),
        volumes: vec![],
        ports: vec![],
        cpu_cores: 1.0,
        memory_mb: 512,
        storage_gb: 1,
        labels: Default::default(),
    };

    let result = provider.create_container(&config).await;
    assert!(result.is_err(), "create_container should fail");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::NotSupported(_)),
            "Should return NotSupported error"
        );
        assert!(
            e.to_string().contains("not yet implemented"),
            "Error message should be clear: {}",
            e
        );
    }
}

/// Test that ModalProvider stub returns proper NotSupported errors
#[tokio::test]
async fn test_modal_provider_stub_returns_not_supported() {
    let provider = ModalProvider::new("test-id".to_string(), "test-secret".to_string(), None)
        .expect("Failed to create provider");

    // is_available should return Ok(false)
    let available = provider.is_available().await.expect("is_available failed");
    assert!(!available, "ModalProvider stub should not be available");

    // get_info should work and indicate NotAvailable status
    let info = provider.get_info().await.expect("get_info failed");
    assert_eq!(info.name, "Modal");
    assert_eq!(info.version, "stub");
    assert!(
        matches!(
            info.status,
            orkee_sandbox::providers::ProviderStatus::NotAvailable(_)
        ),
        "ModalProvider should report NotAvailable status"
    );

    // Test that operational methods return NotSupported
    let config = orkee_sandbox::providers::ContainerConfig {
        name: "test".to_string(),
        image: "ubuntu:22.04".to_string(),
        command: Some(vec!["bash".to_string()]),
        working_dir: None,
        env_vars: Default::default(),
        volumes: vec![],
        ports: vec![],
        cpu_cores: 1.0,
        memory_mb: 512,
        storage_gb: 1,
        labels: Default::default(),
    };

    let result = provider.create_container(&config).await;
    assert!(result.is_err(), "create_container should fail");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::NotSupported(_)),
            "Should return NotSupported error"
        );
        assert!(
            e.to_string().contains("not yet implemented"),
            "Error message should be clear: {}",
            e
        );
    }
}

/// Test that providers require valid credentials
#[tokio::test]
async fn test_providers_require_credentials() {
    // BeamProvider requires API key
    let result = BeamProvider::new(String::new(), None, None);
    assert!(result.is_err(), "BeamProvider should require API key");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::ConfigError(_)),
            "Should return ConfigError"
        );
        assert!(
            e.to_string().contains("required"),
            "Error should mention required credential: {}",
            e
        );
    }

    // E2BProvider requires API key
    let result = E2BProvider::new(String::new(), None);
    assert!(result.is_err(), "E2BProvider should require API key");
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::ConfigError(_)),
            "Should return ConfigError"
        );
    }

    // ModalProvider requires both token ID and secret
    let result = ModalProvider::new(String::new(), String::new(), None);
    assert!(
        result.is_err(),
        "ModalProvider should require token ID and secret"
    );
    if let Err(e) = result {
        assert!(
            matches!(e, orkee_sandbox::providers::ProviderError::ConfigError(_)),
            "Should return ConfigError"
        );
    }
}
