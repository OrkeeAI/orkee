use axum::{extract::Json, http::StatusCode, response::Json as ResponseJson};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Deserialize)]
pub struct BrowseDirectoriesRequest {
    pub path: Option<String>,
}

#[derive(Serialize)]
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
    Json(request): Json<BrowseDirectoriesRequest>,
) -> Result<ResponseJson<BrowseDirectoriesResponse>, StatusCode> {
    let target_path = match &request.path {
        Some(p) if !p.is_empty() => expand_path(p),
        _ => get_home_directory(),
    };

    let path = Path::new(&target_path);

    if !path.exists() {
        return Ok(ResponseJson(BrowseDirectoriesResponse {
            success: false,
            data: None,
            error: Some("Directory does not exist".to_string()),
        }));
    }

    if !path.is_dir() {
        return Ok(ResponseJson(BrowseDirectoriesResponse {
            success: false,
            data: None,
            error: Some("Path is not a directory".to_string()),
        }));
    }

    let mut directories = Vec::new();

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries.flatten() {
                let entry_path = entry.path();
                if entry_path.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        // Skip hidden directories (starting with .)
                        if !name.starts_with('.') {
                            directories.push(DirectoryItem {
                                name: name.to_string(),
                                path: entry_path.to_string_lossy().to_string(),
                                is_directory: true,
                            });
                        }
                    }
                }
            }
        }
        Err(e) => {
            return Ok(ResponseJson(BrowseDirectoriesResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to read directory: {}", e)),
            }));
        }
    }

    // Sort directories alphabetically
    directories.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));

    let current_path_str = path.to_string_lossy().to_string();
    let parent_path = path.parent().map(|p| p.to_string_lossy().to_string());
    let is_root = parent_path.is_none();

    let data = DirectoryData {
        current_path: current_path_str,
        parent_path,
        directories,
        is_root,
        separator: std::path::MAIN_SEPARATOR.to_string(),
    };

    Ok(ResponseJson(BrowseDirectoriesResponse {
        success: true,
        data: Some(data),
        error: None,
    }))
}

fn expand_path(path: &str) -> String {
    if path.starts_with("~/") {
        let home = get_home_directory();
        path.replacen("~", &home, 1)
    } else {
        path.to_string()
    }
}

fn get_home_directory() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| "/".to_string())
}
