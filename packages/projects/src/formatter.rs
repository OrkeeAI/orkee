use crate::types::Project;
use crate::validator::truncate;

/// Formats projects into a table string
pub fn format_projects_table(projects: &[Project], current_project: Option<&Project>) -> String {
    if projects.is_empty() {
        return "No projects found".to_string();
    }

    // Calculate column widths (add 2 for the indicator)
    let max_id_length = std::cmp::max(6, projects.iter().map(|p| p.id.len()).max().unwrap_or(0));
    let max_name_length =
        std::cmp::max(20, projects.iter().map(|p| p.name.len()).max().unwrap_or(0));
    let max_path_length = std::cmp::max(
        30,
        projects
            .iter()
            .map(|p| p.project_root.len())
            .max()
            .unwrap_or(0),
    );
    let max_status_length = std::cmp::max(
        8,
        projects
            .iter()
            .map(|p| format!("{:?}", p.status).len())
            .max()
            .unwrap_or(0),
    );

    // Create header
    let header = format!(
        "  {:<width_id$} | {:<width_name$} | {:<width_path$} | {:<width_status$}",
        "ID",
        "Name",
        "Project Root",
        "Status",
        width_id = max_id_length,
        width_name = max_name_length,
        width_path = max_path_length,
        width_status = max_status_length
    );

    let separator = "-".repeat(header.len());

    // Create rows
    let rows: Vec<String> = projects
        .iter()
        .map(|project| {
            let is_selected = current_project.map_or(false, |cp| cp.id == project.id);
            let indicator = if is_selected { "▸ " } else { "  " };

            let id = format!("{:<width$}", project.id, width = max_id_length);
            let name = format!(
                "{:<width$}",
                truncate(&project.name, max_name_length),
                width = max_name_length
            );
            let path = format!(
                "{:<width$}",
                truncate(&project.project_root, max_path_length),
                width = max_path_length
            );
            let status = format!(
                "{:<width$}",
                format!("{:?}", project.status).to_lowercase(),
                width = max_status_length
            );

            format!("{}{} | {} | {} | {}", indicator, id, name, path, status)
        })
        .collect();

    let mut result = vec![header, separator];
    result.extend(rows);
    result.join("\n")
}

/// Formats project details into a string
pub fn format_project_details(project: &Project) -> String {
    let mut lines = vec![
        format!("ID: {}", project.id),
        format!("Name: {}", project.name),
        format!("Project Root: {}", project.project_root),
        format!("Status: {}", format!("{:?}", project.status).to_lowercase()),
        format!(
            "Created: {}",
            project.created_at.format("%m/%d/%Y, %l:%M:%S %p")
        ),
        format!(
            "Updated: {}",
            project.updated_at.format("%m/%d/%Y, %l:%M:%S %p")
        ),
    ];

    if let Some(ref description) = project.description {
        lines.push(format!("Description: {}", description));
    }

    if let Some(ref tags) = project.tags {
        if !tags.is_empty() {
            lines.push(format!("Tags: {}", tags.join(", ")));
        }
    }

    if let Some(ref priority) = Some(&project.priority) {
        lines.push(format!(
            "Priority: {}",
            format!("{:?}", priority).to_lowercase()
        ));
    }

    if let Some(rank) = project.rank {
        lines.push(format!("Rank: {}", rank));
    }

    if let Some(ref setup_script) = project.setup_script {
        if !setup_script.is_empty() {
            lines.push(format!("Setup Script: {}", setup_script));
        }
    }

    if let Some(ref dev_script) = project.dev_script {
        if !dev_script.is_empty() {
            lines.push(format!("Dev Script: {}", dev_script));
        }
    }

    if let Some(ref cleanup_script) = project.cleanup_script {
        if !cleanup_script.is_empty() {
            lines.push(format!("Cleanup Script: {}", cleanup_script));
        }
    }

    if let Some(ref task_source) = project.task_source {
        lines.push(format!(
            "Task Source: {}",
            format!("{:?}", task_source).to_lowercase()
        ));
    }

    if let Some(ref manual_tasks) = project.manual_tasks {
        if !manual_tasks.is_empty() {
            lines.push(format!("Manual Tasks: {} tasks", manual_tasks.len()));
        }
    }

    if let Some(ref mcp_servers) = project.mcp_servers {
        let enabled_count = mcp_servers.values().filter(|&&v| v).count();
        let total_count = mcp_servers.len();
        if total_count > 0 {
            lines.push(format!(
                "MCP Servers: {}/{} enabled",
                enabled_count, total_count
            ));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Priority, ProjectStatus};
    use chrono::Utc;

    fn create_test_project(id: &str, name: &str, project_root: &str) -> Project {
        Project {
            id: id.to_string(),
            name: name.to_string(),
            project_root: project_root.to_string(),
            setup_script: None,
            dev_script: None,
            cleanup_script: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: None,
            description: None,
            status: ProjectStatus::Active,
            rank: None,
            priority: Priority::Medium,
            task_source: None,
            manual_tasks: None,
            mcp_servers: None,
            git_repository: None,
        }
    }

    #[test]
    fn test_format_projects_table_empty() {
        let projects = vec![];
        let result = format_projects_table(&projects, None);
        assert_eq!(result, "No projects found");
    }

    #[test]
    fn test_format_projects_table() {
        let projects = vec![
            create_test_project("abc123", "Test Project", "/path/to/project"),
            create_test_project("def456", "Another Project", "/path/to/another"),
        ];

        let result = format_projects_table(&projects, None);
        assert!(result.contains("ID"));
        assert!(result.contains("Name"));
        assert!(result.contains("Project Root"));
        assert!(result.contains("Status"));
        assert!(result.contains("Test Project"));
        assert!(result.contains("Another Project"));
    }

    #[test]
    fn test_format_projects_table_with_current() {
        let projects = vec![
            create_test_project("abc123", "Test Project", "/path/to/project"),
            create_test_project("def456", "Another Project", "/path/to/another"),
        ];

        let result = format_projects_table(&projects, Some(&projects[0]));
        assert!(result.contains("▸ abc123"));
        assert!(result.contains("  def456"));
    }

    #[test]
    fn test_format_project_details() {
        let mut project = create_test_project("abc123", "Test Project", "/path/to/project");
        project.description = Some("A test project description".to_string());
        project.tags = Some(vec!["rust".to_string(), "cli".to_string()]);
        project.setup_script = Some("cargo build".to_string());

        let result = format_project_details(&project);
        assert!(result.contains("ID: abc123"));
        assert!(result.contains("Name: Test Project"));
        assert!(result.contains("Description: A test project description"));
        assert!(result.contains("Tags: rust, cli"));
        assert!(result.contains("Setup Script: cargo build"));
    }

    #[test]
    fn test_format_project_details_minimal() {
        let project = create_test_project("minimal", "Minimal Project", "/minimal");

        let result = format_project_details(&project);
        assert!(result.contains("ID: minimal"));
        assert!(result.contains("Name: Minimal Project"));
        assert!(result.contains("Status: active"));
        assert!(!result.contains("Description:"));
        assert!(!result.contains("Tags:"));
    }
}
