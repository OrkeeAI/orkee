#[cfg(test)]
mod protocol_tests;

#[cfg(test)]
mod tool_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
pub mod test_helpers {
    use orkee_projects::{initialize_storage, get_storage_manager};
    use std::sync::{Once, Mutex};
    use std::env;

    static INIT: Once = Once::new();
    static TEST_LOCK: Mutex<()> = Mutex::new(());

    /// Initialize test storage and clean up any existing data
    /// This should be called at the beginning of each test that needs storage
    pub async fn setup_test_storage() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Use a global test lock to ensure tests run sequentially to avoid database conflicts
        let _guard = TEST_LOCK.lock().unwrap();
        
        INIT.call_once(|| {
            // Set up a unique test database location to avoid conflicts
            let test_dir = env::temp_dir().join(format!("orkee_test_{}", std::process::id()));
            std::fs::create_dir_all(&test_dir).ok();
            env::set_var("HOME", test_dir);
        });
        
        // Always try to initialize storage - it will only actually initialize once
        match initialize_storage().await {
            Ok(_) => {},
            Err(e) => {
                // If already initialized, that's fine
                if !e.to_string().contains("already initialized") {
                    return Err(Box::new(e));
                }
            }
        }

        // Clean up any existing data to isolate tests
        if let Ok(storage_manager) = get_storage_manager().await {
            let storage = storage_manager.storage();
            
            // Get all projects and delete them to clean the state
            if let Ok(projects) = storage.list_projects().await {
                for project in projects {
                    let _ = storage.delete_project(&project.id).await;
                }
            }
        }

        Ok(())
    }
}