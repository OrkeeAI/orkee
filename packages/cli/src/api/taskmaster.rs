use axum::{extract::Json, http::StatusCode, response::IntoResponse};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct TasksRequest {
    #[serde(rename = "projectPath")]
    project_path: String,
}

#[derive(Debug, Serialize)]
pub struct TasksResponse {
    success: bool,
    data: Option<TaskmasterData>,
    error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskmasterData {
    master: TaskmasterMaster,
    metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskmasterMaster {
    tasks: Vec<serde_json::Value>,
}

pub async fn get_tasks(Json(request): Json<TasksRequest>) -> impl IntoResponse {
    let tasks_file = Path::new(&request.project_path)
        .join(".taskmaster")
        .join("tasks")
        .join("tasks.json");

    match tokio::fs::read_to_string(&tasks_file).await {
        Ok(content) => match serde_json::from_str::<TaskmasterData>(&content) {
            Ok(data) => (
                StatusCode::OK,
                Json(TasksResponse {
                    success: true,
                    data: Some(data),
                    error: None,
                }),
            ),
            Err(e) => (
                StatusCode::OK,
                Json(TasksResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to parse tasks.json: {}", e)),
                }),
            ),
        },
        Err(e) => (
            StatusCode::OK,
            Json(TasksResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to read tasks.json: {}", e)),
            }),
        ),
    }
}

pub async fn save_tasks(Json(request): Json<SaveTasksRequest>) -> impl IntoResponse {
    let tasks_file = Path::new(&request.project_path)
        .join(".taskmaster")
        .join("tasks")
        .join("tasks.json");

    let content = match serde_json::to_string_pretty(&request.data) {
        Ok(json) => json,
        Err(e) => {
            return (
                StatusCode::OK,
                Json(TasksResponse {
                    success: false,
                    data: None,
                    error: Some(format!("Failed to serialize tasks data: {}", e)),
                }),
            )
        }
    };

    match tokio::fs::write(&tasks_file, content).await {
        Ok(_) => (
            StatusCode::OK,
            Json(TasksResponse {
                success: true,
                data: Some(request.data),
                error: None,
            }),
        ),
        Err(e) => (
            StatusCode::OK,
            Json(TasksResponse {
                success: false,
                data: None,
                error: Some(format!("Failed to write tasks.json: {}", e)),
            }),
        ),
    }
}

#[derive(Debug, Deserialize)]
pub struct SaveTasksRequest {
    #[serde(rename = "projectPath")]
    project_path: String,
    data: TaskmasterData,
}
