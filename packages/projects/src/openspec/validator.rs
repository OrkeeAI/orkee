// ABOUTME: OpenSpec validator for parsed specs
// ABOUTME: Validates spec structure, requirements, and scenarios against database constraints

use super::types::{ParsedCapability, ParsedRequirement, ParsedScenario, ParsedSpec};
use std::collections::{HashMap, HashSet};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Duplicate capability name: {0}")]
    DuplicateCapability(String),

    #[error("Duplicate requirement name '{0}' in capability '{1}'")]
    DuplicateRequirement(String, String),

    #[error("Empty capability name")]
    EmptyCapabilityName,

    #[error("Empty requirement name in capability '{0}'")]
    EmptyRequirementName(String),

    #[error("Capability '{0}' has no requirements")]
    NoRequirements(String),

    #[error("Requirement '{0}' has no scenarios")]
    NoScenarios(String),

    #[error("Invalid scenario in requirement '{0}': {1}")]
    InvalidScenario(String, String),

    #[error("Scenario name too long: {0} characters (max 200)")]
    ScenarioNameTooLong(usize),

    #[error("Capability name too long: {0} characters (max 200)")]
    CapabilityNameTooLong(usize),

    #[error("Requirement name too long: {0} characters (max 200)")]
    RequirementNameTooLong(usize),

    #[error("Purpose text too long: {0} characters (max 5000)")]
    PurposeTooLong(usize),

    #[error("Description too long: {0} characters (max 5000)")]
    DescriptionTooLong(usize),

    #[error("WHEN clause empty in scenario '{0}'")]
    EmptyWhenClause(String),

    #[error("THEN clause empty in scenario '{0}'")]
    EmptyThenClause(String),

    #[error("Spec contains no capabilities")]
    EmptySpec,
}

pub type ValidationResult<T> = Result<T, ValidationError>;

const MAX_NAME_LENGTH: usize = 200;
const MAX_TEXT_LENGTH: usize = 5000;
const MAX_CLAUSE_LENGTH: usize = 1000;

/// Validate a complete parsed spec
pub fn validate_spec(spec: &ParsedSpec) -> ValidationResult<()> {
    if spec.capabilities.is_empty() {
        return Err(ValidationError::EmptySpec);
    }

    // Check for duplicate capability names
    let mut capability_names = HashSet::new();

    for capability in &spec.capabilities {
        validate_capability(capability)?;

        if !capability_names.insert(&capability.name) {
            return Err(ValidationError::DuplicateCapability(
                capability.name.clone(),
            ));
        }
    }

    Ok(())
}

/// Validate a single capability
pub fn validate_capability(capability: &ParsedCapability) -> ValidationResult<()> {
    // Check name
    if capability.name.trim().is_empty() {
        return Err(ValidationError::EmptyCapabilityName);
    }

    if capability.name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::CapabilityNameTooLong(
            capability.name.len(),
        ));
    }

    // Check purpose length
    if capability.purpose.len() > MAX_TEXT_LENGTH {
        return Err(ValidationError::PurposeTooLong(capability.purpose.len()));
    }

    // Must have at least one requirement
    if capability.requirements.is_empty() {
        return Err(ValidationError::NoRequirements(capability.name.clone()));
    }

    // Check for duplicate requirement names within this capability
    let mut requirement_names = HashSet::new();

    for requirement in &capability.requirements {
        validate_requirement(requirement, &capability.name)?;

        if !requirement_names.insert(&requirement.name) {
            return Err(ValidationError::DuplicateRequirement(
                requirement.name.clone(),
                capability.name.clone(),
            ));
        }
    }

    Ok(())
}

/// Validate a single requirement
pub fn validate_requirement(
    requirement: &ParsedRequirement,
    capability_name: &str,
) -> ValidationResult<()> {
    // Check name
    if requirement.name.trim().is_empty() {
        return Err(ValidationError::EmptyRequirementName(
            capability_name.to_string(),
        ));
    }

    if requirement.name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::RequirementNameTooLong(
            requirement.name.len(),
        ));
    }

    // Check description length
    if requirement.description.len() > MAX_TEXT_LENGTH {
        return Err(ValidationError::DescriptionTooLong(
            requirement.description.len(),
        ));
    }

    // Must have at least one scenario
    if requirement.scenarios.is_empty() {
        return Err(ValidationError::NoScenarios(requirement.name.clone()));
    }

    // Validate each scenario
    for scenario in &requirement.scenarios {
        validate_scenario(scenario, &requirement.name)?;
    }

    Ok(())
}

/// Validate a single scenario
pub fn validate_scenario(
    scenario: &ParsedScenario,
    requirement_name: &str,
) -> ValidationResult<()> {
    // Check name
    if scenario.name.len() > MAX_NAME_LENGTH {
        return Err(ValidationError::ScenarioNameTooLong(scenario.name.len()));
    }

    // Check WHEN clause
    if scenario.when.trim().is_empty() {
        return Err(ValidationError::EmptyWhenClause(scenario.name.clone()));
    }

    if scenario.when.len() > MAX_CLAUSE_LENGTH {
        return Err(ValidationError::InvalidScenario(
            requirement_name.to_string(),
            format!("WHEN clause too long: {} characters", scenario.when.len()),
        ));
    }

    // Check THEN clause
    if scenario.then.trim().is_empty() {
        return Err(ValidationError::EmptyThenClause(scenario.name.clone()));
    }

    if scenario.then.len() > MAX_CLAUSE_LENGTH {
        return Err(ValidationError::InvalidScenario(
            requirement_name.to_string(),
            format!("THEN clause too long: {} characters", scenario.then.len()),
        ));
    }

    // Validate AND clauses
    for and_clause in &scenario.and {
        if and_clause.trim().is_empty() {
            return Err(ValidationError::InvalidScenario(
                requirement_name.to_string(),
                "Empty AND clause".to_string(),
            ));
        }

        if and_clause.len() > MAX_CLAUSE_LENGTH {
            return Err(ValidationError::InvalidScenario(
                requirement_name.to_string(),
                format!("AND clause too long: {} characters", and_clause.len()),
            ));
        }
    }

    Ok(())
}

/// Validation statistics for reporting
#[derive(Debug, Clone)]
pub struct ValidationStats {
    pub total_capabilities: usize,
    pub total_requirements: usize,
    pub total_scenarios: usize,
    pub avg_requirements_per_capability: f64,
    pub avg_scenarios_per_requirement: f64,
}

/// Calculate validation statistics for a spec
pub fn calculate_stats(spec: &ParsedSpec) -> ValidationStats {
    let total_capabilities = spec.capabilities.len();
    let total_requirements: usize = spec
        .capabilities
        .iter()
        .map(|c| c.requirements.len())
        .sum();
    let total_scenarios: usize = spec
        .capabilities
        .iter()
        .flat_map(|c| &c.requirements)
        .map(|r| r.scenarios.len())
        .sum();

    let avg_requirements_per_capability = if total_capabilities > 0 {
        total_requirements as f64 / total_capabilities as f64
    } else {
        0.0
    };

    let avg_scenarios_per_requirement = if total_requirements > 0 {
        total_scenarios as f64 / total_requirements as f64
    } else {
        0.0
    };

    ValidationStats {
        total_capabilities,
        total_requirements,
        total_scenarios,
        avg_requirements_per_capability,
        avg_scenarios_per_requirement,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_valid_spec() {
        let spec = ParsedSpec {
            capabilities: vec![ParsedCapability {
                name: "Authentication".to_string(),
                purpose: "Handle user authentication".to_string(),
                requirements: vec![ParsedRequirement {
                    name: "User Login".to_string(),
                    description: "Users can log in".to_string(),
                    scenarios: vec![ParsedScenario {
                        name: "Valid login".to_string(),
                        when: "user enters valid credentials".to_string(),
                        then: "user is logged in".to_string(),
                        and: vec!["session is created".to_string()],
                    }],
                }],
            }],
            raw_markdown: String::new(),
        };

        assert!(validate_spec(&spec).is_ok());
    }

    #[test]
    fn test_duplicate_capability_names() {
        let spec = ParsedSpec {
            capabilities: vec![
                ParsedCapability {
                    name: "Auth".to_string(),
                    purpose: "Purpose 1".to_string(),
                    requirements: vec![ParsedRequirement {
                        name: "Req1".to_string(),
                        description: "Desc".to_string(),
                        scenarios: vec![ParsedScenario {
                            name: "S1".to_string(),
                            when: "when".to_string(),
                            then: "then".to_string(),
                            and: vec![],
                        }],
                    }],
                },
                ParsedCapability {
                    name: "Auth".to_string(),
                    purpose: "Purpose 2".to_string(),
                    requirements: vec![ParsedRequirement {
                        name: "Req2".to_string(),
                        description: "Desc".to_string(),
                        scenarios: vec![ParsedScenario {
                            name: "S2".to_string(),
                            when: "when".to_string(),
                            then: "then".to_string(),
                            and: vec![],
                        }],
                    }],
                },
            ],
            raw_markdown: String::new(),
        };

        assert!(matches!(
            validate_spec(&spec),
            Err(ValidationError::DuplicateCapability(_))
        ));
    }

    #[test]
    fn test_empty_when_clause() {
        let scenario = ParsedScenario {
            name: "Test".to_string(),
            when: "   ".to_string(),
            then: "something happens".to_string(),
            and: vec![],
        };

        assert!(matches!(
            validate_scenario(&scenario, "Test Req"),
            Err(ValidationError::EmptyWhenClause(_))
        ));
    }

    #[test]
    fn test_calculate_stats() {
        let spec = ParsedSpec {
            capabilities: vec![
                ParsedCapability {
                    name: "Cap1".to_string(),
                    purpose: "Purpose".to_string(),
                    requirements: vec![
                        ParsedRequirement {
                            name: "Req1".to_string(),
                            description: "Desc".to_string(),
                            scenarios: vec![
                                ParsedScenario {
                                    name: "S1".to_string(),
                                    when: "when".to_string(),
                                    then: "then".to_string(),
                                    and: vec![],
                                },
                                ParsedScenario {
                                    name: "S2".to_string(),
                                    when: "when".to_string(),
                                    then: "then".to_string(),
                                    and: vec![],
                                },
                            ],
                        },
                        ParsedRequirement {
                            name: "Req2".to_string(),
                            description: "Desc".to_string(),
                            scenarios: vec![ParsedScenario {
                                name: "S3".to_string(),
                                when: "when".to_string(),
                                then: "then".to_string(),
                                and: vec![],
                            }],
                        },
                    ],
                },
                ParsedCapability {
                    name: "Cap2".to_string(),
                    purpose: "Purpose".to_string(),
                    requirements: vec![ParsedRequirement {
                        name: "Req3".to_string(),
                        description: "Desc".to_string(),
                        scenarios: vec![ParsedScenario {
                            name: "S4".to_string(),
                            when: "when".to_string(),
                            then: "then".to_string(),
                            and: vec![],
                        }],
                    }],
                },
            ],
            raw_markdown: String::new(),
        };

        let stats = calculate_stats(&spec);
        assert_eq!(stats.total_capabilities, 2);
        assert_eq!(stats.total_requirements, 3);
        assert_eq!(stats.total_scenarios, 4);
        assert_eq!(stats.avg_requirements_per_capability, 1.5);
        assert!((stats.avg_scenarios_per_requirement - 1.333).abs() < 0.01);
    }
}
