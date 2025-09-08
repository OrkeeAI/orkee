use crate::mcp::*;
use rstest::rstest;
use serde_json::json;
use pretty_assertions::assert_eq;

#[tokio::test]
async fn test_initialize_request() {
    let request = InitializeRequest {
        protocol_version: "2024-11-05".to_string(),
        capabilities: ClientCapabilities {
            roots: Some(RootsCapability {
                list_changed: Some(true),
            }),
            sampling: None,
        },
        client_info: ClientInfo {
            name: "test-client".to_string(),
            version: "1.0.0".to_string(),
        },
    };

    let result = initialize(Some(request)).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.protocol_version, "2024-11-05");
    assert_eq!(response.server_info.name, "orkee");
    assert!(response.capabilities.tools.is_some());
    assert!(response.capabilities.prompts.is_some());
    assert!(response.capabilities.resources.is_some());
}

#[tokio::test]
async fn test_initialize_without_request() {
    let result = initialize(None).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response.protocol_version, "2024-11-05");
}

#[tokio::test]
async fn test_ping() {
    let result = ping(None).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response, json!({}));
}

#[tokio::test]
async fn test_ping_with_params() {
    let params = json!({"test": "value"});
    let result = ping(Some(params)).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response, json!({}));
}

#[tokio::test]
async fn test_set_logging_level() {
    let request = LoggingLevel {
        level: "debug".to_string(),
    };
    
    let result = logging_set_level(Some(request)).await;
    assert!(result.is_ok());
    
    let response = result.unwrap();
    assert_eq!(response, json!({}));
}

#[rstest]
#[case("debug")]
#[case("info")]
#[case("warn")]
#[case("error")]
#[tokio::test]
async fn test_logging_levels(#[case] level: &str) {
    let request = LoggingLevel {
        level: level.to_string(),
    };
    
    let result = logging_set_level(Some(request)).await;
    assert!(result.is_ok());
}

#[test]
fn test_server_capabilities_serialization() {
    let capabilities = ServerCapabilities {
        logging: None,
        prompts: Some(PromptsCapability {
            list_changed: Some(true),
        }),
        resources: Some(ResourcesCapability {
            subscribe: Some(false),
            list_changed: Some(true),
        }),
        tools: Some(ToolsCapability {
            list_changed: Some(true),
        }),
    };
    
    let json = serde_json::to_value(&capabilities);
    assert!(json.is_ok());
    
    let value = json.unwrap();
    assert!(value.get("prompts").is_some());
    assert!(value.get("resources").is_some());
    assert!(value.get("tools").is_some());
}

#[test]
fn test_client_info_deserialization() {
    let json = json!({
        "name": "test-client",
        "version": "1.0.0"
    });
    
    let result: Result<ClientInfo, _> = serde_json::from_value(json);
    assert!(result.is_ok());
    
    let info = result.unwrap();
    assert_eq!(info.name, "test-client");
    assert_eq!(info.version, "1.0.0");
}

#[test]
fn test_initialize_request_deserialization() {
    let json = json!({
        "protocolVersion": "2024-11-05",
        "capabilities": {
            "roots": {
                "listChanged": true
            }
        },
        "clientInfo": {
            "name": "claude",
            "version": "1.0"
        }
    });
    
    let result: Result<InitializeRequest, _> = serde_json::from_value(json);
    assert!(result.is_ok());
    
    let request = result.unwrap();
    assert_eq!(request.protocol_version, "2024-11-05");
    assert_eq!(request.client_info.name, "claude");
}