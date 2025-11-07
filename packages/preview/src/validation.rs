// ABOUTME: Validation utilities for preview server operations
// ABOUTME: Provides input validation to prevent security issues like path traversal

/// Validates a project ID for security
///
/// Checks that the project ID:
/// - Is not empty
/// - Does not contain path traversal sequences (..)
/// - Does not contain path separators (/ or \)
///
/// # Arguments
///
/// * `project_id` - The project ID string to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, or `Err(String)` with an error message if invalid.
///
/// # Examples
///
/// ```
/// use orkee_preview::validation::validate_project_id;
///
/// assert!(validate_project_id("proj-123").is_ok());
/// assert!(validate_project_id("").is_err());
/// assert!(validate_project_id("../etc/passwd").is_err());
/// ```
pub fn validate_project_id(project_id: &str) -> Result<(), String> {
    // Check for empty project_id
    if project_id.is_empty() {
        return Err("Project ID cannot be empty".to_string());
    }

    // Check for path traversal sequences
    if project_id.contains("..") {
        return Err(format!(
            "Invalid project ID '{}': contains path traversal sequence",
            project_id
        ));
    }

    // Check for path separators (both Unix and Windows)
    if project_id.contains('/') || project_id.contains('\\') {
        return Err(format!(
            "Invalid project ID '{}': contains path separator",
            project_id
        ));
    }

    // Check for null bytes (can cause security issues in some contexts)
    if project_id.contains('\0') {
        return Err(format!(
            "Invalid project ID '{}': contains null byte",
            project_id
        ));
    }

    // Check for newlines or control characters (can cause log injection)
    if project_id.chars().any(|c| c.is_control()) {
        return Err(format!(
            "Invalid project ID '{}': contains control characters",
            project_id
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_project_ids() {
        assert!(validate_project_id("proj-123").is_ok());
        assert!(validate_project_id("project_name").is_ok());
        assert!(validate_project_id("abc123").is_ok());
        assert!(validate_project_id("my-project-2024").is_ok());
    }

    #[test]
    fn test_empty_project_id() {
        assert!(validate_project_id("").is_err());
    }

    #[test]
    fn test_path_traversal() {
        assert!(validate_project_id("..").is_err());
        assert!(validate_project_id("../etc").is_err());
        assert!(validate_project_id("etc/..").is_err());
        assert!(validate_project_id("a/../b").is_err());
    }

    #[test]
    fn test_path_separators() {
        assert!(validate_project_id("/etc/passwd").is_err());
        assert!(validate_project_id("C:\\Windows").is_err());
        assert!(validate_project_id("a/b").is_err());
        assert!(validate_project_id("a\\b").is_err());
    }

    #[test]
    fn test_null_bytes() {
        assert!(validate_project_id("test\0null").is_err());
        assert!(validate_project_id("\0").is_err());
    }

    #[test]
    fn test_control_characters() {
        assert!(validate_project_id("test\nline").is_err());
        assert!(validate_project_id("test\rcarriage").is_err());
        assert!(validate_project_id("test\ttab").is_err());
    }
}
