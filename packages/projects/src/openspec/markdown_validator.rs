// ABOUTME: Markdown-level validator for OpenSpec format compliance
// ABOUTME: Validates delta markdown against OpenSpec formatting requirements

use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownValidationError {
    pub line: Option<usize>,
    pub error_type: MarkdownValidationErrorType,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum MarkdownValidationErrorType {
    MissingScenarioHeader,
    InvalidScenarioFormat,
    MissingNormativeLanguage,
    NoScenariosFound,
    InvalidRequirementHeader,
    InvalidDeltaOperation,
}

pub struct OpenSpecMarkdownValidator {
    strict: bool,
}

impl OpenSpecMarkdownValidator {
    pub fn new(strict: bool) -> Self {
        Self { strict }
    }

    /// Validate delta markdown for OpenSpec compliance
    pub fn validate_delta_markdown(&self, markdown: &str) -> Vec<MarkdownValidationError> {
        let mut errors = Vec::new();

        // Check for delta operation headers
        if !self.has_delta_operation(markdown) {
            errors.push(MarkdownValidationError {
                line: None,
                error_type: MarkdownValidationErrorType::InvalidDeltaOperation,
                message: "Delta must start with ## ADDED, ## MODIFIED, or ## REMOVED Requirements"
                    .to_string(),
            });
        }

        // Validate requirements
        errors.extend(self.validate_requirements(markdown));

        // Validate scenarios
        errors.extend(self.validate_scenarios(markdown));

        // Check normative language
        if self.strict {
            errors.extend(self.validate_normative_language(markdown));
        }

        errors
    }

    fn has_delta_operation(&self, markdown: &str) -> bool {
        markdown.contains("## ADDED Requirements")
            || markdown.contains("## MODIFIED Requirements")
            || markdown.contains("## REMOVED Requirements")
            || markdown.contains("## RENAMED Requirements")
    }

    fn validate_requirements(&self, markdown: &str) -> Vec<MarkdownValidationError> {
        let mut errors = Vec::new();
        let req_regex = Regex::new(r"### Requirement: .+").unwrap();

        if !req_regex.is_match(markdown) {
            errors.push(MarkdownValidationError {
                line: None,
                error_type: MarkdownValidationErrorType::InvalidRequirementHeader,
                message: "Requirements must use '### Requirement: [Name]' format".to_string(),
            });
        }

        errors
    }

    fn validate_scenarios(&self, markdown: &str) -> Vec<MarkdownValidationError> {
        let mut errors = Vec::new();

        // Check for proper scenario headers (exactly 4 hashtags)
        let scenario_header_regex = Regex::new(r"#### Scenario: .+").unwrap();
        if !scenario_header_regex.is_match(markdown) {
            errors.push(MarkdownValidationError {
                line: None,
                error_type: MarkdownValidationErrorType::MissingScenarioHeader,
                message: "Scenarios must use '#### Scenario: [Name]' format (exactly 4 hashtags)"
                    .to_string(),
            });
        }

        // Check WHEN/THEN format
        let when_then_regex = Regex::new(r"- \*\*WHEN\*\* .+\n- \*\*THEN\*\* .+").unwrap();
        if !when_then_regex.is_match(markdown) {
            errors.push(MarkdownValidationError {
                line: None,
                error_type: MarkdownValidationErrorType::InvalidScenarioFormat,
                message: "Scenarios must use '- **WHEN** ...' and '- **THEN** ...' format"
                    .to_string(),
            });
        }

        errors
    }

    fn validate_normative_language(&self, markdown: &str) -> Vec<MarkdownValidationError> {
        let mut errors = Vec::new();

        // Check for SHALL or MUST in requirements
        let has_normative = markdown.contains(" SHALL ") || markdown.contains(" MUST ");

        if !has_normative && markdown.contains("### Requirement:") {
            errors.push(MarkdownValidationError {
                line: None,
                error_type: MarkdownValidationErrorType::MissingNormativeLanguage,
                message: "Requirements must use SHALL or MUST (not should/may)".to_string(),
            });
        }

        errors
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_delta_markdown() {
        let validator = OpenSpecMarkdownValidator::new(true);
        let markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication using JWT tokens.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
- **AND** the token expires after 24 hours
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(errors.is_empty(), "Valid markdown should have no errors");
    }

    #[test]
    fn test_missing_delta_operation() {
        let validator = OpenSpecMarkdownValidator::new(false);
        let markdown = r#"### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            errors
                .iter()
                .any(|e| e.error_type == MarkdownValidationErrorType::InvalidDeltaOperation),
            "Should detect missing delta operation"
        );
    }

    #[test]
    fn test_invalid_scenario_header() {
        let validator = OpenSpecMarkdownValidator::new(false);
        let markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            errors
                .iter()
                .any(|e| e.error_type == MarkdownValidationErrorType::MissingScenarioHeader),
            "Should detect incorrect scenario header (3 hashtags instead of 4)"
        );
    }

    #[test]
    fn test_missing_when_then_format() {
        let validator = OpenSpecMarkdownValidator::new(false);
        let markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication.

#### Scenario: Successful login
WHEN valid credentials are provided
THEN a JWT token is returned
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            errors
                .iter()
                .any(|e| e.error_type == MarkdownValidationErrorType::InvalidScenarioFormat),
            "Should detect missing bullet point format"
        );
    }

    #[test]
    fn test_missing_normative_language_strict() {
        let validator = OpenSpecMarkdownValidator::new(true);
        let markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system should provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            errors
                .iter()
                .any(|e| e.error_type == MarkdownValidationErrorType::MissingNormativeLanguage),
            "Strict mode should detect missing SHALL/MUST"
        );
    }

    #[test]
    fn test_missing_normative_language_relaxed() {
        let validator = OpenSpecMarkdownValidator::new(false);
        let markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system should provide secure user authentication.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            !errors
                .iter()
                .any(|e| e.error_type == MarkdownValidationErrorType::MissingNormativeLanguage),
            "Relaxed mode should not check normative language"
        );
    }

    #[test]
    fn test_multiple_scenarios() {
        let validator = OpenSpecMarkdownValidator::new(true);
        let markdown = r#"## ADDED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication using JWT tokens.

#### Scenario: Successful login
- **WHEN** valid credentials are provided
- **THEN** a JWT token is returned
- **AND** the token expires after 24 hours

#### Scenario: Failed login
- **WHEN** invalid credentials are provided
- **THEN** an error message is returned
- **AND** no token is generated
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(errors.is_empty(), "Multiple valid scenarios should pass");
    }

    #[test]
    fn test_modified_requirements() {
        let validator = OpenSpecMarkdownValidator::new(true);
        let markdown = r#"## MODIFIED Requirements

### Requirement: User Authentication
The system SHALL provide secure user authentication using OAuth 2.0.

#### Scenario: OAuth login
- **WHEN** user initiates OAuth flow
- **THEN** redirect to OAuth provider
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            errors.is_empty(),
            "MODIFIED delta operation should be valid"
        );
    }

    #[test]
    fn test_removed_requirements() {
        let validator = OpenSpecMarkdownValidator::new(false);
        let markdown = r#"## REMOVED Requirements

### Requirement: Legacy Authentication
The old authentication system is being removed.

#### Scenario: Migration complete
- **WHEN** all users have migrated
- **THEN** old system can be removed
"#;

        let errors = validator.validate_delta_markdown(markdown);
        assert!(
            errors.is_empty(),
            "REMOVED delta operation should be valid"
        );
    }
}
