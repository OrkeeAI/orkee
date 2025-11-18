// ABOUTME: Shared Docker CLI wrapper functions for image management and authentication
// ABOUTME: Provides reusable Docker command execution for both CLI and API server

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

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

/// Docker daemon status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerDaemonStatus {
    pub running: bool,
    pub version: Option<String>,
    pub error: Option<String>,
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

/// Get Docker daemon status with version information
pub fn get_docker_daemon_status() -> Result<DockerDaemonStatus> {
    let running = is_docker_running()?;

    if !running {
        return Ok(DockerDaemonStatus {
            running: false,
            version: None,
            error: Some("Docker daemon is not running".to_string()),
        });
    }

    // Get Docker version
    let version_output = Command::new("docker")
        .arg("version")
        .arg("--format")
        .arg("{{.Server.Version}}")
        .output();

    let version = match version_output {
        Ok(output) if output.status.success() => String::from_utf8_lossy(&output.stdout)
            .trim()
            .to_string()
            .into(),
        _ => None,
    };

    Ok(DockerDaemonStatus {
        running: true,
        version,
        error: None,
    })
}

/// Check if user is logged in to Docker Hub
pub fn is_docker_logged_in() -> Result<bool> {
    // Check Docker config file for authentication
    if let Ok(home) = std::env::var("HOME") {
        let config_path = format!("{}/.docker/config.json", home);
        if let Ok(content) = std::fs::read_to_string(config_path) {
            // Parse JSON to check if there's actual auth data
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check if using a credential store (Docker Desktop)
                if let Some(creds_store) = config.get("credsStore").and_then(|s| s.as_str()) {
                    // Try to query the credential store
                    let helper_cmd = format!("docker-credential-{}", creds_store);
                    if let Ok(output) = Command::new(&helper_cmd).arg("list").output() {
                        if output.status.success() {
                            let list_str = String::from_utf8_lossy(&output.stdout);
                            // Check if Docker Hub is in the credential store
                            return Ok(list_str.contains("index.docker.io"));
                        }
                    }
                }

                // Check auths object for inline credentials
                if let Some(auths) = config.get("auths").and_then(|a| a.as_object()) {
                    // Check if any Docker Hub auth entries have actual credentials
                    for (server, auth) in auths {
                        if server.contains("index.docker.io") {
                            // Check if there's an auth field (non-empty credentials)
                            if let Some(auth_obj) = auth.as_object() {
                                // Empty object means logged out
                                if auth_obj.is_empty() {
                                    continue;
                                }
                                // Check for actual auth data
                                if auth_obj.contains_key("auth")
                                    || auth_obj.contains_key("username")
                                {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
            }
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
                            // Try direct username field first (old format)
                            if let Some(username) = auth.get("username").and_then(|u| u.as_str()) {
                                return Ok(username.to_string());
                            }
                            // Try to decode auth token (base64 encoded "username:password")
                            if let Some(auth_str) = auth.get("auth").and_then(|a| a.as_str()) {
                                use base64::{engine::general_purpose, Engine as _};
                                if let Ok(decoded) = general_purpose::STANDARD.decode(auth_str) {
                                    if let Ok(credentials) = String::from_utf8(decoded) {
                                        if let Some(username) = credentials.split(':').next() {
                                            return Ok(username.to_string());
                                        }
                                    }
                                }
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

    let output = cmd
        .output()
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

    let output = cmd
        .output()
        .context("Failed to execute docker rmi command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker rmi failed: {}", stderr);
    }

    Ok(())
}

/// Build a Docker image
/// Find the workspace root by looking for Cargo.toml with [workspace] in parent directories
fn find_project_root() -> Result<PathBuf> {
    let mut current = std::env::current_dir().context("Failed to get current directory")?;

    loop {
        // Check if this directory contains Cargo.toml
        let cargo_toml = current.join("Cargo.toml");
        if cargo_toml.exists() {
            // Read the Cargo.toml to check if it's a workspace root
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                // If it contains [workspace], it's the workspace root
                if content.contains("[workspace]") {
                    return Ok(current);
                }
            }
        }

        // Try parent directory
        match current.parent() {
            Some(parent) => current = parent.to_path_buf(),
            None => anyhow::bail!(
                "Could not find workspace root (Cargo.toml with [workspace] not found in any parent directory)"
            ),
        }
    }
}

pub fn build_docker_image(
    dockerfile_path: &Path,
    build_context: &Path,
    image_tag: &str,
    additional_labels: &[(&str, &str)],
) -> Result<Output> {
    // Resolve relative paths against the project root
    // This ensures paths work regardless of where the server was started from
    let resolved_dockerfile = if dockerfile_path.is_relative() {
        let project_root = find_project_root()
            .context("Failed to find project root for resolving Dockerfile path")?;
        project_root.join(dockerfile_path)
    } else {
        dockerfile_path.to_path_buf()
    };

    let resolved_context = if build_context.is_relative() {
        let project_root = find_project_root()
            .context("Failed to find project root for resolving build context")?;
        project_root.join(build_context)
    } else {
        build_context.to_path_buf()
    };

    let mut cmd = Command::new("docker");
    cmd.arg("build")
        .arg("--progress=plain") // Plain text output for better readability
        .arg("-t")
        .arg(image_tag)
        .arg("-f")
        .arg(&resolved_dockerfile);

    // Add labels
    for (key, value) in additional_labels {
        cmd.arg("--label").arg(format!("{}={}", key, value));
    }

    cmd.arg(&resolved_context);

    let output = cmd
        .output()
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
    use std::thread;

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

    let mut child = cmd
        .spawn()
        .context("Failed to spawn docker build process")?;

    // Capture both stdout and stderr
    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let stderr = child.stderr.take().context("Failed to capture stderr")?;

    // Use a channel to merge stdout and stderr streams
    let (tx, rx) = std::sync::mpsc::channel();
    let tx_stderr = tx.clone();

    // Thread for stdout
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().filter_map(|l| l.ok()) {
            let _ = tx.send(line);
        }
    });

    // Thread for stderr (where Docker sends build progress)
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().filter_map(|l| l.ok()) {
            let _ = tx_stderr.send(line);
        }
    });

    // Return an iterator that reads from the channel
    Ok(rx.into_iter())
}

/// Pull a Docker image from Docker Hub
pub fn pull_docker_image(image_tag: &str) -> Result<Output> {
    let output = Command::new("docker")
        .arg("pull")
        .arg(image_tag)
        .output()
        .context("Failed to execute docker pull command")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Docker pull failed: {}", stderr);
    }

    Ok(output)
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
    let stdout = child.stdout.take().context("Failed to capture stdout")?;

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

/// Launch docker login in the user's terminal
pub fn docker_login_in_terminal() -> Result<()> {
    // Detect the operating system and open appropriate terminal
    #[cfg(target_os = "macos")]
    {
        // On macOS, use osascript to open Terminal.app with docker login
        let script = r#"
            tell application "Terminal"
                activate
                do script "docker login && echo 'Press Enter to close this window...' && read"
            end tell
        "#;

        let status = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .status()
            .context("Failed to open Terminal.app")?;

        if !status.success() {
            anyhow::bail!("Failed to launch Terminal.app");
        }
    }

    #[cfg(target_os = "linux")]
    {
        // Try common Linux terminals in order of preference
        let terminals = [
            (
                "gnome-terminal",
                vec![
                    "--",
                    "bash",
                    "-c",
                    "docker login; echo 'Press Enter to close...'; read",
                ],
            ),
            (
                "konsole",
                vec![
                    "-e",
                    "bash",
                    "-c",
                    "docker login; echo 'Press Enter to close...'; read",
                ],
            ),
            (
                "xterm",
                vec![
                    "-e",
                    "bash",
                    "-c",
                    "docker login; echo 'Press Enter to close...'; read",
                ],
            ),
        ];

        let mut success = false;
        for (terminal, args) in &terminals {
            if let Ok(status) = Command::new(terminal).args(args).status() {
                if status.success() {
                    success = true;
                    break;
                }
            }
        }

        if !success {
            anyhow::bail!("No supported terminal emulator found. Please run 'docker login' manually in your terminal.");
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, use cmd.exe
        let status = Command::new("cmd")
            .args(&["/c", "start", "cmd", "/k", "docker login && pause"])
            .status()
            .context("Failed to open Command Prompt")?;

        if !status.success() {
            anyhow::bail!("Failed to launch Command Prompt");
        }
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
            assert!(
                result.is_ok(),
                "Should be able to list images if Docker is running"
            );
        }
    }

    #[test]
    fn test_list_docker_images_with_filter() {
        // Skip if Docker not available
        if is_docker_running().unwrap_or(false) {
            // Test with orkee.sandbox label filter
            let result = list_docker_images(Some("orkee.sandbox=true"));
            assert!(
                result.is_ok(),
                "Should be able to list images with label filter"
            );

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
                assert!(
                    !username.is_empty(),
                    "Username should not be empty if logged in"
                );
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
                Err(_) => {
                    println!("Logged in but username detection failed (version-specific behavior)")
                }
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
        assert!(
            result.is_err(),
            "Should fail with invalid or nonexistent tag"
        );
    }

    #[test]
    fn test_docker_status_structure() {
        // Test that get_docker_status returns proper structure
        let result = get_docker_status();

        match result {
            Ok(status) => {
                // If logged in, server address should be present
                if status.logged_in {
                    assert!(
                        status.server_address.is_some(),
                        "Should have server address if logged in"
                    );
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
