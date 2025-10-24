// ABOUTME: Task parser for OpenSpec change task markdown
// ABOUTME: Parses markdown task lists into structured task items with hierarchy support

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTask {
    pub number: String,          // e.g., "1.1", "2.3", "1"
    pub text: String,            // Task description
    pub is_completed: bool,      // Checkbox state: [ ] = false, [x] = true
    pub display_order: usize,    // Order in the document
    pub parent_number: Option<String>, // Parent task number (e.g., "1" for "1.1")
    pub level: usize,            // Nesting level (0 = top-level)
}

#[derive(Debug, thiserror::Error)]
pub enum TaskParseError {
    #[error("Invalid task format: {0}")]
    InvalidFormat(String),

    #[error("Invalid task number: {0}")]
    InvalidNumber(String),

    #[error("Empty task description")]
    EmptyDescription,
}

pub type TaskParseResult<T> = Result<T, TaskParseError>;

/// Parse tasks from markdown task list
///
/// Supports formats:
/// - `- [ ] 1.1 Task description`
/// - `- [x] 2 Completed task`
/// - `  - [ ] 1.1.1 Nested task` (indentation)
///
/// Returns tasks in document order with hierarchy information
pub fn parse_tasks_from_markdown(markdown: &str) -> TaskParseResult<Vec<ParsedTask>> {
    let mut tasks = Vec::new();
    let mut display_order = 0;

    // Regex pattern to match task lines
    // Captures: (indent)(checkbox)(number)(description)
    let task_pattern = Regex::new(
        r"^(\s*)- \[([ xX])\]\s*(?:(\d+(?:\.\d+)*)\s+)?(.+)$"
    ).map_err(|e| TaskParseError::InvalidFormat(format!("Regex compilation failed: {}", e)))?;

    for line in markdown.lines() {
        if let Some(captures) = task_pattern.captures(line) {
            let indent = captures.get(1).map_or("", |m| m.as_str());
            let checkbox = captures.get(2).map_or(" ", |m| m.as_str());
            let number = captures.get(3).map(|m| m.as_str().to_string());
            let text = captures.get(4).map_or("", |m| m.as_str()).trim().to_string();

            if text.is_empty() {
                return Err(TaskParseError::EmptyDescription);
            }

            // Calculate nesting level from indentation (2 spaces = 1 level)
            let level = indent.len() / 2;

            // Determine if task is completed
            let is_completed = checkbox.eq_ignore_ascii_case("x");

            // Extract parent number from task number (e.g., "1.2.3" -> parent is "1.2")
            let parent_number = if let Some(ref num) = number {
                extract_parent_number(num)
            } else {
                None
            };

            // Use task number if provided, otherwise generate from order
            let task_number = number.unwrap_or_else(|| format!("{}", display_order + 1));

            tasks.push(ParsedTask {
                number: task_number,
                text,
                is_completed,
                display_order,
                parent_number,
                level,
            });

            display_order += 1;
        }
    }

    Ok(tasks)
}

/// Extract parent number from a task number
/// Examples:
/// - "1.2.3" -> Some("1.2")
/// - "1.2" -> Some("1")
/// - "1" -> None
fn extract_parent_number(number: &str) -> Option<String> {
    let parts: Vec<&str> = number.split('.').collect();
    if parts.len() > 1 {
        Some(parts[..parts.len() - 1].join("."))
    } else {
        None
    }
}

/// Regenerate markdown task list from parsed tasks
/// Useful for round-trip testing and updating checkbox states
pub fn generate_tasks_markdown(tasks: &[ParsedTask]) -> String {
    let mut lines = Vec::new();

    for task in tasks {
        let indent = "  ".repeat(task.level);
        let checkbox = if task.is_completed { "[x]" } else { "[ ]" };
        let line = format!("{}- {} {} {}", indent, checkbox, task.number, task.text);
        lines.push(line);
    }

    lines.join("\n")
}

/// Update task completion state in markdown
/// Finds the task by number and updates its checkbox
pub fn update_task_in_markdown(
    markdown: &str,
    task_number: &str,
    is_completed: bool,
) -> TaskParseResult<String> {
    let mut result = Vec::new();
    let task_pattern = Regex::new(
        r"^(\s*- )\[([ xX])\](\s*\d+(?:\.\d+)*\s+.+)$"
    ).map_err(|e| TaskParseError::InvalidFormat(format!("Regex compilation failed: {}", e)))?;

    let mut found = false;

    for line in markdown.lines() {
        if let Some(captures) = task_pattern.captures(line) {
            let prefix = captures.get(1).map_or("", |m| m.as_str());
            let suffix = captures.get(3).map_or("", |m| m.as_str());

            // Check if this line contains the task number we're looking for
            if suffix.trim_start().starts_with(task_number) {
                let new_checkbox = if is_completed { "x" } else { " " };
                result.push(format!("{}[{}]{}", prefix, new_checkbox, suffix));
                found = true;
            } else {
                result.push(line.to_string());
            }
        } else {
            result.push(line.to_string());
        }
    }

    if !found {
        return Err(TaskParseError::InvalidNumber(format!(
            "Task number '{}' not found in markdown",
            task_number
        )));
    }

    Ok(result.join("\n"))
}

/// Calculate task completion statistics
pub fn calculate_task_stats(tasks: &[ParsedTask]) -> TaskStats {
    let total = tasks.len();
    let completed = tasks.iter().filter(|t| t.is_completed).count();
    let percentage = if total > 0 {
        (completed as f64 / total as f64 * 100.0).round() as u8
    } else {
        0
    };

    TaskStats {
        total,
        completed,
        pending: total - completed,
        percentage,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub total: usize,
    pub completed: usize,
    pub pending: usize,
    pub percentage: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_tasks() {
        let markdown = r#"
- [ ] 1 First task
- [x] 2 Second task (completed)
- [ ] 3 Third task
"#;

        let tasks = parse_tasks_from_markdown(markdown).unwrap();
        assert_eq!(tasks.len(), 3);
        assert_eq!(tasks[0].number, "1");
        assert_eq!(tasks[0].text, "First task");
        assert!(!tasks[0].is_completed);
        assert_eq!(tasks[1].number, "2");
        assert!(tasks[1].is_completed);
    }

    #[test]
    fn test_parse_hierarchical_tasks() {
        let markdown = r#"
- [ ] 1 Parent task
  - [ ] 1.1 Child task
  - [x] 1.2 Another child
- [ ] 2 Second parent
"#;

        let tasks = parse_tasks_from_markdown(markdown).unwrap();
        assert_eq!(tasks.len(), 4);
        assert_eq!(tasks[0].number, "1");
        assert_eq!(tasks[0].level, 0);
        assert_eq!(tasks[0].parent_number, None);

        assert_eq!(tasks[1].number, "1.1");
        assert_eq!(tasks[1].level, 1);
        assert_eq!(tasks[1].parent_number, Some("1".to_string()));

        assert_eq!(tasks[2].number, "1.2");
        assert_eq!(tasks[2].parent_number, Some("1".to_string()));
        assert!(tasks[2].is_completed);
    }

    #[test]
    fn test_parse_without_numbers() {
        let markdown = r#"
- [ ] Task without number
- [x] Another task
"#;

        let tasks = parse_tasks_from_markdown(markdown).unwrap();
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].number, "1");
        assert_eq!(tasks[1].number, "2");
    }

    #[test]
    fn test_generate_markdown() {
        let tasks = vec![
            ParsedTask {
                number: "1".to_string(),
                text: "First task".to_string(),
                is_completed: false,
                display_order: 0,
                parent_number: None,
                level: 0,
            },
            ParsedTask {
                number: "1.1".to_string(),
                text: "Nested task".to_string(),
                is_completed: true,
                display_order: 1,
                parent_number: Some("1".to_string()),
                level: 1,
            },
        ];

        let markdown = generate_tasks_markdown(&tasks);
        assert!(markdown.contains("- [ ] 1 First task"));
        assert!(markdown.contains("  - [x] 1.1 Nested task"));
    }

    #[test]
    fn test_update_task_completion() {
        let markdown = r#"
- [ ] 1 First task
- [ ] 2 Second task
- [ ] 3 Third task
"#;

        let updated = update_task_in_markdown(markdown, "2", true).unwrap();
        assert!(updated.contains("- [ ] 1 First task"));
        assert!(updated.contains("- [x] 2 Second task"));
        assert!(updated.contains("- [ ] 3 Third task"));
    }

    #[test]
    fn test_calculate_stats() {
        let tasks = vec![
            ParsedTask {
                number: "1".to_string(),
                text: "Task 1".to_string(),
                is_completed: true,
                display_order: 0,
                parent_number: None,
                level: 0,
            },
            ParsedTask {
                number: "2".to_string(),
                text: "Task 2".to_string(),
                is_completed: false,
                display_order: 1,
                parent_number: None,
                level: 0,
            },
            ParsedTask {
                number: "3".to_string(),
                text: "Task 3".to_string(),
                is_completed: true,
                display_order: 2,
                parent_number: None,
                level: 0,
            },
        ];

        let stats = calculate_task_stats(&tasks);
        assert_eq!(stats.total, 3);
        assert_eq!(stats.completed, 2);
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.percentage, 67);
    }

    #[test]
    fn test_extract_parent_number() {
        assert_eq!(extract_parent_number("1.2.3"), Some("1.2".to_string()));
        assert_eq!(extract_parent_number("1.2"), Some("1".to_string()));
        assert_eq!(extract_parent_number("1"), None);
    }
}
