use crate::config::{Config, ConfigError};
use std::env;
use rstest::rstest;

#[test]
fn test_config_from_env_defaults() {
    // Clear environment variables
    env::remove_var("PORT");
    env::remove_var("CORS_ORIGIN");
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.port, 4001);
    assert_eq!(config.cors_origin, "http://localhost:5173");
}

#[test]
fn test_config_from_env_with_custom_port() {
    env::set_var("PORT", "8080");
    env::remove_var("CORS_ORIGIN");
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.port, 8080);
    assert_eq!(config.cors_origin, "http://localhost:5173");
    
    env::remove_var("PORT");
}

#[test]
fn test_config_from_env_with_custom_cors() {
    env::remove_var("PORT");
    env::set_var("CORS_ORIGIN", "https://example.com");
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.port, 4001);
    assert_eq!(config.cors_origin, "https://example.com");
    
    env::remove_var("CORS_ORIGIN");
}

#[test]
fn test_config_from_env_with_all_custom() {
    env::set_var("PORT", "3000");
    env::set_var("CORS_ORIGIN", "https://app.example.com");
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.port, 3000);
    assert_eq!(config.cors_origin, "https://app.example.com");
    
    env::remove_var("PORT");
    env::remove_var("CORS_ORIGIN");
}

#[test]
fn test_config_invalid_port() {
    env::set_var("PORT", "not-a-number");
    
    let result = Config::from_env();
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::InvalidPort(_)));
    
    env::remove_var("PORT");
}

#[test]
fn test_config_port_zero() {
    env::set_var("PORT", "0");
    
    let result = Config::from_env();
    
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ConfigError::PortOutOfRange(0)));
    
    env::remove_var("PORT");
}

#[rstest]
#[case("1", 1)]
#[case("80", 80)]
#[case("443", 443)]
#[case("8080", 8080)]
#[case("65535", 65535)]
fn test_valid_port_numbers(#[case] port_str: &str, #[case] expected: u16) {
    env::set_var("PORT", port_str);
    
    let config = Config::from_env().unwrap();
    
    assert_eq!(config.port, expected);
    
    env::remove_var("PORT");
}

#[rstest]
#[case("-1")]
#[case("65536")]
#[case("99999")]
#[case("1.5")]
#[case("0x1234")]
fn test_invalid_port_formats(#[case] port_str: &str) {
    env::set_var("PORT", port_str);
    
    let result = Config::from_env();
    
    assert!(result.is_err());
    
    env::remove_var("PORT");
}

#[test]
fn test_config_error_display() {
    let error = ConfigError::PortOutOfRange(0);
    assert_eq!(error.to_string(), "Port 0 is out of valid range (1-65535)");
    
    let parse_error = "123abc".parse::<u16>().unwrap_err();
    let error = ConfigError::InvalidPort(parse_error);
    assert!(error.to_string().contains("Invalid port number"));
}