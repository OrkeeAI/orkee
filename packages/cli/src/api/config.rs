use axum::{http::StatusCode, response::Json};
use serde::Serialize;
use std::env;

#[derive(Serialize)]
pub struct ConfigResponse {
    pub success: bool,
    pub data: Option<Config>,
    pub error: Option<String>,
}

#[derive(Serialize)]
pub struct Config {
    pub cloud_enabled: bool,
}

pub async fn get_config() -> Result<Json<ConfigResponse>, StatusCode> {
    let cloud_enabled = env::var("ORKEE_CLOUD_ENABLED")
        .unwrap_or_else(|_| "false".to_string())
        .to_lowercase()
        == "true";

    let config = Config { cloud_enabled };

    Ok(Json(ConfigResponse {
        success: true,
        data: Some(config),
        error: None,
    }))
}
