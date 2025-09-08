use crate::constants::{orkee_dir, projects_file};
use crate::types::ProjectsConfig;
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, warn};

use super::StorageResult;

/// Ensures the .orkee directory and projects.json file exist
pub async fn ensure_projects_file() -> StorageResult<()> {
    let orkee_path = orkee_dir();
    let projects_path = projects_file();

    // Create .orkee directory if it doesn't exist
    if !orkee_path.exists() {
        debug!("Creating Orkee directory: {:?}", orkee_path);
        fs::create_dir_all(&orkee_path).await?;
    }

    // Create projects.json if it doesn't exist
    if !projects_path.exists() {
        debug!("Creating projects.json file: {:?}", projects_path);
        let default_config = ProjectsConfig::default();
        let json_content = serde_json::to_string_pretty(&default_config)?;
        fs::write(&projects_path, json_content).await?;
    }

    Ok(())
}

/// Reads the projects configuration from disk
pub async fn read_projects_config() -> StorageResult<ProjectsConfig> {
    ensure_projects_file().await?;

    let projects_path = projects_file();
    debug!("Reading projects config from: {:?}", projects_path);

    match fs::read_to_string(&projects_path).await {
        Ok(content) => match serde_json::from_str::<ProjectsConfig>(&content) {
            Ok(config) => {
                debug!("Successfully loaded {} projects", config.projects.len());
                Ok(config)
            }
            Err(e) => {
                error!("Failed to parse projects.json: {}", e);
                warn!("Using default configuration");
                Ok(ProjectsConfig::default())
            }
        },
        Err(e) => {
            error!("Failed to read projects.json: {}", e);
            warn!("Using default configuration");
            Ok(ProjectsConfig::default())
        }
    }
}

/// Writes the projects configuration to disk
pub async fn write_projects_config(config: &ProjectsConfig) -> StorageResult<()> {
    ensure_projects_file().await?;

    let projects_path = projects_file();
    debug!("Writing projects config to: {:?}", projects_path);

    let json_content = serde_json::to_string_pretty(config)?;
    fs::write(&projects_path, json_content).await?;

    debug!(
        "Successfully wrote {} projects to disk",
        config.projects.len()
    );
    Ok(())
}

/// Checks if a path exists
pub async fn path_exists(path: impl AsRef<Path>) -> bool {
    fs::metadata(path).await.is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_helpers::with_temp_home;

    #[tokio::test]
    async fn test_ensure_projects_file() {
        with_temp_home(|| async {
            let result = ensure_projects_file().await;
            assert!(result.is_ok());

            assert!(orkee_dir().exists());
            assert!(projects_file().exists());
        })
        .await;
    }

    #[tokio::test]
    async fn test_read_write_config() {
        with_temp_home(|| async {
            // Make sure we start with a clean slate
            if projects_file().exists() {
                fs::remove_file(projects_file()).await.ok();
            }
            if orkee_dir().exists() {
                fs::remove_dir_all(orkee_dir()).await.ok();
            }

            let mut config = ProjectsConfig::default();
            config.version = "test-version".to_string();

            let write_result = write_projects_config(&config).await;
            assert!(write_result.is_ok());

            let read_config = read_projects_config().await.unwrap();
            assert_eq!(read_config.version, "test-version");
        })
        .await;
    }

    #[tokio::test]
    async fn test_path_exists() {
        with_temp_home(|| async {
            assert!(!path_exists("/nonexistent/path").await);

            ensure_projects_file().await.unwrap();
            assert!(path_exists(projects_file()).await);
        })
        .await;
    }
}
