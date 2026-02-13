// ABOUTME: HTTP handlers for the agent run lifecycle (start, stop, list, detail, SSE events).
// ABOUTME: Manages the Agent Runner subprocess and relays its NDJSON events to SSE clients.

use axum::{
    extract::{Path, Query, State},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::{broadcast, RwLock};
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;
use tracing::{error, info, warn};

use super::response::{created_or_internal_error, ok_or_internal_error};
use orkee_projects::DbState;

// ── Types ──────────────────────────────────────────────────────────────────

/// NDJSON events emitted by the agent runner subprocess.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RunnerEvent {
    #[serde(rename = "run_started")]
    RunStarted {
        run_id: String,
        total_stories: i64,
        completed_stories: i64,
    },
    #[serde(rename = "run_completed")]
    RunCompleted {
        run_id: String,
        total_cost: f64,
        stories_completed: i64,
        duration_secs: i64,
    },
    #[serde(rename = "run_failed")]
    RunFailed { run_id: String, error: String },
    #[serde(rename = "iteration_started")]
    IterationStarted {
        iteration: i64,
        story_id: String,
        story_title: String,
    },
    #[serde(rename = "iteration_completed")]
    IterationCompleted {
        iteration: i64,
        story_id: String,
        cost: f64,
        duration_secs: i64,
        tools: HashMap<String, i64>,
    },
    #[serde(rename = "iteration_failed")]
    IterationFailed {
        iteration: i64,
        story_id: String,
        error: String,
    },
    #[serde(rename = "agent_text")]
    AgentText { text: String },
    #[serde(rename = "agent_tool")]
    AgentTool { tool: String, detail: String },
    #[serde(rename = "branch_created")]
    BranchCreated { branch: String },
    #[serde(rename = "pr_created")]
    PrCreated { pr_number: i64, pr_url: String },
    #[serde(rename = "pr_merged")]
    PrMerged { pr_number: i64 },
    #[serde(rename = "story_completed")]
    StoryCompleted {
        story_id: String,
        passed: i64,
        total: i64,
    },
}

/// Persistent run record stored in SQLite.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRun {
    pub id: String,
    pub project_id: String,
    pub prd_id: Option<String>,
    pub prd_json: String,
    pub system_prompt: Option<String>,
    pub status: String,
    pub max_iterations: i64,
    pub current_iteration: i64,
    pub stories_total: i64,
    pub stories_completed: i64,
    pub total_cost: f64,
    pub total_tokens: i64,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub error: Option<String>,
    pub runner_pid: Option<i64>,
}

/// Shared state for the agent runs subsystem.
#[derive(Clone)]
pub struct AgentRunsState {
    pub db: DbState,
    /// Per-run event broadcast channels.
    pub channels: Arc<RwLock<HashMap<String, broadcast::Sender<RunnerEvent>>>>,
    /// Track runner PIDs for stop requests.
    pub pids: Arc<RwLock<HashMap<String, u32>>>,
}

impl AgentRunsState {
    pub fn new(db: DbState) -> Self {
        Self {
            db,
            channels: Arc::new(RwLock::new(HashMap::new())),
            pids: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get or create a broadcast channel for a run.
    async fn get_or_create_channel(&self, run_id: &str) -> broadcast::Sender<RunnerEvent> {
        let mut channels = self.channels.write().await;
        channels
            .entry(run_id.to_string())
            .or_insert_with(|| broadcast::channel(200).0)
            .clone()
    }
}

// ── Request/Response types ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct StartRunRequest {
    pub project_id: String,
    pub prd_id: Option<String>,
    pub prd_json: serde_json::Value,
    pub max_iterations: Option<i64>,
    pub system_prompt_override: Option<String>,
}

#[derive(Deserialize)]
pub struct ListRunsQuery {
    pub project_id: Option<String>,
    pub status: Option<String>,
}

// ── Handlers ───────────────────────────────────────────────────────────────

/// POST /api/agent-runs - Start a new agent run.
pub async fn start_run(
    State(state): State<AgentRunsState>,
    Json(request): Json<StartRunRequest>,
) -> impl IntoResponse {
    let run_id = nanoid::nanoid!(12);
    let prd_json_str = serde_json::to_string(&request.prd_json).unwrap_or_default();

    // Count stories from the PRD JSON
    let stories_total = request
        .prd_json
        .get("userStories")
        .and_then(|v| v.as_array())
        .map(|a| a.len() as i64)
        .unwrap_or(0);

    let stories_completed = request
        .prd_json
        .get("userStories")
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter(|s| s.get("passes").and_then(|p| p.as_bool()).unwrap_or(false))
                .count() as i64
        })
        .unwrap_or(0);

    info!(
        "Starting agent run {} for project {} ({}/{} stories)",
        run_id, request.project_id, stories_completed, stories_total
    );

    // Insert run record
    let insert_result = sqlx::query(
        "INSERT INTO agent_runs (id, project_id, prd_id, prd_json, system_prompt, status, \
         max_iterations, stories_total, stories_completed, started_at) \
         VALUES (?, ?, ?, ?, ?, 'running', ?, ?, ?, datetime('now'))",
    )
    .bind(&run_id)
    .bind(&request.project_id)
    .bind(&request.prd_id)
    .bind(&prd_json_str)
    .bind(&request.system_prompt_override)
    .bind(request.max_iterations.unwrap_or(10))
    .bind(stories_total)
    .bind(stories_completed)
    .execute(&state.db.pool)
    .await;

    if let Err(e) = insert_result {
        error!("Failed to insert agent run: {}", e);
        return created_or_internal_error(
            Err::<AgentRun, _>(format!("Database error: {}", e)),
            "Failed to create agent run",
        );
    }

    // Look up the project path
    let project_path: Option<String> = sqlx::query_scalar(
        "SELECT project_root FROM projects WHERE id = ?",
    )
    .bind(&request.project_id)
    .fetch_optional(&state.db.pool)
    .await
    .ok()
    .flatten();

    let project_dir = match project_path {
        Some(path) => path,
        None => {
            let _ = sqlx::query("UPDATE agent_runs SET status = 'failed', error = ? WHERE id = ?")
                .bind("Project not found or has no project_root")
                .bind(&run_id)
                .execute(&state.db.pool)
                .await;
            return created_or_internal_error(
                Err::<AgentRun, _>("Project not found or has no project_root".to_string()),
                "Failed to start agent run",
            );
        }
    };

    // Retrieve OAuth token for Claude
    let oauth_token = get_claude_oauth_token(&state.db).await;

    // Write prd.json to a temp file for the runner
    let prd_tmp_path = format!("/tmp/orkee-prd-{}.json", run_id);
    if let Err(e) = tokio::fs::write(&prd_tmp_path, &prd_json_str).await {
        error!("Failed to write temp PRD file: {}", e);
        let _ = sqlx::query("UPDATE agent_runs SET status = 'failed', error = ? WHERE id = ?")
            .bind(format!("Failed to write temp PRD: {}", e))
            .bind(&run_id)
            .execute(&state.db.pool)
            .await;
        return created_or_internal_error(
            Err::<AgentRun, _>(format!("Failed to write temp PRD: {}", e)),
            "Failed to start agent run",
        );
    }

    // Spawn the agent runner subprocess
    let tx = state.get_or_create_channel(&run_id).await;
    let max_iterations = request.max_iterations.unwrap_or(10);

    let spawn_result = spawn_runner(
        &run_id,
        &project_dir,
        &prd_tmp_path,
        max_iterations,
        oauth_token.as_deref(),
        tx.clone(),
    )
    .await;

    match spawn_result {
        Ok(pid) => {
            state.pids.write().await.insert(run_id.clone(), pid);

            // Spawn background task to monitor the runner and update DB
            let state_clone = state.clone();
            let run_id_clone = run_id.clone();
            let prd_tmp_clone = prd_tmp_path.clone();
            tokio::spawn(async move {
                monitor_runner(&state_clone, &run_id_clone, tx).await;
                // Cleanup temp file
                let _ = tokio::fs::remove_file(&prd_tmp_clone).await;
                state_clone.pids.write().await.remove(&run_id_clone);
                state_clone.channels.write().await.remove(&run_id_clone);
            });

            // Fetch the created run and return it
            let run = get_run_from_db(&state.db, &run_id).await;
            created_or_internal_error(run, "Failed to fetch created agent run")
        }
        Err(e) => {
            let _ = sqlx::query("UPDATE agent_runs SET status = 'failed', error = ? WHERE id = ?")
                .bind(format!("Failed to spawn runner: {}", e))
                .bind(&run_id)
                .execute(&state.db.pool)
                .await;
            created_or_internal_error(
                Err::<AgentRun, _>(format!("Failed to spawn runner: {}", e)),
                "Failed to start agent run",
            )
        }
    }
}

/// GET /api/agent-runs - List runs.
pub async fn list_runs(
    State(state): State<AgentRunsState>,
    Query(query): Query<ListRunsQuery>,
) -> impl IntoResponse {
    let mut sql = "SELECT * FROM agent_runs WHERE 1=1".to_string();
    let mut binds: Vec<String> = Vec::new();

    if let Some(ref project_id) = query.project_id {
        sql.push_str(" AND project_id = ?");
        binds.push(project_id.clone());
    }
    if let Some(ref status) = query.status {
        sql.push_str(" AND status = ?");
        binds.push(status.clone());
    }
    sql.push_str(" ORDER BY created_at DESC LIMIT 50");

    let mut q = sqlx::query_as::<_, AgentRunRow>(&sql);
    for bind in &binds {
        q = q.bind(bind);
    }

    let result = q
        .fetch_all(&state.db.pool)
        .await
        .map(|rows| rows.into_iter().map(|r| r.into()).collect::<Vec<AgentRun>>())
        .map_err(|e| format!("Database error: {}", e));

    ok_or_internal_error(result, "Failed to list agent runs")
}

/// GET /api/agent-runs/:id - Get run details.
pub async fn get_run(
    State(state): State<AgentRunsState>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    let result = get_run_from_db(&state.db, &run_id).await;
    ok_or_internal_error(result, "Failed to get agent run")
}

/// POST /api/agent-runs/:id/stop - Stop a running agent.
pub async fn stop_run(
    State(state): State<AgentRunsState>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    info!("Stopping agent run: {}", run_id);

    let pids = state.pids.read().await;
    if let Some(&pid) = pids.get(&run_id) {
        info!("Sending SIGTERM to runner pid={}", pid);
        let _ = std::process::Command::new("kill")
            .arg("-TERM")
            .arg(pid.to_string())
            .output();
    }

    let result = sqlx::query(
        "UPDATE agent_runs SET status = 'cancelled', completed_at = datetime('now') WHERE id = ? AND status = 'running'",
    )
    .bind(&run_id)
    .execute(&state.db.pool)
    .await
    .map(|_| serde_json::json!({"stopped": true}))
    .map_err(|e| format!("Database error: {}", e));

    ok_or_internal_error(result, "Failed to stop agent run")
}

/// GET /api/agent-runs/:id/events - SSE stream for a run.
pub async fn run_events(
    State(state): State<AgentRunsState>,
    Path(run_id): Path<String>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let tx = state.get_or_create_channel(&run_id).await;
    let rx = tx.subscribe();

    let stream = BroadcastStream::new(rx).filter_map(|result| match result {
        Ok(event) => {
            let json = serde_json::to_string(&event).unwrap_or_default();
            Some(Ok(Event::default().data(json)))
        }
        Err(_) => None,
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

/// DELETE /api/agent-runs/:id - Delete a run.
pub async fn delete_run(
    State(state): State<AgentRunsState>,
    Path(run_id): Path<String>,
) -> impl IntoResponse {
    info!("Deleting agent run: {}", run_id);

    let result = sqlx::query("DELETE FROM agent_runs WHERE id = ?")
        .bind(&run_id)
        .execute(&state.db.pool)
        .await
        .map(|_| serde_json::json!({"deleted": true}))
        .map_err(|e| format!("Database error: {}", e));

    ok_or_internal_error(result, "Failed to delete agent run")
}

// ── Internal helpers ───────────────────────────────────────────────────────

/// Spawn the agent runner as a bun subprocess.
async fn spawn_runner(
    run_id: &str,
    project_dir: &str,
    prd_path: &str,
    max_iterations: i64,
    oauth_token: Option<&str>,
    _tx: broadcast::Sender<RunnerEvent>,
) -> Result<u32, String> {
    let runner_script = std::env::current_dir()
        .map(|d| {
            d.join("packages/agent-runner/src/index.ts")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|_| "packages/agent-runner/src/index.ts".to_string());

    let mut cmd = Command::new("bun");
    cmd.arg("run")
        .arg(&runner_script)
        .arg("--project-dir")
        .arg(project_dir)
        .arg("--prd")
        .arg(prd_path)
        .arg("--run-id")
        .arg(run_id)
        .arg("--max-iterations")
        .arg(max_iterations.to_string())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    if let Some(token) = oauth_token {
        cmd.env("CLAUDE_CODE_OAUTH_TOKEN", token);
    }

    let child = cmd.spawn().map_err(|e| format!("Failed to spawn runner: {}", e))?;
    let pid = child.id().ok_or("Failed to get runner PID")?;

    // Store the child handle for the monitor task
    // The monitor task will read from stdout
    tokio::spawn(read_runner_output(child, _tx));

    Ok(pid)
}

/// Read NDJSON from the runner's stdout and broadcast events.
async fn read_runner_output(
    mut child: tokio::process::Child,
    tx: broadcast::Sender<RunnerEvent>,
) {
    if let Some(stdout) = child.stdout.take() {
        let reader = BufReader::new(stdout);
        let mut lines = reader.lines();

        while let Ok(Some(line)) = lines.next_line().await {
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<RunnerEvent>(&line) {
                Ok(event) => {
                    let _ = tx.send(event);
                }
                Err(e) => {
                    warn!("Failed to parse runner event: {} (line: {})", e, &line[..line.len().min(100)]);
                }
            }
        }
    }

    // Wait for the process to exit
    let _ = child.wait().await;
}

/// Background task that monitors a runner and updates DB on events.
async fn monitor_runner(
    state: &AgentRunsState,
    run_id: &str,
    tx: broadcast::Sender<RunnerEvent>,
) {
    let mut rx = tx.subscribe();

    while let Ok(event) = rx.recv().await {
        match &event {
            RunnerEvent::IterationStarted { iteration, .. } => {
                let _ = sqlx::query(
                    "UPDATE agent_runs SET current_iteration = ? WHERE id = ?",
                )
                .bind(*iteration)
                .bind(run_id)
                .execute(&state.db.pool)
                .await;
            }
            RunnerEvent::IterationCompleted { cost, .. } => {
                let _ = sqlx::query(
                    "UPDATE agent_runs SET total_cost = total_cost + ? WHERE id = ?",
                )
                .bind(*cost)
                .bind(run_id)
                .execute(&state.db.pool)
                .await;
            }
            RunnerEvent::StoryCompleted { passed, total, .. } => {
                let _ = sqlx::query(
                    "UPDATE agent_runs SET stories_completed = ?, stories_total = ? WHERE id = ?",
                )
                .bind(*passed)
                .bind(*total)
                .bind(run_id)
                .execute(&state.db.pool)
                .await;
            }
            RunnerEvent::RunCompleted {
                total_cost,
                stories_completed,
                ..
            } => {
                let _ = sqlx::query(
                    "UPDATE agent_runs SET status = 'completed', total_cost = ?, \
                     stories_completed = ?, completed_at = datetime('now') WHERE id = ?",
                )
                .bind(*total_cost)
                .bind(*stories_completed)
                .bind(run_id)
                .execute(&state.db.pool)
                .await;
                break;
            }
            RunnerEvent::RunFailed { error, .. } => {
                let _ = sqlx::query(
                    "UPDATE agent_runs SET status = 'failed', error = ?, \
                     completed_at = datetime('now') WHERE id = ?",
                )
                .bind(error)
                .bind(run_id)
                .execute(&state.db.pool)
                .await;
                break;
            }
            _ => {}
        }
    }
}

/// Retrieve a run from the database.
async fn get_run_from_db(db: &DbState, run_id: &str) -> Result<AgentRun, String> {
    sqlx::query_as::<_, AgentRunRow>("SELECT * FROM agent_runs WHERE id = ?")
        .bind(run_id)
        .fetch_optional(&db.pool)
        .await
        .map_err(|e| format!("Database error: {}", e))?
        .map(|row| row.into())
        .ok_or_else(|| "Run not found".to_string())
}

/// Retrieve Claude OAuth token from the auth store.
async fn get_claude_oauth_token(db: &DbState) -> Option<String> {
    // Try to get the token from the OAuth storage
    match orkee_auth::oauth::OAuthManager::new(db.pool.clone()) {
        Ok(manager) => {
            match manager
                .get_token("default-user", orkee_auth::oauth::OAuthProvider::Claude)
                .await
            {
                Ok(Some(token)) if token.is_valid() => Some(token.access_token),
                Ok(_) => {
                    warn!("Claude OAuth token not found or expired. Run `orkee auth login claude`.");
                    None
                }
                Err(e) => {
                    warn!("Failed to retrieve Claude OAuth token: {}", e);
                    None
                }
            }
        }
        Err(e) => {
            warn!("Failed to initialize OAuthManager: {}", e);
            None
        }
    }
}

// ── SQLx row mapping ───────────────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct AgentRunRow {
    id: String,
    project_id: String,
    prd_id: Option<String>,
    prd_json: String,
    system_prompt: Option<String>,
    status: String,
    max_iterations: i64,
    current_iteration: i64,
    stories_total: i64,
    stories_completed: i64,
    total_cost: f64,
    total_tokens: i64,
    started_at: Option<String>,
    completed_at: Option<String>,
    created_at: String,
    updated_at: String,
    error: Option<String>,
    runner_pid: Option<i64>,
}

impl From<AgentRunRow> for AgentRun {
    fn from(row: AgentRunRow) -> Self {
        AgentRun {
            id: row.id,
            project_id: row.project_id,
            prd_id: row.prd_id,
            prd_json: row.prd_json,
            system_prompt: row.system_prompt,
            status: row.status,
            max_iterations: row.max_iterations,
            current_iteration: row.current_iteration,
            stories_total: row.stories_total,
            stories_completed: row.stories_completed,
            total_cost: row.total_cost,
            total_tokens: row.total_tokens,
            started_at: row.started_at,
            completed_at: row.completed_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            error: row.error,
            runner_pid: row.runner_pid,
        }
    }
}
