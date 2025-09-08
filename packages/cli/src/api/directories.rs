use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson, Extension};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, sync::Arc};
use tracing::{debug, info, warn};

use crate::api::path_validator::{PathValidator, ValidationError};

#[derive(Deserialize)]
pub struct BrowseDirectoriesRequest {
    pub path: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct DirectoryItem {
    pub name: String,
    pub path: String,
    #[serde(rename = "isDirectory")]
    pub is_directory: bool,
}

#[derive(Serialize)]
pub struct DirectoryData {
    #[serde(rename = "currentPath")]
    pub current_path: String,
    #[serde(rename = "parentPath")]
    pub parent_path: Option<String>,
    pub directories: Vec<DirectoryItem>,
    #[serde(rename = "isRoot")]
    pub is_root: bool,
    pub separator: String,
}

#[derive(Serialize)]
pub struct BrowseDirectoriesResponse {
    pub success: bool,
    pub data: Option<DirectoryData>,
    pub error: Option<String>,
}

pub async fn browse_directories(
    Extension(validator): Extension<Arc<PathValidator>>,
    Json(request): Json<BrowseDirectoriesRequest>,
) -> Result<ResponseJson<BrowseDirectoriesResponse>, StatusCode> {
    // Get target path or use safe default
    let target_path = match &request.path {
        Some(p) if !p.is_empty() => p.clone(),
        _ => validator.get_safe_default_path(),
    };

    debug!("Browse request for path: {}", target_path);

    // Validate the path through sandbox
    let validated_path = match validator.validate_path(&target_path) {
        Ok(path) => path,
        Err(ValidationError::BlockedPath(blocked)) => {
            warn!("Blocked access to path: {}", blocked);
            log_directory_access(None, &target_path, None, false, Some("blocked_path"), None);
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Access denied: Path is restricted".to_string()),
            }));
        }
        Err(ValidationError::PathTraversal) => {
            warn!("Path traversal attempt detected: {}", target_path);
            log_directory_access(
                None,
                &target_path,
                None,
                false,
                Some("path_traversal"),
                None,
            );
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Path traversal detected and blocked".to_string()),
            }));
        }
        Err(ValidationError::NotInAllowedPaths) => {
            warn!("Access to non-allowed path: {}", target_path);
            log_directory_access(
                None,
                &target_path,
                None,
                false,
                Some("not_in_allowed_paths"),
                None,
            );
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Path is outside allowed directories".to_string()),
            }));
        }
        Err(ValidationError::SensitiveDirectory(dir)) => {
            warn!("Access to sensitive directory blocked: {}", dir);
            log_directory_access(
                None,
                &target_path,
                None,
                false,
                Some("sensitive_directory"),
                None,
            );
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Access to sensitive directory blocked".to_string()),
            }));
        }
        Err(ValidationError::PathDoesNotExist) => {
            log_directory_access(
                None,
                &target_path,
                None,
                false,
                Some("path_does_not_exist"),
                None,
            );
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Directory does not exist".to_string()),
            }));
        }
        Err(e) => {
            warn!("Path validation failed: {:?}", e);
            log_directory_access(
                None,
                &target_path,
                None,
                false,
                Some("validation_failed"),
                None,
            );
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Invalid path".to_string()),
            }));
        }
    };

    if !validated_path.is_dir() {
        return Ok(ResponseJson(BrowseDirectoriesResponse {
            success: false,
            data: None,
            error: Some("Path is not a directory".to_string()),
        }));
    }

    // Read directory with additional filtering
    let mut directories = Vec::new();

    match fs::read_dir(&validated_path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let entry_path = entry.path();

                // Only include directories
                if entry_path.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Skip hidden directories and system directories
                        if !name.starts_with('.') && !is_system_directory(name) {
                            // Additional validation - check if subdirectory would be allowed
                            if validator.would_allow_subdirectory(&entry_path) {
                                directories.push(DirectoryItem {
                                    name: name.to_string(),
                                    path: entry_path.to_string_lossy().to_string(),
                                    is_directory: true,
                                });
                            } else {
                                debug!("Filtered out restricted subdirectory: {}", name);
                            }
                        }
                    }
                }
            }
        }
        Err(e) => {
            warn!(
                "Failed to read directory {}: {}",
                validated_path.display(),
                e
            );
            log_directory_access(
                None,
                &target_path,
                Some(&validated_path.to_string_lossy()),
                false,
                Some("read_permission_denied"),
                None,
            );
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some("Permission denied or directory inaccessible".to_string()),
            }));
        }
    }

    // Sort directories alphabetically
    directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let current_path_str = validated_path.to_string_lossy().to_string();

    // Build response with restricted parent navigation
    let parent_path = if let Some(parent) = validated_path.parent() {
        // Only allow parent if it's also in allowed paths
        match validator.validate_path(&parent.to_string_lossy()) {
            Ok(p) => {
                debug!("Parent path allowed: {}", p.display());
                Some(p.to_string_lossy().to_string())
            }
            Err(_) => {
                debug!("Parent path blocked, not providing parent navigation");
                None
            }
        }
    } else {
        None
    };

    let is_root = parent_path.is_none();

    let data = DirectoryData {
        current_path: current_path_str.clone(),
        parent_path,
        directories: directories.clone(),
        is_root,
        separator: std::path::MAIN_SEPARATOR.to_string(),
    };

    // Log successful directory access
    log_directory_access(
        None,
        &target_path,
        Some(&current_path_str),
        true,
        None,
        Some(directories.len()),
    );

    Ok(ResponseJson(BrowseDirectoriesResponse {
        success: true,
        data: Some(data),
        error: None,
    }))
}

fn log_directory_access(
    user: Option<&str>,
    requested_path: &str,
    resolved_path: Option<&str>,
    allowed: bool,
    error_type: Option<&str>,
    entries_count: Option<usize>,
) {
    let timestamp = chrono::Utc::now();
    let log_entry = json!({
        "timestamp": timestamp.to_rfc3339(),
        "user": user.unwrap_or("anonymous"),
        "action": "browse_directory",
        "requested_path": requested_path,
        "resolved_path": resolved_path,
        "allowed": allowed,
        "error_type": error_type,
        "entries_count": entries_count,
        "source": "directory_browser"
    });

    // Use structured logging with audit flag
    if allowed {
        info!(audit = true, directory_access = %log_entry, "Directory access granted");
    } else {
        warn!(audit = true, directory_access = %log_entry, "Directory access denied: {}", 
              error_type.unwrap_or("unknown"));
    }
}

fn is_system_directory(name: &str) -> bool {
    const SYSTEM_DIRS: &[&str] = &[
        "System",
        "Windows",
        "Program Files",
        "Program Files (x86)",
        "ProgramData",
        "usr",
        "var",
        "opt",
        "etc",
        "proc",
        "sys",
        "bin",
        "sbin",
        "boot",
        "dev",
        "mnt",
        "media",
        "root",
        "run",
        "tmp",
        "Applications",
        "Library",
        "System",
        "Volumes", // macOS
        "node_modules",
        "__pycache__",
        ".git",
        ".svn",
        ".hg", // Development
    ];

    SYSTEM_DIRS.contains(&name)
}
