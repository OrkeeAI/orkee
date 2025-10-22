use axum::{extract::Extension, response::Result, Json};
use serde_json::{json, Value};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::middleware::CsrfLayer;

pub async fn health_check() -> Result<Json<Value>> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0); // Fallback to 0 if system time is somehow before UNIX_EPOCH

    Ok(Json(json!({
        "status": "healthy",
        "timestamp": timestamp,
        "version": env!("CARGO_PKG_VERSION"),
        "service": "orkee-cli"
    })))
}

pub async fn status_check() -> Result<Json<Value>> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0); // Fallback to 0 if system time is somehow before UNIX_EPOCH

    // This would contain more detailed system information in a real implementation
    Ok(Json(json!({
        "status": "healthy",
        "timestamp": timestamp,
        "version": env!("CARGO_PKG_VERSION"),
        "service": "orkee-cli",
        "uptime": timestamp, // In a real app, this would be actual uptime
        "memory": {
            "used": "unknown",
            "available": "unknown"
        },
        "connections": {
            "active": 0,
            "total": 0
        }
    })))
}

pub async fn get_csrf_token(Extension(csrf_layer): Extension<CsrfLayer>) -> Result<Json<Value>> {
    Ok(Json(json!({
        "csrf_token": csrf_layer.token()
    })))
}
