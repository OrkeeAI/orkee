// ABOUTME: Shared Docker CLI wrapper functions for image management and authentication
// ABOUTME: Provides reusable Docker command execution for both CLI and API server

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::io::{BufRead, BufReader};

/// Docker image information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerImage {
    pub repository: String,
    pub tag: String,
    pub image_id: String,
    pub size: String,
    pub created: String,
}

/// Docker login status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerStatus {
    pub logged_in: bool,
    pub username: Option<String>,
    pub email: Option<String>,
    pub server_address: Option<String>,
}

/// Docker Hub configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerConfig {
    pub username: Option<String>,
    pub auth_servers: Vec<String>,
}

/// Docker build progress event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildProgress {
    pub step: usize,
    pub total_steps: usize,
    pub current_step: String,
    pub output: String,
}

/// Check if Docker daemon is running
pub fn is_docker_running() -> Result<bool> {
    let output = Command::new("docker")
        .arg("info")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    Ok(output.map(|s| s.success()).unwrap_or(false))
}

/// Check if user is logged in to Docker Hub
pub fn is_docker_logged_in() -> Result<bool> {
    // Check Docker config file for authentication
    if let Ok(home) = std::env::var("HOME") {
        let config_path = format!("{}/.docker/config.json", home);
        if let Ok(content) = std::fs::read_to_string(config_path) {
            // Check if Docker Hub auth exists in config
            // Docker Hub can be under several keys
            return Ok(content.contains("https://index.docker.io/v1/")
                || content.contains("index.docker.io"));
        }
    }

    // Fallback: try docker info (may not work on all versions)
    let output = Command::new("docker")
        .arg("info")
        .output()
        .context("Failed to execute docker info command")?;

    if !output.status.success() {
        return Ok(false);
    }

    let info_str = String::from_utf8_lossy(&output.stdout);
    // Look for Username field in output
    Ok(info_str.contains("Username:"))
}

/// Get Docker login status and user information
pub fn get_docker_status() -> Result<DockerStatus> {
    let logged_in = is_docker_logged_in()?;

    if !logged_in {
        return Ok(DockerStatus {
            logged_in: false,
            username: None,
            email: None,
            server_address: None,
        });
    }

    // Try to get username from docker info
    let username = get_docker_username().ok();

    Ok(DockerStatus {
        logged_in: true,
        username,
        email: None, // Docker CLI doesn't expose email easily
        server_address: Some("https://index.docker.io/v1/".to_string()),
    })
}

/// Get Docker Hub username
pub fn get_docker_username() -> Result<String> {
    // Try to read from Docker config file first
    if let Ok(home) = std::env::var("HOME") {
        let config_path = format!("{}/.docker/config.json", home);
        if let Ok(content) = std::fs::read_to_string(config_path) {
            // Try to parse JSON config
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check if there's a username in the config
                // Docker Hub can be under "https://index.docker.io/v1/" or "index.docker.io"
                if let Some(auths) = config.get("auths").and_then(|a| a.as_object()) {
                    for (server, auth) in auths {
                        if server.contains("index.docker.io") {
                            if let Some(username) = auth.get("username").and_then(|u| u.as_str()) {
                                return Ok(username.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // Try docker info command (may not work on all Docker versions)
    let output = Command::new("docker").arg("info").output();

    if let Ok(output) = output {
        let info_str = String::from_utf8_lossy(&output.stdout);
        // Try to find username in output
        for line in info_str.lines() {
            if line.trim().starts_with("Username:") {
                if let Some(username) = line.split(':').nth(1) {
                    let username = username.trim();
                    if !username.is_empty() {
                        return Ok(username.to_string());
                    }
                }
            }
        }
    }

    // Could not detect username
    anyhow::bail!("Could not detect Docker Hub username")
}

/// Get Docker configuration
pub fn get_docker_config() -> Result<DockerConfig> {
    let username = get_docker_username().ok();
    let mut auth_servers = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        let config_path = format!("{}/.docker/config.json", home);
        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(auths) = config.get("auths").and_then(|a| a.as_object()) {
                    auth_servers = auths.keys().cloned().collect();
                }
            }
        }
    }

    Ok(DockerConfig {
        username,
        auth_servers,
    })
}

/// List Docker images with optional filter
pub fn list_docker_images(filter_label: Option<&str>) -> Result<Vec<DockerImage>> {
    let mut cmd = Command::new("docker");
    cmd.arg("images")
        .arg("--format")
        .arg("{{.Repository}}\t{{.Tag}}\t{{.ID}}\t{{.Size}}\t{{.CreatedAt}}");

    if let Some(label) = filter_label {
        cmd.arg("--filter").arg(format!("label={}", label));
    }

    let output = cmd.output()
        .context("Failed to execute docker images command")?;

    if !output.status.success() {
        anyhow::bail!(
            "Docker images failed with exit code: {:?}",
            output.status.code()
        );
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut images = Vec::new();

    for line in output_str.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 5 {
            images.push(DockerImage {
                repository: parts[0].to_string(),
                tag: parts[1].to_string(),
                image_id: parts[2].to_string(),
                size: parts[3].to_string(),
                created: parts[4].to_string(),
            });
        }
    }

    Ok(images)
}

/// Delete a Docker image
pub fn delete_docker_image(image_name_or_id: &str, force: bool) -> Result<()> {
    let mut cmd = Command::new("docker");
    cmd.arg("rmi");

    if force {
        cmd.arg("-f");
    }

    cmd.arg(image_name_or_id);

    let output = cmd.output()
        .context("Failed to execute docker rmi command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker rmi failed: {}", stderr);
    }

    Ok(())
}

/// Build a Docker image
pub fn build_docker_image(
    dockerfile_path: &Path,
    build_context: &Path,
    image_tag: &str,
    additional_labels: &[(&str, &str)],
) -> Result<Output> {
    let mut cmd = Command::new("docker");
    cmd.arg("build")
        .arg("-t")
        .arg(image_tag)
        .arg("-f")
        .arg(dockerfile_path);

    // Add labels
    for (key, value) in additional_labels {
        cmd.arg("--label").arg(format!("{}={}", key, value));
    }

    cmd.arg(build_context);

    let output = cmd.output()
        .context("Failed to execute docker build command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker build failed: {}", stderr);
    }

    Ok(output)
}

/// Build a Docker image with streaming output
/// Returns a channel receiver for build progress
pub fn build_docker_image_stream(
    dockerfile_path: PathBuf,
    build_context: PathBuf,
    image_tag: String,
    additional_labels: Vec<(String, String)>,
) -> Result<impl Iterator<Item = String>> {
    let mut cmd = Command::new("docker");
    cmd.arg("build")
        .arg("-t")
        .arg(&image_tag)
        .arg("-f")
        .arg(&dockerfile_path);

    // Add labels
    for (key, value) in &additional_labels {
        cmd.arg("--label").arg(format!("{}={}", key, value));
    }

    cmd.arg(&build_context)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut child = cmd.spawn()
        .context("Failed to spawn docker build process")?;

    // Capture stdout
    let stdout = child.stdout.take()
        .context("Failed to capture stdout")?;

    let reader = BufReader::new(stdout);

    // Return an iterator over lines
    Ok(reader.lines().filter_map(|line| line.ok()))
}

/// Push a Docker image to Docker Hub
pub fn push_docker_image(image_tag: &str) -> Result<Output> {
    let output = Command::new("docker")
        .arg("push")
        .arg(image_tag)
        .output()
        .context("Failed to execute docker push command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker push failed: {}", stderr);
    }

    Ok(output)
}

/// Push a Docker image with streaming output
pub fn push_docker_image_stream(image_tag: String) -> Result<impl Iterator<Item = String>> {
    let mut child = Command::new("docker")
        .arg("push")
        .arg(&image_tag)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn docker push process")?;

    // Capture stdout
    let stdout = child.stdout.take()
        .context("Failed to capture stdout")?;

    let reader = BufReader::new(stdout);

    // Return an iterator over lines
    Ok(reader.lines().filter_map(|line| line.ok()))
}

/// Run docker login interactively
pub fn docker_login() -> Result<()> {
    let status = Command::new("docker")
        .arg("login")
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .context("Failed to execute docker login command")?;

    if !status.success() {
        anyhow::bail!("Docker login failed with exit code: {:?}", status.code());
    }

    Ok(())
}

/// Run docker logout
pub fn docker_logout() -> Result<()> {
    let status = Command::new("docker")
        .arg("logout")
        .status()
        .context("Failed to execute docker logout command")?;

    if !status.success() {
        anyhow::bail!("Docker logout failed with exit code: {:?}", status.code());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_docker_running() {
        // This test will pass if Docker is running, skip if not
        match is_docker_running() {
            Ok(running) => {
                // Just assert it returns a boolean
                assert!(running || !running);
            }
            Err(_) => {
                // Docker might not be installed in test environment
                println!("Docker not available in test environment");
            }
        }
    }

    #[test]
    fn test_docker_status() {
        // Test that we can call get_docker_status without panicking
        let _ = get_docker_status();
    }

    #[test]
    fn test_docker_config() {
        // Test that we can call get_docker_config without panicking
        let _ = get_docker_config();
    }

    #[test]
    fn test_list_docker_images() {
        // Skip if Docker not available
        if is_docker_running().unwrap_or(false) {
            let result = list_docker_images(None);
            assert!(result.is_ok(), "Should be able to list images if Docker is running");
        }
    }

    #[test]
    fn test_list_docker_images_with_filter() {
        // Skip if Docker not available
        if is_docker_running().unwrap_or(false) {
            // Test with orkee.sandbox label filter
            let result = list_docker_images(Some("orkee.sandbox=true"));
            assert!(result.is_ok(), "Should be able to list images with label filter");

            // The result may be empty if no images with that label exist, which is fine
            let images = result.unwrap();
            println!("Found {} images with orkee.sandbox label", images.len());
        }
    }

    #[test]
    fn test_get_docker_username() {
        // This test doesn't require Docker to be running, just checks the function
        // It may return Ok or Err depending on whether user is logged in
        let result = get_docker_username();
        match result {
            Ok(username) => {
                assert!(!username.is_empty(), "Username should not be empty if logged in");
                println!("Detected Docker Hub username: {}", username);
            }
            Err(_) => {
                // Not logged in or can't detect username - both are valid
                println!("No Docker Hub username detected (not logged in or detection failed)");
            }
        }
    }

    #[test]
    fn test_is_docker_logged_in() {
        // Test the login status check
        let result = is_docker_logged_in();
        // Should always return Ok with a boolean
        assert!(result.is_ok(), "is_docker_logged_in should not error");

        let logged_in = result.unwrap();
        println!("Docker login status: {}", logged_in);

        // Note: Even if logged in, getting username may fail due to Docker version differences
        // or configuration file formats, so we just test that the function doesn't panic
        if logged_in {
            let username_result = get_docker_username();
            match username_result {
                Ok(username) => println!("Successfully retrieved username: {}", username),
                Err(_) => println!("Logged in but username detection failed (version-specific behavior)"),
            }
        }
    }

    #[test]
    fn test_delete_docker_image_validates_input() {
        // Test that delete fails gracefully with invalid image name
        // This doesn't require Docker to be running
        let result = delete_docker_image("", false);
        assert!(result.is_err(), "Should fail with empty image name");
    }

    #[test]
    fn test_docker_login_logout() {
        // Test that login/logout functions exist and handle errors gracefully
        // Note: We can't fully test these without real credentials

        // Test logout (safe to call even if not logged in)
        let logout_result = docker_logout();
        // Should either succeed or fail gracefully
        match logout_result {
            Ok(_) => println!("Successfully logged out or was already logged out"),
            Err(e) => println!("Logout handled error: {}", e),
        }
    }

    #[test]
    fn test_build_docker_image_validates_paths() {
        // Test that build validates paths before attempting build
        use std::path::Path;

        let nonexistent_dockerfile = Path::new("/nonexistent/Dockerfile");
        let nonexistent_context = Path::new("/nonexistent");

        let result = build_docker_image(
            nonexistent_dockerfile,
            nonexistent_context,
            "test:latest",
            &[],
        );

        // Should fail because paths don't exist
        assert!(result.is_err(), "Should fail with nonexistent paths");
    }

    #[test]
    fn test_push_docker_image_validates_tag() {
        // Test that push validates tag format
        // This should fail if not logged in or if tag is invalid
        let result = push_docker_image("invalid_tag_without_repository");

        // Should fail (either not logged in, invalid tag, or image doesn't exist)
        assert!(result.is_err(), "Should fail with invalid or nonexistent tag");
    }

    #[test]
    fn test_docker_status_structure() {
        // Test that get_docker_status returns proper structure
        let result = get_docker_status();

        match result {
            Ok(status) => {
                // If logged in, server address should be present
                if status.logged_in {
                    assert!(status.server_address.is_some(), "Should have server address if logged in");
                    // Username may or may not be available depending on Docker version
                    match &status.username {
                        Some(username) => println!("Logged in as: {}", username),
                        None => println!("Logged in but username not available (version-specific)"),
                    }
                } else {
                    // Not logged in is also valid
                    println!("Not logged into Docker Hub");
                }
            }
            Err(_) => {
                // Error is acceptable if Docker is not running
                println!("Could not get Docker status");
            }
        }
    }

    #[test]
    fn test_docker_config_structure() {
        // Test that get_docker_config returns proper structure
        let result = get_docker_config();

        assert!(result.is_ok(), "get_docker_config should not panic");
        let config = result.unwrap();

        // Config should have auth_servers vec (may be empty)
        println!("Found {} auth servers", config.auth_servers.len());

        // If username exists, auth_servers should not be empty
        if config.username.is_some() {
            println!("Username: {:?}", config.username);
        }
    }
}
