// ABOUTME: Markdown parser for OpenSpec format
// ABOUTME: Parses PRDs, specs, requirements, and WHEN/THEN/AND scenarios from markdown

use super::types::{ParsedCapability, ParsedRequirement, ParsedScenario, ParsedSpec};

#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    #[error("Invalid markdown format: {0}")]
    InvalidFormat(String),

    #[error("Missing required section: {0}")]
    MissingSection(String),

    #[error("Invalid scenario format in requirement '{0}': {1}")]
    InvalidScenario(String, String),

    #[error("Input validation failed: {0}")]
    ValidationError(String),
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Maximum length for a single line in characters
const MAX_LINE_LENGTH: usize = 10_000;

/// Maximum total document size in bytes (1MB)
const MAX_DOCUMENT_SIZE: usize = 1_048_576;

/// Dangerous HTML tags that should be rejected
const DANGEROUS_TAGS: &[&str] = &[
    "<script", "<iframe", "<object", "<embed", "<link", "<style", "<meta", "<base", "<form",
    "<input", "<button",
];

/// Dangerous URL protocols
const DANGEROUS_PROTOCOLS: &[&str] = &["javascript:", "data:text/html", "vbscript:"];

/// Validate markdown input for security issues
fn validate_input(markdown: &str) -> ParseResult<()> {
    // Check total document size
    if markdown.len() > MAX_DOCUMENT_SIZE {
        return Err(ParseError::ValidationError(format!(
            "Document too large: {} bytes (max: {} bytes)",
            markdown.len(),
            MAX_DOCUMENT_SIZE
        )));
    }

    // Check for extremely long lines
    for (line_num, line) in markdown.lines().enumerate() {
        if line.len() > MAX_LINE_LENGTH {
            return Err(ParseError::ValidationError(format!(
                "Line {} exceeds maximum length: {} characters (max: {})",
                line_num + 1,
                line.len(),
                MAX_LINE_LENGTH
            )));
        }
    }

    // Check for dangerous HTML tags (case-insensitive)
    let markdown_lower = markdown.to_lowercase();
    for tag in DANGEROUS_TAGS {
        if markdown_lower.contains(tag) {
            return Err(ParseError::ValidationError(format!(
                "Potentially malicious HTML tag detected: {}",
                tag
            )));
        }
    }

    // Check for dangerous URL protocols
    for protocol in DANGEROUS_PROTOCOLS {
        if markdown_lower.contains(protocol) {
            return Err(ParseError::ValidationError(format!(
                "Potentially malicious URL protocol detected: {}",
                protocol
            )));
        }
    }

    Ok(())
}

/// Sanitize a string by removing null bytes and control characters
fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| !c.is_control() || c.is_whitespace())
        .collect()
}

/// Parse a complete spec markdown document into structured capabilities
pub fn parse_spec_markdown(markdown: &str) -> ParseResult<ParsedSpec> {
    // Validate input before parsing
    validate_input(markdown)?;
    let mut capabilities = Vec::new();

    // Split by capability sections (## headings)
    let capability_sections = split_by_heading(markdown, 2);

    for (capability_name, content) in capability_sections {
        if capability_name.is_empty() {
            continue;
        }

        let capability = parse_capability(&capability_name, &content)?;
        capabilities.push(capability);
    }

    if capabilities.is_empty() {
        return Err(ParseError::MissingSection(
            "No capabilities found".to_string(),
        ));
    }

    Ok(ParsedSpec {
        capabilities,
        raw_markdown: markdown.to_string(),
    })
}

/// Parse a single capability section
fn parse_capability(name: &str, content: &str) -> ParseResult<ParsedCapability> {
    let lines: Vec<&str> = content.lines().collect();

    // Extract purpose (first paragraph after heading)
    let purpose = extract_purpose(&lines);

    // Parse requirements (### headings)
    let requirement_sections = split_by_heading(content, 3);
    let mut requirements = Vec::new();

    for (req_name, req_content) in requirement_sections {
        if req_name.is_empty() {
            continue;
        }

        let requirement = parse_requirement(&req_name, &req_content)?;
        requirements.push(requirement);
    }

    Ok(ParsedCapability {
        name: name.trim().to_string(),
        purpose: purpose.unwrap_or_else(|| "No purpose specified".to_string()),
        requirements,
    })
}

/// Parse a single requirement section with scenarios
fn parse_requirement(name: &str, content: &str) -> ParseResult<ParsedRequirement> {
    let lines: Vec<&str> = content.lines().collect();

    // Extract description (text before scenarios)
    let description = extract_description(&lines);

    // Parse scenarios (WHEN/THEN/AND format)
    let scenarios = parse_scenarios(&lines, name)?;

    Ok(ParsedRequirement {
        name: name.trim().to_string(),
        description: description.unwrap_or_else(|| "No description".to_string()),
        scenarios,
    })
}

/// Parse WHEN/THEN/AND scenarios from lines
fn parse_scenarios(lines: &[&str], _requirement_name: &str) -> ParseResult<Vec<ParsedScenario>> {
    let mut scenarios = Vec::new();
    let mut current_scenario: Option<(String, String, Vec<String>)> = None;
    let mut scenario_name = String::new();

    for line in lines {
        let trimmed = line.trim();

        // Scenario name (#### or **Scenario:**)
        if trimmed.starts_with("####") {
            if let Some((when, then, and)) = current_scenario.take() {
                scenarios.push(ParsedScenario {
                    name: scenario_name.clone(),
                    when,
                    then,
                    and,
                });
            }
            scenario_name = sanitize_string(trimmed.trim_start_matches('#').trim());
        } else if trimmed.to_lowercase().starts_with("**scenario:") {
            if let Some((when, then, and)) = current_scenario.take() {
                scenarios.push(ParsedScenario {
                    name: scenario_name.clone(),
                    when,
                    then,
                    and,
                });
            }
            scenario_name = sanitize_string(
                trimmed
                    .trim_start_matches("**")
                    .trim_end_matches("**")
                    .trim_start_matches("Scenario:")
                    .trim_start_matches("scenario:")
                    .trim(),
            );
        }
        // WHEN clause
        else if starts_with_ignore_case(trimmed, "WHEN ") || trimmed.starts_with("**WHEN") {
            let when_text = extract_clause_text(trimmed, "WHEN");
            if let Some((_, then, and)) = current_scenario.take() {
                scenarios.push(ParsedScenario {
                    name: scenario_name.clone(),
                    when: when_text.clone(),
                    then,
                    and,
                });
            }
            current_scenario = Some((when_text, String::new(), Vec::new()));
        }
        // THEN clause
        else if starts_with_ignore_case(trimmed, "THEN ") || trimmed.starts_with("**THEN") {
            if let Some((when, _, and)) = current_scenario.take() {
                let then_text = extract_clause_text(trimmed, "THEN");
                current_scenario = Some((when, then_text, and));
            }
        }
        // AND clause
        else if starts_with_ignore_case(trimmed, "AND ") || trimmed.starts_with("**AND") {
            if let Some((when, then, mut and)) = current_scenario.take() {
                let and_text = extract_clause_text(trimmed, "AND");
                and.push(and_text);
                current_scenario = Some((when, then, and));
            }
        }
    }

    // Add final scenario
    if let Some((when, then, and)) = current_scenario {
        scenarios.push(ParsedScenario {
            name: if scenario_name.is_empty() {
                "Default scenario".to_string()
            } else {
                scenario_name
            },
            when,
            then,
            and,
        });
    }

    Ok(scenarios)
}

/// Check if string starts with prefix (case-insensitive, ASCII only)
fn starts_with_ignore_case(text: &str, prefix: &str) -> bool {
    text.len() >= prefix.len() && text[..prefix.len()].eq_ignore_ascii_case(prefix)
}

/// Extract clause text (WHEN/THEN/AND)
fn extract_clause_text(line: &str, clause_type: &str) -> String {
    let mut text = line.trim();

    // Remove leading/trailing bold markers
    text = text.trim_start_matches("**").trim_end_matches("**");

    // Remove clause keyword (case insensitive)
    if starts_with_ignore_case(text, clause_type) {
        text = &text[clause_type.len()..];
    }

    // Remove any remaining bold markers and colon
    text = text
        .trim_start_matches("**")
        .trim_end_matches("**")
        .trim_start_matches(':')
        .trim();

    // Sanitize the extracted text
    sanitize_string(text)
}

/// Split markdown by heading level
fn split_by_heading(content: &str, level: usize) -> Vec<(String, String)> {
    let heading_prefix = "#".repeat(level);
    let mut sections = Vec::new();
    let mut current_name = String::new();
    let mut current_content = String::new();

    for line in content.lines() {
        if line.trim_start().starts_with(&heading_prefix)
            && !line
                .trim_start()
                .starts_with(&format!("{}#", heading_prefix))
        {
            // New section
            if !current_name.is_empty() {
                sections.push((current_name.clone(), current_content.clone()));
            }
            current_name = sanitize_string(line.trim_start_matches('#').trim());
            current_content.clear();
        } else {
            current_content.push_str(line);
            current_content.push('\n');
        }
    }

    // Add final section
    if !current_name.is_empty() {
        sections.push((current_name, current_content));
    }

    sections
}

/// Extract purpose from first paragraph
fn extract_purpose(lines: &[&str]) -> Option<String> {
    let mut purpose = String::new();
    let mut in_purpose = false;

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if in_purpose && !purpose.is_empty() {
                break;
            }
            continue;
        }

        // Skip headings
        if trimmed.starts_with('#') || starts_with_ignore_case(trimmed, "WHEN") {
            break;
        }

        in_purpose = true;
        if !purpose.is_empty() {
            purpose.push(' ');
        }
        purpose.push_str(trimmed);
    }

    if purpose.is_empty() {
        None
    } else {
        Some(sanitize_string(&purpose))
    }
}

/// Extract description (text before scenarios)
fn extract_description(lines: &[&str]) -> Option<String> {
    let mut description = String::new();

    for line in lines {
        let trimmed = line.trim();

        // Stop at scenario markers
        if trimmed.starts_with("####")
            || starts_with_ignore_case(trimmed, "WHEN")
            || starts_with_ignore_case(trimmed, "**WHEN")
        {
            break;
        }

        if trimmed.is_empty() {
            continue;
        }

        if !description.is_empty() {
            description.push(' ');
        }
        description.push_str(trimmed);
    }

    if description.is_empty() {
        None
    } else {
        Some(sanitize_string(&description))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_spec() {
        let markdown = r#"
## Authentication

User authentication and authorization system.

### User Login

Users can log in with email and password.

**Scenario: Valid credentials**
WHEN user submits correct email and password
THEN system authenticates the user
AND user is redirected to dashboard
AND session is created

**Scenario: Invalid credentials**
WHEN user submits incorrect password
THEN system shows error message
"#;

        let result = parse_spec_markdown(markdown);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.capabilities.len(), 1);
        assert_eq!(spec.capabilities[0].name, "Authentication");
        assert_eq!(spec.capabilities[0].requirements.len(), 1);
        assert_eq!(spec.capabilities[0].requirements[0].scenarios.len(), 2);
    }

    #[test]
    fn test_parse_scenario() {
        let lines = vec![
            "**Scenario: Valid input**",
            "WHEN user enters valid data",
            "THEN data is saved",
            "AND confirmation is shown",
        ];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "Valid input");
        assert_eq!(scenarios[0].when, "user enters valid data");
        assert_eq!(scenarios[0].then, "data is saved");
        assert_eq!(scenarios[0].and.len(), 1);
    }

    #[test]
    fn test_parse_empty_spec() {
        let markdown = "";
        let result = parse_spec_markdown(markdown);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ParseError::MissingSection(_)));
    }

    #[test]
    fn test_parse_spec_without_capabilities() {
        let markdown = "# This is just a heading\n\nSome content but no capabilities.";
        let result = parse_spec_markdown(markdown);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_multiple_capabilities() {
        let markdown = r#"
## Authentication

User authentication system.

### Login
Users can log in.
WHEN user enters credentials
THEN user is authenticated

## Data Management

Data storage and retrieval.

### Save Data
Users can save data.
WHEN user clicks save
THEN data is stored
"#;

        let result = parse_spec_markdown(markdown);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.capabilities.len(), 2);
        assert_eq!(spec.capabilities[0].name, "Authentication");
        assert_eq!(spec.capabilities[1].name, "Data Management");
    }

    #[test]
    fn test_parse_capability_without_purpose() {
        let markdown = r#"
## Test Capability

### Requirement One
Description here.
WHEN something happens
THEN result occurs
"#;

        let result = parse_spec_markdown(markdown);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.capabilities.len(), 1);
        assert!(spec.capabilities[0]
            .purpose
            .contains("No purpose specified"));
    }

    #[test]
    fn test_parse_requirement_without_description() {
        let markdown = r#"
## Test Capability

Purpose text.

### Requirement One
WHEN something happens
THEN result occurs
"#;

        let result = parse_spec_markdown(markdown);
        assert!(result.is_ok());

        let spec = result.unwrap();
        assert_eq!(spec.capabilities[0].requirements.len(), 1);
        assert!(spec.capabilities[0].requirements[0]
            .description
            .contains("No description"));
    }

    #[test]
    fn test_parse_scenario_without_name() {
        let lines = vec!["WHEN user enters data", "THEN data is saved"];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "Default scenario");
    }

    #[test]
    fn test_parse_multiple_scenarios() {
        let lines = vec![
            "**Scenario: First**",
            "WHEN condition one",
            "THEN result one",
            "",
            "**Scenario: Second**",
            "WHEN condition two",
            "THEN result two",
            "AND additional result",
        ];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 2);
        assert_eq!(scenarios[0].name, "First");
        assert_eq!(scenarios[1].name, "Second");
        assert_eq!(scenarios[1].and.len(), 1);
    }

    #[test]
    fn test_parse_scenario_with_multiple_and_clauses() {
        let lines = vec![
            "WHEN user submits form",
            "THEN form is validated",
            "AND data is saved",
            "AND confirmation is shown",
            "AND email is sent",
        ];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].and.len(), 3);
    }

    #[test]
    fn test_parse_scenario_with_heading_name() {
        let lines = vec![
            "#### Valid Login",
            "WHEN user enters correct credentials",
            "THEN user is logged in",
        ];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].name, "Valid Login");
    }

    #[test]
    fn test_parse_scenario_with_case_insensitive_keywords() {
        let lines = vec![
            "when user clicks button",
            "then action occurs",
            "and result is shown",
        ];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].when, "user clicks button");
        assert_eq!(scenarios[0].then, "action occurs");
        assert_eq!(scenarios[0].and.len(), 1);
    }

    #[test]
    fn test_parse_scenario_with_bold_keywords() {
        let lines = vec![
            "**WHEN** user performs action",
            "**THEN** system responds",
            "**AND** result is displayed",
        ];

        let scenarios = parse_scenarios(&lines, "test").unwrap();
        assert_eq!(scenarios.len(), 1);
        assert_eq!(scenarios[0].when, "user performs action");
    }

    #[test]
    fn test_extract_clause_text_variations() {
        assert_eq!(
            extract_clause_text("WHEN user clicks", "WHEN"),
            "user clicks"
        );
        assert_eq!(
            extract_clause_text("when user clicks", "WHEN"),
            "user clicks"
        );
        assert_eq!(
            extract_clause_text("**WHEN** user clicks", "WHEN"),
            "user clicks"
        );
        assert_eq!(
            extract_clause_text("WHEN: user clicks", "WHEN"),
            "user clicks"
        );
    }

    #[test]
    fn test_split_by_heading() {
        let content = r#"
## First Section
Content one

## Second Section
Content two

### Subsection
Content three
"#;

        let sections = split_by_heading(content, 2);
        assert_eq!(sections.len(), 2);
        assert_eq!(sections[0].0, "First Section");
        assert_eq!(sections[1].0, "Second Section");
    }
}
