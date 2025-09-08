use crate::storage::path_exists;
use crate::types::{ProjectCreateInput, ProjectUpdateInput};

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

/// Validates project data for creation or update
pub async fn validate_project_data(
    data: &ProjectCreateInput,
    allow_nonexistent_path: bool,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    // Validate required fields
    if data.name.trim().is_empty() {
        errors.push(ValidationError::new("name", "Project name is required"));
    }

    if data.project_root.trim().is_empty() {
        errors.push(ValidationError::new("projectRoot", "Project root path is required"));
    }

    // Validate project root exists if required
    if !data.project_root.is_empty() && !allow_nonexistent_path {
        if !path_exists(&data.project_root).await {
            errors.push(ValidationError::new(
                "projectRoot",
                format!("Project root path does not exist: {}", data.project_root),
            ));
        }
    }

    // Validate tags if present
    if let Some(ref tags) = data.tags {
        for tag in tags {
            if tag.trim().is_empty() {
                errors.push(ValidationError::new("tags", "Tags cannot be empty"));
                break;
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
        }
    }

    // Validate project root if provided
    if let Some(ref project_root) = data.project_root {
        if project_root.trim().is_empty() {
            errors.push(ValidationError::new("projectRoot", "Project root path cannot be empty"));
        } else if !allow_nonexistent_path && !path_exists(project_root).await {
            errors.push(ValidationError::new(
                "projectRoot",
                format!("Project root path does not exist: {}", project_root),
            ));
        }
    }

    // Validate tags if present
    if let Some(ref tags) = data.tags {
        for tag in tags {
            if tag.trim().is_empty() {
                errors.push(ValidationError::new("tags", "Tags cannot be empty"));
                break;
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
    use crate::types::ProjectStatus;
    use crate::storage::generate_project_id;

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
        
        assert_eq!(id1.len(), 8);
        assert_eq!(id2.len(), 8);
        assert_ne!(id1, id2);
        
        // Should only contain hex characters
        assert!(id1.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
        assert_eq!(truncate("hi", 5), "hi");
        assert_eq!(truncate("", 5), "");
    }
}