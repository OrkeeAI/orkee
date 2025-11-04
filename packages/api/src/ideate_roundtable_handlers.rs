// ABOUTME: HTTP request handlers for expert roundtable discussions
// ABOUTME: Handles roundtable creation, expert management, discussion streaming, and insight extraction

use axum::{
    extract::{Path, State},
    response::{
        sse::{Event, KeepAlive},
        IntoResponse, Sse,
    },
    Json,
};
use futures::stream::{self, Stream};
use orkee_ai::AIService;
use orkee_config::constants;
use orkee_ideate::{
    CreateExpertPersonaInput, ExpertModerator, RoundtableEvent, RoundtableManager,
    StartRoundtableRequest, SuggestExpertsRequest, UserInterjectionInput,
};
use orkee_projects::DbState;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tracing::{info, warn};

use super::response::{created_or_internal_error, ok_or_internal_error, ok_or_not_found};

// ============================================================================
// CONFIGURATION & CONSTANTS
// ============================================================================

const DEFAULT_SSE_MAX_DURATION_MINUTES: u64 = 30;
const DEFAULT_SSE_POLL_INTERVAL_SECS: u64 = 1;
const MIN_SSE_MAX_DURATION_MINUTES: u64 = 5;
const MAX_SSE_MAX_DURATION_MINUTES: u64 = 120;
const MIN_SSE_POLL_INTERVAL_SECS: u64 = 1;
const MAX_SSE_POLL_INTERVAL_SECS: u64 = 10;

fn parse_sse_max_duration() -> u64 {
    std::env::var(constants::ORKEE_SSE_MAX_DURATION_MINUTES)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|v: u64| v.clamp(MIN_SSE_MAX_DURATION_MINUTES, MAX_SSE_MAX_DURATION_MINUTES))
        .unwrap_or(DEFAULT_SSE_MAX_DURATION_MINUTES)
}

fn parse_sse_poll_interval() -> u64 {
    std::env::var(constants::ORKEE_SSE_POLL_INTERVAL_SECS)
        .ok()
        .and_then(|s| s.parse::<u64>().ok())
        .map(|v: u64| v.clamp(MIN_SSE_POLL_INTERVAL_SECS, MAX_SSE_POLL_INTERVAL_SECS))
        .unwrap_or(DEFAULT_SSE_POLL_INTERVAL_SECS)
}

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

// ============================================================================
// EXPERT PERSONA ENDPOINTS
// ============================================================================

/// GET /api/ideate/:session_id/experts - List all expert personas
pub async fn list_experts(State(db): State<DbState>) -> impl IntoResponse {
    info!("Listing all expert personas");

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.list_experts(true).await;

    ok_or_internal_error(result, "Failed to list expert personas")
}

/// POST /api/ideate/:session_id/experts - Create custom expert persona
pub async fn create_expert(
    State(db): State<DbState>,
    Json(request): Json<CreateExpertPersonaInput>,
) -> impl IntoResponse {
    info!("Creating custom expert persona: {}", request.name);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.create_expert(request).await;

    created_or_internal_error(result, "Failed to create expert persona")
}

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

    created_or_internal_error(result, "Failed to create roundtable")
}

/// GET /api/ideate/:session_id/roundtables - List roundtables for session
pub async fn list_roundtables(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
) -> impl IntoResponse {
    info!("Listing roundtables for session: {}", session_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.list_roundtables_for_session(&session_id).await;

    ok_or_internal_error(result, "Failed to list roundtables")
}

/// GET /api/ideate/roundtable/:roundtable_id - Get roundtable details
pub async fn get_roundtable(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager
        .get_roundtable_with_participants(&roundtable_id)
        .await;

    ok_or_not_found(result, "Roundtable not found")
}

/// POST /api/ideate/roundtable/:roundtable_id/participants - Add participants
pub async fn add_participants(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
    Json(request): Json<AddParticipantsRequest>,
) -> impl IntoResponse {
    info!(
        "Adding {} participants to roundtable: {}",
        request.expert_ids.len(),
        roundtable_id
    );

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager
        .add_participants(&roundtable_id, request.expert_ids)
        .await;


    Json(SuccessResponse {
        success: true,
        data: serde_json::json!({
            "message": "Discussion started",
            "roundtableId": roundtable_id
        }),
    })
    .into_response()
}

/// Stream state tracking for resource management
struct StreamState {
    manager: RoundtableManager,
    roundtable_id: String,
    last_order: i32,
    start_time: Instant,
    max_duration: Duration,
    poll_interval: Duration,
    is_client_connected: Arc<Mutex<bool>>,
}

/// GET /api/ideate/roundtable/:roundtable_id/stream - SSE stream of messages
pub async fn stream_discussion(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("Starting SSE stream for roundtable: {}", roundtable_id);

    let max_duration_minutes = parse_sse_max_duration();
    let poll_interval_secs = parse_sse_poll_interval();

    info!(
        "SSE stream configured: max_duration={}min, poll_interval={}s",
        max_duration_minutes, poll_interval_secs
    );

    let manager = RoundtableManager::new(db.pool.clone());
    let is_client_connected = Arc::new(Mutex::new(true));

    let state = StreamState {
        manager,
        roundtable_id: roundtable_id.clone(),
        last_order: 0i32,
        start_time: Instant::now(),
        max_duration: Duration::from_secs(max_duration_minutes * 60),
        poll_interval: Duration::from_secs(poll_interval_secs),
        is_client_connected: is_client_connected.clone(),
    };

    // Create stream with timeout and disconnection detection
    let stream = stream::unfold(state, |mut state| async move {
        // Check if stream has exceeded maximum duration
        let elapsed = state.start_time.elapsed();
        if elapsed >= state.max_duration {
            warn!(
                "SSE stream for roundtable {} exceeded max duration of {}min, terminating",
                state.roundtable_id,
                elapsed.as_secs() / 60
            );
            let event = RoundtableEvent::Error {
                error: format!(
                    "Stream exceeded maximum duration of {} minutes",
                    state.max_duration.as_secs() / 60
                ),
            };
            let data = serde_json::to_string(&event).unwrap_or_default();
            return Some((Ok(Event::default().data(data)), state));
        }

        // Check client connection status
        let connected = *state.is_client_connected.lock().await;
        if !connected {
            info!(
                "Client disconnected from roundtable {} stream, terminating",
                state.roundtable_id
            );
            return None;
        }

        // Poll for new messages
        match state
            .manager
            .get_messages_after(&state.roundtable_id, state.last_order)
            .await
        {
            Ok(messages) => {
                if messages.is_empty() {
                    // No new messages, send heartbeat
                    tokio::time::sleep(state.poll_interval).await;
                    let event = RoundtableEvent::Heartbeat;
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    Some((Ok(Event::default().data(data)), state))
                } else {
                    // Update last order with new messages
                    state.last_order = messages
                        .last()
                        .map(|m| m.message_order)
                        .unwrap_or(state.last_order);

                    // Send first message and queue the rest
                    if let Some(first_message) = messages.into_iter().next() {
                        let event = RoundtableEvent::Message {
                            message: first_message,
                        };
                        let data = serde_json::to_string(&event).unwrap_or_default();
                        Some((Ok(Event::default().data(data)), state))
                    } else {
                        // Fallback to heartbeat if no messages after all
                        tokio::time::sleep(state.poll_interval).await;
                        let event = RoundtableEvent::Heartbeat;
                        let data = serde_json::to_string(&event).unwrap_or_default();
                        Some((Ok(Event::default().data(data)), state))
                    }
                }
            }
            Err(e) => {
                warn!(
                    "Error fetching messages for roundtable {}: {}",
                    state.roundtable_id, e
                );
                let event = RoundtableEvent::Error {
                    error: e.to_string(),
                };
                let data = serde_json::to_string(&event).unwrap_or_default();

                // On database errors, terminate stream to release connection
                info!(
                    "Terminating stream for roundtable {} due to error",
                    state.roundtable_id
                );
                Some((Ok(Event::default().data(data)), state))
            }
        }
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// POST /api/ideate/roundtable/:roundtable_id/interjection - User interjection
pub async fn send_interjection(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
    Json(request): Json<UserInterjectionInput>,
) -> impl IntoResponse {
    info!("Handling interjection for roundtable: {}", roundtable_id);

    // Create AI service
    let manager = RoundtableManager::new(db.pool.clone());
    let moderator = ExpertModerator::new(manager);

    let result = moderator
        .handle_interjection(&roundtable_id, &request.message)
        .await;

    ok_or_internal_error(result, "Operation failed")
}

/// GET /api/ideate/roundtable/:roundtable_id/messages - Get all messages
pub async fn get_messages(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting messages for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_messages(&roundtable_id).await;

    ok_or_internal_error(result, "Operation failed")
}

// ============================================================================
// INSIGHT ENDPOINTS
// ============================================================================

/// GET /api/ideate/roundtable/:roundtable_id/insights - Get insights
pub async fn get_insights(
    State(db): State<DbState>,
    Path(roundtable_id): Path<String>,
) -> impl IntoResponse {
    info!("Getting insights for roundtable: {}", roundtable_id);

    let manager = RoundtableManager::new(db.pool.clone());

    let result = manager.get_insights_by_category(&roundtable_id).await;

    ok_or_internal_error(result, "Operation failed")
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

    ok_or_internal_error(result, "Operation failed")
}
