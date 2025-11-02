// ABOUTME: Input validation for system settings
// ABOUTME: Type-specific validation rules with security checks

use orkee_storage::StorageError;
use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ValidationError {
    #[error("Invalid boolean value: {0}. Must be 'true' or 'false'")]
    InvalidBoolean(String),

    #[error("Invalid integer value: {0}. {1}")]
    InvalidInteger(String, String),

    #[error("Invalid port number: {0}. Must be between 1 and 65535")]
    InvalidPort(String),

    #[error("Invalid enum value: {0}. Must be one of: {1}")]
    InvalidEnum(String, String),

    #[error("Invalid path: {0}. {1}")]
    InvalidPath(String, String),

    #[error("Value cannot be empty")]
    EmptyValue,

    #[error("Unknown setting key: {0}")]
    UnknownKey(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

impl From<ValidationError> for StorageError {
    fn from(err: ValidationError) -> Self {
        StorageError::Validation(err.to_string())
    }
}

/// Validate a setting value based on its key and data type
pub fn validate_setting_value(
    key: &str,
    value: &str,
    data_type: &str,
) -> Result<(), ValidationError> {
    // Check for empty value
    if value.is_empty() {
        return Err(ValidationError::EmptyValue);
    }

    // First validate by data type
    match data_type {
        "boolean" => validate_boolean(value)?,
        "integer" => {
            // Parse as i64 for basic validation
            validate_integer(value, None, None)?;
        }
        "string" => {
            // String type passes basic validation, specific rules below
        }
        _ => {
            // Unknown data type, but allow it (forward compatibility)
        }
    }

    // Then apply setting-specific validation rules
    match key {
        // Port settings (env-only, but we still validate for safety)
        "api_port" | "ui_port" => validate_port(value)?,

        // Boolean settings (already validated by data type)
        "dev_mode"
        | "cloud_enabled"
        | "cors_allow_any_localhost"
        | "tls_enabled"
        | "auto_generate_cert"
        | "rate_limit_enabled"
        | "security_headers_enabled"
        | "enable_hsts"
        | "enable_request_id"
        | "telemetry_enabled" => {
            // Already validated as boolean above
        }

        // Enum settings
        "browse_sandbox_mode" => {
            validate_enum(value, &["strict", "relaxed", "disabled"])?;
        }

        // Path settings
        "tls_cert_path" | "tls_key_path" => {
            validate_path(value)?;
        }

        // Path list settings
        "allowed_browse_paths" => {
            validate_path_list(value)?;
        }

        // Rate limit settings (must be >= 1, max 10,000 to prevent misconfiguration)
        "rate_limit_health_rpm"
        | "rate_limit_browse_rpm"
        | "rate_limit_projects_rpm"
        | "rate_limit_preview_rpm"
        | "rate_limit_ai_rpm"
        | "rate_limit_global_rpm"
        | "rate_limit_burst_size" => {
            validate_integer(value, Some(1), Some(10_000))?;
        }

        // URL settings
        "cloud_api_url" => {
            validate_url(value)?;
        }

        // Unknown setting key - allow it (forward compatibility)
        _ => {}
    }

    Ok(())
}

/// Validate boolean value
fn validate_boolean(value: &str) -> Result<(), ValidationError> {
    match value {
        "true" | "false" => Ok(()),
        _ => Err(ValidationError::InvalidBoolean(value.to_string())),
    }
}

/// Validate integer value with optional min/max bounds
fn validate_integer(
    value: &str,
    min: Option<i64>,
    max: Option<i64>,
) -> Result<(), ValidationError> {
    let parsed = value.parse::<i64>().map_err(|_| {
        ValidationError::InvalidInteger(value.to_string(), "Not a valid integer".to_string())
    })?;

    if let Some(min_val) = min {
        if parsed < min_val {
            return Err(ValidationError::InvalidInteger(
                value.to_string(),
                format!("Must be >= {}", min_val),
            ));
        }
    }

    if let Some(max_val) = max {
        if parsed > max_val {
            return Err(ValidationError::InvalidInteger(
                value.to_string(),
                format!("Must be <= {}", max_val),
            ));
        }
    }

    Ok(())
}

/// Validate port number (1-65535)
fn validate_port(value: &str) -> Result<(), ValidationError> {
    validate_integer(value, Some(1), Some(65535))
        .map_err(|_| ValidationError::InvalidPort(value.to_string()))
}

/// Validate enum value
fn validate_enum(value: &str, allowed: &[&str]) -> Result<(), ValidationError> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(ValidationError::InvalidEnum(
            value.to_string(),
            allowed.join(", "),
        ))
    }
}

/// Validate file path (basic checks, allows tilde expansion)
fn validate_path(value: &str) -> Result<(), ValidationError> {
    // Allow empty path (will be handled by application logic)
    if value.is_empty() {
        return Ok(());
    }

    // Basic path validation - check for obvious path traversal
    if value.contains("..") {
        return Err(ValidationError::InvalidPath(
            value.to_string(),
            "Path traversal detected".to_string(),
        ));
    }

    // Try to parse as PathBuf (basic validity check)
    let path = PathBuf::from(value);

    // For security-sensitive paths, check for symlinks and ensure no absolute path escalation
    // Note: This only validates expandable paths (non-tilde paths that exist)
    // Tilde paths (~/) are validated at runtime after expansion
    if !value.starts_with('~') && path.exists() {
        // Check if path is a symlink
        if path.is_symlink() {
            return Err(ValidationError::InvalidPath(
                value.to_string(),
                "Symlinks are not allowed for security reasons".to_string(),
            ));
        }

        // Verify path doesn't escape outside intended boundaries via canonicalization
        if let Ok(canonical) = path.canonicalize() {
            // Check if canonicalized path points to sensitive system directories
            let canonical_str = canonical.to_string_lossy();
            let sensitive_prefixes = ["/etc", "/sys", "/proc", "/dev", "/boot"];
            if sensitive_prefixes
                .iter()
                .any(|prefix| canonical_str.starts_with(prefix))
            {
                return Err(ValidationError::InvalidPath(
                    value.to_string(),
                    "Path resolves to sensitive system directory".to_string(),
                ));
            }
        }
    }

    Ok(())
}

/// Validate comma-separated list of paths
fn validate_path_list(value: &str) -> Result<(), ValidationError> {
    if value.is_empty() {
        return Ok(());
    }

    for path in value.split(',') {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            validate_path(trimmed)?;
        }
    }

    Ok(())
}

/// Validate URL (basic check)
fn validate_url(value: &str) -> Result<(), ValidationError> {
    // Basic URL validation - must start with http:// or https://
    if !value.starts_with("http://") && !value.starts_with("https://") {
        return Err(ValidationError::InvalidUrl(
            "URL must start with http:// or https://".to_string(),
        ));
    }

    // Check for spaces (invalid in URLs)
    if value.contains(' ') {
        return Err(ValidationError::InvalidUrl(
            "URL cannot contain spaces".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_boolean_valid() {
        assert!(validate_boolean("true").is_ok());
        assert!(validate_boolean("false").is_ok());
    }

    #[test]
    fn test_validate_boolean_invalid() {
        assert!(validate_boolean("yes").is_err());
        assert!(validate_boolean("no").is_err());
        assert!(validate_boolean("1").is_err());
        assert!(validate_boolean("0").is_err());
        assert!(validate_boolean("True").is_err());
        assert!(validate_boolean("FALSE").is_err());
    }

    #[test]
    fn test_validate_integer() {
        assert!(validate_integer("123", None, None).is_ok());
        assert!(validate_integer("0", None, None).is_ok());
        assert!(validate_integer("-1", None, None).is_ok());
        assert!(validate_integer("abc", None, None).is_err());
    }

    #[test]
    fn test_validate_integer_with_min() {
        assert!(validate_integer("5", Some(1), None).is_ok());
        assert!(validate_integer("1", Some(1), None).is_ok());
        assert!(validate_integer("0", Some(1), None).is_err());
        assert!(validate_integer("-1", Some(1), None).is_err());
    }

    #[test]
    fn test_validate_integer_with_max() {
        assert!(validate_integer("5", None, Some(10)).is_ok());
        assert!(validate_integer("10", None, Some(10)).is_ok());
        assert!(validate_integer("11", None, Some(10)).is_err());
    }

    #[test]
    fn test_validate_port() {
        assert!(validate_port("80").is_ok());
        assert!(validate_port("443").is_ok());
        assert!(validate_port("8080").is_ok());
        assert!(validate_port("1").is_ok());
        assert!(validate_port("65535").is_ok());
        assert!(validate_port("0").is_err());
        assert!(validate_port("65536").is_err());
        assert!(validate_port("-1").is_err());
        assert!(validate_port("abc").is_err());
    }

    #[test]
    fn test_validate_enum() {
        let allowed = vec!["strict", "relaxed", "disabled"];
        assert!(validate_enum("strict", &allowed).is_ok());
        assert!(validate_enum("relaxed", &allowed).is_ok());
        assert!(validate_enum("disabled", &allowed).is_ok());
        assert!(validate_enum("invalid", &allowed).is_err());
        assert!(validate_enum("STRICT", &allowed).is_err());
    }

    #[test]
    fn test_validate_path() {
        assert!(validate_path("/home/user/file.txt").is_ok());
        assert!(validate_path("~/Documents").is_ok());
        assert!(validate_path("relative/path").is_ok());
        assert!(validate_path("../../../etc/passwd").is_err());
    }

    #[test]
    fn test_validate_path_list() {
        assert!(validate_path_list("~/Documents,~/Projects,~/Desktop").is_ok());
        assert!(validate_path_list("/home/user,/var/www").is_ok());
        assert!(validate_path_list("").is_ok());
        assert!(validate_path_list("~/Documents,../../../etc").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://api.orkee.ai").is_ok());
        assert!(validate_url("http://localhost:3000").is_ok());
        assert!(validate_url("https://example.com/path").is_ok());
        assert!(validate_url("invalid-url").is_err());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("https://example.com/path with spaces").is_err());
    }

    #[test]
    fn test_validate_setting_value_port() {
        assert!(validate_setting_value("api_port", "4001", "integer").is_ok());
        assert!(validate_setting_value("ui_port", "5173", "integer").is_ok());
        assert!(validate_setting_value("api_port", "0", "integer").is_err());
        assert!(validate_setting_value("api_port", "65536", "integer").is_err());
    }

    #[test]
    fn test_validate_setting_value_boolean() {
        assert!(validate_setting_value("dev_mode", "true", "boolean").is_ok());
        assert!(validate_setting_value("dev_mode", "false", "boolean").is_ok());
        assert!(validate_setting_value("dev_mode", "yes", "boolean").is_err());
    }

    #[test]
    fn test_validate_setting_value_enum() {
        assert!(validate_setting_value("browse_sandbox_mode", "strict", "string").is_ok());
        assert!(validate_setting_value("browse_sandbox_mode", "relaxed", "string").is_ok());
        assert!(validate_setting_value("browse_sandbox_mode", "disabled", "string").is_ok());
        assert!(validate_setting_value("browse_sandbox_mode", "invalid", "string").is_err());
    }

    #[test]
    fn test_validate_setting_value_rate_limit() {
        assert!(validate_setting_value("rate_limit_health_rpm", "60", "integer").is_ok());
        assert!(validate_setting_value("rate_limit_health_rpm", "1", "integer").is_ok());
        assert!(validate_setting_value("rate_limit_health_rpm", "0", "integer").is_err());
        assert!(validate_setting_value("rate_limit_health_rpm", "-1", "integer").is_err());
    }

    #[test]
    fn test_validate_setting_value_empty() {
        assert!(validate_setting_value("api_port", "", "integer").is_err());
    }

    #[test]
    fn test_validate_setting_value_url() {
        assert!(validate_setting_value("cloud_api_url", "https://api.orkee.ai", "string").is_ok());
        assert!(validate_setting_value("cloud_api_url", "invalid-url", "string").is_err());
    }
}
