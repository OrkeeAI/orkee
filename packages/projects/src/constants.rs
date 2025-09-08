use std::env;
use std::path::PathBuf;

/// Current version of the projects configuration format
pub const PROJECTS_VERSION: &str = "1.0.0";

/// Get the path to the Orkee directory (~/.orkee)
pub fn orkee_dir() -> PathBuf {
    // First try HOME environment variable (useful for tests)
    if let Ok(home) = env::var("HOME") {
        PathBuf::from(home).join(".orkee")
    } else {
        // Fall back to dirs crate for normal usage
        dirs::home_dir()
            .expect("Unable to get home directory")
            .join(".orkee")
    }
}

/// Get the path to the projects.json file (~/.orkee/projects.json)
pub fn projects_file() -> PathBuf {
    orkee_dir().join("projects.json")
}
