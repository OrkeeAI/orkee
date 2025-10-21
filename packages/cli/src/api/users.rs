// ABOUTME: User management API endpoints
// ABOUTME: Handles user credentials and settings updates

use axum::{extract::State, http::StatusCode, Json};
use orkee_projects::users::{MaskedUser, UserStorage, UserUpdateInput};
use serde::{Deserialize, Serialize};
use tracing::{error, info};

use crate::{auth::CurrentUser, AppState};

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

pub async fn get_current_user(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<MaskedUser>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Fetching current user");

    let user_storage = match UserStorage::new(state.db.clone()) {
        Ok(storage) => storage,
        Err(e) => {
            error!("Failed to initialize user storage: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to initialize storage: {}", e))),
            ));
        }
    };

    match user_storage.get_current_user().await {
        Ok(user) => {
            let masked_user: MaskedUser = user.into();
            Ok(Json(ApiResponse::success(masked_user)))
        }
        Err(e) => {
            error!("Failed to get current user: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to get user: {}", e))),
            ))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCredentialsRequest {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub xai_api_key: Option<String>,
    pub ai_gateway_enabled: Option<bool>,
    pub ai_gateway_url: Option<String>,
    pub ai_gateway_key: Option<String>,
}

pub async fn update_credentials(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(payload): Json<UpdateCredentialsRequest>,
) -> Result<Json<ApiResponse<MaskedUser>>, (StatusCode, Json<ApiResponse<()>>)> {
    info!("Updating user credentials for user: {}", current_user.id);

    let user_storage = match UserStorage::new(state.db.clone()) {
        Ok(storage) => storage,
        Err(e) => {
            error!("Failed to initialize user storage: {}", e);
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!("Failed to initialize storage: {}", e))),
            ));
        }
    };

    let input = UserUpdateInput {
        openai_api_key: payload.openai_api_key,
        anthropic_api_key: payload.anthropic_api_key,
        google_api_key: payload.google_api_key,
        xai_api_key: payload.xai_api_key,
        ai_gateway_enabled: payload.ai_gateway_enabled,
        ai_gateway_url: payload.ai_gateway_url,
        ai_gateway_key: payload.ai_gateway_key,
    };

    match user_storage.update_credentials(&current_user.id, input).await {
        Ok(user) => {
            let masked_user: MaskedUser = user.into();
            Ok(Json(ApiResponse::success(masked_user)))
        }
        Err(e) => {
            error!("Failed to update credentials: {}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(format!(
                    "Failed to update credentials: {}",
                    e
                ))),
            ))
        }
    }
}
