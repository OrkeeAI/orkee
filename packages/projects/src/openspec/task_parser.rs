// ABOUTME: Task parser for OpenSpec change task markdown
// ABOUTME: Parses markdown task lists into structured task items with hierarchy support

use regex::Regex;
use serde::{Deserialize, Serialize};

// Security: Input length limits to prevent ReDoS attacks
/// Maximum size for entire markdown input (1MB - same as content size limit in db.rs)
const MAX_MARKDOWN_SIZE: usize = 1024 * 1024;

/// Maximum length for a single task line (10KB - reasonable for task descriptions)
const MAX_LINE_LENGTH: usize = 10 * 1024;

/// Maximum nesting depth for tasks (prevents stack overflow from extreme indentation)
const MAX_NESTING_DEPTH: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTask {
    pub number: String,                // e.g., "1.1", "2.3", "1"
    pub text: String,                  // Task description
    pub is_completed: bool,            // Checkbox state: [ ] = false, [x] = true
    pub display_order: usize,          // Order in the document
    pub parent_number: Option<String>, // Parent task number (e.g., "1" for "1.1")
    pub level: usize,                  // Nesting level (0 = top-level)
}

#[derive(Debug, thiserror::Error)]
pub enum TaskParseError {
    #[error("Invalid task format: {0}")]
    InvalidFormat(String),

    #[error("Invalid task number: {0}")]
    InvalidNumber(String),

    #[error("Empty task description")]
    EmptyDescription,

    #[error("Input too large: {0}")]
    InputTooLarge(String),
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
    // Security: Validate input size before regex processing to prevent ReDoS
    if markdown.len() > MAX_MARKDOWN_SIZE {
        return Err(TaskParseError::InputTooLarge(format!(
            "Markdown input exceeds maximum size of {} bytes (got {} bytes)",
            MAX_MARKDOWN_SIZE,
            markdown.len()
        )));
    }

    let mut tasks = Vec::new();
    let mut display_order = 0;
    let mut malformed_lines = Vec::new();

    // Regex pattern to match task lines
    // Captures: (indent)(checkbox)(number)(description)
    let task_pattern = Regex::new(r"^(\s*)- \[([ xX])\]\s*(?:(\d+(?:\.\d+)*)\s+)?(.+)$")
        .map_err(|e| TaskParseError::InvalidFormat(format!("Regex compilation failed: {}", e)))?;

    // Pattern to detect lines that look like they're trying to be tasks but are malformed
    let potential_task_pattern = Regex::new(r"^(\s*)-\s*\[")
        .map_err(|e| TaskParseError::InvalidFormat(format!("Regex compilation failed: {}", e)))?;

    for line in markdown.lines() {
        // Security: Validate line length before regex processing
        if line.len() > MAX_LINE_LENGTH {
            return Err(TaskParseError::InputTooLarge(format!(
                "Task line exceeds maximum length of {} bytes (got {} bytes)",
                MAX_LINE_LENGTH,
                line.len()
            )));
        }

        let trimmed = line.trim();

        // Skip empty lines and non-task lines (like headers)
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if let Some(captures) = task_pattern.captures(line) {
            let indent = captures.get(1).map_or("", |m| m.as_str());
            let checkbox = captures.get(2).map_or(" ", |m| m.as_str());
            let number = captures.get(3).map(|m| m.as_str().to_string());
            let text = captures
                .get(4)
                .map_or("", |m| m.as_str())
                .trim()
                .to_string();

            if text.is_empty() {
                return Err(TaskParseError::EmptyDescription);
            }

            // Calculate nesting level from indentation (2 spaces = 1 level)
            let level = indent.len() / 2;

            // Security: Prevent excessive nesting that could cause performance issues
            if level > MAX_NESTING_DEPTH {
                return Err(TaskParseError::InvalidFormat(format!(
                    "Task nesting depth exceeds maximum of {} levels (got {} levels)",
                    MAX_NESTING_DEPTH, level
                )));
            }

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
        } else if potential_task_pattern.is_match(line) {
            // This line looks like it's trying to be a task but is malformed
            malformed_lines.push(line.to_string());
        }
    }

    // If we found malformed task lines, return an error with guidance
    if !malformed_lines.is_empty() {
        let examples = malformed_lines
            .iter()
            .take(3)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");
        return Err(TaskParseError::InvalidFormat(format!(
            "Found {} malformed task line(s). Tasks must follow the format:\n\
             - [ ] Task description\n\
             - [x] Completed task\n\
             - [ ] 1.1 Numbered task\n\n\
             Malformed lines:\n{}",
            malformed_lines.len(),
            examples
        )));
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
    // Security: Validate input size before regex processing
    if markdown.len() > MAX_MARKDOWN_SIZE {
        return Err(TaskParseError::InputTooLarge(format!(
            "Markdown input exceeds maximum size of {} bytes (got {} bytes)",
            MAX_MARKDOWN_SIZE,
            markdown.len()
        )));
    }

    let mut result = Vec::new();
    let task_pattern = Regex::new(r"^(\s*- )\[([ xX])\](\s*\d+(?:\.\d+)*\s+.+)$")
        .map_err(|e| TaskParseError::InvalidFormat(format!("Regex compilation failed: {}", e)))?;

    let mut found = false;

    for line in markdown.lines() {
        // Security: Validate line length before regex processing
        if line.len() > MAX_LINE_LENGTH {
            return Err(TaskParseError::InputTooLarge(format!(
                "Task line exceeds maximum length of {} bytes (got {} bytes)",
                MAX_LINE_LENGTH,
                line.len()
            )));
        }
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

    #[test]
    fn test_malformed_task_error_with_guidance() {
        // Missing space after dash
        let markdown = r#"
-[ ] Task 1
-[x] Task 2
"#;

        let result = parse_tasks_from_markdown(markdown);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("malformed task line"));
        assert!(err_msg.contains("Tasks must follow the format"));
        assert!(err_msg.contains("- [ ] Task description"));
    }

    #[test]
    fn test_empty_markdown_returns_empty_vec() {
        let markdown = "";
        let tasks = parse_tasks_from_markdown(markdown).unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_markdown_with_only_headers_returns_empty_vec() {
        let markdown = r#"
## Tasks
### Subtasks
"#;
        let tasks = parse_tasks_from_markdown(markdown).unwrap();
        assert_eq!(tasks.len(), 0);
    }

    #[test]
    fn test_input_size_limit() {
        // Create markdown that exceeds MAX_MARKDOWN_SIZE (1MB)
        let large_task = "- [ ] ".to_string() + &"x".repeat(MAX_MARKDOWN_SIZE);
        let result = parse_tasks_from_markdown(&large_task);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskParseError::InputTooLarge(_)));
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("exceeds maximum size"));
    }

    #[test]
    fn test_line_length_limit() {
        // Create a single task line that exceeds MAX_LINE_LENGTH (10KB)
        let large_line = "- [ ] 1 ".to_string() + &"x".repeat(MAX_LINE_LENGTH);
        let result = parse_tasks_from_markdown(&large_line);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskParseError::InputTooLarge(_)));
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("Task line exceeds maximum length"));
    }

    #[test]
    fn test_nesting_depth_limit() {
        // Create a task with excessive nesting (> MAX_NESTING_DEPTH = 20)
        let deep_indent = " ".repeat((MAX_NESTING_DEPTH + 1) * 2); // 21 levels
        let markdown = format!("{}- [ ] 1 Deeply nested task", deep_indent);
        let result = parse_tasks_from_markdown(&markdown);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskParseError::InvalidFormat(_)));
        let err_msg = format!("{}", err);
        assert!(err_msg.contains("nesting depth exceeds maximum"));
    }

    #[test]
    fn test_reasonable_input_size_succeeds() {
        // Test with a reasonable but large markdown (under limits)
        let mut tasks_markdown = String::new();
        for i in 0..100 {
            tasks_markdown.push_str(&format!("- [ ] {} Task {}\n", i, "x".repeat(100)));
        }

        let result = parse_tasks_from_markdown(&tasks_markdown);
        assert!(result.is_ok());
        let tasks = result.unwrap();
        assert_eq!(tasks.len(), 100);
    }

    #[test]
    fn test_update_task_input_size_limit() {
        // Create markdown that exceeds MAX_MARKDOWN_SIZE
        let large_markdown = "- [ ] 1 ".to_string() + &"x".repeat(MAX_MARKDOWN_SIZE);
        let result = update_task_in_markdown(&large_markdown, "1", true);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskParseError::InputTooLarge(_)));
    }

    #[test]
    fn test_update_task_line_length_limit() {
        // Create a task line that exceeds MAX_LINE_LENGTH
        let large_line = "- [ ] 1 ".to_string() + &"x".repeat(MAX_LINE_LENGTH);
        let result = update_task_in_markdown(&large_line, "1", true);

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(err, TaskParseError::InputTooLarge(_)));
    }
}
