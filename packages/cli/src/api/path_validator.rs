use crate::config::{Config, SandboxMode};
use std::fs;
use std::path::{Component, Path, PathBuf};
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug)]
pub struct PathValidator {
    allowed_paths: Vec<PathBuf>,
    blocked_paths: Vec<PathBuf>,
    sandbox_mode: SandboxMode,
}

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Path traversal attempt detected")]
    PathTraversal,
    #[error("Root directory access denied")]
    RootAccess,
    #[error("Access denied to blocked path: {0}")]
    BlockedPath(String),
    #[error("Access denied to sensitive directory: {0}")]
    SensitiveDirectory(String),
    #[error("Path is outside allowed directories")]
    NotInAllowedPaths,
    #[error("Symlink error")]
    SymlinkError,
    #[error("Invalid or inaccessible path")]
    InvalidPath,
    #[error("Path does not exist")]
    PathDoesNotExist,
    #[error("Path expansion failed")]
    ExpansionError,
}

impl PathValidator {
    pub fn new(config: &Config) -> Self {
        // System directories that should ALWAYS be blocked
        const SYSTEM_BLOCKED: &[&str] = &[
            "/etc",
            "/private/etc", // macOS has /etc -> private/etc symlink
            "/sys",
            "/proc",
            "/dev",
            "/boot",
            "/root",
            "/usr/bin",
            "/usr/sbin",
            "/bin",
            "/sbin",
            "/var/log",
            "/var/run",
            "/var/lock",
            "/mnt",
            "/media",
            "/opt",
            "/tmp",
            "/var/tmp", // Block temp directories in relaxed mode
            "C:\\Windows",
            "C:\\Program Files",
            "C:\\Program Files (x86)",
            "C:\\ProgramData",
            "C:\\System32",
        ];

        // Sensitive user directories to block (relative to home)
        const USER_BLOCKED_RELATIVE: &[&str] = &[
            ".ssh",
            ".aws",
            ".gnupg",
            ".docker",
            ".kube",
            ".config/git",
            ".npm",
            ".cargo/credentials",
            ".gitconfig",
            "Library/Keychains",       // macOS
            "AppData/Local/Microsoft", // Windows
            ".env",
            ".env.local",
            ".env.production",
        ];

        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();

        // Expand allowed paths
        let allowed_paths: Vec<PathBuf> = config
            .allowed_browse_paths
            .iter()
            .filter_map(|path| match Self::expand_path_static(path, &home_dir) {
                Ok(expanded) => Some(expanded),
                Err(e) => {
                    warn!("Failed to expand allowed path '{}': {:?}", path, e);
                    None
                }
            })
            .collect();

        // Build blocked paths list
        let mut blocked_paths: Vec<PathBuf> = SYSTEM_BLOCKED.iter().map(PathBuf::from).collect();

        // Add user-specific blocked paths
        if !home_dir.is_empty() {
            for blocked in USER_BLOCKED_RELATIVE {
                blocked_paths.push(PathBuf::from(&home_dir).join(blocked));
            }
        }

        debug!(
            "PathValidator initialized with {} allowed paths, {} blocked paths, mode: {:?}",
            allowed_paths.len(),
            blocked_paths.len(),
            config.browse_sandbox_mode
        );

        Self {
            allowed_paths,
            blocked_paths,
            sandbox_mode: config.browse_sandbox_mode.clone(),
        }
    }

    pub fn validate_path(&self, path: &str) -> Result<PathBuf, ValidationError> {
        debug!("Validating path: {}", path);

        // Step 1: Expand path (handle ~, etc.)
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();
        let expanded = Self::expand_path_static(path, &home_dir)?;

        // Step 2: In strict mode, check allowed paths first
        if self.sandbox_mode == SandboxMode::Strict {
            self.check_allowed_paths(&expanded)?;
        }

        // Step 3: Check against blocked paths
        self.check_blocked_paths(&expanded)?;

        // Step 4: Basic path safety checks (but allow root access for allowed paths in strict mode)
        self.check_path_components(&expanded)?;

        // Step 5: Verify path exists and is accessible
        if !expanded.exists() {
            return Err(ValidationError::PathDoesNotExist);
        }

        // Step 6: Check for symlinks pointing outside sandbox
        self.check_symlinks(&expanded)?;

        debug!("Path validation successful: {}", expanded.display());
        Ok(expanded)
    }

    pub fn would_allow_subdirectory(&self, path: &Path) -> bool {
        // Quick check if a subdirectory would be allowed without full validation
        self.validate_path(&path.to_string_lossy()).is_ok()
    }

    fn expand_path_static(path: &str, home_dir: &str) -> Result<PathBuf, ValidationError> {
        let expanded = if path.starts_with("~/") && !home_dir.is_empty() {
            path.replacen("~", home_dir, 1)
        } else {
            path.to_string()
        };

        let path_buf = PathBuf::from(expanded);

        // Canonicalize if possible, but don't fail if path doesn't exist yet
        match path_buf.canonicalize() {
            Ok(canonical) => Ok(canonical),
            Err(_) => {
                // If canonicalization fails, return the expanded path
                // This allows for paths that don't exist yet
                Ok(path_buf)
            }
        }
    }

    fn check_path_components(&self, path: &Path) -> Result<(), ValidationError> {
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    // Allow parent dir navigation only in relaxed/disabled modes
                    if self.sandbox_mode == SandboxMode::Strict {
                        return Err(ValidationError::PathTraversal);
                    }
                }
                Component::RootDir => {
                    // In strict mode, root access is only blocked if the path is NOT in allowed paths
                    // Since we check allowed paths first in strict mode, if we reach here, it means
                    // the path was approved or we're not in strict mode
                    if self.sandbox_mode == SandboxMode::Strict {
                        // If we reached here, the path was already approved by check_allowed_paths
                        // so root access is OK
                    }
                }
                Component::Normal(name) => {
                    if let Some(name_str) = name.to_str() {
                        if self.is_sensitive_directory(name_str) {
                            return Err(ValidationError::SensitiveDirectory(name_str.to_string()));
                        }
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn check_blocked_paths(&self, path: &Path) -> Result<(), ValidationError> {
        // In disabled mode, don't block any paths
        if self.sandbox_mode == SandboxMode::Disabled {
            return Ok(());
        }

        // If path is explicitly allowed, don't block it
        for allowed in &self.allowed_paths {
            if path.starts_with(allowed) {
                return Ok(());
            }
        }

        // In relaxed mode, be more permissive with temp directories
        if self.sandbox_mode == SandboxMode::Relaxed {
            // Allow temp directories unless they contain sensitive patterns
            if path.starts_with("/tmp") || path.starts_with("/var/tmp") {
                // Check for sensitive patterns in the path components
                for component in path.components() {
                    if let std::path::Component::Normal(name) = component {
                        if let Some(name_str) = name.to_str() {
                            if self.is_sensitive_directory(name_str) {
                                return Err(ValidationError::SensitiveDirectory(
                                    name_str.to_string(),
                                ));
                            }
                        }
                    }
                }
                return Ok(());
            }
        }

        // Check blocked paths for strict mode and remaining relaxed mode paths
        for blocked in &self.blocked_paths {
            if path.starts_with(blocked) {
                return Err(ValidationError::BlockedPath(blocked.display().to_string()));
            }
        }
        Ok(())
    }

    fn check_allowed_paths(&self, path: &Path) -> Result<(), ValidationError> {
        // In strict mode, path must be under one of the allowed paths
        for allowed in &self.allowed_paths {
            if path.starts_with(allowed) {
                return Ok(());
            }
        }

        Err(ValidationError::NotInAllowedPaths)
    }

    fn check_symlinks(&self, path: &Path) -> Result<(), ValidationError> {
        // If sandbox is disabled, don't check symlinks
        if self.sandbox_mode == SandboxMode::Disabled {
            return Ok(());
        }

        if path.is_symlink() {
            let target = fs::read_link(path).map_err(|_| ValidationError::SymlinkError)?;

            // Recursively validate the symlink target
            self.validate_path(&target.to_string_lossy())?;
        }

        Ok(())
    }

    fn is_sensitive_directory(&self, name: &str) -> bool {
        const SENSITIVE_PATTERNS: &[&str] = &[
            ".ssh",
            ".aws",
            ".gnupg",
            ".docker",
            ".kube",
            ".git",
            ".svn",
            ".hg",
            ".env",
            ".credentials",
            ".config",
            "node_modules",
            ".npm",
            ".yarn",
            "__pycache__",
            ".pytest_cache",
        ];

        // Check for exact matches and patterns
        for pattern in SENSITIVE_PATTERNS {
            if name == *pattern || name.starts_with(&format!("{}.", pattern)) {
                return true;
            }
        }

        // Additional checks for hidden system files
        if name.starts_with('.')
            && (name.ends_with("_history")
                || name.ends_with("_token")
                || name.contains("secret")
                || name.contains("key")
                || name.contains("password"))
        {
            return true;
        }

        false
    }

    pub fn get_safe_default_path(&self) -> String {
        // Return a safe default path to start browsing from
        let home_dir = std::env::var("HOME")
            .or_else(|_| std::env::var("USERPROFILE"))
            .unwrap_or_default();

        // Try to use the first allowed path as default
        if !self.allowed_paths.is_empty() {
            return self.allowed_paths[0].to_string_lossy().to_string();
        }

        // Fallback to Documents if it exists
        if !home_dir.is_empty() {
            let documents = PathBuf::from(&home_dir).join("Documents");
            if documents.exists() {
                return documents.to_string_lossy().to_string();
            }
        }

        // Last resort: use home directory or temp
        if !home_dir.is_empty() {
            home_dir
        } else {
            "/tmp".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SandboxMode;
    use tempfile::tempdir;

    fn create_test_config(mode: SandboxMode, allowed: Vec<&str>) -> Config {
        Config {
            port: 4001,
            cors_origin: "http://localhost:5173".to_string(),
            cors_allow_any_localhost: true,
            allowed_browse_paths: allowed.iter().map(|s| s.to_string()).collect(),
            browse_sandbox_mode: mode,
            rate_limit: crate::middleware::RateLimitConfig::default(),
            security_headers_enabled: true,
            enable_hsts: false,
            enable_request_id: true,
            tls: crate::tls::TlsConfig {
                enabled: false,
                cert_path: "/tmp/cert.pem".into(),
                key_path: "/tmp/key.pem".into(),
                auto_generate: false,
            },
        }
    }

    #[test]
    fn test_strict_mode_blocks_unauthorized_paths() {
        let temp_dir = tempdir().unwrap();
        let allowed_path = temp_dir.path().to_str().unwrap();

        let config = create_test_config(SandboxMode::Strict, vec![allowed_path]);
        let validator = PathValidator::new(&config);

        // Should allow access to allowed path (which exists as a temp directory)
        let result = validator.validate_path(allowed_path);
        if let Err(e) = &result {
            eprintln!(
                "Validation failed for allowed path '{}': {:?}",
                allowed_path, e
            );
        }
        assert!(
            result.is_ok(),
            "Should allow access to explicitly allowed temp dir"
        );

        // Test with a non-existent path that should fail
        let result = validator.validate_path("/definitely/nonexistent/path/12345");
        assert!(result.is_err(), "Should block access to nonexistent paths");

        // If /etc exists, it should be blocked
        if std::path::Path::new("/etc").exists() {
            let result = validator.validate_path("/etc");
            assert!(result.is_err(), "Should block access to /etc");
        }
    }

    #[test]
    fn test_relaxed_mode_blocks_sensitive_paths() {
        let config = create_test_config(SandboxMode::Relaxed, vec![]);
        let validator = PathValidator::new(&config);

        // Test that blocked paths are properly configured
        assert!(
            !validator.blocked_paths.is_empty(),
            "Should have blocked paths configured"
        );

        // Create a temporary directory to test with
        let temp_dir = tempdir().unwrap();
        let temp_path = temp_dir.path().to_str().unwrap();

        // This should work in relaxed mode (temp directory)
        let result = validator.validate_path(temp_path);
        assert!(
            result.is_ok(),
            "Should allow temp directory in relaxed mode"
        );

        // Test with clearly blocked paths
        let blocked_tests = vec!["/etc", "/sys", "/proc", "/usr/bin"];

        for blocked_path in blocked_tests {
            if std::path::Path::new(blocked_path).exists() {
                let result = validator.validate_path(blocked_path);
                if result.is_ok() {
                    eprintln!(
                        "ERROR: Path '{}' was allowed but should be blocked!",
                        blocked_path
                    );
                    eprintln!("Blocked paths: {:?}", validator.blocked_paths);
                }
                assert!(
                    result.is_err(),
                    "Should block system path: {}",
                    blocked_path
                );
            }
        }

        // Test with home-based sensitive directories
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let sensitive_paths = vec![
                format!("{}/.ssh", home),
                format!("{}/.aws", home),
                format!("{}/.gnupg", home),
            ];

            for sensitive_path in sensitive_paths {
                let result = validator.validate_path(&sensitive_path);
                // Should fail either because it's blocked OR because it doesn't exist
                assert!(
                    result.is_err(),
                    "Should block or fail for sensitive path: {}",
                    sensitive_path
                );
            }
        }
    }

    #[test]
    fn test_path_traversal_detection() {
        let config = create_test_config(SandboxMode::Strict, vec!["/tmp"]);
        let validator = PathValidator::new(&config);

        // Should detect path traversal attempts
        let result = validator.validate_path("/tmp/../etc/passwd");
        assert!(result.is_err());
    }

    #[test]
    fn test_tilde_expansion() {
        let config = create_test_config(SandboxMode::Relaxed, vec!["~/Documents"]);
        let validator = PathValidator::new(&config);

        // Should expand tilde properly
        let home = std::env::var("HOME").unwrap_or_default();
        if !home.is_empty() {
            let expected_path = format!("{}/Documents", home);
            assert!(validator
                .allowed_paths
                .iter()
                .any(|p| p.to_string_lossy() == expected_path));
        }
    }

    #[test]
    fn test_disabled_mode_allows_most_paths() {
        let temp_dir = tempdir().unwrap();
        let test_path = temp_dir.path().to_str().unwrap();

        let config = create_test_config(SandboxMode::Disabled, vec![]);
        let validator = PathValidator::new(&config);

        // Should allow access to most paths when disabled
        assert!(validator.validate_path(test_path).is_ok());
    }
}
