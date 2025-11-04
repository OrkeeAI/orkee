# Vibekit Integration Plan for Orkee

## Executive Summary

This document outlines the comprehensive plan to integrate Vibekit SDK into Orkee for containerized AI agent execution. The integration enables users to execute tasks with AI coding agents in secure Docker containers with full observability and tracking.

### Key Features
- Local Docker sandbox execution (cloud providers in future phases)
- AI agent selection (Claude, OpenAI, Gemini, etc.)
- Real-time log streaming via SSE
- Comprehensive execution tracking in SQLite
- Resource monitoring and limits
- Artifact collection and download

### Architecture Overview
- **Hybrid Rust + Node.js**: Vibekit TypeScript SDK via Node.js child process, bollard for Docker monitoring
- **API Design**: RESTful endpoints nested under tasks
- **Database**: Extend existing tables + new execution tracking tables
- **Real-time**: Server-Sent Events for log streaming

### Important Naming Conventions
**NO "vibekit_" prefixes in database tables or fields!** Tables are named for their purpose, not the technology:
- ✅ `execution_logs` - NOT ❌ vibekit_logs
- ✅ `execution_artifacts` - NOT ❌ vibekit_artifacts
- ✅ Extend existing `agent_executions` table - NOT ❌ create new vibekit_executions
- ✅ Field: `sandbox_provider` - NOT ❌ vibekit_provider
- ✅ Field: `container_id` - NOT ❌ vibekit_container_id

The only "vibekit" references should be:
- `vibekit_session_id` field - specifically for tracking Vibekit SDK sessions
- `vibekit_version` field - for tracking SDK version
- Package name: `packages/sandboxes` - NOT packages/vibekit

---

## Implementation Approach

### Database Migration Strategy
Since no one is using the app yet, we're taking the simpler approach:
1. **Direct updates to `001_initial_schema.sql`** - No need for new migration files
2. **Direct updates to `001_initial_schema.down.sql`** - Keep down migration in sync
3. This avoids unnecessary migration complexity during pre-production development

### Naming Philosophy
- **Technology-agnostic naming** - Tables and fields describe their purpose, not the implementation
- **"sandbox" not "vibekit"** - We're building a sandbox execution system that happens to use Vibekit
- **Generic execution tracking** - The system could work with other SDKs in the future
- Only use "vibekit" prefix for SDK-specific tracking fields (session_id, version)

### Package Structure
- **`packages/sandboxes/`** - Main Rust package for sandbox orchestration
- **`packages/sandboxes/vibekit/`** - Node.js bridge for Vibekit SDK
- This keeps the Rust code separate from the TypeScript/Node.js integration

---

## Phase 1: Database Schema & Foundation (Week 1)

### Goals
Establish database schema, create sandbox package structure, and define core types.

### Tasks

#### 1.1 Database Schema Updates

**IMPORTANT**: We're updating the existing migration file directly since no one is using the app yet. This avoids unnecessary migration complexity.

- [x] Extend `agent_executions` table in `packages/storage/migrations/001_initial_schema.sql`

  Add these fields to the existing `agent_executions` table (before the closing `created_at` and `updated_at` fields):
  ```sql
  -- Sandbox execution fields (generic, not Vibekit-specific)
  sandbox_provider TEXT CHECK(sandbox_provider IN ('local', 'e2b', 'modal') OR sandbox_provider IS NULL),
  container_id TEXT,
  container_image TEXT,
  container_status TEXT CHECK(container_status IN ('creating', 'running', 'stopped', 'error') OR container_status IS NULL),

  -- Resource usage tracking
  memory_limit_mb INTEGER,
  memory_used_mb INTEGER,
  cpu_limit_cores REAL,
  cpu_usage_percent REAL,

  -- File system tracking
  workspace_path TEXT,
  output_files TEXT, -- JSON array of file paths

  -- SDK-specific metadata (only these two fields reference Vibekit)
  vibekit_session_id TEXT,  -- Tracks Vibekit SDK session
  vibekit_version TEXT,     -- SDK version for debugging
  environment_variables TEXT, -- JSON object
  ```

- [x] Create `execution_logs` table for streaming and replay
  ```sql
  CREATE TABLE execution_logs (
      id TEXT PRIMARY KEY CHECK(length(id) >= 8),
      execution_id TEXT NOT NULL REFERENCES agent_executions(id) ON DELETE CASCADE,
      timestamp TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ', 'now')),
      log_level TEXT NOT NULL CHECK(log_level IN ('debug', 'info', 'warn', 'error', 'fatal')),
      message TEXT NOT NULL,
      source TEXT, -- 'vibekit', 'agent', 'container', 'system'
      metadata TEXT, -- JSON object for structured logging
      stack_trace TEXT,
      sequence_number INTEGER NOT NULL,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
      UNIQUE(execution_id, sequence_number)
  );
  ```

- [x] Create `execution_artifacts` table for outputs
  ```sql
  CREATE TABLE execution_artifacts (
      id TEXT PRIMARY KEY CHECK(length(id) >= 8),
      execution_id TEXT NOT NULL REFERENCES agent_executions(id) ON DELETE CASCADE,
      artifact_type TEXT NOT NULL CHECK(artifact_type IN ('file', 'screenshot', 'test_report', 'coverage', 'output')),
      file_path TEXT NOT NULL,
      file_name TEXT NOT NULL,
      file_size_bytes INTEGER,
      mime_type TEXT,
      stored_path TEXT,
      storage_backend TEXT DEFAULT 'local' CHECK(storage_backend IN ('local', 's3', 'gcs')),
      description TEXT,
      metadata TEXT,
      checksum TEXT,
      created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
  );
  ```

- [x] Add indexes for all new tables
  - `idx_execution_logs_execution` on `execution_logs(execution_id, sequence_number)`
  - `idx_execution_logs_timestamp` on `execution_logs(timestamp)`
  - `idx_execution_logs_level` on `execution_logs(log_level)`
  - `idx_artifacts_execution` on `execution_artifacts(execution_id)`
  - `idx_artifacts_type` on `execution_artifacts(artifact_type)`
  - `idx_artifacts_created` on `execution_artifacts(created_at)`

- [x] Update `001_initial_schema.down.sql` with DROP statements
  ```sql
  -- Add to down migration (in correct order)
  DROP TABLE IF EXISTS execution_artifacts;
  DROP TABLE IF EXISTS execution_logs;
  -- Note: agent_executions columns will be removed when table is dropped
  ```

- [x] Add sandbox configuration to `system_settings` seed data
  ```sql
  -- Add to INSERT OR IGNORE INTO system_settings
  ('sandbox.default_provider', 'local', 'sandbox', 'Default sandbox provider', 'string', 0, 0),
  ('sandbox.default_image', 'ubuntu:22.04', 'sandbox', 'Default container image', 'string', 0, 0),
  ('sandbox.max_concurrent', '5', 'sandbox', 'Maximum concurrent executions', 'integer', 0, 0),
  ('sandbox.default_memory_mb', '2048', 'sandbox', 'Default memory limit (MB)', 'integer', 0, 0),
  ('sandbox.default_cpu_cores', '2.0', 'sandbox', 'Default CPU cores', 'number', 0, 0),
  ('sandbox.default_timeout_seconds', '3600', 'sandbox', 'Default execution timeout (seconds)', 'integer', 0, 0),
  ('sandbox.log_retention_days', '30', 'sandbox', 'Days to retain execution logs', 'integer', 0, 0),
  ('sandbox.artifact_retention_days', '30', 'sandbox', 'Days to retain execution artifacts', 'integer', 0, 0),
  ('sandbox.cleanup_interval_minutes', '5', 'sandbox', 'Interval for container cleanup task', 'integer', 0, 0)
  ```

#### 1.2 Create Sandbox Package Structure
- [x] Create `packages/sandboxes/` directory (plural to match workspace conventions)
- [x] Create `packages/sandboxes/Cargo.toml` with dependencies:
  - `bollard = "0.15"` (Docker API)
  - `tokio = { version = "1", features = ["full"] }`
  - `serde = { version = "1", features = ["derive"] }`
  - `serde_json = "1"`
  - `sqlx = { version = "0.7", features = ["sqlite", "runtime-tokio-rustls"] }`
  - `thiserror = "1"`
  - `tracing = "0.1"`
  - `async-stream = "0.3"`

- [x] Create package structure (created lib.rs, types.rs, error.rs, and sandboxes.json; provider.rs, node_bridge.rs, and container.rs are for Phase 2 & 3):
  ```
  packages/sandboxes/  (note: plural form)
  ├── Cargo.toml
  ├── src/
  │   ├── lib.rs           # Public API ✅
  │   ├── provider.rs      # SandboxProvider enum and registry (Phase 2)
  │   ├── node_bridge.rs   # Node.js child process management (Phase 2)
  │   ├── container.rs     # Container lifecycle via bollard (Phase 3)
  │   ├── types.rs         # Core types ✅
  │   └── error.rs         # Error types ✅
  └── config/
      └── sandboxes.json   # Sandbox provider configurations ✅
  ```

#### 1.3 Define Core Types
- [x] Create `packages/sandboxes/src/types.rs`:
  - [x] `SandboxProvider` enum (Local, E2B, Modal)
  - [x] `ExecutionRequest` struct
  - [x] `ExecutionResponse` struct
  - [x] `ContainerStatus` enum
  - [x] `LogEntry` struct
  - [x] `Artifact` struct
  - [x] `ResourceLimits` struct
  - [x] `ExecutionStatus` enum

- [x] Create `packages/sandboxes/src/error.rs`:
  - [x] `SandboxError` enum with thiserror
  - [x] Error variants for Docker, Vibekit, Node.js, Resources, Timeout

#### 1.4 Create Configuration
- [x] Create `packages/sandboxes/config/sandboxes.json`:
  ```json
  {
    "version": "2025-11-04",
    "sandboxes": [
      {
        "id": "local-docker",
        "name": "Local Docker",
        "provider": "local",
        "description": "Run sandboxes in local Docker containers",
        "supported_images": ["ubuntu:22.04", "node:20", "python:3.11"],
        "default_image": "ubuntu:22.04",
        "max_concurrent": 5,
        "resource_limits": {
          "memory_mb": 2048,
          "cpu_cores": 2.0,
          "timeout_seconds": 3600
        },
        "is_available": true,
        "requires_config": false
      }
    ]
  }
  ```

#### 1.5 Testing & Validation
- [x] Add `packages/sandboxes` to workspace `Cargo.toml`
- [ ] Run database migration and verify tables created (deferred - will test when implementing Phase 2)
- [ ] Write schema validation tests (deferred - will add during integration testing)
- [x] Verify package compiles

### Deliverables
- ✅ Database schema updated with Vibekit fields
- ✅ Sandbox package skeleton created
- ✅ Core type definitions complete
- ✅ Configuration structure defined
- ✅ All tests passing

---

## Phase 2: Node.js Vibekit Bridge (Week 1-2) ✅

### Goals
Implement Node.js bridge for Vibekit TypeScript SDK with IPC communication to Rust.

### Tasks

#### 2.1 Create Node.js Project
- [x] Create `packages/sandboxes/vibekit/` directory (using "vibekit" not "vibekit-bridge")
- [x] Initialize Node.js project:
  ```json
  // package.json
  {
    "name": "@orkee/vibekit-bridge",
    "version": "1.0.0",
    "main": "dist/index.js",
    "scripts": {
      "build": "tsc",
      "dev": "tsx src/index.ts",
      "test": "jest"
    },
    "dependencies": {
      "@vibe-kit/sdk": "^0.0.70",
      "@vibe-kit/docker": "^0.0.70"
    },
    "devDependencies": {
      "@types/node": "^20.0.0",
      "typescript": "^5.0.0",
      "tsx": "^4.0.0",
      "jest": "^29.0.0"
    }
  }
  ```

- [x] Create `tsconfig.json`:
  ```json
  {
    "compilerOptions": {
      "target": "ES2022",
      "module": "commonjs",
      "outDir": "./dist",
      "rootDir": "./src",
      "strict": true,
      "esModuleInterop": true,
      "skipLibCheck": true,
      "forceConsistentCasingInFileNames": true
    }
  }
  ```

#### 2.2 Implement IPC Communication
- [x] Create `packages/sandboxes/vibekit/src/index.ts`:
  - [x] Parse JSON from stdin
  - [x] Initialize Vibekit SDK
  - [x] Handle execution requests
  - [x] Stream responses to stdout
  - [x] Handle graceful shutdown

- [x] Create `packages/sandboxes/vibekit/src/types.ts`:
  - [x] `ExecutionRequest` interface
  - [x] `ExecutionResponse` interface
  - [x] `LogMessage` interface
  - [x] `IPCMessage` type union

- [x] Create `packages/sandboxes/vibekit/src/vibekit.ts`:
  - [x] Vibekit session management
  - [x] Agent execution logic
  - [x] Error handling
  - [x] Resource monitoring

#### 2.3 Implement Rust Bridge
- [x] Implement `packages/sandboxes/src/node_bridge.rs`:
  - [x] `NodeBridge` struct
  - [x] `spawn_bridge()` function to start Node.js process (implemented as `start()`)
  - [x] `send_request()` for JSON communication
  - [x] `receive_response()` for parsing responses
  - [x] Process lifecycle management (start, stop, restart)
  - [x] Error recovery logic

- [x] Implement message serialization:
  - [x] Request serialization to JSON
  - [x] Response deserialization from JSON
  - [x] Stream handling for logs
  - [x] Error message parsing

#### 2.4 Testing
- [x] Write unit tests for Rust bridge:
  - [x] Test bridge path detection
  - [x] Test IPC request serialization
  - [x] Test IPC response deserialization
- [ ] Write unit tests for Node.js bridge (deferred to Phase 7):
  - [ ] Mock stdin/stdout
  - [ ] Test message parsing
  - [ ] Test error scenarios
  - [ ] Test graceful shutdown
- [ ] Integration test: Rust → Node.js → Mock Vibekit (deferred to Phase 4)

### Deliverables
- ✅ Node.js Vibekit bridge implemented
- ✅ Bidirectional IPC protocol defined
- ✅ Error handling robust
- ✅ Process management complete
- ✅ Basic unit tests passing (full test suite in Phase 7)

---

## Phase 3: Container Management with bollard (Week 2) ✅

### Goals
Integrate bollard for Docker container monitoring and management.

### Tasks

#### 3.1 Implement Container Manager
- [x] Create `packages/sandboxes/src/container.rs`:
  - [x] `ContainerManager` struct
  - [x] `connect_docker()` - Connect to Docker daemon
  - [x] `create_container()` - Create secure container
  - [x] `start_container()` - Start container
  - [x] `stop_container()` - Stop container
  - [x] `remove_container()` - Clean up container
  - [x] `list_containers()` - List by project/execution

- [x] Implement container security:
  - [x] Resource limits (memory, CPU)
  - [x] Capability dropping
  - [x] Network isolation
  - [x] Read-only rootfs options
  - [x] No privileged mode

#### 3.2 Container Monitoring
- [x] Implement resource monitoring:
  - [x] `get_container_stats()` - CPU and memory usage
  - [x] `stream_container_logs()` - Real-time log streaming
  - [x] `get_container_info()` - Status and metadata
  - [x] Parse Docker events (via bollard)

- [x] Implement log processing:
  - [x] Parse container log format
  - [x] Add timestamps and sequence numbers
  - [x] Buffer logs for batch insertion (infrastructure ready)
  - [x] Handle log overflow (via async streams)

#### 3.3 Container Cleanup
- [x] Implement cleanup logic:
  - [x] `cleanup_stale_containers()` function
  - [x] Detect orphaned containers
  - [x] Force stop hung containers (`force_stop_hung_containers()`)
  - [x] Remove stopped containers
  - [ ] Update database status (deferred to Phase 4 - execution lifecycle integration)

- [ ] Background cleanup task (deferred to Phase 4):
  - [ ] Schedule periodic cleanup (5 minutes)
  - [ ] Log cleanup actions
  - [ ] Handle cleanup errors gracefully
  - [ ] Respect resource limits

#### 3.4 Container API Endpoints
- [x] Add container management endpoints:
  - [x] GET `/api/containers` - List all containers
  - [x] GET `/api/containers/:id` - Get container details
  - [x] GET `/api/containers/:id/stats` - Resource usage
  - [x] POST `/api/containers/:id/restart` - Restart container
  - [x] POST `/api/containers/:id/stop` - Force stop
  - [x] DELETE `/api/containers/:id` - Remove container

#### 3.5 Testing
- [x] Unit tests for container manager (basic tests in container.rs):
  - [x] Test Docker connection
  - [x] Test container lifecycle
  - [x] Test list containers
  - [ ] Test resource limit enforcement (requires running container)
  - [ ] Test cleanup logic (deferred - requires integration tests)
  - [ ] Test error scenarios (basic coverage in handler implementations)

- [ ] Integration tests with Docker (deferred to Phase 7):
  - [ ] Use testcontainers for real Docker
  - [ ] Test container lifecycle
  - [ ] Test resource monitoring
  - [ ] Test concurrent containers

### Deliverables
- ✅ bollard integration complete
- ✅ Container lifecycle management working
- ✅ Resource monitoring implemented
- ⏸️ Cleanup task deferred to Phase 4 (will integrate with execution lifecycle)
- ⏸️ Full test suite deferred to Phase 7 (basic tests in place)

---

## Phase 4: API Endpoints & Execution Logic (Week 3) ✅

### Goals
Implement REST API for execution management and core execution lifecycle.

### Tasks

#### 4.1 Create Execution Handlers
- [x] Create `packages/api/src/sandbox_execution_handlers.rs`:
  - [x] `stop_execution()` - Cancel running execution
  - [x] `retry_execution()` - Retry failed execution (placeholder for Phase 5)

- [x] Implement log endpoints:
  - [x] `get_execution_logs()` - Paginated logs
  - [x] `stream_execution_logs()` - SSE streaming (placeholder for Phase 5)
  - [x] `search_logs()` - Search with filters

- [x] Implement artifact endpoints:
  - [x] `list_artifacts()` - List execution outputs
  - [x] `get_artifact()` - Get artifact metadata
  - [x] `download_artifact()` - Stream file download
  - [x] `delete_artifact()` - Remove artifact

**Note**: `create_execution()`, `list_executions()`, and `get_execution()` already exist in `executions_handlers.rs` for the general `agent_executions` table. Sandbox-specific execution creation will be implemented when integrating with CLI server.

#### 4.2 Update API Router
- [x] Update `packages/api/src/lib.rs`:
  - [x] Add `create_sandbox_executions_router()` function
  - [ ] Apply authentication middleware (deferred to CLI integration)
  - [ ] Add rate limiting for execution endpoints (deferred to CLI integration)
  - [ ] Configure CORS for SSE (deferred to Phase 5)

- [x] Route structure implemented:
  ```
  /api/sandbox/executions/:execution_id/stop       - Stop execution
  /api/sandbox/executions/:execution_id/retry      - Retry execution
  /api/sandbox/executions/:execution_id/logs       - Get logs (paginated)
  /api/sandbox/executions/:execution_id/logs/stream - Stream logs (SSE placeholder)
  /api/sandbox/executions/:execution_id/logs/search - Search logs
  /api/sandbox/executions/:execution_id/artifacts  - List artifacts
  /api/sandbox/artifacts/:artifact_id              - Get artifact
  /api/sandbox/artifacts/:artifact_id/download     - Download artifact
  /api/sandbox/artifacts/:artifact_id              - Delete artifact
  ```

#### 4.3 Implement Execution Lifecycle
- [x] Create `packages/sandboxes/src/execution.rs`:
  - [x] `ExecutionOrchestrator` struct
  - [x] `start_execution()` - Main execution flow
  - [x] `monitor_execution()` - Track progress
  - [x] `finalize_execution()` - Cleanup and save results
  - [x] `stream_logs()` - Stream logs from container
  - [x] `collect_artifacts()` - Collect execution artifacts

- [x] Execution flow:
  1. [x] Validate request and check quotas
  2. [x] Create execution record in database (via external handler)
  3. [ ] Spawn Vibekit bridge process (infrastructure ready, full integration in Phase 5)
  4. [x] Create and start container
  5. [ ] Execute agent prompt (infrastructure ready, full integration in Phase 5)
  6. [x] Stream logs to database
  7. [x] Collect artifacts
  8. [x] Update execution status
  9. [x] Cleanup resources

#### 4.4 Database Storage Layer
- [x] Create `packages/sandboxes/src/storage.rs`:
  - [x] `ExecutionStorage` struct
  - [x] CRUD operations for logs and artifacts
  - [x] Log batch insertion
  - [x] Artifact tracking
  - [x] Status updates

- [x] Implement queries:
  - [x] Insert logs with sequence numbers
  - [x] Query logs by execution (with pagination)
  - [x] Search logs with filters
  - [x] Store artifact metadata
  - [x] Update container status
  - [x] Update execution status
  - [x] Update resource usage

#### 4.5 Testing
- [ ] API integration tests (deferred to Phase 7):
  - [ ] Test full execution flow
  - [ ] Test concurrent executions
  - [ ] Test error scenarios
  - [ ] Test resource limits
  - [ ] Test authentication

- [ ] Load testing (deferred to Phase 7):
  - [ ] Test with 10 concurrent executions
  - [ ] Measure API response times
  - [ ] Check database performance
  - [ ] Verify cleanup works under load

### Deliverables
- ✅ Storage layer implemented (logs and artifacts)
- ✅ Execution orchestrator implemented with lifecycle management
- ✅ REST API handlers implemented (stop, retry placeholder, logs, artifacts)
- ✅ Router configured with sandbox execution routes
- ⏸️ Full integration testing deferred to Phase 7
- ⏸️ SSE streaming placeholder - full implementation in Phase 5

---

## Phase 5: Real-time Log Streaming with SSE (Week 3) ✅

### Goals
Implement Server-Sent Events for real-time log streaming from executions.

### Tasks

#### 5.1 Implement SSE Endpoint
- [x] Implemented `packages/api/src/sandbox_execution_handlers.rs`:
  - [x] `stream_execution_logs()` SSE handler with full implementation
  - [x] Set correct SSE headers via create_sse_response helper
  - [x] Query existing logs from database with lastSequence support
  - [x] Subscribe to ExecutionOrchestrator broadcast channel for new log entries
  - [x] Format as SSE events with proper event types
  - [x] Handle client disconnection via GuardedSseStream (RAII cleanup)

- [x] SSE event types implemented in `packages/sandboxes/src/types.rs`:
  - [x] `log` - New log entry with full LogEntry data
  - [x] `status` - Execution status change
  - [x] `container_status` - Container status change
  - [x] `resource_usage` - CPU and memory usage updates
  - [x] `complete` - Execution completed with success flag
  - [x] `heartbeat` - Keep-alive events
  - [x] `sync` - Client lag recovery events

#### 5.2 Log Ingestion Pipeline
- [x] Enhanced `packages/sandboxes/src/execution.rs`:
  - [x] Stream logs from Docker via bollard (already in Phase 4)
  - [x] Parse Docker log format (already in Phase 4)
  - [x] Extract log level and metadata (already in Phase 4)
  - [x] Assign sequence numbers (already in Phase 4)
  - [x] Broadcast log events in real-time via ExecutionEvent::Log

- [x] Log streaming infrastructure:
  - [x] Logs inserted to database and broadcast simultaneously
  - [x] Configurable event channel size (ORKEE_EXECUTION_EVENT_CHANNEL_SIZE)
  - [x] Handle backpressure via tokio broadcast channel
  - [x] Error recovery with lag detection and sync events

#### 5.3 SSE Infrastructure
- [x] Created `packages/api/src/sse.rs` with reusable components:
  - [x] SseConnectionTracker - Per-IP connection rate limiting (configurable via ORKEE_SSE_MAX_CONNECTIONS_PER_IP)
  - [x] SseConnectionGuard - RAII cleanup pattern
  - [x] GuardedSseStream - Guaranteed cleanup even if stream drops
  - [x] create_sse_response() helper with standard keep-alive settings
  - [x] Comprehensive error handling and logging

- [x] Event broadcasting infrastructure:
  - [x] tokio::sync::broadcast channel in ExecutionOrchestrator (capacity: 200, configurable)
  - [x] Broadcast to multiple clients simultaneously
  - [x] Handle slow clients with lag detection (sends sync event instead of disconnecting)
  - [x] Buffer overflow protection via tokio broadcast semantics
  - [x] Graceful degradation on serialization errors

#### 5.4 Frontend SSE Client
- [x] Created `packages/dashboard/src/services/execution-stream.ts`:
  - [x] ExecutionStreamClient class with EventSource connection
  - [x] Auto-reconnect logic with exponential backoff (max 5 attempts)
  - [x] Event parsing for all event types (log, status, complete, heartbeat, sync)
  - [x] Comprehensive error handling with callbacks
  - [x] Connection state management (connecting/connected/disconnected/error)

- [x] Created React hook `packages/dashboard/src/hooks/useExecutionLogs.ts`:
  - [x] useExecutionLogs hook for easy component integration
  - [x] Auto-connect on mount with cleanup on unmount
  - [x] State management for logs array, status, completion
  - [x] Callback support for status changes and completion
  - [x] Connection state tracking
  - [x] Manual connect/disconnect/clearLogs controls
  - [x] Comprehensive JSDoc with usage examples

#### 5.5 Testing
- [x] Backend compilation tests passed
- [ ] SSE integration tests (deferred to Phase 7):
  - [ ] Test streaming with real execution
  - [ ] Test reconnection after disconnect
  - [ ] Test multiple concurrent clients
  - [ ] Test log overflow scenarios

- [ ] Performance testing (deferred to Phase 7):
  - [ ] Stream 1000 logs/second
  - [ ] Test with 50 concurrent SSE clients
  - [ ] Measure latency
  - [ ] Check memory usage

### Deliverables
- ✅ SSE endpoint fully implemented with lag handling and error recovery
- ✅ Real-time log streaming infrastructure complete
- ✅ Auto-reconnect functioning with exponential backoff
- ✅ Multiple client support via broadcast channel
- ✅ Frontend React hook ready for UI integration
- ⏸️ Performance testing deferred to Phase 7 (infrastructure ready)

---

## Phase 6: Dashboard UI Components (Week 4)

### Goals
Build comprehensive UI for agent execution in the dashboard.

### Tasks

#### 6.1 Task Detail Page Updates
- [ ] Update `packages/dashboard/src/pages/TaskDetail.tsx`:
  - [ ] Add "Execute with Agent" section
  - [ ] Conditionally show based on task status
  - [ ] Load agent and sandbox options
  - [ ] Handle execution state

- [ ] Agent selector dropdown:
  - [ ] Load agents from `/api/agents` endpoint
  - [ ] Show agent name, description, capabilities
  - [ ] Default to task's assigned agent
  - [ ] Save preference per task

- [ ] Sandbox provider dropdown:
  - [ ] Load providers from `/api/sandboxes`
  - [ ] Currently only "Local Docker"
  - [ ] Show resource limits
  - [ ] Disable if Docker not available

#### 6.2 Execution Modal
- [ ] Create `packages/dashboard/src/components/ExecutionModal.tsx`:
  - [ ] Show selected agent and sandbox
  - [ ] Prompt customization textarea
  - [ ] Resource limit configuration
  - [ ] Environment variables input
  - [ ] Confirmation before start

- [ ] Modal features:
  - [ ] Validate inputs
  - [ ] Show estimated cost (future)
  - [ ] Warning for resource limits
  - [ ] Loading state during creation

#### 6.3 Execution Viewer Component
- [ ] Create `packages/dashboard/src/components/ExecutionViewer.tsx`:
  - [ ] Execution status badge (running/completed/failed)
  - [ ] Resource usage gauges (CPU, memory)
  - [ ] Duration timer
  - [ ] Stop button for running executions
  - [ ] Retry button for failed executions

- [ ] Real-time updates:
  - [ ] Poll execution status every 5 seconds
  - [ ] Update resource usage
  - [ ] Show container status
  - [ ] Handle state transitions

#### 6.4 Log Viewer Component
- [ ] Create `packages/dashboard/src/components/LogViewer.tsx`:
  - [ ] Virtual scrolling for performance
  - [ ] Log level filtering (debug/info/warn/error)
  - [ ] Search functionality
  - [ ] Export logs button
  - [ ] Auto-scroll toggle

- [ ] SSE integration:
  - [ ] Connect to SSE endpoint
  - [ ] Handle reconnection
  - [ ] Show connection status
  - [ ] Buffer logs locally

#### 6.5 Artifact Gallery
- [ ] Create `packages/dashboard/src/components/ArtifactGallery.tsx`:
  - [ ] Grid view of artifacts
  - [ ] Preview images inline
  - [ ] Download links for files
  - [ ] File size and type display
  - [ ] Bulk download option

- [ ] Artifact types:
  - [ ] Code files with syntax highlighting
  - [ ] Images with lightbox
  - [ ] Test reports with formatting
  - [ ] JSON/YAML with tree view

#### 6.6 Execution History
- [ ] Create `packages/dashboard/src/components/ExecutionHistory.tsx`:
  - [ ] List past executions for task
  - [ ] Status indicators
  - [ ] Duration and resource usage
  - [ ] Filter by status/agent/date
  - [ ] Pagination

- [ ] History item actions:
  - [ ] View details
  - [ ] View logs
  - [ ] Download artifacts
  - [ ] Retry execution

#### 6.7 Testing
- [ ] Component unit tests:
  - [ ] Test with mock data
  - [ ] Test user interactions
  - [ ] Test error states
  - [ ] Test loading states

- [ ] E2E tests:
  - [ ] Test full execution flow
  - [ ] Test SSE log streaming
  - [ ] Test artifact download
  - [ ] Test error scenarios

### Deliverables
- ✅ Task detail page updated with execution UI
- ✅ Execution viewer fully functional
- ✅ Real-time log streaming working
- ✅ Artifact gallery complete
- ✅ All UI tests passing

---

## Phase 7: Testing & Documentation (Week 4)

### Goals
Comprehensive testing, documentation, and production readiness.

### Tasks

#### 7.1 Integration Testing
- [ ] End-to-end execution tests:
  - [ ] Create task → Execute → View logs → Download artifacts
  - [ ] Test with different agents
  - [ ] Test resource limits
  - [ ] Test timeout scenarios
  - [ ] Test cleanup after completion

- [ ] Error scenario tests:
  - [ ] Docker daemon unavailable
  - [ ] Container creation failure
  - [ ] Node.js bridge crash
  - [ ] Network interruption
  - [ ] Database errors

#### 7.2 Performance Testing
- [ ] Load testing:
  - [ ] 20 concurrent executions
  - [ ] 100 SSE clients
  - [ ] 10,000 logs per execution
  - [ ] Measure response times
  - [ ] Check memory usage

- [ ] Stress testing:
  - [ ] Max out resource limits
  - [ ] Rapid start/stop cycles
  - [ ] Large artifact uploads
  - [ ] Database connection pool

#### 7.3 Security Testing
- [ ] Container escape testing
- [ ] Resource limit enforcement
- [ ] Authentication bypass attempts
- [ ] SQL injection testing
- [ ] Path traversal testing
- [ ] Secret leakage checks

#### 7.4 Documentation
- [ ] API documentation:
  - [ ] OpenAPI/Swagger spec
  - [ ] Example requests/responses
  - [ ] Error codes
  - [ ] Rate limits

- [ ] User guide:
  - [ ] How to execute tasks
  - [ ] Understanding logs
  - [ ] Downloading artifacts
  - [ ] Troubleshooting

- [ ] Developer documentation:
  - [ ] Architecture overview
  - [ ] Adding new agents
  - [ ] Adding sandbox providers
  - [ ] Contributing guide

#### 7.5 Production Readiness
- [ ] Monitoring setup:
  - [ ] Prometheus metrics
  - [ ] Grafana dashboards
  - [ ] Alert rules
  - [ ] Log aggregation

- [ ] Deployment:
  - [ ] Docker compose configuration
  - [ ] Kubernetes manifests
  - [ ] CI/CD pipeline
  - [ ] Rollback procedure

### Deliverables
- ✅ All tests passing
- ✅ Documentation complete
- ✅ Security review passed
- ✅ Monitoring configured
- ✅ Ready for production

---

## Phase 8: Future Enhancements (Post-MVP)

### Future Features to Consider

#### 8.1 Cloud Sandbox Providers
- [ ] E2B integration
- [ ] Modal.com integration
- [ ] AWS Fargate support
- [ ] Google Cloud Run support
- [ ] Provider selection logic

#### 8.2 Advanced Execution Features
- [ ] Execution templates (common tasks)
- [ ] Scheduled executions (cron)
- [ ] Execution chaining/pipelines
- [ ] Parallel execution groups
- [ ] Conditional execution logic

#### 8.3 Collaboration Features
- [ ] Multi-agent collaboration
- [ ] Human-in-the-loop approval
- [ ] Execution sharing
- [ ] Team quotas
- [ ] Audit logging

#### 8.4 Enhanced Monitoring
- [ ] Execution analytics dashboard
- [ ] Cost tracking and budgets
- [ ] Performance profiling
- [ ] Dependency analysis
- [ ] Success rate tracking

#### 8.5 Developer Experience
- [ ] VS Code extension
- [ ] CLI execution commands
- [ ] Execution API SDK
- [ ] Webhook notifications
- [ ] Slack/Discord integration

---

## Technical Specifications

### API Endpoints
```
POST   /api/projects/:project_id/tasks/:task_id/executions
GET    /api/projects/:project_id/tasks/:task_id/executions
GET    /api/projects/:project_id/tasks/:task_id/executions/:id
POST   /api/projects/:project_id/tasks/:task_id/executions/:id/stop
GET    /api/projects/:project_id/tasks/:task_id/executions/:id/logs
GET    /api/projects/:project_id/tasks/:task_id/executions/:id/logs/stream
GET    /api/projects/:project_id/tasks/:task_id/executions/:id/artifacts
GET    /api/projects/:project_id/tasks/:task_id/executions/:id/artifacts/:artifact_id/download
```

### Database Schema Summary
- Extended `agent_executions` table with Vibekit fields
- New `execution_logs` table for log storage
- New `execution_artifacts` table for outputs
- Indexes for performance
- Foreign key constraints

### Resource Limits (Defaults)
- Memory: 2048 MB
- CPU: 2.0 cores
- Timeout: 3600 seconds (1 hour)
- Max file size: 100 MB
- Max artifacts: 50 per execution
- Max concurrent: 5 per user

### Security Measures
- Container capability dropping
- Network isolation
- Resource quotas
- Secret redaction in logs
- Authentication required
- Rate limiting

---

## Success Criteria

### Functional Requirements
- [ ] Users can execute tasks with AI agents
- [ ] Execution runs in isolated Docker containers
- [ ] Real-time log streaming works
- [ ] Artifacts can be downloaded
- [ ] Resource limits are enforced
- [ ] Cleanup happens automatically

### Non-Functional Requirements
- [ ] Execution starts within 10 seconds
- [ ] Logs stream with < 500ms latency
- [ ] Supports 20 concurrent executions
- [ ] 99% execution success rate
- [ ] Zero container escapes
- [ ] No resource leaks after 24 hours

### Quality Metrics
- [ ] > 80% code coverage
- [ ] All critical paths tested
- [ ] Documentation complete
- [ ] Security review passed
- [ ] Performance targets met

---

## Timeline Summary

### Week 1
- Phase 1: Database & Foundation ✅
- Phase 2: Vibekit Bridge ✅

### Week 2
- Phase 3: Container Management (pending)
- Phase 4: API Endpoints (pending)

### Week 3
- Phase 4: API Endpoints ✅
- Phase 5: SSE Streaming ✅

### Week 4
- Phase 6: Dashboard UI ✅
- Phase 7: Testing & Documentation ✅

### Total Duration
**4 weeks** from start to production ready

---

## Risk Mitigation

### Technical Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| Node.js bridge instability | High | Implement restart logic, health checks |
| Container resource exhaustion | Medium | Enforce strict limits, monitoring |
| Log overflow | Medium | Buffer limits, truncation, rotation |
| Docker daemon failure | High | Graceful degradation, clear errors |
| Database performance | Medium | Indexes, batch operations, cleanup |

### Operational Risks
| Risk | Impact | Mitigation |
|------|--------|------------|
| Runaway costs (future cloud) | High | Quotas, budgets, alerts |
| Security breach | Critical | Container isolation, secret management |
| Data loss | High | Regular backups, transaction logs |
| User confusion | Medium | Clear UI, documentation, tutorials |

---

## Dependencies

### External Dependencies
- Docker Engine (required)
- Node.js 20+ (for bridge)
- Vibekit SDK (npm package)
- bollard (Rust Docker client)

### Internal Dependencies
- Existing agent_executions table
- Task management system
- Authentication system
- SSE infrastructure (from preview servers)

---

## Team & Resources

### Required Skills
- Rust development (Axum, SQLx, bollard)
- TypeScript/Node.js (Vibekit SDK)
- React development (Dashboard UI)
- Docker expertise (Security, networking)
- Database design (SQLite, migrations)

### Estimated Effort
- Backend development: 2 developers × 3 weeks
- Frontend development: 1 developer × 2 weeks
- Testing & documentation: 1 developer × 1 week
- **Total: ~6 developer-weeks**

---

## Approval & Sign-off

### Stakeholders
- [ ] Engineering Lead approval
- [ ] Product Manager approval
- [ ] Security review completed
- [ ] Infrastructure review completed

### Go/No-Go Criteria
- [ ] Docker available on target systems
- [ ] Resource limits acceptable
- [ ] Security measures sufficient
- [ ] Performance targets achievable
- [ ] Timeline acceptable

---

## Appendix

### A. Sample Configuration Files
[Configuration examples provided in plan above]

### B. API Request/Response Examples
[To be added during implementation]

### C. Error Codes
[To be defined during implementation]

### D. Monitoring Metrics
[To be defined during implementation]

---

**Document Version**: 1.0
**Last Updated**: 2024-11-04
**Status**: DRAFT - Awaiting Approval