// ABOUTME: Environment variable parsing utilities
// ABOUTME: Provides helper functions for parsing and validating environment variables

use std::str::FromStr;
use tracing;

/// Parse an environment variable with a fallback default value
/// Returns the parsed value or the default if the variable is not set or cannot be parsed
pub fn parse_env_or_default<T>(var_name: &str, default: T) -> T
where
    T: FromStr,
{
    std::env::var(var_name)
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}

/// Parse an environment variable with validation
/// Returns the parsed value if it passes validation, otherwise returns the default
/// Logs warnings when environment variables are set but fail validation or parsing
pub fn parse_env_or_default_with_validation<T, F>(var_name: &str, default: T, validator: F) -> T
where
    T: FromStr + Copy + std::fmt::Display,
    F: Fn(T) -> bool,
{
    match std::env::var(var_name) {
        Ok(raw_value) => {
            match raw_value.parse::<T>() {
                Ok(parsed_value) => {
                    if validator(parsed_value) {
                        parsed_value
                    } else {
                        tracing::warn!(
                            "Environment variable {} has invalid value '{}', using default: {}",
                            var_name,
                            raw_value,
                            default
                        );
                        default
                    }
                }
                Err(_) => {
                    tracing::warn!(
                        "Environment variable {} has unparseable value '{}', using default: {}",
                        var_name,
                        raw_value,
                        default
                    );
                    default
                }
            }
        }
        Err(_) => {
            // Variable not set - no warning needed, this is expected behavior
            default
        }
    }
}

/// Parse an environment variable with fallback to another variable
/// Tries the primary variable first, then falls back to the secondary, then to the default
pub fn parse_env_with_fallback<T>(primary_var: &str, fallback_var: &str, default: T) -> T
where
    T: FromStr,
{
    std::env::var(primary_var)
        .or_else(|_| std::env::var(fallback_var))
        .ok()
        .and_then(|v| v.parse::<T>().ok())
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_or_default_not_set() {
        std::env::remove_var("TEST_VAR_NOT_SET");
        let result: i32 = parse_env_or_default("TEST_VAR_NOT_SET", 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_env_or_default_set() {
        std::env::set_var("TEST_VAR_SET", "100");
        let result: i32 = parse_env_or_default("TEST_VAR_SET", 42);
        assert_eq!(result, 100);
        std::env::remove_var("TEST_VAR_SET");
    }

    #[test]
    fn test_parse_env_or_default_invalid() {
        std::env::set_var("TEST_VAR_INVALID", "not_a_number");
        let result: i32 = parse_env_or_default("TEST_VAR_INVALID", 42);
        assert_eq!(result, 42);
        std::env::remove_var("TEST_VAR_INVALID");
    }

    #[test]
    fn test_parse_env_with_validation() {
        std::env::set_var("TEST_VAR_VALIDATION", "150");
        let result =
            parse_env_or_default_with_validation("TEST_VAR_VALIDATION", 100, |v| v > 0 && v <= 200);
        assert_eq!(result, 150);
        std::env::remove_var("TEST_VAR_VALIDATION");
    }

    #[test]
    fn test_parse_env_with_validation_fails() {
        std::env::set_var("TEST_VAR_VALIDATION_FAIL", "300");
        let result = parse_env_or_default_with_validation("TEST_VAR_VALIDATION_FAIL", 100, |v| {
            v > 0 && v <= 200
        });
        assert_eq!(result, 100); // Should return default because validation failed
        std::env::remove_var("TEST_VAR_VALIDATION_FAIL");
    }

    #[test]
    fn test_parse_env_with_fallback_primary() {
        std::env::set_var("PRIMARY_VAR", "100");
        std::env::set_var("FALLBACK_VAR", "200");
        let result: i32 = parse_env_with_fallback("PRIMARY_VAR", "FALLBACK_VAR", 42);
        assert_eq!(result, 100);
        std::env::remove_var("PRIMARY_VAR");
        std::env::remove_var("FALLBACK_VAR");
    }

    #[test]
    fn test_parse_env_with_fallback_secondary() {
        std::env::remove_var("PRIMARY_VAR_2");
        std::env::set_var("FALLBACK_VAR_2", "200");
        let result: i32 = parse_env_with_fallback("PRIMARY_VAR_2", "FALLBACK_VAR_2", 42);
        assert_eq!(result, 200);
        std::env::remove_var("FALLBACK_VAR_2");
    }

    #[test]
    fn test_parse_env_with_fallback_default() {
        std::env::remove_var("PRIMARY_VAR_3");
        std::env::remove_var("FALLBACK_VAR_3");
        let result: i32 = parse_env_with_fallback("PRIMARY_VAR_3", "FALLBACK_VAR_3", 42);
        assert_eq!(result, 42);
    }
}
