#[cfg(test)]
mod protocol_tests;

#[cfg(test)]
mod tool_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
pub mod test_helpers {
    use crate::context::ToolContext;

    /// Create a test context with isolated in-memory storage
    /// Each test gets its own isolated database that doesn't interfere with other tests
    pub async fn create_test_context() -> Result<ToolContext, Box<dyn std::error::Error + Send + Sync>> {
        ToolContext::test_context().await
    }
    
    /// Create a temporary test environment with proper directory structure
    /// Returns the temp directory and sets up HOME environment variable
    pub fn setup_temp_home() -> (tempfile::TempDir, Option<String>) {
        let temp_dir = tempfile::TempDir::new().unwrap();
        let original_home = std::env::var("HOME").ok();
        
        // Create the .orkee subdirectory that the storage system expects
        let orkee_dir = temp_dir.path().join(".orkee");
        std::fs::create_dir_all(&orkee_dir).expect("Failed to create .orkee directory");
        
        std::env::set_var("HOME", temp_dir.path());
        (temp_dir, original_home)
    }
    
    /// Restore the original HOME environment variable
    pub fn restore_home(original_home: Option<String>) {
        if let Some(home) = original_home {
            std::env::set_var("HOME", home);
        } else {
            std::env::remove_var("HOME");
        }
    }
}
