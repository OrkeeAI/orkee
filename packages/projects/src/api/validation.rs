// ABOUTME: Input validation utilities for API request handlers
// ABOUTME: Provides field-specific validation, sanitization, and security checks

use openspec::db::DbError;

/// Maximum size limits for different markdown content types (in bytes)
pub const MAX_PROPOSAL_MARKDOWN_SIZE: usize = 100 * 1024; // 100KB
pub const MAX_TASKS_MARKDOWN_SIZE: usize = 50 * 1024; // 50KB
pub const MAX_DESIGN_MARKDOWN_SIZE: usize = 200 * 1024; // 200KB
pub const MAX_DELTA_MARKDOWN_SIZE: usize = 100 * 1024; // 100KB
pub const MAX_CAPABILITY_NAME_SIZE: usize = 500; // 500 chars

/// Maximum size for user identifiers
pub const MAX_USER_ID_SIZE: usize = 255;

/// Minimum size for required markdown content (after trimming)
pub const MIN_MARKDOWN_SIZE: usize = 10;

/// Validate and sanitize proposal markdown
pub fn validate_proposal_markdown(content: &str) -> Result<String, DbError> {
    validate_markdown_field(
        content,
        "Proposal markdown",
        MAX_PROPOSAL_MARKDOWN_SIZE,
        true,
    )
}

/// Validate and sanitize tasks markdown
pub fn validate_tasks_markdown(content: &str) -> Result<String, DbError> {
    validate_markdown_field(content, "Tasks markdown", MAX_TASKS_MARKDOWN_SIZE, true)
}

/// Validate and sanitize optional design markdown
pub fn validate_design_markdown(content: Option<&str>) -> Result<Option<String>, DbError> {
    match content {
        Some(c) => {
            let trimmed = c.trim();
            // If empty after trimming, treat as None
            if trimmed.is_empty() {
                return Ok(None);
            }

            let validated =
                validate_markdown_field(c, "Design markdown", MAX_DESIGN_MARKDOWN_SIZE, false)?;
            Ok(Some(validated))
        }
        None => Ok(None),
    }
}

/// Validate and sanitize delta markdown
pub fn validate_delta_markdown(content: &str) -> Result<String, DbError> {
    validate_markdown_field(content, "Delta markdown", MAX_DELTA_MARKDOWN_SIZE, true)
}

/// Validate capability name
pub fn validate_capability_name(name: &str) -> Result<String, DbError> {
    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(DbError::InvalidInput(
            "Capability name cannot be empty".to_string(),
        ));
    }

    if trimmed.len() > MAX_CAPABILITY_NAME_SIZE {
        return Err(DbError::InvalidInput(format!(
            "Capability name exceeds maximum size of {} characters (got {} characters)",
            MAX_CAPABILITY_NAME_SIZE,
            trimmed.len()
        )));
    }

    // Check for null bytes which could cause issues
    if trimmed.contains('\0') {
        return Err(DbError::InvalidInput(
            "Capability name contains invalid null bytes".to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

/// Validate user ID format and size
pub fn validate_user_id(user_id: &str) -> Result<String, DbError> {
    let trimmed = user_id.trim();

    if trimmed.is_empty() {
        return Err(DbError::InvalidInput("User ID cannot be empty".to_string()));
    }

    if trimmed.len() > MAX_USER_ID_SIZE {
        return Err(DbError::InvalidInput(format!(
            "User ID exceeds maximum size of {} characters (got {} characters)",
            MAX_USER_ID_SIZE,
            trimmed.len()
        )));
    }

    // Check for null bytes
    if trimmed.contains('\0') {
        return Err(DbError::InvalidInput(
            "User ID contains invalid null bytes".to_string(),
        ));
    }

    // Check for path traversal attempts
    if trimmed.contains("..") || trimmed.contains('/') || trimmed.contains('\\') {
        return Err(DbError::InvalidInput(
            "User ID contains invalid characters".to_string(),
        ));
    }

    Ok(trimmed.to_string())
}

/// Validate markdown field with size limits and sanitization
fn validate_markdown_field(
    content: &str,
    field_name: &str,
    max_size: usize,
    required: bool,
) -> Result<String, DbError> {
    // Trim leading and trailing whitespace
    let trimmed = content.trim();

    // Check if content is required
    if required && trimmed.is_empty() {
        return Err(DbError::InvalidInput(format!(
            "{} cannot be empty",
            field_name
        )));
    }

    // Check minimum size for required fields
    if required && trimmed.len() < MIN_MARKDOWN_SIZE {
        return Err(DbError::InvalidInput(format!(
            "{} is too short (minimum {} characters, got {} characters)",
            field_name,
            MIN_MARKDOWN_SIZE,
            trimmed.len()
        )));
    }

    // Check maximum size (using byte count to prevent DoS)
    if trimmed.len() > max_size {
        return Err(DbError::InvalidInput(format!(
            "{} exceeds maximum size of {} bytes (got {} bytes). Consider splitting into multiple sections.",
            field_name,
            max_size,
            trimmed.len()
        )));
    }

    // Check for null bytes which could cause database issues
    if trimmed.contains('\0') {
        return Err(DbError::InvalidInput(format!(
            "{} contains invalid null bytes",
            field_name
        )));
    }

    Ok(trimmed.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_proposal_markdown_valid() {
        let content = "# Proposal\n\nThis is a valid proposal with enough content.";
        assert!(validate_proposal_markdown(content).is_ok());
    }

    #[test]
    fn test_validate_proposal_markdown_empty() {
        assert!(validate_proposal_markdown("").is_err());
        assert!(validate_proposal_markdown("   ").is_err());
    }

    #[test]
    fn test_validate_proposal_markdown_too_short() {
        assert!(validate_proposal_markdown("short").is_err());
    }

    #[test]
    fn test_validate_proposal_markdown_too_large() {
        let large_content = "a".repeat(MAX_PROPOSAL_MARKDOWN_SIZE + 1);
        assert!(validate_proposal_markdown(&large_content).is_err());
    }

    #[test]
    fn test_validate_proposal_markdown_null_bytes() {
        let content = "Valid content\0with null byte";
        assert!(validate_proposal_markdown(content).is_err());
    }

    #[test]
    fn test_validate_proposal_markdown_trims_whitespace() {
        let content = "  # Proposal\n\nContent with whitespace  ";
        let result = validate_proposal_markdown(content).unwrap();
        assert!(!result.starts_with(' '));
        assert!(!result.ends_with(' '));
    }

    #[test]
    fn test_validate_user_id_valid() {
        assert!(validate_user_id("user-123").is_ok());
        assert!(validate_user_id("default-user").is_ok());
    }

    #[test]
    fn test_validate_user_id_empty() {
        assert!(validate_user_id("").is_err());
        assert!(validate_user_id("   ").is_err());
    }

    #[test]
    fn test_validate_user_id_path_traversal() {
        assert!(validate_user_id("../admin").is_err());
        assert!(validate_user_id("user/admin").is_err());
        assert!(validate_user_id("user\\admin").is_err());
    }

    #[test]
    fn test_validate_user_id_null_bytes() {
        assert!(validate_user_id("user\0id").is_err());
    }

    #[test]
    fn test_validate_capability_name_valid() {
        assert!(validate_capability_name("User Authentication").is_ok());
    }

    #[test]
    fn test_validate_capability_name_empty() {
        assert!(validate_capability_name("").is_err());
        assert!(validate_capability_name("   ").is_err());
    }

    #[test]
    fn test_validate_capability_name_too_large() {
        let large_name = "a".repeat(MAX_CAPABILITY_NAME_SIZE + 1);
        assert!(validate_capability_name(&large_name).is_err());
    }

    #[test]
    fn test_validate_design_markdown_optional() {
        assert!(validate_design_markdown(None).unwrap().is_none());
        assert!(validate_design_markdown(Some("  ")).unwrap().is_none());
    }

    #[test]
    fn test_validate_design_markdown_with_content() {
        let content = "# Design\n\nThis is design documentation.";
        let result = validate_design_markdown(Some(content)).unwrap();
        assert!(result.is_some());
    }
}
