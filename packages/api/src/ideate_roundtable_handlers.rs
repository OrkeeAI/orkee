// ABOUTME: HTTP request handlers for expert roundtable discussions
// ABOUTME: Handles roundtable creation, expert management, discussion streaming, and insight extraction

use ai::AIService;
use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json,
};
use futures::stream::{self, Stream};
use ideate::{
    CreateExpertPersonaInput, ExpertModerator, ExpertPersona, MessageRole, RoundtableEvent,
    RoundtableManager, RoundtableStatistics, StartRoundtableRequest, SuggestExpertsRequest,
    UserInterjectionInput,
};
use orkee_projects::DbState;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::StreamExt as _;
use tracing::{debug, info, warn};

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};

// TODO: Replace with proper user authentication
const DEFAULT_USER_ID: &str = "default-user";

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

/// Request to create a roundtable session
#[derive(Deserialize)]
pub struct CreateRoundtableRequest {
    pub topic: String,
    #[serde(rename = "numExperts")]
    pub num_experts: i32,
}

/// Request to add participants to roundtable
#[derive(Deserialize)]
pub struct AddParticipantsRequest {
    #[serde(rename = "expertIds")]
    pub expert_ids: Vec<String>,
}

/// Request to extract insights
#[derive(Deserialize)]
pub struct ExtractInsightsRequest {
    pub categories: Option<Vec<String>>,
}

/// Generic success response
#[derive(Serialize)]
struct SuccessResponse<T> {
    success: bool,
    data: T,
}

/// Error response
#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

// ============================================================================
// EXPERT PERSONA ENDPOINTS
// ============================================================================

/// GET /api/ideate/:session_id/experts - List all expert personas
pub async fn list_experts(State(db): State<DbState>) -> impl IntoResponse {
    info!("Listing all expert personas");

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.list_experts(true).await;

    ok_or_internal_error(result)
}

/// POST /api/ideate/:session_id/experts - Create custom expert persona
pub async fn create_expert(
    State(db): State<DbState>,
    Json(request): Json<CreateExpertPersonaInput>,
) -> impl IntoResponse {
    info!("Creating custom expert persona: {}", request.name);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.create_expert(request).await;

    created_or_internal_error(result)
}

/// POST /api/ideate/:session_id/experts/suggest - Get AI-suggested experts
pub async fn suggest_experts(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<SuggestExpertsRequest>,
) -> impl IntoResponse {
    info!("Suggesting experts for session: {}", session_id);

    // Get user's API key
    let user = match db.user_storage.get_user(DEFAULT_USER_ID).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Failed to retrieve user: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("User not found: {}", e),
            })
            .into_response();
        }
    };

    if user.api_key.is_none() {
        return Json(ErrorResponse {
            success: false,
            error: "API key not configured. Please set your API key in settings.".to_string(),
        })
        .into_response();
    }

    // Create AI service with user's API key
    let ai_service = match AIService::new(
        user.api_key.unwrap(),
        db.system_storage.clone(),
        db.ai_usage_logger.clone(),
    )
    .await
    {
        Ok(service) => service,
        Err(e) => {
            warn!("Failed to create AI service: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("Failed to initialize AI service: {}", e),
            })
            .into_response();
        }
    };

    let manager = RoundtableManager::new(db.pool.clone());
    let moderator = ExpertModerator::new(manager, ai_service);

    let result = moderator.suggest_experts(&request).await;

    ok_or_internal_error(result)
}

// ============================================================================
// ROUNDTABLE SESSION ENDPOINTS
// ============================================================================

/// POST /api/ideate/:session_id/roundtable - Create roundtable session
pub async fn create_roundtable(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(request): Json<CreateRoundtableRequest>,
) -> impl IntoResponse {
    info!("Creating roundtable for session: {}", session_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager
        .create_roundtable(&session_id, request.topic, request.num_experts)
        .await;

    created_or_internal_error(result)
}

/// GET /api/ideate/:session_id/roundtables - List roundtables for session
pub async fn list_roundtables(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing roundtables for session: {}", session_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.list_roundtables_for_session(&session_id).await;

    ok_or_internal_error(result)
}

/// GET /api/ideate/roundtable/:roundtable_id - Get roundtable details
pub async fn get_roundtable(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_roundtable_with_participants(&roundtable_id).await;

    ok_or_not_found(result, "Roundtable not found")
}

/// POST /api/ideate/roundtable/:roundtable_id/participants - Add participants
pub async fn add_participants(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
    Json(request): Json<AddParticipantsRequest>,
) -> impl IntoResponse {
    info!("Adding {} participants to roundtable: {}", request.expert_ids.len(), roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager
        .add_participants(&roundtable_id, request.expert_ids)
        .await;

    created_or_internal_error(result)
}

/// GET /api/ideate/roundtable/:roundtable_id/participants - Get participants
pub async fn get_participants(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting participants for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_participants(&roundtable_id).await;

    ok_or_internal_error(result)
}

// ============================================================================
// DISCUSSION ENDPOINTS
// ============================================================================

/// POST /api/ideate/roundtable/:roundtable_id/start - Start discussion
pub async fn start_discussion(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
    Json(request): Json<StartRoundtableRequest>,
) -> impl IntoResponse {
    info!("Starting discussion for roundtable: {}", roundtable_id);

    // Get user's API key
    let user = match db.user_storage.get_user(DEFAULT_USER_ID).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Failed to retrieve user: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("User not found: {}", e),
            })
            .into_response();
        }
    };

    if user.api_key.is_none() {
        return Json(ErrorResponse {
            success: false,
            error: "API key not configured. Please set your API key in settings.".to_string(),
        })
        .into_response();
    }

    // Create AI service
    let ai_service = match AIService::new(
        user.api_key.unwrap(),
        db.system_storage.clone(),
        db.ai_usage_logger.clone(),
    )
    .await
    {
        Ok(service) => service,
        Err(e) => {
            warn!("Failed to create AI service: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("Failed to initialize AI service: {}", e),
            })
            .into_response();
        }
    };

    let manager = RoundtableManager::new(db.pool.clone());
    let moderator = ExpertModerator::new(manager, ai_service);

    // Run discussion in background
    let roundtable_id_clone = roundtable_id.clone();
    let topic = request.topic.clone();
    tokio::spawn(async move {
        if let Err(e) = moderator.run_discussion(&roundtable_id_clone, &topic).await {
            warn!("Discussion error for roundtable {}: {}", roundtable_id_clone, e);
        }
    });

    Json(SuccessResponse {
        success: true,
        data: serde_json::json!({
            "message": "Discussion started",
            "roundtableId": roundtable_id
        }),
    })
    .into_response()
}

/// GET /api/ideate/roundtable/:roundtable_id/stream - SSE stream of messages
pub async fn stream_discussion(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("Starting SSE stream for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    // Send initial connected event
    let connected_event = RoundtableEvent::Connected {
        roundtable_id: roundtable_id.clone(),
    };

    // Create stream that polls for new messages
    let stream = stream::unfold(
        (manager, roundtable_id, 0i32),
        |(manager, roundtable_id, last_order)| async move {
            // Poll for new messages
            match manager.get_messages_after(&roundtable_id, last_order).await {
                Ok(messages) => {
                    if messages.is_empty() {
                        // No new messages, send heartbeat
                        tokio::time::sleep(Duration::from_secs(1)).await;
                        let event = RoundtableEvent::Heartbeat;
                        let data = serde_json::to_string(&event).unwrap();
                        Some((
                            Ok(Event::default().data(data)),
                            (manager, roundtable_id, last_order),
                        ))
                    } else {
                        // Send new messages
                        let new_last_order = messages.last().map(|m| m.message_order).unwrap_or(last_order);

                        let events: Vec<_> = messages
                            .into_iter()
                            .map(|message| {
                                let event = RoundtableEvent::Message { message };
                                let data = serde_json::to_string(&event).unwrap();
                                Ok(Event::default().data(data))
                            })
                            .collect();

                        // Return first event and continue with rest
                        if let Some(first) = events.into_iter().next() {
                            Some((first, (manager, roundtable_id, new_last_order)))
                        } else {
                            tokio::time::sleep(Duration::from_secs(1)).await;
                            let event = RoundtableEvent::Heartbeat;
                            let data = serde_json::to_string(&event).unwrap();
                            Some((
                                Ok(Event::default().data(data)),
                                (manager, roundtable_id, last_order),
                            ))
                        }
                    }
                }
                Err(e) => {
                    warn!("Error fetching messages: {}", e);
                    let event = RoundtableEvent::Error {
                        error: e.to_string(),
                    };
                    let data = serde_json::to_string(&event).unwrap();
                    Some((
                        Ok(Event::default().data(data)),
                        (manager, roundtable_id, last_order),
                    ))
                }
            }
        },
    );

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// POST /api/ideate/roundtable/:roundtable_id/interjection - User interjection
pub async fn send_interjection(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
    Json(request): Json<UserInterjectionInput>,
) -> impl IntoResponse {
    info!("Handling interjection for roundtable: {}", roundtable_id);

    // Get user's API key
    let user = match db.user_storage.get_user(DEFAULT_USER_ID).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Failed to retrieve user: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("User not found: {}", e),
            })
            .into_response();
        }
    };

    if user.api_key.is_none() {
        return Json(ErrorResponse {
            success: false,
            error: "API key not configured. Please set your API key in settings.".to_string(),
        })
        .into_response();
    }

    // Create AI service
    let ai_service = match AIService::new(
        user.api_key.unwrap(),
        db.system_storage.clone(),
        db.ai_usage_logger.clone(),
    )
    .await
    {
        Ok(service) => service,
        Err(e) => {
            warn!("Failed to create AI service: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("Failed to initialize AI service: {}", e),
            })
            .into_response();
        }
    };

    let manager = RoundtableManager::new(db.pool.clone());
    let moderator = ExpertModerator::new(manager, ai_service);

    let result = moderator
        .handle_interjection(&roundtable_id, &request.message)
        .await;

    ok_or_internal_error(result)
}

/// GET /api/ideate/roundtable/:roundtable_id/messages - Get all messages
pub async fn get_messages(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting messages for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_messages(&roundtable_id).await;

    ok_or_internal_error(result)
}

// ============================================================================
// INSIGHT ENDPOINTS
// ============================================================================

/// POST /api/ideate/roundtable/:roundtable_id/insights/extract - Extract insights
pub async fn extract_insights(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
    Json(request): Json<ExtractInsightsRequest>,
) -> impl IntoResponse {
    info!("Extracting insights for roundtable: {}", roundtable_id);

    // Get user's API key
    let user = match db.user_storage.get_user(DEFAULT_USER_ID).await {
        Ok(u) => u,
        Err(e) => {
            warn!("Failed to retrieve user: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("User not found: {}", e),
            })
            .into_response();
        }
    };

    if user.api_key.is_none() {
        return Json(ErrorResponse {
            success: false,
            error: "API key not configured. Please set your API key in settings.".to_string(),
        })
        .into_response();
    }

    // Create AI service
    let ai_service = match AIService::new(
        user.api_key.unwrap(),
        db.system_storage.clone(),
        db.ai_usage_logger.clone(),
    )
    .await
    {
        Ok(service) => service,
        Err(e) => {
            warn!("Failed to create AI service: {}", e);
            return Json(ErrorResponse {
                success: false,
                error: format!("Failed to initialize AI service: {}", e),
            })
            .into_response();
        }
    };

    let manager = RoundtableManager::new(db.pool.clone());
    let moderator = ExpertModerator::new(manager, ai_service);

    let result = moderator
        .extract_insights(&roundtable_id, request.categories)
        .await;

    ok_or_internal_error(result)
}

/// GET /api/ideate/roundtable/:roundtable_id/insights - Get insights
pub async fn get_insights(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting insights for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_insights_by_category(&roundtable_id).await;

    ok_or_internal_error(result)
}

// ============================================================================
// STATISTICS ENDPOINTS
// ============================================================================

/// GET /api/ideate/roundtable/:roundtable_id/statistics - Get roundtable statistics
pub async fn get_statistics(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting statistics for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_statistics(&roundtable_id).await;

    ok_or_internal_error(result)
}
