// ABOUTME: Common test utilities for integration tests
// ABOUTME: Provides test server setup, database helpers, and HTTP client utilities

use api::{
    create_ai_router, create_ai_usage_router, create_changes_router, create_prds_router,
    create_specs_router, create_task_spec_router,
};
use axum::Router;
use orkee_projects::DbState;
use sqlx::SqlitePool;
use tempfile::TempDir;
use tokio::sync::Mutex;

/// Global mutex to ensure thread-safe test execution
static TEST_MUTEX: Mutex<()> = Mutex::const_new(());

/// Test context containing server URL and database pool
pub struct TestContext {
    pub base_url: String,
    #[allow(dead_code)]
    pub pool: SqlitePool,
    pub _temp_dir: TempDir,
}

/// Create a test server with isolated database for OpenSpec endpoints
pub async fn setup_test_server() -> TestContext {
    let _guard = TEST_MUTEX.lock().await;

    // Create temporary directory to keep temp_dir alive
    let temp_dir = TempDir::new().unwrap();

    // Use in-memory database for tests (simpler and faster)
    let database_url = "sqlite::memory:";

    // Create database pool directly
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
        .expect("Failed to create database pool");

    // Configure SQLite settings
    sqlx::query("PRAGMA journal_mode = WAL")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&pool)
        .await
        .unwrap();

    // Run migrations
    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    // Create DbState for API handlers
    let db_state = DbState::new(pool.clone()).expect("Failed to create DbState");

    let app = Router::new()
        .merge(create_prds_router())
        .merge(create_specs_router())
        .merge(create_changes_router())
        .merge(create_task_spec_router())
        .merge(create_ai_router())
        .merge(create_ai_usage_router())
        .with_state(db_state);

    // Bind to random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let base_url = format!("http://{}", addr);

    // Spawn server
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    // Give server time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    TestContext {
        base_url,
        pool,
        _temp_dir: temp_dir,
    }
}

/// Helper to make GET requests
pub async fn get(base_url: &str, path: &str) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .get(format!("{}{}", base_url, path))
        .send()
        .await
        .expect("Failed to make GET request")
}

/// Helper to make POST requests with JSON body
#[allow(dead_code)]
pub async fn post_json<T: serde::Serialize>(
    base_url: &str,
    path: &str,
    body: &T,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .post(format!("{}{}", base_url, path))
        .json(body)
        .send()
        .await
        .expect("Failed to make POST request")
}

/// Helper to make PUT requests with JSON body
#[allow(dead_code)]
pub async fn put_json<T: serde::Serialize>(
    base_url: &str,
    path: &str,
    body: &T,
) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .put(format!("{}{}", base_url, path))
        .json(body)
        .send()
        .await
        .expect("Failed to make PUT request")
}

/// Helper to make DELETE requests
#[allow(dead_code)]
pub async fn delete(base_url: &str, path: &str) -> reqwest::Response {
    let client = reqwest::Client::new();
    client
        .delete(format!("{}{}", base_url, path))
        .send()
        .await
        .expect("Failed to make DELETE request")
}

/// Create a test project in the database
#[allow(dead_code)]
pub async fn create_test_project(pool: &SqlitePool, name: &str, path: &str) -> String {
    let id = nanoid::nanoid!(8);
    // Use runtime query to avoid compile-time validation issues in tests
    sqlx::query("INSERT INTO projects (id, name, project_root, description) VALUES (?, ?, ?, ?)")
        .bind(&id)
        .bind(name)
        .bind(path)
        .bind("Test project")
        .execute(pool)
        .await
        .expect("Failed to create test project");
    id
}
