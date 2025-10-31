// ABOUTME: GitHub CLI wrapper providing high-level interface to gh CLI commands
// ABOUTME: Handles authentication detection, issue operations, and automatic error handling

use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GitHubCliError {
    #[error("gh CLI not available: {0}")]
    NotAvailable(String),

    #[error("gh not authenticated")]
    NotAuthenticated,

    #[error("gh command failed: {0}")]
    CommandFailed(String),

    #[error("Failed to parse gh output: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GitHubCliError>;

/// GitHub issue structure returned by gh CLI
#[derive(Debug, Deserialize, Serialize)]
pub struct GhIssue {
    pub number: i32,
    pub url: String,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

/// GitHub CLI wrapper
pub struct GitHubCli {
    gh_path: String,
}

impl GitHubCli {
    /// Check if gh CLI is available and authenticated
    pub fn new() -> Result<Self> {
        // Find gh in PATH
        let gh_path = which::which("gh")
            .map_err(|e| GitHubCliError::NotAvailable(format!("gh not found in PATH: {}", e)))?
            .to_string_lossy()
            .to_string();

        let cli = Self { gh_path };

        // Verify authentication
        if !cli.is_authenticated()? {
            return Err(GitHubCliError::NotAuthenticated);
        }

        Ok(cli)
    }

    /// Check if gh CLI is authenticated
    pub fn is_authenticated(&self) -> Result<bool> {
        let output = Command::new(&self.gh_path)
            .args(["auth", "status"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()?;

        Ok(output.success())
    }

    /// Create a GitHub issue using gh CLI
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `title` - Issue title
    /// * `body` - Issue body (markdown)
    /// * `labels` - Optional labels to add
    /// * `assignees` - Optional assignees
    pub async fn create_issue(
        &self,
        owner: &str,
        repo: &str,
        title: &str,
        body: &str,
        labels: Option<Vec<String>>,
        assignees: Option<Vec<String>>,
    ) -> Result<GhIssue> {
        let mut args = vec![
            "issue".to_string(),
            "create".to_string(),
            "--repo".to_string(),
            format!("{}/{}", owner, repo),
            "--title".to_string(),
            title.to_string(),
            "--body".to_string(),
            body.to_string(),
            "--json".to_string(),
            "number,url,title,body,state,updatedAt".to_string(),
        ];

        // Add labels
        if let Some(label_vec) = labels {
            for label in label_vec {
                args.push("--label".to_string());
                args.push(label);
            }
        }

        // Add assignees
        if let Some(assignee_vec) = assignees {
            for assignee in assignee_vec {
                args.push("--assignee".to_string());
                args.push(assignee);
            }
        }

        let output = Command::new(&self.gh_path)
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubCliError::CommandFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| GitHubCliError::ParseError(format!("Failed to parse issue JSON: {}", e)))
    }

    /// Update a GitHub issue using gh CLI
    ///
    /// # Arguments
    /// * `owner` - Repository owner
    /// * `repo` - Repository name
    /// * `issue_number` - Issue number to update
    /// * `title` - Optional new title
    /// * `body` - Optional new body
    /// * `state` - Optional new state ("open" or "closed")
    /// * `labels` - Optional labels (replaces existing)
    pub async fn update_issue(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        title: Option<String>,
        body: Option<String>,
        state: Option<String>,
        labels: Option<Vec<String>>,
    ) -> Result<GhIssue> {
        let mut args = vec![
            "issue".to_string(),
            "edit".to_string(),
            issue_number.to_string(),
            "--repo".to_string(),
            format!("{}/{}", owner, repo),
        ];

        // Add title if provided
        if let Some(t) = title {
            args.push("--title".to_string());
            args.push(t);
        }

        // Add body if provided
        if let Some(b) = body {
            args.push("--body".to_string());
            args.push(b);
        }

        // Add labels if provided
        if let Some(label_vec) = labels {
            // gh CLI replaces all labels when using --label
            for label in label_vec {
                args.push("--label".to_string());
                args.push(label);
            }
        }

        // Execute edit command
        let output = Command::new(&self.gh_path)
            .args(&args)
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubCliError::CommandFailed(stderr.to_string()));
        }

        // Handle state change separately (gh issue edit doesn't support --state)
        if let Some(new_state) = state {
            let state_cmd = if new_state == "closed" { "close" } else { "reopen" };
            let state_output = Command::new(&self.gh_path)
                .args([
                    "issue",
                    state_cmd,
                    &issue_number.to_string(),
                    "--repo",
                    &format!("{}/{}", owner, repo),
                ])
                .output()?;

            if !state_output.status.success() {
                let stderr = String::from_utf8_lossy(&state_output.stderr);
                return Err(GitHubCliError::CommandFailed(format!(
                    "Failed to change issue state: {}",
                    stderr
                )));
            }
        }

        // Fetch updated issue
        self.get_issue(owner, repo, issue_number).await
    }

    /// Get issue details
    async fn get_issue(&self, owner: &str, repo: &str, issue_number: i32) -> Result<GhIssue> {
        let output = Command::new(&self.gh_path)
            .args([
                "issue",
                "view",
                &issue_number.to_string(),
                "--repo",
                &format!("{}/{}", owner, repo),
                "--json",
                "number,url,title,body,state,updatedAt",
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubCliError::CommandFailed(stderr.to_string()));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        serde_json::from_str(&stdout)
            .map_err(|e| GitHubCliError::ParseError(format!("Failed to parse issue JSON: {}", e)))
    }

    /// Add a comment to an issue
    pub async fn add_comment(
        &self,
        owner: &str,
        repo: &str,
        issue_number: i32,
        body: &str,
    ) -> Result<()> {
        let output = Command::new(&self.gh_path)
            .args([
                "issue",
                "comment",
                &issue_number.to_string(),
                "--repo",
                &format!("{}/{}", owner, repo),
                "--body",
                body,
            ])
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(GitHubCliError::CommandFailed(stderr.to_string()));
        }

        Ok(())
    }

    /// Check if the current user has the specified scopes
    pub fn check_scopes(&self, required_scopes: &[&str]) -> Result<bool> {
        let output = Command::new(&self.gh_path)
            .args(["auth", "status", "-t"])
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        let combined = format!("{}{}", stdout, stderr);

        // gh auth status outputs scopes like: Token scopes: 'repo', 'workflow'
        let has_all_scopes = required_scopes.iter().all(|scope| {
            combined.contains(&format!("'{}'", scope)) || combined.contains(&format!("\"{}", scope))
        });

        Ok(has_all_scopes)
    }
}

impl Default for GitHubCli {
    fn default() -> Self {
        Self::new().expect("gh CLI not available or not authenticated")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gh_availability() {
        // This test will pass if gh is installed and authenticated
        match GitHubCli::new() {
            Ok(cli) => {
                println!("gh CLI is available and authenticated");
                assert!(cli.is_authenticated().unwrap());
            }
            Err(e) => {
                println!("gh CLI not available: {}", e);
                // Test passes either way - we're just checking detection works
            }
        }
    }

    #[tokio::test]
    async fn test_scope_checking() {
        if let Ok(cli) = GitHubCli::new() {
            // Check for basic repo scope
            let has_repo = cli.check_scopes(&["repo"]).unwrap();
            println!("Has 'repo' scope: {}", has_repo);
        }
    }
}
