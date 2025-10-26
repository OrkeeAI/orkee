// ABOUTME: Git integration utilities for extracting repository information from projects.
// ABOUTME: Handles GitHub URL parsing (SSH/HTTPS formats) and retrieves branch/remote data.

use git2::Repository;
use orkee_core::types::GitRepositoryInfo;
use tracing::debug;

pub fn get_git_repository_info(project_path: &str) -> Option<GitRepositoryInfo> {
    debug!("Getting git repository info for path: {}", project_path);

    // Try to open the git repository
    let repo = match Repository::open(project_path) {
        Ok(repo) => repo,
        Err(e) => {
            debug!("No git repository found at {}: {}", project_path, e);
            return None;
        }
    };

    // Get the remote origin URL
    let remote = match repo.find_remote("origin") {
        Ok(remote) => remote,
        Err(e) => {
            debug!("No origin remote found: {}", e);
            return None;
        }
    };

    let url = match remote.url() {
        Some(url) => url,
        None => {
            debug!("No URL found for origin remote");
            return None;
        }
    };

    // Parse GitHub repository info from URL
    let repo_info = match parse_github_url(url) {
        Some(info) => info,
        None => {
            debug!("Could not parse GitHub info from URL: {}", url);
            return None;
        }
    };

    // Get current branch
    let branch = repo
        .head()
        .ok()
        .and_then(|head| head.shorthand().map(|s| s.to_string()));

    debug!("Found git repository: {}/{}", repo_info.0, repo_info.1);

    Some(GitRepositoryInfo {
        owner: repo_info.0,
        repo: repo_info.1,
        url: url.to_string(),
        branch,
    })
}

fn parse_github_url(url: &str) -> Option<(String, String)> {
    // Handle different GitHub URL formats:
    // https://github.com/owner/repo.git
    // git@github.com:owner/repo.git
    // https://github.com/owner/repo

    if url.starts_with("git@github.com:") {
        // SSH format: git@github.com:owner/repo.git
        let path = url.strip_prefix("git@github.com:")?;
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() == 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    } else if url.contains("github.com/") {
        // HTTPS format with or without username:
        // https://github.com/owner/repo.git
        // https://github.com/owner/repo
        // https://username@github.com/owner/repo.git

        // Find the position of "github.com/" and extract everything after it
        let github_pos = url
            .find("github.com/")
            .map(|pos| pos + "github.com/".len())?;
        let path = &url[github_pos..];
        let path = path.strip_suffix(".git").unwrap_or(path);
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 2 {
            return Some((parts[0].to_string(), parts[1].to_string()));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        // Test HTTPS URLs
        assert_eq!(
            parse_github_url("https://github.com/joedanz/vibekit.git"),
            Some(("joedanz".to_string(), "vibekit".to_string()))
        );

        assert_eq!(
            parse_github_url("https://github.com/joedanz/vibekit"),
            Some(("joedanz".to_string(), "vibekit".to_string()))
        );

        // Test HTTPS URLs with username
        assert_eq!(
            parse_github_url("https://joedanz@github.com/joedanz/vibe-kanban.git"),
            Some(("joedanz".to_string(), "vibe-kanban".to_string()))
        );

        // Test SSH URLs
        assert_eq!(
            parse_github_url("git@github.com:joedanz/vibekit.git"),
            Some(("joedanz".to_string(), "vibekit".to_string()))
        );

        // Test invalid URLs
        assert_eq!(parse_github_url("not-a-valid-url"), None);
    }
}
