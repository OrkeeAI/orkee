use axum::{extract::State, http::StatusCode, response::Json};
use orkee_projects::db::DbState;
use serde::Serialize;
use std::env;
use tracing::warn;

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

pub async fn get_config(State(db): State<DbState>) -> Result<Json<ConfigResponse>, StatusCode> {
    // Try to get cloud_enabled from database first, fall back to env var
    let cloud_enabled = match db.settings_storage.get("cloud_enabled").await {
        Ok(setting) => setting.value.to_lowercase() == "true",
        Err(e) => {
            warn!("Failed to get cloud_enabled from database, falling back to env var: {}", e);
            env::var("ORKEE_CLOUD_ENABLED")
                .unwrap_or_else(|_| "false".to_string())
                .to_lowercase()
                == "true"
        }
    };

    let config = Config { cloud_enabled };

    Ok(Json(ConfigResponse {
        success: true,
        data: Some(config),
        error: None,
    }))
}
