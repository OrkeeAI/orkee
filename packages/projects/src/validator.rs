use crate::storage::path_exists;
use crate::types::{ProjectCreateInput, ProjectUpdateInput};

// Validation limits
const MAX_NAME_LENGTH: usize = 100;
const MAX_DESCRIPTION_LENGTH: usize = 1000;
const MAX_TAG_LENGTH: usize = 50;
const MAX_SCRIPT_LENGTH: usize = 500;
const MAX_PATH_LENGTH: usize = 500;

// Validation patterns
const NAME_PATTERN: &str = r"^[a-zA-Z0-9][a-zA-Z0-9_\-\s]{0,98}[a-zA-Z0-9]$";
const TAG_PATTERN: &str = r"^[a-zA-Z0-9_\-]+$";

// Dangerous commands to block in scripts
const DANGEROUS_COMMANDS: &[&str] = &[
    "rm -rf /",
    "rm -rf /*",
    ":(){ :|:& };:", // Fork bomb
    "dd if=/dev/zero",
    "mkfs.",
    "format ",
    "> /dev/sda",
    "chmod -R 777 /",
];

/// Validation errors for project data
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        ValidationError {
            field: field.into(),
            message: message.into(),
        }
    }
}

use regex::Regex;
use std::path::{Component, Path};

fn validate_name(name: &str) -> Result<(), ValidationError> {
    // Check length
    if name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::new(
            "name",
            format!("Name too long (max {} characters)", MAX_NAME_LENGTH),
        ));
    }

    // Check pattern
    let re = Regex::new(NAME_PATTERN).expect("Invalid NAME_PATTERN regex");
    if !re.is_match(name) {
        return Err(ValidationError::new(
            "name",
            "Name must contain only letters, numbers, hyphens, underscores, and spaces",
        ));
    }

    Ok(())
}

fn validate_path_safety(path_str: &str) -> Result<(), ValidationError> {
    // Check length
    if path_str.len() > MAX_PATH_LENGTH {
        return Err(ValidationError::new(
            "projectRoot",
            format!("Path too long (max {} characters)", MAX_PATH_LENGTH),
        ));
    }

    let path = Path::new(path_str);

    // Check for path traversal attempts
    for component in path.components() {
        match component {
            Component::ParentDir => {
                return Err(ValidationError::new(
                    "projectRoot",
                    "Path cannot contain '..' (parent directory references)",
                ));
            }
            Component::Normal(os_str) => {
                if let Some(s) = os_str.to_str() {
                    // Check for hidden directories in path
                    if s.starts_with('.') && s != "." {
                        return Err(ValidationError::new(
                            "projectRoot",
                            "Path cannot contain hidden directories",
                        ));
                    }
                }
            }
            _ => {}
        }
    }

    // Block access to sensitive system directories
    const BLOCKED_PATHS: &[&str] = &[
        "/etc",
        "/sys",
        "/proc",
        "/dev",
        "/boot",
        "/usr/bin",
        "/usr/sbin",
        "/bin",
        "/sbin",
        "~/.ssh",
        "~/.aws",
        "~/.gnupg",
        "~/.config/git",
    ];

    let normalized_path = if path_str.starts_with("~/") {
        path_str.replace("~", &std::env::var("HOME").unwrap_or_default())
    } else {
        path_str.to_string()
    };

    for blocked in BLOCKED_PATHS {
        let blocked_expanded = if blocked.starts_with("~/") {
            blocked.replace("~", &std::env::var("HOME").unwrap_or_default())
        } else {
            blocked.to_string()
        };

        if normalized_path.starts_with(&blocked_expanded) {
            return Err(ValidationError::new(
                "projectRoot",
                format!("Access to {} is not allowed", blocked),
            ));
        }
    }

    Ok(())
}

fn validate_script(script: &str, field_name: &str) -> Result<(), ValidationError> {
    // Check length
    if script.len() > MAX_SCRIPT_LENGTH {
        return Err(ValidationError::new(
            field_name,
            format!("Script too long (max {} characters)", MAX_SCRIPT_LENGTH),
        ));
    }

    // Check for dangerous commands
    let script_lower = script.to_lowercase();
    for dangerous in DANGEROUS_COMMANDS {
        if script_lower.contains(&dangerous.to_lowercase()) {
            return Err(ValidationError::new(
                field_name,
                format!(
                    "Script contains potentially dangerous command: {}",
                    dangerous
                ),
            ));
        }
    }

    // Check for shell injection patterns
    const INJECTION_PATTERNS: &[&str] =
        &["$(", "${", "`", "&&", "||", ";", "|", ">", "<", ">>", "<<"];

    for pattern in INJECTION_PATTERNS {
        if script.contains(pattern) {
            return Err(ValidationError::new(
                field_name,
                format!("Script contains potentially dangerous pattern: {}", pattern),
            ));
        }
    }

    Ok(())
}

/// Validates project data for creation or update
pub async fn validate_project_data(
    data: &ProjectCreateInput,
    allow_nonexistent_path: bool,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate name (required)
    if data.name.trim().is_empty() {
        errors.push(ValidationError::new("name", "Project name is required"));
    } else if let Err(e) = validate_name(&data.name) {
        errors.push(e);
    }

    // Validate project root (required)
    if data.project_root.trim().is_empty() {
        errors.push(ValidationError::new(
            "projectRoot",
            "Project root path is required",
        ));
    } else {
        // Validate path safety
        if let Err(e) = validate_path_safety(&data.project_root) {
            errors.push(e);
        }

        // Check if path exists (if required)
        if !allow_nonexistent_path && !path_exists(&data.project_root).await {
            errors.push(ValidationError::new(
                "projectRoot",
                format!("Project root path does not exist: {}", data.project_root),
            ));
        }
    }

    // Validate description if present
    if let Some(ref description) = data.description {
        if description.len() > MAX_DESCRIPTION_LENGTH {
            errors.push(ValidationError::new(
                "description",
                format!(
                    "Description too long (max {} characters)",
                    MAX_DESCRIPTION_LENGTH
                ),
            ));
        }

        // Check for HTML/script injection
        if description.contains("<script") || description.contains("javascript:") {
            errors.push(ValidationError::new(
                "description",
                "Description cannot contain script tags or javascript",
            ));
        }
    }

    // Validate scripts if present
    if let Some(ref script) = data.setup_script {
        if let Err(e) = validate_script(script, "setup_script") {
            errors.push(e);
        }
    }

    if let Some(ref script) = data.dev_script {
        if let Err(e) = validate_script(script, "dev_script") {
            errors.push(e);
        }
    }

    if let Some(ref script) = data.cleanup_script {
        if let Err(e) = validate_script(script, "cleanup_script") {
            errors.push(e);
        }
    }

    // Validate tags if present
    if let Some(ref tags) = data.tags {
        let tag_re = Regex::new(TAG_PATTERN).expect("Invalid TAG_PATTERN regex");
        for tag in tags {
            if tag.trim().is_empty() {
                errors.push(ValidationError::new("tags", "Tags cannot be empty"));
                break;
            }
            if tag.len() > MAX_TAG_LENGTH {
                errors.push(ValidationError::new(
                    "tags",
                    format!("Tag '{}' too long (max {} characters)", tag, MAX_TAG_LENGTH),
                ));
            }
            if !tag_re.is_match(tag) {
                errors.push(ValidationError::new(
                    "tags",
                    format!("Tag '{}' contains invalid characters", tag),
                ));
            }
        }
    }

    errors
}

/// Validates project update data
pub async fn validate_project_update(
    data: &ProjectUpdateInput,
    allow_nonexistent_path: bool,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate name if provided
    if let Some(ref name) = data.name {
        if name.trim().is_empty() {
            errors.push(ValidationError::new("name", "Project name cannot be empty"));
        } else if let Err(e) = validate_name(name) {
            errors.push(e);
        }
    }

    // Validate project root if provided
    if let Some(ref project_root) = data.project_root {
        if project_root.trim().is_empty() {
            errors.push(ValidationError::new(
                "projectRoot",
                "Project root path cannot be empty",
            ));
        } else {
            // Validate path safety
            if let Err(e) = validate_path_safety(project_root) {
                errors.push(e);
            }

            // Check if path exists (if required)
            if !allow_nonexistent_path && !path_exists(project_root).await {
                errors.push(ValidationError::new(
                    "projectRoot",
                    format!("Project root path does not exist: {}", project_root),
                ));
            }
        }
    }

    // Validate description if present
    if let Some(ref description) = data.description {
        if description.len() > MAX_DESCRIPTION_LENGTH {
            errors.push(ValidationError::new(
                "description",
                format!(
                    "Description too long (max {} characters)",
                    MAX_DESCRIPTION_LENGTH
                ),
            ));
        }

        // Check for HTML/script injection
        if description.contains("<script") || description.contains("javascript:") {
            errors.push(ValidationError::new(
                "description",
                "Description cannot contain script tags or javascript",
            ));
        }
    }

    // Validate scripts if present
    if let Some(ref script) = data.setup_script {
        if let Err(e) = validate_script(script, "setup_script") {
            errors.push(e);
        }
    }

    if let Some(ref script) = data.dev_script {
        if let Err(e) = validate_script(script, "dev_script") {
            errors.push(e);
        }
    }

    if let Some(ref script) = data.cleanup_script {
        if let Err(e) = validate_script(script, "cleanup_script") {
            errors.push(e);
        }
    }

    // Validate tags if present
    if let Some(ref tags) = data.tags {
        let tag_re = Regex::new(TAG_PATTERN).expect("Invalid TAG_PATTERN regex");
        for tag in tags {
            if tag.trim().is_empty() {
                errors.push(ValidationError::new("tags", "Tags cannot be empty"));
                break;
            }
            if tag.len() > MAX_TAG_LENGTH {
                errors.push(ValidationError::new(
                    "tags",
                    format!("Tag '{}' too long (max {} characters)", tag, MAX_TAG_LENGTH),
                ));
            }
            if !tag_re.is_match(tag) {
                errors.push(ValidationError::new(
                    "tags",
                    format!("Tag '{}' contains invalid characters", tag),
                ));
            }
        }
    }

    errors
}

/// Truncates a string to a maximum length with ellipsis
pub fn truncate(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::generate_project_id;
    use crate::types::ProjectStatus;

    #[tokio::test]
    async fn test_validate_project_data_valid() {
        let data = ProjectCreateInput {
            name: "Test Project".to_string(),
            project_root: "/tmp".to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            tags: Some(vec!["rust".to_string(), "web".to_string()]),
            description: Some("A test project".to_string()),
            status: Some(ProjectStatus::Active),
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let errors = validate_project_data(&data, true).await;
        assert!(errors.is_empty());
    }

    #[tokio::test]
    async fn test_validate_project_data_empty_name() {
        let data = ProjectCreateInput {
            name: "".to_string(),
            project_root: "/tmp".to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            tags: None,
            description: None,
            status: None,
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let errors = validate_project_data(&data, true).await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "name");
    }

    #[tokio::test]
    async fn test_validate_project_data_empty_path() {
        let data = ProjectCreateInput {
            name: "Test".to_string(),
            project_root: "".to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            tags: None,
            description: None,
            status: None,
            rank: None,
            priority: None,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
        };

        let errors = validate_project_data(&data, true).await;
        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].field, "projectRoot");
    }

    #[test]
    fn test_generate_project_id() {
        let id1 = generate_project_id();
        let id2 = generate_project_id();

        // IDs are 8 characters long (cloud-compatible format)
        assert_eq!(id1.len(), 8);
        assert_eq!(id2.len(), 8);
        assert_ne!(id1, id2);

        // Should be alphanumeric characters only
        assert!(id1.chars().all(|c| c.is_ascii_alphanumeric()));
        assert!(id2.chars().all(|c| c.is_ascii_alphanumeric()));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(truncate("", 5), "");
    }
}
