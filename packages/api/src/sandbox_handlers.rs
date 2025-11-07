// ABOUTME: HTTP request handlers for sandbox settings and instance operations
// ABOUTME: Handles sandbox configuration management and lifecycle operations

use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{error, info};

use super::auth::CurrentUser;
use super::response::ok_or_internal_error;
use orkee_projects::DbState;
use orkee_sandbox::{CreateSandboxRequest, ProviderSettings, Sandbox, SandboxSettings};

/// Get sandbox settings
pub async fn get_sandbox_settings(State(db): State<DbState>) -> impl IntoResponse {
    info!("Getting sandbox settings");

    let result = db.sandbox_settings.get_sandbox_settings().await;
    ok_or_internal_error(result, "Failed to get sandbox settings")
}

/// Request body for updating sandbox settings
#[derive(Deserialize)]
pub struct UpdateSandboxSettingsRequest {
    #[serde(flatten)]
    pub settings: SandboxSettings,
}

/// Update sandbox settings
pub async fn update_sandbox_settings(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(request): Json<UpdateSandboxSettingsRequest>,
) -> impl IntoResponse {
    info!("Updating sandbox settings for user: {}", current_user.id);

    let updated_by = Some(current_user.id.as_str());

    let result = db
        .sandbox_settings
        .update_sandbox_settings(&request.settings, updated_by)
        .await
        .map(|_| serde_json::json!({"message": "Sandbox settings updated successfully"}));

    ok_or_internal_error(result, "Failed to update sandbox settings")
}

/// List all provider settings
pub async fn list_provider_settings(State(db): State<DbState>) -> impl IntoResponse {
    info!("Listing all provider settings");

    let result = db.sandbox_settings.list_provider_settings().await;
    ok_or_internal_error(result, "Failed to list provider settings")
}

/// Get provider settings by provider ID
pub async fn get_provider_settings(
    State(db): State<DbState>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    info!("Getting provider settings for: {}", provider);

    let result = db.sandbox_settings.get_provider_settings(&provider).await;
    ok_or_internal_error(result, "Failed to get provider settings")
}

/// Request body for updating provider settings
#[derive(Deserialize)]
pub struct UpdateProviderSettingsRequest {
    #[serde(flatten)]
    pub settings: ProviderSettings,
}

/// Update provider settings
pub async fn update_provider_settings(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path(provider): Path<String>,
    Json(request): Json<UpdateProviderSettingsRequest>,
) -> impl IntoResponse {
    info!(
        "Updating provider settings for: {} (user: {})",
        provider, current_user.id
    );

    // Validate that the provider in the path matches the request body
    if request.settings.provider != provider {
        let error: Result<(), orkee_storage::StorageError> =
            Err(orkee_storage::StorageError::InvalidInput(
                "Provider in path does not match provider in request body".to_string(),
            ));
        return ok_or_internal_error(error, "Provider mismatch");
    }

    let updated_by = Some(current_user.id.as_str());

    let result = db
        .sandbox_settings
        .update_provider_settings(&request.settings, updated_by)
        .await
        .map(|_| serde_json::json!({"message": "Provider settings updated successfully"}));

    ok_or_internal_error(result, "Failed to update provider settings")
}

/// Delete provider settings
pub async fn delete_provider_settings(
    State(db): State<DbState>,
    Path(provider): Path<String>,
) -> impl IntoResponse {
    info!("Deleting provider settings for: {}", provider);

    let result = db
        .sandbox_settings
        .delete_provider_settings(&provider)
        .await
        .map(|_| serde_json::json!({"message": "Provider settings deleted successfully"}));

    ok_or_internal_error(result, "Failed to delete provider settings")
}

// ============================================================================
// SANDBOX INSTANCE OPERATIONS
// ============================================================================

/// Request body for creating a sandbox
#[derive(Deserialize)]
pub struct CreateSandboxRequestBody {
    pub name: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub cpu_cores: Option<f32>,
    #[serde(default)]
    pub memory_mb: Option<u32>,
    #[serde(default)]
    pub disk_gb: Option<u32>,
    #[serde(default)]
    pub gpu_type: Option<String>,
    #[serde(default)]
    pub network_mode: Option<String>,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub template_id: Option<String>,
    #[serde(default)]
    pub env_vars: Option<HashMap<String, String>>,
    #[serde(default)]
    pub volumes: Option<Vec<VolumeRequest>>,
}

#[derive(Deserialize)]
pub struct VolumeRequest {
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
}

/// Query parameters for listing sandboxes
#[derive(Deserialize)]
pub struct ListSandboxesQuery {
    #[serde(default)]
    pub user_id: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
}

/// List all sandboxes
pub async fn list_sandboxes(
    State(db): State<DbState>,
    Query(query): Query<ListSandboxesQuery>,
) -> impl IntoResponse {
    info!("Listing sandboxes");

    // Parse status if provided
    let status = query.status.as_ref().and_then(|s| {
        use orkee_sandbox::SandboxStatus;
        SandboxStatus::from_str(s).ok()
    });

    let result = db
        .sandbox_manager
        .list_sandboxes(query.user_id.as_deref(), status)
        .await;

    ok_or_internal_error(result, "Failed to list sandboxes")
}

/// Get sandbox by ID
pub async fn get_sandbox(State(db): State<DbState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("Getting sandbox: {}", id);

    let result = db.sandbox_manager.get_sandbox(&id).await;

    ok_or_internal_error(result, "Failed to get sandbox")
}

/// Create a new sandbox
pub async fn create_sandbox(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Json(body): Json<CreateSandboxRequestBody>,
) -> impl IntoResponse {
    info!("Creating sandbox: {} for user: {}", body.name, current_user.id);

    // Get default values from settings
    let settings = match db.sandbox_settings.get_sandbox_settings().await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to get sandbox settings: {}", e);
            return ok_or_internal_error::<Sandbox, _>(
                Err(orkee_sandbox::ManagerError::NotFound(
                    "Sandbox settings not found".to_string(),
                )),
                "Failed to get sandbox settings",
            );
        }
    };

    // Build the sandbox request
    let request = CreateSandboxRequest {
        name: body.name,
        provider: body.provider.unwrap_or(settings.default_provider),
        agent_id: body.agent_id.unwrap_or_else(|| "claude".to_string()),
        user_id: current_user.id.clone(),
        project_id: None, // TODO: Get from request if available
        image: body.image,
        cpu_cores: body.cpu_cores,
        memory_mb: body.memory_mb,
        storage_gb: body.disk_gb,
        gpu_enabled: body.gpu_type.is_some(),
        gpu_model: body.gpu_type,
        env_vars: body.env_vars.unwrap_or_default(),
        volumes: body
            .volumes
            .unwrap_or_default()
            .into_iter()
            .map(|v| orkee_sandbox::providers::VolumeMount {
                host_path: v.host_path,
                container_path: v.container_path,
                readonly: v.read_only,
            })
            .collect(),
        ports: vec![], // TODO: Add port configuration
        ssh_enabled: false,
        config: None,
        metadata: None,
    };

    let result = db.sandbox_manager.create_sandbox(request).await;

    ok_or_internal_error(result, "Failed to create sandbox")
}

/// Start a sandbox
pub async fn start_sandbox(State(db): State<DbState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("Starting sandbox: {}", id);

    let result = db
        .sandbox_manager
        .start_sandbox(&id)
        .await
        .map(|_| serde_json::json!({"message": "Sandbox started successfully"}));

    ok_or_internal_error(result, "Failed to start sandbox")
}

/// Stop a sandbox
pub async fn stop_sandbox(State(db): State<DbState>, Path(id): Path<String>) -> impl IntoResponse {
    info!("Stopping sandbox: {}", id);

    // Get sandbox first to check its state
    let sandbox = match db.sandbox_manager.get_sandbox(&id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Failed to get sandbox");
        }
    };

    // Check if container exists
    if sandbox.container_id.is_none() {
        error!(
            "Cannot stop sandbox {}: no container ID (status: {:?})",
            id, sandbox.status
        );
        let result: Result<serde_json::Value, orkee_sandbox::ManagerError> =
            Err(orkee_sandbox::ManagerError::ConfigError(
                "Sandbox has no container. Please delete the sandbox.".to_string(),
            ));
        return ok_or_internal_error(result, "Failed to stop sandbox");
    }

    // Check if sandbox is in a stoppable state
    use orkee_sandbox::SandboxStatus;
    match sandbox.status {
        SandboxStatus::Running => {
            // Stoppable state - proceed
        }
        SandboxStatus::Stopped => {
            // Already stopped
            let result: Result<serde_json::Value, orkee_sandbox::ManagerError> =
                Ok(serde_json::json!({"message": "Sandbox is already stopped"}));
            return ok_or_internal_error(result, "Sandbox already stopped");
        }
        SandboxStatus::Error | SandboxStatus::Creating => {
            error!("Cannot stop sandbox {} in state {:?}", id, sandbox.status);
            let result: Result<serde_json::Value, orkee_sandbox::ManagerError> = Err(
                orkee_sandbox::ManagerError::InvalidStateTransition(format!(
                    "Cannot stop sandbox in state {:?}. Please delete the sandbox.",
                    sandbox.status
                )),
            );
            return ok_or_internal_error(result, "Failed to stop sandbox");
        }
        _ => {
            error!("Cannot stop sandbox {} in state {:?}", id, sandbox.status);
            let result: Result<serde_json::Value, orkee_sandbox::ManagerError> =
                Err(orkee_sandbox::ManagerError::InvalidStateTransition(
                    format!("Cannot stop sandbox in state {:?}", sandbox.status),
                ));
            return ok_or_internal_error(result, "Failed to stop sandbox");
        }
    }

    let timeout = 10; // 10 second timeout
    let result = db
        .sandbox_manager
        .stop_sandbox(&id, timeout)
        .await
        .map(|_| serde_json::json!({"message": "Sandbox stopped successfully"}));

    ok_or_internal_error(result, "Failed to stop sandbox")
}

/// Restart a sandbox
pub async fn restart_sandbox(
    State(db): State<DbState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Restarting sandbox: {}", id);

    // Get sandbox first to check its state
    let sandbox = match db.sandbox_manager.get_sandbox(&id).await {
        Ok(s) => s,
        Err(e) => {
            return ok_or_internal_error::<serde_json::Value, _>(Err(e), "Failed to get sandbox");
        }
    };

    // Check if container exists
    if sandbox.container_id.is_none() {
        error!(
            "Cannot restart sandbox {}: no container ID (status: {:?})",
            id, sandbox.status
        );
        let result: Result<serde_json::Value, orkee_sandbox::ManagerError> =
            Err(orkee_sandbox::ManagerError::ConfigError(
                "Sandbox has no container. Please delete and recreate the sandbox.".to_string(),
            ));
        return ok_or_internal_error(result, "Failed to restart sandbox");
    }

    // Check if sandbox is in a restartable state
    use orkee_sandbox::SandboxStatus;
    match sandbox.status {
        SandboxStatus::Running | SandboxStatus::Stopped => {
            // Restartable states - proceed
        }
        SandboxStatus::Error | SandboxStatus::Creating => {
            error!(
                "Cannot restart sandbox {} in state {:?}",
                id, sandbox.status
            );
            let result: Result<serde_json::Value, orkee_sandbox::ManagerError> = Err(
                orkee_sandbox::ManagerError::InvalidStateTransition(format!(
                    "Cannot restart sandbox in state {:?}. Please delete and recreate the sandbox.",
                    sandbox.status
                )),
            );
            return ok_or_internal_error(result, "Failed to restart sandbox");
        }
        _ => {
            error!(
                "Cannot restart sandbox {} in state {:?}",
                id, sandbox.status
            );
            let result: Result<serde_json::Value, orkee_sandbox::ManagerError> =
                Err(orkee_sandbox::ManagerError::InvalidStateTransition(
                    format!("Cannot restart sandbox in state {:?}", sandbox.status),
                ));
            return ok_or_internal_error(result, "Failed to restart sandbox");
        }
    }

    // Stop and then start
    let timeout = 10;
    let result: Result<serde_json::Value, orkee_sandbox::ManagerError> = async {
        db.sandbox_manager.stop_sandbox(&id, timeout).await?;
        db.sandbox_manager.start_sandbox(&id).await?;
        Ok(serde_json::json!({"message": "Sandbox restarted successfully"}))
    }
    .await;

    ok_or_internal_error(result, "Failed to restart sandbox")
}

/// Delete a sandbox
pub async fn delete_sandbox(
    State(db): State<DbState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting sandbox: {}", id);

    let result = db
        .sandbox_manager
        .remove_sandbox(&id, false)
        .await
        .map(|_| serde_json::json!({"message": "Sandbox deleted successfully"}));

    ok_or_internal_error(result, "Failed to delete sandbox")
}

/// Request body for executing a command
#[derive(Deserialize)]
pub struct ExecuteCommandRequestBody {
    pub command: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// Execute a command in a sandbox
pub async fn execute_command(
    State(db): State<DbState>,
    current_user: CurrentUser,
    Path(id): Path<String>,
    Json(body): Json<ExecuteCommandRequestBody>,
) -> impl IntoResponse {
    info!(
        "Executing command in sandbox {} for user {}: {}",
        id, current_user.id, body.command
    );

    // Create execution record
    let result = db
        .sandbox_manager
        .create_execution(
            &id,
            body.command,
            Some("/workspace".to_string()),
            Some(current_user.id),
            None,
        )
        .await;

    ok_or_internal_error(result, "Failed to execute command")
}

/// Get executions for a sandbox
pub async fn get_sandbox_executions(
    State(db): State<DbState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Getting executions for sandbox: {}", id);

    let result = db.sandbox_manager.list_executions(&id).await;

    ok_or_internal_error(result, "Failed to get executions")
}

/// Get sandbox metrics
pub async fn get_sandbox_metrics(
    State(db): State<DbState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    info!("Getting metrics for sandbox: {}", id);

    let result = db
        .sandbox_manager
        .get_container_info(&id)
        .await
        .map(|info| {
            // Convert ContainerInfo to a serializable JSON value
            serde_json::json!({
                "id": info.id,
                "name": info.name,
                "status": format!("{:?}", info.status),
                "ip_address": info.ip_address,
                "ports": info.ports,
                "created_at": info.created_at,
                "started_at": info.started_at,
                "metrics": info.metrics.as_ref().map(|m| serde_json::json!({
                    "cpu_usage_percent": m.cpu_usage_percent,
                    "memory_usage_mb": m.memory_usage_mb,
                    "memory_limit_mb": m.memory_limit_mb,
                    "network_rx_bytes": m.network_rx_bytes,
                    "network_tx_bytes": m.network_tx_bytes,
                })),
            })
        });

    ok_or_internal_error(result, "Failed to get metrics")
}
