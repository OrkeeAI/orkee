// ABOUTME: Storage layer for sandboxes, executions, environment variables and volumes
// ABOUTME: Provides CRUD operations for sandbox data in SQLite database

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::{Row, SqlitePool};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Sandbox not found: {0}")]
    NotFound(String),
    #[error("Invalid status: {0}")]
    InvalidStatus(String),
    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, StorageError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStatus {
    Creating,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error,
    Terminated,
}

impl SandboxStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Creating => "creating",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Stopped => "stopped",
            Self::Error => "error",
            Self::Terminated => "terminated",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "creating" => Ok(Self::Creating),
            "starting" => Ok(Self::Starting),
            "running" => Ok(Self::Running),
            "stopping" => Ok(Self::Stopping),
            "stopped" => Ok(Self::Stopped),
            "error" => Ok(Self::Error),
            "terminated" => Ok(Self::Terminated),
            _ => Err(StorageError::InvalidStatus(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sandbox {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub agent_id: String,
    pub status: SandboxStatus,
    pub container_id: Option<String>,
    pub port: Option<u16>,

    // Resource configuration
    pub cpu_cores: f32,
    pub memory_mb: u32,
    pub storage_gb: u32,
    pub gpu_enabled: bool,
    pub gpu_model: Option<String>,

    // Networking
    pub public_url: Option<String>,
    pub ssh_enabled: bool,
    pub ssh_key: Option<String>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub stopped_at: Option<DateTime<Utc>>,
    pub terminated_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub cost_estimate: Option<f64>,

    // Association
    pub project_id: Option<String>,
    pub user_id: String,

    // JSON fields
    pub config: Option<JsonValue>,
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl ExecutionStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self> {
        match s {
            "queued" => Ok(Self::Queued),
            "running" => Ok(Self::Running),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            "cancelled" => Ok(Self::Cancelled),
            _ => Err(StorageError::InvalidStatus(s.to_string())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxExecution {
    pub id: String,
    pub sandbox_id: String,
    pub command: String,
    pub working_directory: String,
    pub status: ExecutionStatus,

    // Execution details
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub exit_code: Option<i32>,
    pub stdout: Option<String>,
    pub stderr: Option<String>,

    // Resource tracking
    pub cpu_time_seconds: Option<f64>,
    pub memory_peak_mb: Option<u32>,

    // Metadata
    pub created_at: DateTime<Utc>,
    pub created_by: Option<String>,
    pub agent_execution_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub id: Option<i32>,
    pub sandbox_id: String,
    pub name: String,
    pub value: String,
    pub is_secret: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Volume {
    pub id: Option<i32>,
    pub sandbox_id: String,
    pub host_path: String,
    pub container_path: String,
    pub read_only: bool,
    pub created_at: DateTime<Utc>,
}

pub struct SandboxStorage {
    pool: SqlitePool,
}

impl SandboxStorage {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    // ========================================================================
    // SANDBOX OPERATIONS
    // ========================================================================

    pub async fn create_sandbox(&self, mut sandbox: Sandbox) -> Result<Sandbox> {
        // Generate ID if not provided
        if sandbox.id.is_empty() {
            sandbox.id = format!("sbx_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        }

        let config_json = serde_json::to_string(&sandbox.config)?;
        let metadata_json = serde_json::to_string(&sandbox.metadata)?;

        sqlx::query(
            r#"
            INSERT INTO sandboxes (
                id, name, provider, agent_id, status, container_id, port,
                cpu_cores, memory_mb, storage_gb, gpu_enabled, gpu_model,
                public_url, ssh_enabled, ssh_key,
                created_at, started_at, stopped_at, terminated_at,
                error_message, cost_estimate,
                project_id, user_id, config, metadata
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7,
                ?8, ?9, ?10, ?11, ?12,
                ?13, ?14, ?15,
                ?16, ?17, ?18, ?19,
                ?20, ?21,
                ?22, ?23, ?24, ?25
            )
            "#,
        )
        .bind(&sandbox.id)
        .bind(&sandbox.name)
        .bind(&sandbox.provider)
        .bind(&sandbox.agent_id)
        .bind(sandbox.status.as_str())
        .bind(&sandbox.container_id)
        .bind(sandbox.port.map(|p| p as i32))
        .bind(sandbox.cpu_cores)
        .bind(sandbox.memory_mb as i32)
        .bind(sandbox.storage_gb as i32)
        .bind(sandbox.gpu_enabled)
        .bind(&sandbox.gpu_model)
        .bind(&sandbox.public_url)
        .bind(sandbox.ssh_enabled)
        .bind(&sandbox.ssh_key)
        .bind(sandbox.created_at.to_rfc3339())
        .bind(sandbox.started_at.map(|d| d.to_rfc3339()))
        .bind(sandbox.stopped_at.map(|d| d.to_rfc3339()))
        .bind(sandbox.terminated_at.map(|d| d.to_rfc3339()))
        .bind(&sandbox.error_message)
        .bind(sandbox.cost_estimate)
        .bind(&sandbox.project_id)
        .bind(&sandbox.user_id)
        .bind(&config_json)
        .bind(&metadata_json)
        .execute(&self.pool)
        .await?;

        Ok(sandbox)
    }

    pub async fn get_sandbox(&self, id: &str) -> Result<Sandbox> {
        let row = sqlx::query(
            r#"
            SELECT id, name, provider, agent_id, status, container_id, port,
                   cpu_cores, memory_mb, storage_gb, gpu_enabled, gpu_model,
                   public_url, ssh_enabled, ssh_key,
                   created_at, started_at, stopped_at, terminated_at,
                   error_message, cost_estimate,
                   project_id, user_id, config, metadata
            FROM sandboxes
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(self.row_to_sandbox(row)?),
            None => Err(StorageError::NotFound(id.to_string())),
        }
    }

    pub async fn list_sandboxes(
        &self,
        user_id: Option<&str>,
        project_id: Option<&str>,
        status: Option<SandboxStatus>,
    ) -> Result<Vec<Sandbox>> {
        let mut query = String::from(
            r#"
            SELECT id, name, provider, agent_id, status, container_id, port,
                   cpu_cores, memory_mb, storage_gb, gpu_enabled, gpu_model,
                   public_url, ssh_enabled, ssh_key,
                   created_at, started_at, stopped_at, terminated_at,
                   error_message, cost_estimate,
                   project_id, user_id, config, metadata
            FROM sandboxes
            WHERE 1=1
            "#,
        );

        let mut param_count = 0;
        if user_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND user_id = ?{}", param_count));
        }
        if project_id.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND project_id = ?{}", param_count));
        }
        if status.is_some() {
            param_count += 1;
            query.push_str(&format!(" AND status = ?{}", param_count));
        }
        query.push_str(" ORDER BY created_at DESC");

        let mut q = sqlx::query(&query);

        if let Some(uid) = user_id {
            q = q.bind(uid);
        }
        if let Some(pid) = project_id {
            q = q.bind(pid);
        }
        if let Some(s) = &status {
            q = q.bind(s.as_str());
        }

        let rows = q.fetch_all(&self.pool).await?;

        rows.into_iter()
            .map(|row| self.row_to_sandbox(row))
            .collect()
    }

    pub async fn update_sandbox_status(
        &self,
        id: &str,
        status: SandboxStatus,
        error: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        let mut query = String::from("UPDATE sandboxes SET status = ?1");
        let mut bind_index = 2;

        if let Some(ref _err) = error {
            query.push_str(&format!(", error_message = ?{}", bind_index));
            bind_index += 1;
        }

        // Update timestamps based on status
        match status {
            SandboxStatus::Running => {
                query.push_str(&format!(", started_at = ?{}", bind_index));
                bind_index += 1;
            }
            SandboxStatus::Stopped => {
                query.push_str(&format!(", stopped_at = ?{}", bind_index));
                bind_index += 1;
            }
            SandboxStatus::Terminated => {
                query.push_str(&format!(", terminated_at = ?{}", bind_index));
                bind_index += 1;
            }
            _ => {}
        }

        query.push_str(&format!(" WHERE id = ?{}", bind_index));

        let mut q = sqlx::query(&query).bind(status.as_str());

        if let Some(err) = error {
            q = q.bind(err);
        }

        match status {
            SandboxStatus::Running | SandboxStatus::Stopped | SandboxStatus::Terminated => {
                q = q.bind(now.to_rfc3339());
            }
            _ => {}
        }

        q = q.bind(id);

        let result = q.execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn update_sandbox_container(
        &self,
        id: &str,
        container_id: &str,
        port: Option<u16>,
    ) -> Result<()> {
        let result = sqlx::query("UPDATE sandboxes SET container_id = ?1, port = ?2 WHERE id = ?3")
            .bind(container_id)
            .bind(port.map(|p| p as i32))
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    /// Atomically update sandbox container_id and status
    /// This ensures both fields are updated together, preventing inconsistent state
    pub async fn update_sandbox_with_container(
        &self,
        id: &str,
        container_id: &str,
        status: SandboxStatus,
        port: Option<u16>,
    ) -> Result<()> {
        let now = Utc::now();
        let mut query =
            String::from("UPDATE sandboxes SET container_id = ?1, port = ?2, status = ?3");

        // Update timestamps based on status
        match status {
            SandboxStatus::Running => {
                query.push_str(", started_at = ?4");
            }
            SandboxStatus::Stopped => {
                query.push_str(", stopped_at = ?4");
            }
            SandboxStatus::Terminated => {
                query.push_str(", terminated_at = ?4");
            }
            _ => {}
        }

        query.push_str(" WHERE id = ?5");

        let mut q = sqlx::query(&query)
            .bind(container_id)
            .bind(port.map(|p| p as i32))
            .bind(status.as_str());

        // Bind timestamp if needed
        match status {
            SandboxStatus::Running | SandboxStatus::Stopped | SandboxStatus::Terminated => {
                q = q.bind(now.to_rfc3339());
            }
            _ => {}
        }

        let result = q.bind(id).execute(&self.pool).await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    pub async fn delete_sandbox(&self, id: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM sandboxes WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        if result.rows_affected() == 0 {
            return Err(StorageError::NotFound(id.to_string()));
        }

        Ok(())
    }

    /// Create a sandbox with its environment variables and volumes in a single transaction
    /// This ensures all related data is created atomically
    pub async fn create_sandbox_with_resources(
        &self,
        sandbox: Sandbox,
        env_vars: Vec<EnvVar>,
        volumes: Vec<Volume>,
    ) -> Result<Sandbox> {
        let mut tx = self.pool.begin().await?;

        // Create sandbox
        let sandbox_id = if sandbox.id.is_empty() {
            uuid::Uuid::new_v4().to_string()
        } else {
            sandbox.id.clone()
        };

        let config_json = sandbox
            .config
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let metadata_json = sandbox
            .metadata
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        sqlx::query(
            "INSERT INTO sandboxes (
                id, name, provider, agent_id, status, container_id, port,
                cpu_cores, memory_mb, storage_gb, gpu_enabled, gpu_model,
                public_url, ssh_enabled, ssh_key,
                created_at, started_at, stopped_at, terminated_at,
                error_message, cost_estimate, project_id, user_id,
                config, metadata
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25)",
        )
        .bind(&sandbox_id)
        .bind(&sandbox.name)
        .bind(&sandbox.provider)
        .bind(&sandbox.agent_id)
        .bind(sandbox.status.as_str())
        .bind(&sandbox.container_id)
        .bind(sandbox.port.map(|p| p as i32))
        .bind(sandbox.cpu_cores)
        .bind(sandbox.memory_mb as i64)
        .bind(sandbox.storage_gb as i64)
        .bind(sandbox.gpu_enabled)
        .bind(&sandbox.gpu_model)
        .bind(&sandbox.public_url)
        .bind(sandbox.ssh_enabled)
        .bind(&sandbox.ssh_key)
        .bind(sandbox.created_at.to_rfc3339())
        .bind(sandbox.started_at.map(|d| d.to_rfc3339()))
        .bind(sandbox.stopped_at.map(|d| d.to_rfc3339()))
        .bind(sandbox.terminated_at.map(|d| d.to_rfc3339()))
        .bind(&sandbox.error_message)
        .bind(sandbox.cost_estimate)
        .bind(&sandbox.project_id)
        .bind(&sandbox.user_id)
        .bind(config_json.as_deref())
        .bind(metadata_json.as_deref())
        .execute(&mut *tx)
        .await?;

        // Create environment variables
        for env_var in env_vars.iter() {
            let env_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO sandbox_env_vars (id, sandbox_id, name, value, is_secret, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&env_id)
            .bind(&sandbox_id)
            .bind(&env_var.name)
            .bind(&env_var.value)
            .bind(env_var.is_secret)
            .bind(env_var.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }

        // Create volumes
        for volume in volumes.iter() {
            let vol_id = uuid::Uuid::new_v4().to_string();
            sqlx::query(
                "INSERT INTO sandbox_volumes (id, sandbox_id, host_path, container_path, read_only, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .bind(&vol_id)
            .bind(&sandbox_id)
            .bind(&volume.host_path)
            .bind(&volume.container_path)
            .bind(volume.read_only)
            .bind(volume.created_at.to_rfc3339())
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        Ok(Sandbox {
            id: sandbox_id,
            ..sandbox
        })
    }

    // ========================================================================
    // EXECUTION OPERATIONS
    // ========================================================================

    pub async fn create_execution(
        &self,
        mut execution: SandboxExecution,
    ) -> Result<SandboxExecution> {
        // Generate ID if not provided
        if execution.id.is_empty() {
            execution.id = format!("exec_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
        }

        sqlx::query(
            r#"
            INSERT INTO sandbox_executions (
                id, sandbox_id, command, working_directory, status,
                started_at, completed_at, exit_code, stdout, stderr,
                cpu_time_seconds, memory_peak_mb,
                created_at, created_by, agent_execution_id
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5,
                ?6, ?7, ?8, ?9, ?10,
                ?11, ?12,
                ?13, ?14, ?15
            )
            "#,
        )
        .bind(&execution.id)
        .bind(&execution.sandbox_id)
        .bind(&execution.command)
        .bind(&execution.working_directory)
        .bind(execution.status.as_str())
        .bind(execution.started_at.map(|d| d.to_rfc3339()))
        .bind(execution.completed_at.map(|d| d.to_rfc3339()))
        .bind(execution.exit_code)
        .bind(&execution.stdout)
        .bind(&execution.stderr)
        .bind(execution.cpu_time_seconds)
        .bind(execution.memory_peak_mb.map(|m| m as i32))
        .bind(execution.created_at.to_rfc3339())
        .bind(&execution.created_by)
        .bind(&execution.agent_execution_id)
        .execute(&self.pool)
        .await?;

        Ok(execution)
    }

    pub async fn get_execution(&self, id: &str) -> Result<SandboxExecution> {
        let row = sqlx::query(
            r#"
            SELECT id, sandbox_id, command, working_directory, status,
                   started_at, completed_at, exit_code, stdout, stderr,
                   cpu_time_seconds, memory_peak_mb,
                   created_at, created_by, agent_execution_id
            FROM sandbox_executions
            WHERE id = ?1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        match row {
            Some(row) => Ok(self.row_to_execution(row)?),
            None => Err(StorageError::NotFound(id.to_string())),
        }
    }

    pub async fn list_executions(&self, sandbox_id: &str) -> Result<Vec<SandboxExecution>> {
        let rows = sqlx::query(
            r#"
            SELECT id, sandbox_id, command, working_directory, status,
                   started_at, completed_at, exit_code, stdout, stderr,
                   cpu_time_seconds, memory_peak_mb,
                   created_at, created_by, agent_execution_id
            FROM sandbox_executions
            WHERE sandbox_id = ?1
            ORDER BY created_at DESC
            "#,
        )
        .bind(sandbox_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_execution(row))
            .collect()
    }

    pub async fn update_execution_status(
        &self,
        id: &str,
        status: ExecutionStatus,
        exit_code: Option<i32>,
        stdout: Option<String>,
        stderr: Option<String>,
    ) -> Result<()> {
        let now = Utc::now();
        let completed_at = match status {
            ExecutionStatus::Completed | ExecutionStatus::Failed | ExecutionStatus::Cancelled => {
                Some(now.to_rfc3339())
            }
            _ => None,
        };
        let started_at = match status {
            ExecutionStatus::Running => Some(now.to_rfc3339()),
            _ => None,
        };

        sqlx::query(
            r#"
            UPDATE sandbox_executions
            SET status = ?1, exit_code = ?2, stdout = ?3, stderr = ?4,
                completed_at = COALESCE(?5, completed_at),
                started_at = COALESCE(?6, started_at)
            WHERE id = ?7
            "#,
        )
        .bind(status.as_str())
        .bind(exit_code)
        .bind(stdout)
        .bind(stderr)
        .bind(completed_at)
        .bind(started_at)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ========================================================================
    // ENVIRONMENT VARIABLE OPERATIONS
    // ========================================================================

    pub async fn add_env_var(&self, env_var: EnvVar) -> Result<EnvVar> {
        let id: i32 = sqlx::query(
            r#"
            INSERT INTO sandbox_env_vars (sandbox_id, name, value, is_secret)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING id
            "#,
        )
        .bind(&env_var.sandbox_id)
        .bind(&env_var.name)
        .bind(&env_var.value)
        .bind(env_var.is_secret)
        .fetch_one(&self.pool)
        .await?
        .get(0);

        let mut result = env_var;
        result.id = Some(id);
        Ok(result)
    }

    pub async fn list_env_vars(&self, sandbox_id: &str) -> Result<Vec<EnvVar>> {
        let rows = sqlx::query(
            r#"
            SELECT id, sandbox_id, name, value, is_secret, created_at
            FROM sandbox_env_vars
            WHERE sandbox_id = ?1
            ORDER BY name
            "#,
        )
        .bind(sandbox_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_env_var(row))
            .collect()
    }

    pub async fn delete_env_var(&self, sandbox_id: &str, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM sandbox_env_vars WHERE sandbox_id = ?1 AND name = ?2")
            .bind(sandbox_id)
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // VOLUME OPERATIONS
    // ========================================================================

    pub async fn add_volume(&self, volume: Volume) -> Result<Volume> {
        let id: i32 = sqlx::query(
            r#"
            INSERT INTO sandbox_volumes (sandbox_id, host_path, container_path, read_only)
            VALUES (?1, ?2, ?3, ?4)
            RETURNING id
            "#,
        )
        .bind(&volume.sandbox_id)
        .bind(&volume.host_path)
        .bind(&volume.container_path)
        .bind(volume.read_only)
        .fetch_one(&self.pool)
        .await?
        .get(0);

        let mut result = volume;
        result.id = Some(id);
        Ok(result)
    }

    pub async fn list_volumes(&self, sandbox_id: &str) -> Result<Vec<Volume>> {
        let rows = sqlx::query(
            r#"
            SELECT id, sandbox_id, host_path, container_path, read_only, created_at
            FROM sandbox_volumes
            WHERE sandbox_id = ?1
            ORDER BY container_path
            "#,
        )
        .bind(sandbox_id)
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| self.row_to_volume(row))
            .collect()
    }

    pub async fn delete_volume(&self, id: i32) -> Result<()> {
        sqlx::query("DELETE FROM sandbox_volumes WHERE id = ?1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    fn row_to_sandbox(&self, row: sqlx::sqlite::SqliteRow) -> Result<Sandbox> {
        use sqlx::Row;

        Ok(Sandbox {
            id: row.get("id"),
            name: row.get("name"),
            provider: row.get("provider"),
            agent_id: row.get("agent_id"),
            status: SandboxStatus::from_str(&row.get::<String, _>("status"))?,
            container_id: row.get("container_id"),
            port: row.get::<Option<i32>, _>("port").map(|p| p as u16),
            cpu_cores: row.get("cpu_cores"),
            memory_mb: row.get::<i32, _>("memory_mb") as u32,
            storage_gb: row.get::<i32, _>("storage_gb") as u32,
            gpu_enabled: row.get("gpu_enabled"),
            gpu_model: row.get("gpu_model"),
            public_url: row.get("public_url"),
            ssh_enabled: row.get("ssh_enabled"),
            ssh_key: row.get("ssh_key"),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
            started_at: row
                .get::<Option<String>, _>("started_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            stopped_at: row
                .get::<Option<String>, _>("stopped_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            terminated_at: row
                .get::<Option<String>, _>("terminated_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            error_message: row.get("error_message"),
            cost_estimate: row.get("cost_estimate"),
            project_id: row.get("project_id"),
            user_id: row.get("user_id"),
            config: row
                .get::<Option<String>, _>("config")
                .and_then(|s| serde_json::from_str(&s).ok()),
            metadata: row
                .get::<Option<String>, _>("metadata")
                .and_then(|s| serde_json::from_str(&s).ok()),
        })
    }

    fn row_to_execution(&self, row: sqlx::sqlite::SqliteRow) -> Result<SandboxExecution> {
        use sqlx::Row;

        Ok(SandboxExecution {
            id: row.get("id"),
            sandbox_id: row.get("sandbox_id"),
            command: row.get("command"),
            working_directory: row.get("working_directory"),
            status: ExecutionStatus::from_str(&row.get::<String, _>("status"))?,
            started_at: row
                .get::<Option<String>, _>("started_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            completed_at: row
                .get::<Option<String>, _>("completed_at")
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc)),
            exit_code: row.get("exit_code"),
            stdout: row.get("stdout"),
            stderr: row.get("stderr"),
            cpu_time_seconds: row.get("cpu_time_seconds"),
            memory_peak_mb: row
                .get::<Option<i32>, _>("memory_peak_mb")
                .map(|m| m as u32),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
            created_by: row.get("created_by"),
            agent_execution_id: row.get("agent_execution_id"),
        })
    }

    fn row_to_env_var(&self, row: sqlx::sqlite::SqliteRow) -> Result<EnvVar> {
        use sqlx::Row;

        Ok(EnvVar {
            id: Some(row.get("id")),
            sandbox_id: row.get("sandbox_id"),
            name: row.get("name"),
            value: row.get("value"),
            is_secret: row.get("is_secret"),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
        })
    }

    fn row_to_volume(&self, row: sqlx::sqlite::SqliteRow) -> Result<Volume> {
        use sqlx::Row;

        Ok(Volume {
            id: Some(row.get("id")),
            sandbox_id: row.get("sandbox_id"),
            host_path: row.get("host_path"),
            container_path: row.get("container_path"),
            read_only: row.get("read_only"),
            created_at: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_at"))
                .unwrap()
                .with_timezone(&Utc),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn setup_test_db() -> SqlitePool {
        // Create an in-memory SQLite pool for testing
        let pool = SqlitePool::connect("sqlite::memory:")
            .await
            .expect("Failed to create in-memory database");

        // Disable foreign keys for testing to simplify setup
        sqlx::query("PRAGMA foreign_keys = OFF")
            .execute(&pool)
            .await
            .expect("Failed to disable foreign keys");

        // Run the initial schema migration
        let migration_sql = include_str!("../../storage/migrations/001_initial_schema.sql");
        sqlx::query(migration_sql)
            .execute(&pool)
            .await
            .expect("Failed to run migrations");

        // Create test user (required for sandbox user_id reference)
        sqlx::query(
            "INSERT OR REPLACE INTO users (id, email, name, created_at, updated_at)
             VALUES ('testuser123', 'test@example.com', 'Test User', datetime('now'), datetime('now'))"
        )
        .execute(&pool)
        .await
        .expect("Failed to create test user");

        // Make sure local provider exists
        sqlx::query(
            "INSERT OR REPLACE INTO sandbox_provider_settings (provider, enabled, updated_at)
             VALUES ('local', TRUE, datetime('now'))",
        )
        .execute(&pool)
        .await
        .expect("Failed to create local provider");

        pool
    }

    #[tokio::test]
    async fn test_create_and_get_sandbox() {
        let pool = setup_test_db().await;
        let storage = SandboxStorage::new(pool.clone());

        let sandbox = Sandbox {
            id: String::new(),
            name: "Test Sandbox".to_string(),
            provider: "local".to_string(),
            agent_id: "claude-code".to_string(),
            status: SandboxStatus::Creating,
            container_id: None,
            port: None,
            cpu_cores: 2.0,
            memory_mb: 4096,
            storage_gb: 20,
            gpu_enabled: false,
            gpu_model: None,
            public_url: None,
            ssh_enabled: false,
            ssh_key: None,
            created_at: Utc::now(),
            started_at: None,
            stopped_at: None,
            terminated_at: None,
            error_message: None,
            cost_estimate: None,
            project_id: None,
            user_id: "testuser123".to_string(),
            config: None,
            metadata: None,
        };

        let created = storage.create_sandbox(sandbox).await.unwrap();
        assert!(!created.id.is_empty());

        let retrieved = storage.get_sandbox(&created.id).await.unwrap();
        assert_eq!(retrieved.name, "Test Sandbox");
        assert_eq!(retrieved.provider, "local");
    }

    #[tokio::test]
    async fn test_update_sandbox_status() {
        let pool = setup_test_db().await;
        let storage = SandboxStorage::new(pool.clone());

        let sandbox = Sandbox {
            id: String::new(),
            name: "Status Test".to_string(),
            provider: "local".to_string(),
            agent_id: "claude-code".to_string(),
            status: SandboxStatus::Creating,
            container_id: None,
            port: None,
            cpu_cores: 1.0,
            memory_mb: 2048,
            storage_gb: 10,
            gpu_enabled: false,
            gpu_model: None,
            public_url: None,
            ssh_enabled: false,
            ssh_key: None,
            created_at: Utc::now(),
            started_at: None,
            stopped_at: None,
            terminated_at: None,
            error_message: None,
            cost_estimate: None,
            project_id: None,
            user_id: "testuser123".to_string(),
            config: None,
            metadata: None,
        };

        let created = storage.create_sandbox(sandbox).await.unwrap();

        // Update to running
        storage
            .update_sandbox_status(&created.id, SandboxStatus::Running, None)
            .await
            .unwrap();
        let running = storage.get_sandbox(&created.id).await.unwrap();
        assert_eq!(running.status, SandboxStatus::Running);
        assert!(running.started_at.is_some());

        // Update to stopped
        storage
            .update_sandbox_status(&created.id, SandboxStatus::Stopped, None)
            .await
            .unwrap();
        let stopped = storage.get_sandbox(&created.id).await.unwrap();
        assert_eq!(stopped.status, SandboxStatus::Stopped);
        assert!(stopped.stopped_at.is_some());
    }

    #[tokio::test]
    async fn test_sandbox_executions() {
        let pool = setup_test_db().await;
        let storage = SandboxStorage::new(pool.clone());

        // Create sandbox first
        let sandbox = Sandbox {
            id: String::new(),
            name: "Exec Test".to_string(),
            provider: "local".to_string(),
            agent_id: "claude-code".to_string(),
            status: SandboxStatus::Running,
            container_id: Some("container123".to_string()),
            port: Some(8080),
            cpu_cores: 1.0,
            memory_mb: 2048,
            storage_gb: 10,
            gpu_enabled: false,
            gpu_model: None,
            public_url: None,
            ssh_enabled: false,
            ssh_key: None,
            created_at: Utc::now(),
            started_at: Some(Utc::now()),
            stopped_at: None,
            terminated_at: None,
            error_message: None,
            cost_estimate: None,
            project_id: None,
            user_id: "testuser123".to_string(),
            config: None,
            metadata: None,
        };

        let created_sandbox = storage.create_sandbox(sandbox).await.unwrap();

        // Create execution
        let execution = SandboxExecution {
            id: String::new(),
            sandbox_id: created_sandbox.id.clone(),
            command: "echo 'Hello World'".to_string(),
            working_directory: "/workspace".to_string(),
            status: ExecutionStatus::Queued,
            started_at: None,
            completed_at: None,
            exit_code: None,
            stdout: None,
            stderr: None,
            cpu_time_seconds: None,
            memory_peak_mb: None,
            created_at: Utc::now(),
            created_by: Some("testuser123".to_string()),
            agent_execution_id: None,
        };

        let created_exec = storage.create_execution(execution).await.unwrap();
        assert!(!created_exec.id.is_empty());

        // Update execution status
        storage
            .update_execution_status(
                &created_exec.id,
                ExecutionStatus::Completed,
                Some(0),
                Some("Hello World".to_string()),
                None,
            )
            .await
            .unwrap();

        let updated = storage.get_execution(&created_exec.id).await.unwrap();
        assert_eq!(updated.status, ExecutionStatus::Completed);
        assert_eq!(updated.exit_code, Some(0));
        assert_eq!(updated.stdout, Some("Hello World".to_string()));
    }
}
