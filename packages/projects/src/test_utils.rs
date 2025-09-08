//! Test utilities for ensuring thread-safe test execution

#[cfg(test)]
pub mod test_helpers {
    use std::env;
    use std::sync::Mutex;
    use tempfile::TempDir;

    /// Global mutex to ensure thread-safe access to HOME environment variable across all tests
    static HOME_MUTEX: Mutex<()> = Mutex::new(());

    /// Run a test with a temporary HOME directory, ensuring thread-safe execution
    pub async fn with_temp_home<F, Fut>(test: F)
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let _guard = HOME_MUTEX
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let temp_dir = TempDir::new().unwrap();
        let original_home = env::var("HOME").ok();

        env::set_var("HOME", temp_dir.path());

        test().await;

        if let Some(home) = original_home {
            env::set_var("HOME", home);
        } else {
            env::remove_var("HOME");
        }
    }
}
