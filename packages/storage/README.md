# Orkee Storage Package

> SQLite-based storage layer for the Orkee project with migrations, schema management, and test infrastructure.

## Overview

The storage package provides the database foundation for Orkee, including:

- **SQLite Database**: Local-first architecture with `~/.orkee/orkee.db`
- **Migration System**: SQLx-managed migrations with comprehensive schema
- **Schema**: 35+ tables covering projects, users, tasks, PRDs, ideation, telemetry, and cloud sync
- **Test Infrastructure**: 1200+ lines of integration tests with established patterns

### Key Numbers

- **Tables**: 35+
- **Indexes**: 40+
- **Triggers**: 35+ (FTS, timestamps, cascade operations)
- **Migration Files**: 2 (001 up/down)
- **Test Files**: 8 integration test files
- **Test Cases**: 30+
- **JSON Fields**: 4 primary fields with validation

---

## Migration Strategy

### Pre-1.0: Consolidated Migration
- Single initial migration (`001_initial_schema.sql`)
- No production instances exist yet (safe to consolidate)
- SQLx tracks migrations in `_sqlx_migrations` table

### Post-1.0: Incremental Migrations
- All schema changes will be separate, incremental migrations
- Application-generated IDs (not auto-increment)
- Down migrations available for development resets

### Migration Files

**Location**: `migrations/`

1. **`001_initial_schema.sql`** (108 KB)
   - Complete initial schema with all tables, views, triggers, and indexes
   - 95+ CREATE statements
   - Seed data with `INSERT OR IGNORE` for idempotency

2. **`001_initial_schema.down.sql`** (7.5 KB)
   - Full rollback migration for development resets
   - Drops triggers, views, then tables in reverse dependency order
   - Uses `DROP IF EXISTS` for idempotency

---

## Database Schema

### Core Tables

**Project Management**:
- `projects` - Main project entity with JSON fields (tags, manual_tasks, mcp_servers, git_repository)
- `projects_fts` - Full-text search virtual table with triggers

**User & Security**:
- `users` - User profiles with encrypted API keys (ChaCha20-Poly1305)
- `api_tokens` - API token storage
- `encryption_settings` - Encryption mode configuration (machine or password-based)
- `password_attempts` - Brute-force protection tracking

**Task Management**:
- `tasks` - Task records with FTS support
- `tasks_fts` - Task full-text search with triggers

**PRD & Ideation**:
- `prds` - Product Requirements Documents with version tracking
- `ideate_sessions` - PRD ideation sessions (guided and quick modes)
- `ideate_overview`, `ideate_ux`, `ideate_technical`, etc. - Structured PRD sections
- `prd_output_templates` - Customizable PRD output templates

**Telemetry & Cloud**:
- `telemetry_events`, `telemetry_settings`, `telemetry_stats` - Telemetry data
- `sync_state`, `sync_snapshots` - Cloud sync state tracking
- `storage_metadata` - Storage type and creation info

**AI & Context**:
- `ai_usage_logs` - API usage tracking
- `agent_executions` - Agent execution records
- `user_agents` - User-agent associations
- `context_configurations`, `context_snapshots` - Context management

### JSON Fields with Validation

All JSON fields use `json_valid()` CHECK constraints:

```sql
-- projects table
CHECK (json_valid(tags) OR tags IS NULL)
CHECK (json_valid(manual_tasks) OR manual_tasks IS NULL)
CHECK (json_valid(mcp_servers) OR mcp_servers IS NULL)
CHECK (json_valid(git_repository) OR git_repository IS NULL)
```

**Mapped to Rust types** (see `packages/core/src/types.rs`):
- `projects.tags` → `Option<Vec<String>>`
- `projects.manual_tasks` → `Option<Vec<ManualTask>>`
- `projects.mcp_servers` → `Option<HashMap<String, bool>>`
- `projects.git_repository` → `Option<GitRepositoryInfo>`

### Constraints & Features

- **Foreign Keys**: Enabled via `PRAGMA foreign_keys = ON`
- **CHECK Constraints**:
  - Status validation (projects, tasks status enums)
  - Priority validation (low, medium, high)
  - ID length minimum (8 characters)
  - JSON validation for complex fields
- **Cascade Rules**: ON DELETE CASCADE for FK relationships
- **Indexes**: 40+ indexes covering query patterns (filters, sorts, FTS, composite keys)
- **Triggers**: 35+ triggers for FTS maintenance, timestamp updates, cascade operations

---

## Serialization & Type Mapping

### Field Name Mapping

Rust snake_case ↔ JSON camelCase via `#[serde(rename)]`:

```rust
pub struct ManualTask {
    #[serde(rename = "createdAt")]
    pub created_at: DateTime<Utc>,

    #[serde(rename = "testStrategy")]
    pub test_strategy: Option<String>,
}
```

### Enum Case Conversion

```rust
#[serde(rename_all = "kebab-case")]
pub enum ProjectStatus {
    Planning,      // → "planning"
    InProgress,    // → "in-progress"
    OnHold,        // → "on-hold"
}

#[serde(rename_all = "lowercase")]
pub enum Priority {
    High,    // → "high"
    Medium,  // → "medium"
    Low,     // → "low"
}
```

### Null Handling

- All JSON fields are `Option<T>`
- SQL NULL maps to Rust `None`, not JSON null
- Empty collections stored as JSON `[]`, not NULL

### Deserialization Pattern

**Location**: `src/sqlite.rs` (lines 96-124)

```rust
fn row_to_project(&self, row: &SqliteRow) -> StorageResult<Project> {
    let tags_json: Option<String> = row.try_get("tags")?;
    let tags = if let Some(json) = tags_json {
        Some(serde_json::from_str(&json)?)  // Deserialize JSON string
    } else {
        None
    };
    // Similar for: manual_tasks, mcp_servers, git_repository
}
```

---

## Seed Data

All seed data uses `INSERT OR IGNORE` for idempotent migrations:

- `storage_metadata` - Storage type and creation timestamp
- `users` - Default user (id: "default-user", required for FK dependencies)
- `tags` - Default "main" tag (required for task FK dependencies)
- `encryption_settings` - Machine-based encryption by default
- `password_attempts` - Password attempt tracking initialization
- `telemetry_settings` - Telemetry configuration defaults
- `system_settings` - Default configuration (ports, security, TLS, rate limiting)

**Location**: `migrations/001_initial_schema.sql:1062-1128`

---

## Test Infrastructure

### Test Files

**Location**: `packages/projects/tests/`

**Main Test File**: `migration_integration_tests.rs` (1200+ lines)

Test coverage includes:
- **Schema Validation** (10+ tests): Tables, indexes, FTS tables, migration tracking
- **Foreign Key Tests** (3+ tests): FK constraints, relationships, cascade behavior
- **Data Integrity** (5+ tests): CHECK constraints, enum validation, ID length, type checking
- **Cascade & Deletion** (3+ tests): Parent-child deletion, CASCADE behavior
- **Seed Data** (5+ tests): Default user, metadata, idempotency
- **FTS Triggers** (2+ tests): Insert/delete triggers for full-text search
- **Quality Checks** (1+ test): Orphaned index detection

**Other Integration Tests**:
- `prd_integration_tests.rs` - PRD CRUD operation tests
- `ai_usage_integration_tests.rs` - AI usage tracking tests
- `ai_proxy_integration_tests.rs` - AI proxy tests
- `type_serialization_tests.rs` - Type serialization/deserialization tests

**Common Utilities**: `common/mod.rs` - Test helpers and setup functions

### Test Patterns

#### Pattern 1: In-Memory Database Setup

```rust
async fn setup_migrated_db() -> Pool<Sqlite> {
    let pool = Pool::<Sqlite>::connect(":memory:").await.unwrap();
    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .expect("Migration should succeed");
    pool
}
```

#### Pattern 2: Raw SQL Testing

```rust
#[tokio::test]
async fn test_schema_feature() {
    let pool = setup_migrated_db().await;
    let result: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM users")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(result, 1);
}
```

#### Pattern 3: JSON Validation in SQL

```rust
let result = sqlx::query(
    "INSERT INTO projects (id, name, project_root, tags, created_at, updated_at)
     VALUES (?, ?, ?, ?, datetime('now', 'utc'), datetime('now', 'utc'))"
)
.bind(id)
.bind(name)
.bind(root)
.bind(json_data)  // Must be valid JSON or NULL
.execute(&pool)
.await;
```

#### Pattern 4: Integration Test Server

```rust
pub async fn setup_test_server() -> TestContext {
    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(":memory:")
        .await
        .expect("Failed to create database pool");

    sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await.unwrap();
    sqlx::query("PRAGMA foreign_keys = ON").execute(&pool).await.unwrap();

    sqlx::migrate!("../storage/migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let db_state = DbState::new(pool.clone()).expect("Failed to create DbState");
    let app = create_router().with_state(db_state);

    // Spawn server, return TestContext with base_url and pool
}
```

#### Pattern 5: JSON Integration Tests

```rust
let response = post_json(
    &ctx.base_url,
    &format!("/{}/specs", project_id),
    &json!({
        "name": "User Authentication",
        "specMarkdown": "# Requirements\n\nLogin: Users can login",
    }),
).await;

let body: serde_json::Value = response.json().await.unwrap();
assert_eq!(body["data"]["name"], "User Authentication");
```

#### Pattern 6: Trigger Testing

```rust
#[tokio::test]
async fn test_fts_trigger_on_insert() {
    let pool = setup_migrated_db().await;

    // Insert a project
    sqlx::query("INSERT INTO projects (...) VALUES (...)")
        .execute(&pool)
        .await
        .unwrap();

    // Verify FTS entry was created by trigger
    let fts_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM projects_fts WHERE id = ?"
    )
    .bind(project_id)
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(fts_count, 1);
}
```

#### Pattern 7: Error Validation

```rust
let result = sqlx::query("INSERT INTO projects ...")
    .execute(&pool)
    .await;

assert!(result.is_err());
let error_msg = result.unwrap_err().to_string();
assert!(error_msg.to_lowercase().contains("check"));
```

---

## Development Workflow

### Running Migrations

```bash
# Migrations run automatically in tests
cargo test

# Reset development database
rm ~/.orkee/orkee.db && cargo run

# Create new migration (post-1.0)
sqlx migrate add <migration_name>
```

### Migration Checksum Mismatches (Pre-1.0 Only)

**Context**: During pre-1.0 development, we modify the initial migration (`001_initial_schema.sql`) instead of creating new migrations. This is safe because there are no production instances, but it breaks migration checksums for existing dev databases.

**Symptoms**:
- `sqlx::migrate()` fails with checksum mismatch error
- Application won't start due to migration verification failure
- Error message: "migration checksum mismatch"

**Solution** (choose one):

1. **Full Database Reset** (recommended for clean slate):
   ```bash
   # Delete database and restart
   rm ~/.orkee/orkee.db && cargo run
   ```

2. **Manual Checksum Reset** (if you need to preserve data):
   ```bash
   # Backup your data first!
   cp ~/.orkee/orkee.db ~/.orkee/orkee.db.backup

   # Reset migration tracking table
   sqlite3 ~/.orkee/orkee.db "DELETE FROM _sqlx_migrations WHERE version = 1"

   # Restart application (will re-run migration with new checksum)
   cargo run
   ```

3. **Test Database Reset** (for test failures):
   ```bash
   # Tests use in-memory databases - just re-run tests
   cargo test
   ```

**Why This Happens**:
- SQLx tracks migration checksums in `_sqlx_migrations.checksum`
- Modifying a migration file changes its checksum
- SQLx detects mismatch between file and database record
- This prevents accidental schema drift in production

**Post-1.0 Behavior**:
- We'll use incremental migrations (002, 003, etc.)
- Never modify existing migrations
- Checksums will always match
- This workflow is temporary for pre-release development

### Testing

```bash
# Run all tests
cargo test

# Run specific test file
cargo test --test migration_integration_tests

# Run specific test
cargo test test_initial_schema_migration_succeeds
```

### Database Management

```bash
# Inspect database schema
sqlite3 ~/.orkee/orkee.db ".schema projects"

# Check migration status
sqlite3 ~/.orkee/orkee.db "SELECT * FROM _sqlx_migrations"

# Query JSON fields
sqlite3 ~/.orkee/orkee.db "SELECT id, json_extract(tags, '$') FROM projects"
```

---

## Key Insights

### JSON Serialization
- Database enforces `json_valid()` CHECK constraints
- Field names use snake_case in Rust, camelCase in JSON
- Null values stored as SQL NULL, not JSON null
- All JSON fields are `Option<T>` (can be NULL)

### Test Infrastructure
- In-memory SQLite for fast test execution
- Migrations run automatically in test setup
- Full HTTP server available for integration tests
- 1200+ line comprehensive test suite

### Database Design
- Pre-1.0: Single consolidated migration
- Post-1.0: Incremental migrations per schema change
- Application-generated IDs (not auto-increment)
- Foreign keys enabled with explicit cascade rules
- 40+ indexes for performance
- Full-text search with triggers

### Migration Strategy
- SQLx manages migration tracking via `_sqlx_migrations` table
- Down migration available for development resets
- Seed data idempotent via INSERT OR IGNORE
- No production instances yet (safe to consolidate)

---

## File Structure

```
packages/storage/
├── migrations/
│   ├── 001_initial_schema.sql (108 KB)
│   └── 001_initial_schema.down.sql (7.5 KB)
└── src/
    ├── sqlite.rs          # SQLite implementation, row_to_project deserialization
    ├── factory.rs         # Storage factory pattern
    ├── lib.rs             # Public API
    ├── legacy.rs          # Legacy JSON migration
    └── test_utils.rs      # Test utilities

packages/projects/tests/
├── migration_integration_tests.rs    # Main migration tests (1200+ lines)
├── prd_integration_tests.rs          # PRD CRUD operation tests
├── ai_usage_integration_tests.rs     # AI usage tracking tests
├── ai_proxy_integration_tests.rs     # AI proxy tests
├── type_serialization_tests.rs       # Type serialization tests
└── common/
    └── mod.rs                        # Test utilities and helpers

packages/core/src/
└── types.rs                          # Type definitions (37+ types with Serialize/Deserialize)
```

---

## Quick Reference

### Table Dependency Order (for FK tests)

1. `users` (no dependencies)
2. `projects` (no dependencies)
3. `tasks` (FK: projects.id, users.id)
4. `prds` (FK: projects.id)
5. `ideate_sessions` (FK: projects.id)
6. `ideate_overview`, `ideate_ux`, etc. (FK: ideate_sessions.id)

### Critical Constants

```rust
// Migration tracking
_sqlx_migrations table - SQLx internal tracking

// Default user (required for FK dependencies)
id: "default-user"
email: "user@localhost"

// Database settings
PRAGMA foreign_keys = ON
PRAGMA journal_mode = WAL
```

### Important Notes

- Always create default user before inserting tasks (FK dependency)
- FTS tables maintained automatically by triggers
- Enum values in DB must match serde rename attributes
- Timestamps always use `datetime('now', 'utc')`
- IDs must be at least 8 characters (CHECK constraint)

---

## Testing Checklist

When adding new features or modifying schema:

- [ ] Add migration file for schema changes (post-1.0)
- [ ] Update type definitions in `packages/core/src/types.rs`
- [ ] Add deserialization logic in `packages/storage/src/sqlite.rs`
- [ ] Write integration tests in `packages/projects/tests/`
- [ ] Test round-trip: Rust type → JSON → DB → Rust type
- [ ] Verify field names match serde attributes
- [ ] Check enum string representations match DB constraints
- [ ] Test edge cases: null, empty, special characters
- [ ] Verify FK relationships and cascade behavior
- [ ] Test migration idempotency

---

**Last Updated**: 2025-10-30
**Documentation Generated From**: Database foundation work (PR #01)
