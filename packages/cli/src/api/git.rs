use axum::{
    extract::{Path, Query},
    response::Json,
    Extension,
};
use git2::{Commit, DiffOptions, Repository, Oid, DiffFormat};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, warn};

use orkee_projects::manager::ProjectsManager;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommitInfo {
    pub id: String,
    pub short_id: String,
    pub message: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub timestamp: i64,
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Serialize)]
pub struct CommitDetail {
    pub commit: CommitInfo,
    pub files: Vec<FileChange>,
    pub parent_ids: Vec<String>,
    pub stats: CommitStats,
}

#[derive(Debug, Serialize)]
pub struct FileChange {
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
    pub insertions: usize,
    pub deletions: usize,
}

#[derive(Debug, Serialize)]
pub struct CommitStats {
    pub files_changed: usize,
    pub total_insertions: usize,
    pub total_deletions: usize,
}

#[derive(Debug, Serialize)]
pub struct FileDiff {
    pub path: String,
    pub old_path: Option<String>,
    pub status: String,
    pub content: String,
    pub is_binary: bool,
}

#[derive(Debug, Deserialize)]
pub struct CommitHistoryQuery {
    pub page: Option<usize>,
    pub per_page: Option<usize>,
    pub branch: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FileDiffQuery {
    pub context: Option<usize>,
}

pub async fn get_commit_history(
    Path(project_id): Path<String>,
    Query(params): Query<CommitHistoryQuery>,
    Extension(project_manager): Extension<Arc<ProjectsManager>>,
) -> Json<ApiResponse<Vec<CommitInfo>>> {
    debug!("Getting commit history for project: {}", project_id);

    let page = params.page.unwrap_or(1).max(1);
    let per_page = params.per_page.unwrap_or(50).clamp(1, 100);
    let skip = (page - 1) * per_page;

    // Get project details
    let project = match project_manager.get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Project not found".to_string()),
            });
        }
        Err(e) => {
            error!("Failed to get project: {}", e);
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to get project".to_string()),
            });
        }
    };

    // Open git repository
    let repo = match Repository::open(&project.project_root) {
        Ok(repo) => repo,
        Err(e) => {
            warn!("No git repository found at {}: {}", project.project_root, e);
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("No git repository found".to_string()),
            });
        }
    };

    // Get commits
    match get_commits_from_repo(&repo, skip, per_page, params.branch.as_deref()) {
        Ok(commits) => Json(ApiResponse {
            success: true,
            data: Some(commits),
            error: None,
        }),
        Err(e) => {
            error!("Failed to get commit history: {}", e);
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to get commit history".to_string()),
            })
        }
    }
}

pub async fn get_commit_details(
    Path((project_id, commit_id)): Path<(String, String)>,
    Extension(project_manager): Extension<Arc<ProjectsManager>>,
) -> Json<ApiResponse<CommitDetail>> {
    debug!("Getting commit details for project: {}, commit: {}", project_id, commit_id);

    // Get project details
    let project = match project_manager.get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Project not found".to_string()),
            });
        }
        Err(e) => {
            error!("Failed to get project: {}", e);
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to get project".to_string()),
            });
        }
    };

    // Open git repository
    let repo = match Repository::open(&project.project_root) {
        Ok(repo) => repo,
        Err(e) => {
            warn!("No git repository found at {}: {}", project.project_root, e);
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("No git repository found".to_string()),
            });
        }
    };

    // Get commit details
    match get_commit_detail_from_repo(&repo, &commit_id) {
        Ok(detail) => Json(ApiResponse {
            success: true,
            data: Some(detail),
            error: None,
        }),
        Err(e) => {
            error!("Failed to get commit details: {}", e);
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to get commit details".to_string()),
            })
        }
    }
}

pub async fn get_file_diff(
    Path((project_id, commit_id, file_path)): Path<(String, String, String)>,
    Query(params): Query<FileDiffQuery>,
    Extension(project_manager): Extension<Arc<ProjectsManager>>,
) -> Json<ApiResponse<FileDiff>> {
    debug!("Getting file diff for project: {}, commit: {}, file: {}", project_id, commit_id, file_path);

    let context = params.context.unwrap_or(3);

    // Get project details
    let project = match project_manager.get_project(&project_id).await {
        Ok(Some(project)) => project,
        Ok(None) => {
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Project not found".to_string()),
            });
        }
        Err(e) => {
            error!("Failed to get project: {}", e);
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to get project".to_string()),
            });
        }
    };

    // Open git repository
    let repo = match Repository::open(&project.project_root) {
        Ok(repo) => repo,
        Err(e) => {
            warn!("No git repository found at {}: {}", project.project_root, e);
            return Json(ApiResponse {
                success: false,
                data: None,
                error: Some("No git repository found".to_string()),
            });
        }
    };

    // Get file diff
    match get_file_diff_from_repo(&repo, &commit_id, &file_path, context) {
        Ok(diff) => Json(ApiResponse {
            success: true,
            data: Some(diff),
            error: None,
        }),
        Err(e) => {
            error!("Failed to get file diff: {}", e);
            Json(ApiResponse {
                success: false,
                data: None,
                error: Some("Failed to get file diff".to_string()),
            })
        }
    }
}

fn get_commits_from_repo(
    repo: &Repository,
    skip: usize,
    per_page: usize,
    branch: Option<&str>,
) -> Result<Vec<CommitInfo>, git2::Error> {
    let mut revwalk = repo.revwalk()?;
    
    // Use specified branch or HEAD
    if let Some(branch_name) = branch {
        let branch_ref = format!("refs/heads/{}", branch_name);
        if let Ok(_reference) = repo.find_reference(&branch_ref) {
            revwalk.push_ref(&branch_ref)?;
        } else {
            // Fallback to HEAD if branch not found
            revwalk.push_head()?;
        }
    } else {
        revwalk.push_head()?;
    }
    
    revwalk.set_sorting(git2::Sort::TIME)?;
    
    let mut commits = Vec::new();
    let mut count = 0;
    
    for (index, commit_id) in revwalk.enumerate() {
        if index < skip {
            continue;
        }
        
        if count >= per_page {
            break;
        }
        
        let commit_id = commit_id?;
        let commit = repo.find_commit(commit_id)?;
        
        // Calculate diff stats
        let (files_changed, insertions, deletions) = calculate_commit_stats(repo, &commit)?;
        
        commits.push(CommitInfo {
            id: commit.id().to_string(),
            short_id: commit.id().to_string()[..7].to_string(),
            message: commit.message().unwrap_or("").to_string(),
            author: commit.author().name().unwrap_or("").to_string(),
            email: commit.author().email().unwrap_or("").to_string(),
            date: format_timestamp(commit.time().seconds()),
            timestamp: commit.time().seconds(),
            files_changed,
            insertions,
            deletions,
        });
        
        count += 1;
    }
    
    Ok(commits)
}

fn get_commit_detail_from_repo(repo: &Repository, commit_id: &str) -> Result<CommitDetail, git2::Error> {
    let oid = Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    
    // Get parent IDs
    let parent_ids: Vec<String> = commit.parent_ids().map(|id| id.to_string()).collect();
    
    // Calculate diff stats and file changes
    let (files_changed, insertions, deletions) = calculate_commit_stats(repo, &commit)?;
    let files = get_commit_file_changes(repo, &commit)?;
    
    let commit_info = CommitInfo {
        id: commit.id().to_string(),
        short_id: commit.id().to_string()[..7].to_string(),
        message: commit.message().unwrap_or("").to_string(),
        author: commit.author().name().unwrap_or("").to_string(),
        email: commit.author().email().unwrap_or("").to_string(),
        date: format_timestamp(commit.time().seconds()),
        timestamp: commit.time().seconds(),
        files_changed,
        insertions,
        deletions,
    };
    
    Ok(CommitDetail {
        commit: commit_info,
        files,
        parent_ids,
        stats: CommitStats {
            files_changed,
            total_insertions: insertions,
            total_deletions: deletions,
        },
    })
}

fn get_file_diff_from_repo(
    repo: &Repository,
    commit_id: &str,
    file_path: &str,
    context: usize,
) -> Result<FileDiff, git2::Error> {
    let oid = Oid::from_str(commit_id)?;
    let commit = repo.find_commit(oid)?;
    
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };
    
    let mut diff_opts = DiffOptions::new();
    diff_opts.context_lines(context as u32);
    diff_opts.pathspec(file_path);
    
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;
    
    let mut content = String::new();
    let mut old_path = None;
    let mut status = "modified".to_string();
    let mut is_binary = false;
    
    diff.print(DiffFormat::Patch, |_delta, _hunk, line| {
        match line.origin() {
            ' ' | '+' | '-' => {
                content.push(line.origin());
                content.push_str(std::str::from_utf8(line.content()).unwrap_or(""));
            }
            'F' => {
                // File header
                if let Ok(header) = std::str::from_utf8(line.content()) {
                    content.push_str(header);
                }
            }
            'H' => {
                // Hunk header  
                content.push_str("@@");
                if let Ok(header) = std::str::from_utf8(line.content()) {
                    content.push_str(header);
                }
            }
            'B' => {
                is_binary = true;
            }
            _ => {}
        }
        true
    })?;
    
    // Get file status and old path from delta
    diff.foreach(
        &mut |delta, _progress| {
            match delta.status() {
                git2::Delta::Added => status = "added".to_string(),
                git2::Delta::Deleted => status = "deleted".to_string(),
                git2::Delta::Modified => status = "modified".to_string(),
                git2::Delta::Renamed => {
                    status = "renamed".to_string();
                    if let Some(old_file) = delta.old_file().path() {
                        old_path = old_file.to_str().map(|s| s.to_string());
                    }
                }
                git2::Delta::Copied => status = "copied".to_string(),
                _ => {}
            }
            true
        },
        None,
        None,
        None,
    )?;
    
    Ok(FileDiff {
        path: file_path.to_string(),
        old_path,
        status,
        content,
        is_binary,
    })
}

fn calculate_commit_stats(repo: &Repository, commit: &Commit) -> Result<(usize, usize, usize), git2::Error> {
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };
    
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let stats = diff.stats()?;
    
    Ok((
        stats.files_changed(),
        stats.insertions(),
        stats.deletions(),
    ))
}

fn get_commit_file_changes(repo: &Repository, commit: &Commit) -> Result<Vec<FileChange>, git2::Error> {
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() > 0 {
        Some(commit.parent(0)?.tree()?)
    } else {
        None
    };
    
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let mut file_changes = Vec::new();
    
    diff.foreach(
        &mut |delta, _progress| {
            let new_file = delta.new_file();
            let old_file = delta.old_file();
            
            let path = new_file.path()
                .or_else(|| old_file.path())
                .and_then(|p| p.to_str())
                .unwrap_or("")
                .to_string();
            
            let old_path = if delta.status() == git2::Delta::Renamed {
                old_file.path().and_then(|p| p.to_str()).map(|s| s.to_string())
            } else {
                None
            };
            
            let status = match delta.status() {
                git2::Delta::Added => "added",
                git2::Delta::Deleted => "deleted", 
                git2::Delta::Modified => "modified",
                git2::Delta::Renamed => "renamed",
                git2::Delta::Copied => "copied",
                _ => "unknown",
            }.to_string();
            
            file_changes.push(FileChange {
                path,
                old_path,
                status,
                insertions: 0, // Will be filled by patch analysis
                deletions: 0,  // Will be filled by patch analysis
            });
            
            true
        },
        None,
        None,
        None,
    )?;
    
    // Get per-file stats by creating individual diffs for each file
    for file_change in file_changes.iter_mut() {
        // Create a diff options that targets only this specific file
        let mut diff_opts = DiffOptions::new();
        diff_opts.pathspec(&file_change.path);
        
        // Create a diff for just this file
        let file_diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), Some(&mut diff_opts))?;
        
        // Count additions and deletions for this specific file
        let mut insertions = 0;
        let mut deletions = 0;
        
        file_diff.foreach(
            &mut |_delta, _progress| true,
            None,
            Some(&mut |_delta, _hunk| true),
            Some(&mut |_delta, _hunk, line| {
                match line.origin() {
                    '+' => insertions += 1,
                    '-' => deletions += 1,
                    _ => {}
                }
                true
            }),
        )?;
        
        file_change.insertions = insertions;
        file_change.deletions = deletions;
    }
    
    Ok(file_changes)
}

fn format_timestamp(timestamp: i64) -> String {
    use chrono::{Utc, TimeZone};
    
    match Utc.timestamp_opt(timestamp, 0) {
        chrono::LocalResult::Single(dt) => dt.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        _ => timestamp.to_string(),
    }
}