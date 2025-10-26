use crate::manager::{
    create_project as manager_create_project, delete_project as manager_delete_project,
    export_database as manager_export_database, get_all_projects,
    get_project as manager_get_project, get_project_by_name as manager_get_project_by_name,
    get_project_by_path as manager_get_project_by_path, import_database as manager_import_database,
    update_project as manager_update_project, ManagerError,
};
use axum::{
    body::Body,
    extract::{Json, Path},
    http::{header, StatusCode},
    response::{IntoResponse, Json as ResponseJson, Response},
};
use orkee_core::types::{ProjectCreateInput, ProjectUpdateInput};
use serde::{Deserialize, Serialize};
use std::process::Command;
use tracing::{error, info, warn};

/// Standard API response wrapper
#[derive(Serialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> ApiResponse<()> {
        ApiResponse {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

/// Request body for getting project by path
#[derive(Deserialize)]
pub struct GetProjectByPathRequest {
    #[serde(rename = "projectRoot")]
    project_root: String,
}

/// Request body for checking taskmaster folder
#[derive(Deserialize)]
pub struct CheckTaskmasterRequest {
    #[serde(rename = "projectRoot")]
    project_root: String,
}

/// Response for taskmaster check
#[derive(Serialize)]
pub struct CheckTaskmasterResponse {
    #[serde(rename = "hasTaskmaster")]
    has_taskmaster: bool,
    #[serde(rename = "taskSource")]
    task_source: String,
}

/// Request body for opening project in editor
#[derive(Deserialize, Debug)]
pub struct OpenInEditorRequest {
    #[serde(rename = "projectId")]
    project_id: Option<String>,
    #[serde(rename = "projectPath")]
    project_path: Option<String>,
    #[serde(rename = "editorId")]
    editor_id: Option<String>,
}

/// Response for opening project in editor
#[derive(Serialize)]
pub struct OpenInEditorResponse {
    message: String,
    #[serde(rename = "detectedCommand")]
    detected_command: Option<String>,
}

/// Convert manager errors to HTTP responses
impl IntoResponse for ManagerError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match &self {
            ManagerError::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            ManagerError::DuplicateName(_) | ManagerError::DuplicatePath(_) => {
                (StatusCode::CONFLICT, self.to_string())
            }
            ManagerError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            ManagerError::Storage(e) => {
                error!("Storage error details: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Storage error".to_string(),
                )
            }
        };

        let response = ApiResponse::<()>::error(message);
        (status, ResponseJson(response)).into_response()
    }
}

/// List all projects
pub async fn list_projects() -> impl IntoResponse {
    info!("Listing all projects");

    match get_all_projects().await {
        Ok(projects) => {
            info!("Retrieved {} projects", projects.len());
            (StatusCode::OK, ResponseJson(ApiResponse::success(projects))).into_response()
        }
        Err(e) => {
            error!("Failed to list projects: {}", e);
            e.into_response()
        }
    }
}

/// Get a specific project by ID
pub async fn get_project(Path(id): Path<String>) -> impl IntoResponse {
    info!("Getting project with ID: {}", id);

    match manager_get_project(&id).await {
        Ok(Some(project)) => {
            info!("Found project: {}", project.name);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Ok(None) => {
            info!("Project not found: {}", id);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to get project {}: {}", id, e);
            e.into_response()
        }
    }
}

/// Get a project by name
pub async fn get_project_by_name(Path(name): Path<String>) -> impl IntoResponse {
    info!("Getting project with name: {}", name);

    match manager_get_project_by_name(&name).await {
        Ok(Some(project)) => {
            info!("Found project by name: {}", project.name);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Ok(None) => {
            info!("Project not found by name: {}", name);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to get project by name {}: {}", name, e);
            e.into_response()
        }
    }
}

/// Get a project by path
pub async fn get_project_by_path(
    Json(request): Json<GetProjectByPathRequest>,
) -> impl IntoResponse {
    info!("Getting project with path: {}", request.project_root);

    match manager_get_project_by_path(&request.project_root).await {
        Ok(Some(project)) => {
            info!("Found project by path: {}", project.name);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Ok(None) => {
            info!("Project not found by path: {}", request.project_root);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            )
                .into_response()
        }
        Err(e) => {
            error!(
                "Failed to get project by path {}: {}",
                request.project_root, e
            );
            e.into_response()
        }
    }
}

/// Create a new project
pub async fn create_project(Json(input): Json<ProjectCreateInput>) -> impl IntoResponse {
    info!("Creating project: {}", input.name);

    match manager_create_project(input).await {
        Ok(project) => {
            info!("Created project: {} (ID: {})", project.name, project.id);
            (
                StatusCode::CREATED,
                ResponseJson(ApiResponse::success(project)),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to create project: {}", e);
            e.into_response()
        }
    }
}

/// Update an existing project
pub async fn update_project(
    Path(id): Path<String>,
    Json(updates): Json<ProjectUpdateInput>,
) -> impl IntoResponse {
    info!("Updating project: {}", id);

    match manager_update_project(&id, updates).await {
        Ok(project) => {
            info!("Updated project: {} (ID: {})", project.name, project.id);
            (StatusCode::OK, ResponseJson(ApiResponse::success(project))).into_response()
        }
        Err(e) => {
            error!("Failed to update project {}: {}", id, e);
            e.into_response()
        }
    }
}

/// Delete a project
pub async fn delete_project(Path(id): Path<String>) -> impl IntoResponse {
    info!("Deleting project: {}", id);

    match manager_delete_project(&id).await {
        Ok(true) => {
            info!("Deleted project: {}", id);
            (
                StatusCode::OK,
                ResponseJson(ApiResponse::success("Project deleted successfully")),
            )
                .into_response()
        }
        Ok(false) => {
            info!("Project not found for deletion: {}", id);
            (
                StatusCode::NOT_FOUND,
                ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
            )
                .into_response()
        }
        Err(e) => {
            error!("Failed to delete project {}: {}", id, e);
            e.into_response()
        }
    }
}

/// Check if project has .taskmaster folder
pub async fn check_taskmaster(Json(request): Json<CheckTaskmasterRequest>) -> impl IntoResponse {
    info!("Checking taskmaster folder for: {}", request.project_root);

    let taskmaster_path = std::path::Path::new(&request.project_root).join(".taskmaster");
    let has_taskmaster = taskmaster_path.exists() && taskmaster_path.is_dir();

    let response = CheckTaskmasterResponse {
        has_taskmaster,
        task_source: if has_taskmaster {
            "taskmaster".to_string()
        } else {
            "manual".to_string()
        },
    };

    info!(
        "Taskmaster check result for {}: has_taskmaster={}, task_source={}",
        request.project_root, response.has_taskmaster, response.task_source
    );

    (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
}

/// Open a project in the configured editor
pub async fn open_in_editor(Json(request): Json<OpenInEditorRequest>) -> impl IntoResponse {
    info!("Opening project in editor: {:?}", request);

    // Determine project path - either from projectId lookup or direct projectPath
    let project_path = if let Some(project_id) = request.project_id {
        match manager_get_project(&project_id).await {
            Ok(Some(project)) => project.project_root,
            Ok(None) => {
                error!("Project not found: {}", project_id);
                return (
                    StatusCode::NOT_FOUND,
                    ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
                )
                    .into_response();
            }
            Err(e) => {
                error!("Failed to get project {}: {}", project_id, e);
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    ResponseJson(ApiResponse::<()>::error("Database error".to_string())),
                )
                    .into_response();
            }
        }
    } else if let Some(project_path) = request.project_path {
        project_path
    } else {
        return (
            StatusCode::BAD_REQUEST,
            ResponseJson(ApiResponse::<()>::error(
                "Either projectId or projectPath must be provided".to_string(),
            )),
        )
            .into_response();
    };

    // Use the preferred editor if specified, otherwise default to VS Code
    let result = if let Some(editor_id) = request.editor_id {
        try_open_with_editor(&project_path, &editor_id)
    } else {
        try_open_with_vscode(&project_path)
    };

    match result {
        Ok(message) => {
            info!("Successfully opened project in editor: {}", message);
            let response = OpenInEditorResponse {
                message,
                detected_command: Some("code".to_string()),
            };
            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(error) => {
            error!("Failed to open project in editor: {}", error);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(error)),
            )
                .into_response()
        }
    }
}

/// Test editor configuration (GET endpoint for testing)
pub async fn test_editor_config() -> impl IntoResponse {
    info!("Testing editor configuration");

    // Try to detect VS Code for now
    let result = detect_vscode();

    match result {
        Ok(command) => {
            let response = OpenInEditorResponse {
                message: "Editor configuration test successful".to_string(),
                detected_command: Some(command),
            };
            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(error) => {
            warn!("Editor configuration test failed: {}", error);
            (
                StatusCode::OK,
                ResponseJson(ApiResponse::<()>::error(error)),
            )
                .into_response()
        }
    }
}

/// Helper function to try opening a project with VS Code
fn try_open_with_vscode(project_path: &str) -> Result<String, String> {
    // Check if path exists
    if !std::path::Path::new(project_path).exists() {
        return Err("Project path does not exist".to_string());
    }

    // Try different ways to open VS Code based on platform
    let commands = if cfg!(target_os = "macos") {
        vec![
            // Try command line tool first
            ("code", vec![project_path.to_string()]),
            // Try opening via application bundle
            (
                "open",
                vec![
                    "-a".to_string(),
                    "Visual Studio Code".to_string(),
                    project_path.to_string(),
                ],
            ),
        ]
    } else if cfg!(target_os = "windows") {
        vec![
            ("code", vec![project_path.to_string()]),
            ("code.cmd", vec![project_path.to_string()]),
        ]
    } else {
        // Linux and other Unix-like systems
        vec![
            ("code", vec![project_path.to_string()]),
            ("code-insiders", vec![project_path.to_string()]),
        ]
    };

    let mut last_error = String::new();

    for (command, args) in commands {
        match Command::new(command).args(&args).output() {
            Ok(output) => {
                if output.status.success() {
                    return Ok(format!(
                        "Project opened in VS Code successfully using '{}'",
                        command
                    ));
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    last_error = format!("Command '{}' failed: {}", command, stderr);
                }
            }
            Err(e) => {
                last_error = format!("Failed to execute '{}': {}", command, e);
                continue; // Try the next command
            }
        }
    }

    Err(format!(
        "All VS Code launch attempts failed. Last error: {}",
        last_error
    ))
}

/// Try to open a project with the specified editor
fn try_open_with_editor(project_path: &str, editor_id: &str) -> Result<String, String> {
    // Check if path exists
    if !std::path::Path::new(project_path).exists() {
        return Err("Project path does not exist".to_string());
    }

    // Get editor configuration based on ID
    let commands = match editor_id {
        "vscode" => get_vscode_commands(),
        "cursor" => get_cursor_commands(),
        "sublime" => get_sublime_commands(),
        "atom" => get_atom_commands(),
        "vim" => get_vim_commands(),
        "neovim" => get_neovim_commands(),
        "emacs" => get_emacs_commands(),
        "intellij" => get_intellij_commands(),
        "webstorm" => get_webstorm_commands(),
        "phpstorm" => get_phpstorm_commands(),
        "pycharm" => get_pycharm_commands(),
        "rubymine" => get_rubymine_commands(),
        "goland" => get_goland_commands(),
        "clion" => get_clion_commands(),
        "rider" => get_rider_commands(),
        "appcode" => get_appcode_commands(),
        "datagrip" => get_datagrip_commands(),
        "fleet" => get_fleet_commands(),
        "nova" => get_nova_commands(),
        "textmate" => get_textmate_commands(),
        "brackets" => get_brackets_commands(),
        "notepadpp" => get_notepadpp_commands(),
        "eclipse" => get_eclipse_commands(),
        "netbeans" => get_netbeans_commands(),
        "android-studio" => get_android_studio_commands(),
        "xcode" => get_xcode_commands(),
        "zed" => get_zed_commands(),
        "windsurf" => get_windsurf_commands(),
        "claude-dev" => get_claude_dev_commands(),
        "replit" => get_replit_commands(),
        "stackblitz" => get_stackblitz_commands(),
        "codesandbox" => get_codesandbox_commands(),
        "gitpod" => get_gitpod_commands(),
        "lapce" => get_lapce_commands(),
        "helix" => get_helix_commands(),
        "kakoune" => get_kakoune_commands(),
        "micro" => get_micro_commands(),
        _ => {
            return Err(format!("Unsupported editor: {}", editor_id));
        }
    };

    let mut last_error = String::new();

    for (command, mut args) in commands {
        // Add the project path as the last argument
        args.push(project_path.to_string());

        match Command::new(command).args(&args).output() {
            Ok(output) => {
                if output.status.success() {
                    return Ok(format!(
                        "Project opened in {} successfully using '{}'",
                        editor_id, command
                    ));
                } else {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    last_error = format!("Command '{}' failed: {}", command, stderr);
                }
            }
            Err(e) => {
                last_error = format!("Failed to execute '{}': {}", command, e);
                continue; // Try the next command
            }
        }
    }

    Err(format!(
        "All {} launch attempts failed. Last error: {}",
        editor_id, last_error
    ))
}

/// Get VS Code commands for different platforms
fn get_vscode_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("code", vec![]),
            (
                "open",
                vec!["-a".to_string(), "Visual Studio Code".to_string()],
            ),
        ]
    } else if cfg!(target_os = "windows") {
        vec![("code", vec![]), ("code.cmd", vec![])]
    } else {
        vec![("code", vec![]), ("code-insiders", vec![])]
    }
}

/// Get Cursor commands for different platforms
fn get_cursor_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("cursor", vec![]),
            ("open", vec!["-a".to_string(), "Cursor".to_string()]),
        ]
    } else if cfg!(target_os = "windows") {
        vec![("cursor", vec![]), ("cursor.exe", vec![])]
    } else {
        vec![("cursor", vec![])]
    }
}

// For now, implement a basic fallback for other editors that tries common commands
fn get_sublime_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("subl", vec![]),
            ("open", vec!["-a".to_string(), "Sublime Text".to_string()]),
        ]
    } else {
        vec![("subl", vec![])]
    }
}

fn get_atom_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("atom", vec![])]
}
fn get_vim_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("vim", vec![])]
}
fn get_neovim_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("nvim", vec![])]
}
fn get_emacs_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("emacs", vec![])]
}
fn get_intellij_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("idea", vec![])]
}
fn get_webstorm_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("webstorm", vec![])]
}
fn get_phpstorm_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("phpstorm", vec![])]
}
fn get_pycharm_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("pycharm", vec![])]
}
fn get_rubymine_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("rubymine", vec![])]
}
fn get_goland_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("goland", vec![])]
}
fn get_clion_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("clion", vec![])]
}
fn get_rider_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("rider", vec![])]
}
fn get_appcode_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("appcode", vec![])]
}
fn get_datagrip_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("datagrip", vec![])]
}
fn get_fleet_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("fleet", vec![])]
}
fn get_nova_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("nova", vec![])]
}
fn get_textmate_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("mate", vec![])]
}
fn get_brackets_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("brackets", vec![])]
}
fn get_notepadpp_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("notepad++", vec![])]
}
fn get_eclipse_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("eclipse", vec![])]
}
fn get_netbeans_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("netbeans", vec![])]
}
fn get_android_studio_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("studio", vec![])]
}
fn get_xcode_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("xed", vec![]),
            ("open", vec!["-a".to_string(), "Xcode".to_string()]),
        ]
    } else {
        vec![]
    }
}
fn get_zed_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("zed", vec![])]
}

fn get_windsurf_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("windsurf", vec![]),
            ("open", vec!["-a".to_string(), "Windsurf".to_string()]),
        ]
    } else if cfg!(target_os = "windows") {
        vec![("windsurf", vec![]), ("windsurf.exe", vec![])]
    } else {
        vec![("windsurf", vec![])]
    }
}

fn get_claude_dev_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("claude-dev", vec![]),
            ("open", vec!["-a".to_string(), "Claude Dev".to_string()]),
        ]
    } else {
        vec![("claude-dev", vec![])]
    }
}

fn get_replit_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("replit", vec![])]
}

fn get_stackblitz_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("stackblitz", vec![])]
}

fn get_codesandbox_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("codesandbox", vec![])]
}

fn get_gitpod_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("gitpod", vec![])]
}

fn get_lapce_commands() -> Vec<(&'static str, Vec<String>)> {
    if cfg!(target_os = "macos") {
        vec![
            ("lapce", vec![]),
            ("open", vec!["-a".to_string(), "Lapce".to_string()]),
        ]
    } else if cfg!(target_os = "windows") {
        vec![("lapce", vec![]), ("lapce.exe", vec![])]
    } else {
        vec![("lapce", vec![])]
    }
}

fn get_helix_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("hx", vec![])]
}

fn get_kakoune_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("kak", vec![])]
}

fn get_micro_commands() -> Vec<(&'static str, Vec<String>)> {
    vec![("micro", vec![])]
}

// =============================================================================
// Task Management Handlers
// =============================================================================

/// Request body for task operations
#[derive(Deserialize)]
pub struct GetTasksRequest {
    #[serde(rename = "projectId")]
    project_id: String,
    #[serde(rename = "taskSource")]
    task_source: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateTaskRequest {
    #[serde(rename = "projectId")]
    project_id: String,
    title: String,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateTaskRequest {
    #[serde(rename = "projectId")]
    project_id: String,
    #[serde(rename = "taskId")]
    task_id: String,
    title: Option<String>,
    description: Option<String>,
    status: Option<String>,
    priority: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteTaskRequest {
    #[serde(rename = "projectId")]
    project_id: String,
    #[serde(rename = "taskId")]
    task_id: String,
}

/// Get tasks for a project
pub async fn get_tasks(Json(request): Json<GetTasksRequest>) -> impl IntoResponse {
    info!("Getting tasks for project {}", request.project_id);

    // Get project to find its path
    match manager_get_project(&request.project_id).await {
        Ok(Some(project)) => {
            // For now, return empty tasks array
            // In production, this would delegate to the tasks service
            let response = serde_json::json!({
                "projectId": request.project_id,
                "projectPath": project.project_root,
                "taskSource": request.task_source.unwrap_or_else(|| "taskmaster".to_string()),
                "tasks": []
            });

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(ManagerError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get project {}: {}", request.project_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error("Database error".to_string())),
            )
                .into_response()
        }
    }
}

/// Create a new task
pub async fn create_task(Json(request): Json<CreateTaskRequest>) -> impl IntoResponse {
    info!(
        "Creating task '{}' for project {}",
        request.title, request.project_id
    );

    // Get project to validate it exists
    match manager_get_project(&request.project_id).await {
        Ok(Some(_project)) => {
            // For now, return a mock created task
            let response = serde_json::json!({
                "id": format!("task_{}", uuid::Uuid::new_v4()),
                "projectId": request.project_id,
                "title": request.title,
                "description": request.description,
                "status": request.status.unwrap_or_else(|| "pending".to_string()),
                "priority": request.priority,
                "createdAt": chrono::Utc::now().to_rfc3339(),
            });

            (
                StatusCode::CREATED,
                ResponseJson(ApiResponse::success(response)),
            )
                .into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(ManagerError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get project {}: {}", request.project_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error("Database error".to_string())),
            )
                .into_response()
        }
    }
}

/// Update an existing task
pub async fn update_task(Json(request): Json<UpdateTaskRequest>) -> impl IntoResponse {
    info!(
        "Updating task {} for project {}",
        request.task_id, request.project_id
    );

    // Get project to validate it exists
    match manager_get_project(&request.project_id).await {
        Ok(Some(_project)) => {
            // For now, return a mock updated task
            let response = serde_json::json!({
                "id": request.task_id,
                "projectId": request.project_id,
                "title": request.title,
                "description": request.description,
                "status": request.status,
                "priority": request.priority,
                "updatedAt": chrono::Utc::now().to_rfc3339(),
            });

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(ManagerError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get project {}: {}", request.project_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error("Database error".to_string())),
            )
                .into_response()
        }
    }
}

/// Delete a task
pub async fn delete_task(Json(request): Json<DeleteTaskRequest>) -> impl IntoResponse {
    info!(
        "Deleting task {} from project {}",
        request.task_id, request.project_id
    );

    // Get project to validate it exists
    match manager_get_project(&request.project_id).await {
        Ok(Some(_project)) => {
            // For now, return success
            let response = serde_json::json!({
                "message": format!("Task {} deleted successfully", request.task_id)
            });

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(ManagerError::NotFound(_)) => (
            StatusCode::NOT_FOUND,
            ResponseJson(ApiResponse::<()>::error("Project not found".to_string())),
        )
            .into_response(),
        Err(e) => {
            error!("Failed to get project {}: {}", request.project_id, e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error("Database error".to_string())),
            )
                .into_response()
        }
    }
}

/// Helper function to detect VS Code installation
fn detect_vscode() -> Result<String, String> {
    // Try to run 'code --version' to check if VS Code is available
    let output = Command::new("code")
        .arg("--version")
        .output()
        .map_err(|e| format!("VS Code not found or not in PATH: {}", e))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let version = stdout.lines().next().unwrap_or("unknown");
        Ok(format!("code (VS Code {})", version))
    } else {
        Err("VS Code command failed".to_string())
    }
}

// =============================================================================
// Database Export/Import Handlers
// =============================================================================

/// Export database as compressed snapshot
pub async fn export_database() -> impl IntoResponse {
    info!("Exporting database");

    match manager_export_database().await {
        Ok(data) => {
            // Generate filename with timestamp
            let timestamp = chrono::Utc::now().format("%Y-%m-%d-%H%M%S");
            let filename = format!("orkee-backup-{}.gz", timestamp);

            info!("Database export successful, {} bytes", data.len());

            // Return binary data with appropriate headers
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/gzip")
                .header(
                    header::CONTENT_DISPOSITION,
                    format!("attachment; filename=\"{}\"", filename),
                )
                .body(Body::from(data))
                .unwrap()
        }
        Err(e) => {
            error!("Failed to export database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(format!(
                    "Failed to export database: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

/// Response for database import
#[derive(Serialize)]
pub struct ImportDatabaseResponse {
    #[serde(rename = "projectsImported")]
    projects_imported: usize,
    #[serde(rename = "projectsSkipped")]
    projects_skipped: usize,
    #[serde(rename = "conflictsCount")]
    conflicts_count: usize,
    conflicts: Vec<ImportConflictInfo>,
}

#[derive(Serialize)]
pub struct ImportConflictInfo {
    #[serde(rename = "projectId")]
    project_id: String,
    #[serde(rename = "projectName")]
    project_name: String,
    #[serde(rename = "conflictType")]
    conflict_type: String,
}

/// Import database from compressed snapshot
pub async fn import_database(body: axum::body::Bytes) -> impl IntoResponse {
    info!("Importing database, {} bytes received", body.len());

    // Convert Bytes to Vec<u8>
    let data = body.to_vec();

    match manager_import_database(data).await {
        Ok(result) => {
            let conflicts = result
                .conflicts
                .iter()
                .map(|c| ImportConflictInfo {
                    project_id: c.project_id.clone(),
                    project_name: c.project_name.clone(),
                    conflict_type: format!("{:?}", c.conflict_type),
                })
                .collect();

            let response = ImportDatabaseResponse {
                projects_imported: result.projects_imported,
                projects_skipped: result.projects_skipped,
                conflicts_count: result.conflicts.len(),
                conflicts,
            };

            info!(
                "Database import successful: {} imported, {} skipped, {} conflicts",
                result.projects_imported,
                result.projects_skipped,
                result.conflicts.len()
            );

            (StatusCode::OK, ResponseJson(ApiResponse::success(response))).into_response()
        }
        Err(e) => {
            error!("Failed to import database: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(ApiResponse::<()>::error(format!(
                    "Failed to import database: {}",
                    e
                ))),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::test_helpers::with_temp_home;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use orkee_core::types::ProjectStatus;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_create_and_get_project_api() {
        with_temp_home(|| async {
            let app = crate::api::create_projects_router();

            // Create a project
            let create_request = ProjectCreateInput {
                name: "API Test Project".to_string(),
                project_root: "/tmp/api-test".to_string(),
                setup_script: None,
                dev_script: None,
                cleanup_script: None,
                tags: Some(vec!["api".to_string(), "test".to_string()]),
                description: Some("Test project for API".to_string()),
                status: Some(ProjectStatus::Planning),
                rank: None,
                priority: None,
                task_source: None,
                manual_tasks: None,
                mcp_servers: None,
            };

            let request = Request::builder()
                .method("POST")
                .uri("/")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&create_request).unwrap()))
                .unwrap();

            let response = app.clone().oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::CREATED);

            // List projects
            let request = Request::builder()
                .method("GET")
                .uri("/")
                .body(Body::empty())
                .unwrap();

            let response = app.oneshot(request).await.unwrap();
            assert_eq!(response.status(), StatusCode::OK);
        })
        .await;
    }
}
