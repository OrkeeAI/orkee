// ABOUTME: Database connection management and storage initialization
// ABOUTME: Provides shared access to SQLite pool and storage layers

use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};
use std::sync::Arc;
use tracing::{debug, info};

use crate::agents::AgentStorage;
use crate::executions::ExecutionStorage;
use crate::storage::StorageError;
use crate::tags::TagStorage;
use crate::tasks::TaskStorage;
use crate::users::UserStorage;

/// Shared database state for API handlers
#[derive(Clone)]
pub struct DbState {
    pub pool: SqlitePool,
    pub task_storage: Arc<TaskStorage>,
    pub agent_storage: Arc<AgentStorage>,
    pub user_storage: Arc<UserStorage>,
    pub tag_storage: Arc<TagStorage>,
    pub execution_storage: Arc<ExecutionStorage>,
}

impl DbState {
    /// Create new database state from a SQLite pool
    pub fn new(pool: SqlitePool) -> Self {
        let task_storage = Arc::new(TaskStorage::new(pool.clone()));
        let agent_storage = Arc::new(AgentStorage::new(pool.clone()));
        let user_storage = Arc::new(UserStorage::new(pool.clone()));
        let tag_storage = Arc::new(TagStorage::new(pool.clone()));
        let execution_storage = Arc::new(ExecutionStorage::new(pool.clone()));

        Self {
            pool,
            task_storage,
            agent_storage,
            user_storage,
            tag_storage,
            execution_storage,
        }
    }

    /// Initialize database state with default configuration
    pub async fn init() -> Result<Self, StorageError> {
        let database_path = crate::constants::orkee_dir().join("orkee.db");

        // Ensure parent directory exists
        if let Some(parent) = database_path.parent() {
            std::fs::create_dir_all(parent).map_err(StorageError::Io)?;
        }

        let database_url = format!("sqlite:{}", database_path.display());

        debug!("Connecting to database: {}", database_url);

        // Configure connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .acquire_timeout(std::time::Duration::from_secs(30))
            .connect(&database_url)
            .await
            .map_err(StorageError::Sqlx)?;

        // Configure SQLite settings
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA foreign_keys = ON")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&pool)
            .await
            .map_err(StorageError::Sqlx)?;

        info!("Database connection established");

        Ok(Self::new(pool))
    }
}
