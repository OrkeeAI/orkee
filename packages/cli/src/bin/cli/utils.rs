// ABOUTME: CLI utility functions for project context detection
// ABOUTME: Helps automatically detect project from current working directory

use sqlx::SqlitePool;
use std::env;
use std::path::Path;

/// Maximum number of parent directories to search for project markers
const MAX_PARENT_SEARCH_DEPTH: usize = 10;

/// Detect project ID from current working directory
///
/// Searches up the directory tree for common project markers like:
/// - .git directory
/// - package.json
/// - Cargo.toml
/// - pyproject.toml
///
/// Then queries the database for a project with a matching project_root path.
pub async fn detect_project_from_cwd() -> Option<String> {
    let current_dir = env::current_dir().ok()?;

    // Try to find project markers
    let markers = vec![
        ".git",
        "package.json",
        "Cargo.toml",
        "pyproject.toml",
        "go.mod",
    ];

    let mut search_dir = current_dir.clone();
    for _ in 0..MAX_PARENT_SEARCH_DEPTH {
        for marker in &markers {
            if search_dir.join(marker).exists() {
                // Query database for project with this path
                if let Some(project_id) = find_project_by_path(&search_dir).await {
                    return Some(project_id);
                }
            }
        }

        // Move to parent directory
        if !search_dir.pop() {
            break;
        }
    }

    None
}

/// Find project ID by matching project_root path in database
async fn find_project_by_path(path: &Path) -> Option<String> {
    let path_str = path.to_string_lossy().to_string();

    // Get database pool
    let pool = get_database_pool().await.ok()?;

    // Query for project with matching project_root
    let result: Option<(String,)> =
        sqlx::query_as("SELECT id FROM projects WHERE project_root = ? LIMIT 1")
            .bind(&path_str)
            .fetch_optional(&pool)
            .await
            .ok()?;

    result.map(|(id,)| id)
}

/// Get the shared database connection pool
async fn get_database_pool() -> Result<SqlitePool, Box<dyn std::error::Error>> {
    use sqlx::sqlite::SqlitePoolOptions;
    use std::time::Duration;

    let home = dirs::home_dir().ok_or("Failed to find home directory")?;
    let db_path = home.join(".orkee").join("orkee.db");

    // Ensure parent directory exists
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database_url = format!("sqlite://{}?mode=rwc", db_path.display());

    // Configure connection pool
    let pool = SqlitePoolOptions::new()
        .max_connections(3)
        .acquire_timeout(Duration::from_secs(10))
        .connect(&database_url)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_depth_constant() {
        assert!(MAX_PARENT_SEARCH_DEPTH > 0);
        assert!(MAX_PARENT_SEARCH_DEPTH <= 20); // Reasonable limit
    }
}
