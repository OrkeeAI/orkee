# VibeKit Integration & Legacy AIService Removal Plan

## Project Overview

This document tracks the complete migration from legacy AIService to modern AI SDK with VibeKit OAuth support.

### High-Level Goals
- Completely remove legacy AIService from Rust codebase
- Migrate all AI operations to use the modern AI SDK proxy
- Integrate VibeKit directly into existing CLI server (no extra port)
- Use Dagger for local sandboxing (expandable to other providers later)
- Support both API keys and OAuth tokens (hybrid approach)

### Timeline Summary
**Total Duration:** 6 weeks
- **Weeks 1-2:** Phase 1 - Remove Legacy AIService
- **Week 2:** Phase 2 - Database Schema
- **Week 3:** Phase 3 - VibeKit Package & Phase 4 - Rust Integration (Part 1)
- **Week 4:** Phase 4 - Rust Integration (Part 2) & Phase 5 - CLI Commands
- **Week 5:** Phase 6 - Dashboard Integration & Phase 7 - Dagger Integration
- **Week 6:** Phase 8 - Testing & Documentation

## Phase Progress Tracker

### Phase Status Overview
- [ ] **Phase 1:** Remove Legacy AIService & Migrate to Proxy _(Weeks 1-2)_
- [ ] **Phase 2:** Database Schema for OAuth Support _(Week 2)_
- [ ] **Phase 3:** VibeKit Package Integration _(Week 3)_
- [ ] **Phase 4:** Rust Integration Layer & API Routes _(Weeks 3-4)_
- [ ] **Phase 5:** CLI Commands _(Week 4)_
- [ ] **Phase 6:** Dashboard Integration _(Weeks 4-5)_
- [ ] **Phase 7:** Dagger Integration _(Week 5)_
- [ ] **Phase 8:** Testing & Documentation _(Week 6)_

---

## Phase 1: Remove Legacy AIService & Migrate to Proxy (Weeks 1-2)

### Phase 1 Status: In Progress - Extended Scope 🔄
**Completion:** 14/32 tasks (44%)

**Summary**: Successfully migrated all originally scoped functions (ai_handlers.rs + ideate package). Extended Phase 1 scope to include 3 additional API handler files discovered during implementation.

### Phase 1 Overview
Completely remove the legacy `AIService` from the Rust codebase and migrate all AI operations to use the existing AI SDK proxy endpoints.

### Phase 1 Tasks

#### 1.1 Migrate API Handlers (`packages/api/src/ai_handlers.rs`) ✅
- [x] `analyze_prd()` - Line 42-98
- [x] `generate_spec()` - Line 112-187
- [x] `suggest_tasks()` - Line 201-276
- [x] `refine_spec()` - Line 290-365
- [x] `validate_completion()` - Line 379-454

#### 1.2 Migrate Ideate Package (`packages/ideate/src/`) ✅
- [x] `prd_generator.rs` - 4 of 6 functions migrated (2 streaming functions skipped - see notes)
  - [x] `generate_complete_prd_with_model()`
  - [x] `generate_section()`
  - [x] `generate_from_session()`
  - [x] `generate_section_with_context()`
  - ⚠️ `regenerate_with_template_stream()` - **SKIPPED** (uses streaming API)
  - ⚠️ `regenerate_with_template()` - **SKIPPED** (uses text generation, not structured)
- [x] `insight_extractor.rs::extract_insights_with_ai()` - **MIGRATED** (signature changed to accept user_id)
- [x] `research_analyzer.rs` - All 5 functions migrated:
  - [x] `analyze_competitor()`
  - [x] `analyze_gaps()`
  - [x] `extract_ui_patterns()`
  - [x] `extract_lessons()`
  - [x] `synthesize_research()`
- [x] `expert_moderator.rs` - All 3 functions migrated:
  - [x] `suggest_experts()`
  - [x] `extract_insights()`
  - [x] `generate_expert_response()`
- [x] `dependency_analyzer.rs::analyze_dependencies()` - **MIGRATED**

#### 1.3 Migrate Additional API Handlers (Extended Scope)
- [ ] `ideate_dependency_handlers.rs` - 5 functions:
  - [ ] `analyze_project_dependencies()`
  - [ ] `analyze_file_dependencies()`
  - [ ] `analyze_code_dependencies()`
  - [ ] `get_dependency_analysis()`
  - [ ] `analyze_dependencies_with_ai()`
- [ ] `ideate_research_handlers.rs` - 5 functions:
  - [ ] `analyze_competitor()`
  - [ ] `analyze_gaps()`
  - [ ] `extract_ui_patterns()`
  - [ ] `extract_lessons()`
  - [ ] `synthesize_research()`
- [ ] `ideate_roundtable_handlers.rs` - 4 functions:
  - [ ] `create_roundtable()`
  - [ ] `generate_expert_response()`
  - [ ] `extract_insights()`
  - [ ] `generate_summary()`

#### 1.4 Clean Up Legacy Code
- [ ] Delete `packages/ai/src/service.rs` (entire legacy AIService implementation)
- [ ] Remove AIService exports from `packages/ai/src/lib.rs`
- [ ] Update `packages/ai/Cargo.toml` dependencies (remove unused async-trait, reqwest if not needed)

**Note**: 2 streaming functions in `prd_generator.rs` intentionally skipped (require different API pattern)

### Migration Strategy

Each function will be updated to use HTTP client calls to our existing AI proxy instead of direct AIService calls.

#### Example Migration Pattern

**Before (Legacy AIService):**
```rust
// packages/api/src/ai_handlers.rs
use orkee_ai::{AIService, AIServiceError};

pub async fn analyze_prd(request: AnalyzePRDRequest) -> Result<PRDAnalysisData> {
    let api_key = get_api_key_from_env_or_db().await?;
    let ai_service = AIService::with_api_key_and_model(api_key, request.model);

    let result = ai_service
        .generate_structured::<PRDAnalysisData>(
            &user_prompt,
            &system_prompt
        )
        .await?;

    Ok(result)
}
```

**After (Using AI Proxy):**
```rust
// packages/api/src/ai_handlers.rs
use reqwest::Client;
use serde_json::json;

pub async fn analyze_prd(request: AnalyzePRDRequest) -> Result<PRDAnalysisData> {
    let client = Client::new();

    // Build request matching Anthropic Messages API format
    let request_body = json!({
        "model": request.model,
        "max_tokens": 64000,
        "temperature": 0.7,
        "messages": [{
            "role": "user",
            "content": user_prompt
        }],
        "system": system_prompt
    });

    // Call our own AI proxy endpoint
    let response = client
        .post("http://localhost:4001/api/ai/anthropic/v1/messages")
        .header("X-User-ID", current_user.id)  // For auth lookup
        .json(&request_body)
        .send()
        .await?;

    // Parse Anthropic response
    let anthropic_response: AnthropicResponse = response.json().await?;

    // Extract JSON from response content
    let text = anthropic_response.content.first()
        .ok_or("No content in response")?
        .text.clone();

    // Parse into our structured type
    let data: PRDAnalysisData = serde_json::from_str(&text)?;
    Ok(data)
}
```

#### 1.3 Clean Up Legacy Code
- [ ] Delete `packages/ai/src/service.rs` (entire file - 487 lines)
- [ ] Remove AIService exports from `packages/ai/src/lib.rs`
- [ ] Remove `async-trait` from `packages/ai/Cargo.toml`
- [ ] Remove direct `reqwest` from `packages/ai/Cargo.toml` (if not needed elsewhere)

#### 1.4 Testing & Verification
- [ ] All 5 API handler endpoints work with proxy
- [ ] All 5 Ideate functions work with proxy
- [ ] Verify no references to `AIService` remain (run grep search)
- [ ] All existing tests pass
- [ ] Manual testing of each migrated endpoint

---

## Phase 2: Database Schema for OAuth Support (Week 2)

### Phase 2 Status: Not Started ⏳
**Completion:** 0/8 tasks

### Phase 2 Overview
Add database tables to support OAuth tokens and sandbox sessions.

### Phase 2 Tasks

#### 2.1 Database Migration
- [ ] Create `packages/projects/migrations/002_oauth_vibekit.sql`
- [ ] Add migration to the system
- [ ] Test migration up
- [ ] Test migration down

#### 2.2 Schema Implementation

```sql
-- OAuth tokens for AI providers (VibeKit Auth)
CREATE TABLE oauth_tokens (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL,
    provider TEXT NOT NULL, -- 'claude', 'openai', 'google', 'grok'
    access_token TEXT NOT NULL, -- Encrypted with ApiKeyEncryption
    refresh_token TEXT, -- Encrypted with ApiKeyEncryption
    expires_at INTEGER NOT NULL, -- Unix timestamp
    token_type TEXT DEFAULT 'Bearer',
    scope TEXT, -- Space-separated scopes
    organization_name TEXT, -- From token (e.g., "joe@ticc.net's Organization")
    account_email TEXT, -- From token (e.g., "joe@ticc.net")
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    updated_at INTEGER NOT NULL DEFAULT (unixepoch()),
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE(user_id, provider)
);

-- Track Dagger sandbox sessions
CREATE TABLE sandbox_sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL,
    project_id TEXT,
    provider TEXT NOT NULL DEFAULT 'dagger', -- 'dagger' now, expandable later
    container_id TEXT NOT NULL, -- Dagger container ID
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'terminated')),
    host_url TEXT, -- Local URL (e.g., localhost:8080)
    working_directory TEXT DEFAULT '/app',
    metadata TEXT, -- JSON blob for provider-specific data
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    last_active_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER, -- Auto-cleanup after this time
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL
);

-- Indexes for performance
CREATE INDEX idx_oauth_tokens_user ON oauth_tokens(user_id);
CREATE INDEX idx_oauth_tokens_provider ON oauth_tokens(provider);
CREATE INDEX idx_oauth_tokens_expires ON oauth_tokens(expires_at);
CREATE INDEX idx_sandbox_sessions_user ON sandbox_sessions(user_id);
CREATE INDEX idx_sandbox_sessions_status ON sandbox_sessions(status);
CREATE INDEX idx_sandbox_sessions_expires ON sandbox_sessions(expires_at);

-- Triggers for updated_at
CREATE TRIGGER oauth_tokens_updated_at
    AFTER UPDATE ON oauth_tokens
    FOR EACH ROW
    BEGIN
        UPDATE oauth_tokens SET updated_at = unixepoch()
        WHERE id = NEW.id;
    END;
```

#### 2.3 Rust Storage Layer
- [ ] Create `packages/projects/src/storage/oauth_tokens.rs`
- [ ] Create `packages/projects/src/storage/sandbox_sessions.rs`
- [ ] Update `packages/projects/src/storage/mod.rs` to export new modules
- [ ] Update `packages/projects/src/lib.rs` with new types

```rust
use crate::security::encryption::ApiKeyEncryption;

pub struct OAuthTokenStorage {
    pool: SqlitePool,
    encryption: ApiKeyEncryption,
}

impl OAuthTokenStorage {
    /// Store encrypted OAuth token
    pub async fn store_token(&self, user_id: &str, provider: &str, token: OAuthToken) -> Result<()> {
        let encrypted_access = self.encryption.encrypt(&token.access_token)?;
        let encrypted_refresh = token.refresh_token
            .as_ref()
            .map(|t| self.encryption.encrypt(t))
            .transpose()?;

        sqlx::query!(
            r#"
            INSERT INTO oauth_tokens (user_id, provider, access_token, refresh_token,
                                     expires_at, scope, organization_name, account_email)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT(user_id, provider)
            DO UPDATE SET
                access_token = excluded.access_token,
                refresh_token = excluded.refresh_token,
                expires_at = excluded.expires_at,
                updated_at = unixepoch()
            "#,
            user_id, provider, encrypted_access, encrypted_refresh,
            token.expires_at, token.scope, token.organization_name, token.account_email
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get decrypted OAuth token
    pub async fn get_token(&self, user_id: &str, provider: &str) -> Result<Option<OAuthToken>> {
        // Implementation
    }

    /// Refresh token if expired
    pub async fn refresh_if_needed(&self, user_id: &str, provider: &str) -> Result<OAuthToken> {
        // Implementation
    }
}
```

---

## Phase 3: VibeKit Package Integration (Week 3)

### Phase 3 Status: Not Started ⏳
**Completion:** 0/11 tasks

### Phase 3 Overview
Create a new TypeScript package that wraps VibeKit SDK for use from Rust.

### Phase 3 Tasks

#### 3.1 Package Setup
- [ ] Create `packages/vibekit/` directory
- [ ] Initialize package.json
- [ ] Configure tsconfig.json
- [ ] Set up build scripts
- [ ] Add to turborepo configuration

#### 3.2 Core Implementation Files
- [ ] Create `src/auth.ts` - OAuth token management
- [ ] Create `src/client.ts` - VibeKit SDK wrapper
- [ ] Create `src/dagger.ts` - Dagger sandbox provider
- [ ] Create `src/bridge.ts` - Rust-Node.js bridge
- [ ] Create `src/types.ts` - TypeScript type definitions
- [ ] Create `src/index.ts` - Main exports

```
packages/vibekit/
├── package.json
├── tsconfig.json
├── .gitignore
├── src/
│   ├── index.ts           # Main exports
│   ├── auth.ts            # OAuth token management
│   ├── client.ts          # VibeKit SDK wrapper
│   ├── dagger.ts          # Dagger sandbox provider
│   ├── bridge.ts          # Rust-Node.js bridge utilities
│   └── types.ts           # TypeScript type definitions
├── dist/                  # Compiled JavaScript (gitignored)
└── README.md
```

### Package Configuration

#### package.json
```json
{
  "name": "@orkee/vibekit",
  "version": "1.0.0",
  "private": true,
  "type": "module",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "scripts": {
    "build": "tsc",
    "dev": "tsc --watch",
    "test": "bun test"
  },
  "dependencies": {
    "@vibe-kit/sdk": "latest",
    "@vibe-kit/auth": "latest",
    "@vibe-kit/dagger": "latest",
    "@dagger.io/dagger": "latest",
    "zod": "^3.22.0"
  },
  "devDependencies": {
    "@types/node": "^20.0.0",
    "typescript": "^5.0.0"
  }
}
```

### Implementation Files

#### auth.ts - OAuth Token Management
```typescript
import { readFile, writeFile } from 'fs/promises';
import { homedir } from 'os';
import { join } from 'path';

export interface OAuthToken {
  token_type: string;
  access_token: string;
  expires_in: number;
  refresh_token?: string;
  scope: string;
  organization?: {
    uuid: string;
    name: string;
  };
  account?: {
    uuid: string;
    email_address: string;
  };
  created_at: number;
}

export class VibeKitAuth {
  private tokenPath: string;

  constructor() {
    this.tokenPath = join(homedir(), '.vibekit', 'claude-oauth-token.json');
  }

  async getToken(): Promise<OAuthToken | null> {
    try {
      const content = await readFile(this.tokenPath, 'utf-8');
      return JSON.parse(content);
    } catch {
      return null;
    }
  }

  async isExpired(token: OAuthToken): Promise<boolean> {
    const expiresAt = token.created_at + (token.expires_in * 1000);
    return Date.now() > expiresAt;
  }

  async refreshToken(token: OAuthToken): Promise<OAuthToken> {
    // Call VibeKit Auth refresh endpoint
    // Return new token
  }
}
```

#### client.ts - VibeKit SDK Wrapper
```typescript
import { VibeKit } from '@vibe-kit/sdk';
import { VibeKitAuth } from './auth';

export class VibeKitClient {
  private auth: VibeKitAuth;
  private sdk?: VibeKit;

  constructor() {
    this.auth = new VibeKitAuth();
  }

  async initialize(): Promise<void> {
    const token = await this.auth.getToken();
    if (!token) {
      throw new Error('No OAuth token found. Run: orkee login claude');
    }

    if (await this.auth.isExpired(token)) {
      const newToken = await this.auth.refreshToken(token);
      // Save new token
    }

    this.sdk = new VibeKit()
      .withAgent({
        type: 'claude',
        provider: 'anthropic',
        providerApiKey: token.access_token,
        model: 'claude-sonnet-4-20250514'
      });
  }

  async execute(prompt: string): Promise<any> {
    if (!this.sdk) await this.initialize();
    return this.sdk!.execute(prompt);
  }
}
```

#### dagger.ts - Dagger Sandbox Provider
```typescript
import Client, { connect } from "@dagger.io/dagger";

export interface SandboxConfig {
  template?: string;
  workdir?: string;
  ports?: number[];
  environment?: Record<string, string>;
}

export class DaggerSandbox {
  private client?: Client;

  async connect(): Promise<void> {
    this.client = await connect();
  }

  async createSandbox(config: SandboxConfig = {}): Promise<{
    id: string;
    ports: Map<number, number>;
    logs: AsyncIterator<string>;
  }> {
    if (!this.client) await this.connect();

    const container = this.client!
      .container()
      .from(config.template || 'node:18-alpine')
      .withWorkdir(config.workdir || '/app');

    // Expose ports
    for (const port of config.ports || [3000]) {
      container.withExposedPort(port);
    }

    // Set environment variables
    for (const [key, value] of Object.entries(config.environment || {})) {
      container.withEnvVariable(key, value);
    }

    const id = await container.id();

    return {
      id,
      ports: new Map([[3000, 8080]]), // Port mapping
      logs: container.stdout()
    };
  }

  async stopSandbox(id: string): Promise<void> {
    // Stop container
  }
}
```

#### bridge.ts - Rust-Node.js Bridge
```typescript
// This file handles communication between Rust and Node.js

export interface BridgeRequest {
  command: 'execute' | 'create_sandbox' | 'stop_sandbox' | 'get_token';
  payload: any;
}

export interface BridgeResponse {
  success: boolean;
  data?: any;
  error?: string;
}

// Main entry point when called from Rust
async function main() {
  const input = process.argv[2];
  if (!input) {
    console.error(JSON.stringify({ success: false, error: 'No input provided' }));
    process.exit(1);
  }

  try {
    const request: BridgeRequest = JSON.parse(input);
    let response: BridgeResponse;

    switch (request.command) {
      case 'execute':
        const client = new VibeKitClient();
        const result = await client.execute(request.payload.prompt);
        response = { success: true, data: result };
        break;

      case 'create_sandbox':
        const sandbox = new DaggerSandbox();
        const session = await sandbox.createSandbox(request.payload);
        response = { success: true, data: session };
        break;

      default:
        response = { success: false, error: `Unknown command: ${request.command}` };
    }

    console.log(JSON.stringify(response));
  } catch (error) {
    console.error(JSON.stringify({
      success: false,
      error: error instanceof Error ? error.message : 'Unknown error'
    }));
    process.exit(1);
  }
}

if (require.main === module) {
  main();
}
```

---

## Phase 4: Rust Integration Layer & API Routes (Weeks 3-4)

### Phase 4 Status: Not Started ⏳
**Completion:** 0/14 tasks

### Phase 4 Overview
Integrate the VibeKit TypeScript package into the Rust CLI server.

### Phase 4 Tasks

#### 4.1 Rust VibeKit Module
- [ ] Create `packages/cli/src/vibekit/mod.rs`
- [ ] Add vibekit module to `packages/cli/src/lib.rs`
- [ ] Implement VibeKitBridge struct
- [ ] Implement execute method
- [ ] Implement create_sandbox method
- [ ] Add error handling

#### 4.2 API Route Handlers
- [ ] Create `packages/api/src/vibekit_handlers.rs`
- [ ] Implement `execute_handler`
- [ ] Implement `create_sandbox_handler`
- [ ] Implement `get_sandbox_handler`
- [ ] Implement `delete_sandbox_handler`
- [ ] Implement `auth_status_handler`
- [ ] Implement `auth_refresh_handler`
- [ ] Add routes to main router in `packages/api/src/lib.rs`

```rust
use std::path::PathBuf;
use std::process::Command;
use serde::{Serialize, Deserialize};
use anyhow::Result;

#[derive(Serialize)]
struct BridgeRequest {
    command: String,
    payload: serde_json::Value,
}

#[derive(Deserialize)]
struct BridgeResponse {
    success: bool,
    data: Option<serde_json::Value>,
    error: Option<String>,
}

pub struct VibeKitBridge {
    node_script_path: PathBuf,
}

impl VibeKitBridge {
    pub fn new() -> Result<Self> {
        // Find compiled vibekit bridge script
        let node_script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../vibekit/dist/bridge.js");

        if !node_script_path.exists() {
            anyhow::bail!("VibeKit bridge not built. Run: cd packages/vibekit && bun run build");
        }

        Ok(Self { node_script_path })
    }

    async fn call_bridge(&self, request: BridgeRequest) -> Result<BridgeResponse> {
        let input = serde_json::to_string(&request)?;

        let output = tokio::process::Command::new("node")
            .arg(&self.node_script_path)
            .arg(input)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("VibeKit bridge failed: {}", stderr);
        }

        let response: BridgeResponse = serde_json::from_slice(&output.stdout)?;

        if !response.success {
            anyhow::bail!("VibeKit error: {}", response.error.unwrap_or_default());
        }

        Ok(response)
    }

    pub async fn execute(&self, prompt: String, model: String) -> Result<serde_json::Value> {
        let request = BridgeRequest {
            command: "execute".to_string(),
            payload: serde_json::json!({
                "prompt": prompt,
                "model": model
            }),
        };

        let response = self.call_bridge(request).await?;
        Ok(response.data.unwrap_or_default())
    }

    pub async fn create_sandbox(&self, project_id: String, template: String) -> Result<String> {
        let request = BridgeRequest {
            command: "create_sandbox".to_string(),
            payload: serde_json::json!({
                "projectId": project_id,
                "template": template
            }),
        };

        let response = self.call_bridge(request).await?;
        let data = response.data.unwrap_or_default();

        Ok(data["id"].as_str().unwrap_or_default().to_string())
    }
}
```

### API Route Handlers

Create `packages/api/src/vibekit_handlers.rs`:

- [ ] Create handlers file
- [ ] Implement execute endpoint
- [ ] Implement sandbox endpoints
- [ ] Add to router

```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use crate::vibekit::VibeKitBridge;

#[derive(Deserialize)]
pub struct ExecuteRequest {
    pub code: String,
    pub language: String,
    pub provider: String, // 'claude', 'openai', etc.
}

#[derive(Serialize)]
pub struct ExecuteResponse {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

pub async fn execute_handler(
    State(state): State<AppState>,
    Json(request): Json<ExecuteRequest>,
) -> impl IntoResponse {
    let bridge = match VibeKitBridge::new() {
        Ok(b) => b,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecuteResponse {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                })
            );
        }
    };

    match bridge.execute(request.code, request.provider).await {
        Ok(output) => {
            (
                StatusCode::OK,
                Json(ExecuteResponse {
                    success: true,
                    output: Some(output.to_string()),
                    error: None,
                })
            )
        }
        Err(e) => {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ExecuteResponse {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                })
            )
        }
    }
}

#[derive(Deserialize)]
pub struct CreateSandboxRequest {
    pub project_id: Option<String>,
    pub template: Option<String>,
}

pub async fn create_sandbox_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateSandboxRequest>,
) -> impl IntoResponse {
    // Implementation
}

pub async fn get_sandbox_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Implementation
}

pub async fn delete_sandbox_handler(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    // Implementation
}
```

### Router Updates

Update `packages/api/src/lib.rs`:

- [ ] Import vibekit handlers
- [ ] Add routes to router

```rust
use crate::vibekit_handlers::{
    execute_handler,
    create_sandbox_handler,
    get_sandbox_handler,
    delete_sandbox_handler,
};

pub fn create_router(db: DbState) -> Router {
    Router::new()
        // ... existing routes ...

        // VibeKit execution
        .route("/api/execute", post(execute_handler))

        // Sandbox management
        .route("/api/sandbox", post(create_sandbox_handler))
        .route("/api/sandbox/:id", get(get_sandbox_handler))
        .route("/api/sandbox/:id", delete(delete_sandbox_handler))

        // OAuth status
        .route("/api/auth/status", get(auth_status_handler))
        .route("/api/auth/refresh/:provider", post(auth_refresh_handler))
        .route("/api/auth/:provider", delete(auth_logout_handler))

        .with_state(db)
}
```

---

## Phase 5: CLI Commands (Week 4)

### Phase 5 Status: Not Started ⏳
**Completion:** 0/12 tasks

### Phase 5 Overview
Add intuitive CLI commands for authentication and sandbox management.

### Phase 5 Tasks

#### 5.1 Authentication Commands
- [ ] Create `packages/cli/src/bin/cli/auth.rs`
- [ ] Implement `handle_login()` function
- [ ] Implement `handle_logout()` function
- [ ] Implement `handle_status()` function
- [ ] Add provider-specific login functions (claude, openai, google, grok)

#### 5.2 Sandbox Commands
- [ ] Create `packages/cli/src/bin/cli/sandbox.rs`
- [ ] Implement list command
- [ ] Implement create command
- [ ] Implement stop command
- [ ] Implement logs command
- [ ] Implement clean command

#### 5.3 CLI Integration
- [ ] Update `packages/cli/src/bin/orkee.rs` with new commands

#### auth.rs - Authentication Commands
Create `packages/cli/src/bin/cli/auth.rs`:

- [ ] Create auth module
- [ ] Implement login command
- [ ] Implement logout command
- [ ] Implement status command

```rust
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum AuthCommand {
    /// Show authentication status for all providers
    Status,
}

pub async fn handle_login(provider: &str) -> Result<()> {
    match provider {
        "claude" => login_claude().await,
        "openai" => login_openai().await,
        "google" => login_google().await,
        "grok" => login_grok().await,
        _ => anyhow::bail!("Unknown provider: {}", provider),
    }
}

async fn login_claude() -> Result<()> {
    println!("🔐 Authenticating with Claude (Anthropic)...\n");

    // Check for existing token
    let token_path = dirs::home_dir()
        .unwrap()
        .join(".vibekit")
        .join("claude-oauth-token.json");

    if token_path.exists() {
        println!("✅ Found existing OAuth token");

        // Read and validate token
        let content = std::fs::read_to_string(&token_path)?;
        let token: serde_json::Value = serde_json::from_str(&content)?;

        if let Some(account) = token.get("account") {
            if let Some(email) = account.get("email_address").and_then(|e| e.as_str()) {
                println!("   Email: {}", email);
            }
        }

        // Import to database
        import_oauth_token("claude", token).await?;
        println!("\n✅ Token imported successfully!");
    } else {
        println!("No existing token found.\n");
        println!("To authenticate with Claude:");
        println!("1. Install VibeKit CLI: npm install -g @vibe-kit/cli");
        println!("2. Run: vibekit auth login");
        println!("3. Complete OAuth flow in browser");
        println!("4. Run this command again");

        // Optionally, try to run vibekit auth login for them
        if prompt_yes_no("Would you like me to run 'vibekit auth login' now?")? {
            std::process::Command::new("vibekit")
                .args(["auth", "login"])
                .status()?;

            // After login, try again
            return login_claude().await;
        }
    }

    Ok(())
}

pub async fn handle_logout(provider: &str) -> Result<()> {
    match provider {
        "all" => {
            // Remove all OAuth tokens from database
            remove_all_oauth_tokens().await?;
            println!("✅ Logged out from all providers");
        }
        provider => {
            // Remove specific provider token
            remove_oauth_token(provider).await?;
            println!("✅ Logged out from {}", provider);
        }
    }

    Ok(())
}

pub async fn handle_status() -> Result<()> {
    println!("🔐 Authentication Status\n");
    println!("Provider    | Status         | Account");
    println!("------------|----------------|--------------------------------");

    // Check each provider
    for provider in &["claude", "openai", "google", "grok"] {
        let status = get_oauth_status(provider).await?;

        let status_text = if status.authenticated {
            "✅ Authenticated"
        } else {
            "❌ Not logged in"
        };

        let account = status.email.unwrap_or_else(|| "-".to_string());

        println!("{:<11} | {:<14} | {}", provider, status_text, account);
    }

    Ok(())
}
```

#### sandbox.rs - Sandbox Commands
Create `packages/cli/src/bin/cli/sandbox.rs`:

- [ ] Create sandbox module
- [ ] Implement list command
- [ ] Implement create command
- [ ] Implement stop command
- [ ] Implement logs command

```rust
use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum SandboxCommand {
    /// List active sandboxes
    #[command(visible_alias = "ls")]
    List,

    /// Create a new sandbox
    Create {
        /// Project ID to associate with sandbox
        #[arg(long)]
        project: Option<String>,

        /// Container template (default: node:18)
        #[arg(long, default_value = "node:18")]
        template: String,
    },

    /// Show sandbox logs
    Logs {
        /// Sandbox ID
        id: String,
    },

    /// Stop a sandbox
    Stop {
        /// Sandbox ID
        id: String,
    },

    /// Stop all sandboxes
    Clean,
}

pub async fn handle_sandbox(cmd: SandboxCommand) -> Result<()> {
    match cmd {
        SandboxCommand::List => list_sandboxes().await,
        SandboxCommand::Create { project, template } => {
            create_sandbox(project, template).await
        }
        SandboxCommand::Logs { id } => show_logs(&id).await,
        SandboxCommand::Stop { id } => stop_sandbox(&id).await,
        SandboxCommand::Clean => clean_all_sandboxes().await,
    }
}

async fn list_sandboxes() -> Result<()> {
    println!("📦 Active Sandboxes\n");

    let sandboxes = get_active_sandboxes().await?;

    if sandboxes.is_empty() {
        println!("No active sandboxes");
        return Ok(());
    }

    println!("ID          | Status | Project         | URL");
    println!("------------|--------|-----------------|--------------------");

    for sandbox in sandboxes {
        println!(
            "{:<11} | {:<6} | {:<15} | {}",
            &sandbox.id[..11],
            sandbox.status,
            sandbox.project_name.unwrap_or_else(|| "-".to_string()),
            sandbox.host_url.unwrap_or_else(|| "-".to_string())
        );
    }

    Ok(())
}

async fn create_sandbox(project: Option<String>, template: String) -> Result<()> {
    println!("🚀 Creating sandbox with template: {}", template);

    let sandbox_id = call_create_sandbox_api(project, template).await?;

    println!("✅ Sandbox created: {}", sandbox_id);
    println!("   Access at: http://localhost:8080");
    println!("\nUseful commands:");
    println!("  orkee sandbox logs {}    # View logs", &sandbox_id[..8]);
    println!("  orkee sandbox stop {}    # Stop sandbox", &sandbox_id[..8]);

    Ok(())
}
```

### Update Main CLI

Update `packages/cli/src/bin/orkee.rs`:

- [ ] Add login/logout commands
- [ ] Add sandbox command
- [ ] Update help text

```rust
#[derive(Parser)]
#[command(name = "orkee")]
#[command(about = "AI-powered project orchestration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    // ... existing commands ...

    /// Authenticate with AI providers using OAuth
    Login {
        /// Provider to authenticate with (claude, openai, google, grok)
        provider: String,
    },

    /// Remove OAuth authentication
    Logout {
        /// Provider to logout from (claude, openai, google, grok, all)
        provider: String,
    },

    /// Manage sandbox environments
    Sandbox {
        #[command(subcommand)]
        command: sandbox::SandboxCommand,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Login { provider } => {
            if provider == "status" {
                auth::handle_status().await
            } else {
                auth::handle_login(&provider).await
            }
        }
        Commands::Logout { provider } => auth::handle_logout(&provider).await,
        Commands::Sandbox { command } => sandbox::handle_sandbox(command).await,
        // ... existing commands ...
    }
}
```

---

## Phase 6: Dashboard Integration (Weeks 4-5)

### Phase 6 Status: Not Started ⏳
**Completion:** 0/10 tasks

### Phase 6 Overview
Integrate VibeKit SDK into the React dashboard.

### Phase 6 Tasks

#### 6.1 Dependencies & Setup
- [ ] Add VibeKit dependencies to `packages/dashboard/package.json`
- [ ] Run `bun install`

#### 6.2 OAuth Settings Component
- [ ] Create `packages/dashboard/src/pages/Settings/OAuth.tsx`
- [ ] Implement auth status display
- [ ] Add login/logout functionality

#### 6.3 Sandbox Manager Component
- [ ] Create `packages/dashboard/src/components/SandboxManager.tsx`
- [ ] Implement sandbox list view
- [ ] Add create/stop controls

#### 6.4 Context & Integration
- [ ] Create `packages/dashboard/src/contexts/VibeKitContext.tsx`
- [ ] Update Settings page to include OAuth tab

```json
{
  "dependencies": {
    // ... existing ...
    "@vibe-kit/sdk": "latest",
    "@vibe-kit/auth": "latest",
    "@vibe-kit/dagger": "latest",
    "@dagger.io/dagger": "latest"
  }
}
```

### New Components

#### OAuth Settings Page
Create `packages/dashboard/src/pages/Settings/OAuth.tsx`:

- [ ] Create OAuth settings component
- [ ] Show auth status for each provider
- [ ] Add login/logout buttons

```tsx
import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';

interface AuthStatus {
  claude?: { authenticated: boolean; email?: string };
  openai?: { authenticated: boolean; email?: string };
  google?: { authenticated: boolean; email?: string };
  grok?: { authenticated: boolean; email?: string };
}

export function OAuthSettings() {
  const [authStatus, setAuthStatus] = useState<AuthStatus>({});
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetchAuthStatus();
  }, []);

  const fetchAuthStatus = async () => {
    try {
      const response = await fetch('/api/auth/status');
      const data = await response.json();
      setAuthStatus(data);
    } finally {
      setLoading(false);
    }
  };

  const handleLogin = async (provider: string) => {
    // Open OAuth flow or show instructions
    if (provider === 'claude') {
      alert('Run in terminal: orkee login claude');
      // Or trigger OAuth flow via API
    }
  };

  const handleLogout = async (provider: string) => {
    await fetch(`/api/auth/${provider}`, { method: 'DELETE' });
    await fetchAuthStatus();
  };

  const providers = [
    { id: 'claude', name: 'Claude (Anthropic)', icon: '🤖' },
    { id: 'openai', name: 'OpenAI', icon: '🧠' },
    { id: 'google', name: 'Google Gemini', icon: '🔷' },
    { id: 'grok', name: 'Grok (xAI)', icon: '⚡' },
  ];

  return (
    <div className="space-y-6">
      <div>
        <h3 className="text-lg font-medium">OAuth Authentication</h3>
        <p className="text-sm text-muted-foreground">
          Connect your AI provider subscriptions to use them instead of API keys
        </p>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        {providers.map(provider => {
          const status = authStatus[provider.id];
          const isAuthenticated = status?.authenticated || false;

          return (
            <Card key={provider.id}>
              <CardHeader>
                <CardTitle className="flex items-center justify-between">
                  <span>{provider.icon} {provider.name}</span>
                  {isAuthenticated ? (
                    <Badge variant="success">Connected</Badge>
                  ) : (
                    <Badge variant="secondary">Not connected</Badge>
                  )}
                </CardTitle>
                {isAuthenticated && status?.email && (
                  <CardDescription>{status.email}</CardDescription>
                )}
              </CardHeader>
              <CardContent>
                {isAuthenticated ? (
                  <Button
                    variant="destructive"
                    size="sm"
                    onClick={() => handleLogout(provider.id)}
                  >
                    Disconnect
                  </Button>
                ) : (
                  <Button
                    variant="default"
                    size="sm"
                    onClick={() => handleLogin(provider.id)}
                  >
                    Connect with OAuth
                  </Button>
                )}
              </CardContent>
            </Card>
          );
        })}
      </div>
    </div>
  );
}
```

#### Sandbox Manager Component
Create `packages/dashboard/src/components/SandboxManager.tsx`:

- [ ] Create sandbox manager component
- [ ] Show active sandboxes
- [ ] Add create/stop controls

```tsx
import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';
import { Select } from '@/components/ui/select';

interface Sandbox {
  id: string;
  status: 'active' | 'paused' | 'terminated';
  projectId?: string;
  hostUrl?: string;
  createdAt: number;
}

export function SandboxManager() {
  const [sandboxes, setSandboxes] = useState<Sandbox[]>([]);
  const [creating, setCreating] = useState(false);

  const createSandbox = async (template: string = 'node:18') => {
    setCreating(true);
    try {
      const response = await fetch('/api/sandbox', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ template }),
      });

      const data = await response.json();
      if (data.success) {
        await fetchSandboxes();
      }
    } finally {
      setCreating(false);
    }
  };

  const stopSandbox = async (id: string) => {
    await fetch(`/api/sandbox/${id}`, { method: 'DELETE' });
    await fetchSandboxes();
  };

  const fetchSandboxes = async () => {
    const response = await fetch('/api/sandbox');
    const data = await response.json();
    setSandboxes(data.sandboxes || []);
  };

  useEffect(() => {
    fetchSandboxes();
    const interval = setInterval(fetchSandboxes, 5000); // Poll every 5s
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="space-y-4">
      <div className="flex justify-between items-center">
        <h3 className="text-lg font-medium">Sandbox Environments</h3>
        <Button onClick={() => createSandbox()} disabled={creating}>
          {creating ? 'Creating...' : 'New Sandbox'}
        </Button>
      </div>

      {sandboxes.length === 0 ? (
        <Card className="p-6 text-center text-muted-foreground">
          No active sandboxes
        </Card>
      ) : (
        <div className="space-y-2">
          {sandboxes.map(sandbox => (
            <Card key={sandbox.id} className="p-4">
              <div className="flex justify-between items-center">
                <div>
                  <div className="font-mono text-sm">{sandbox.id.slice(0, 8)}</div>
                  {sandbox.hostUrl && (
                    <a
                      href={sandbox.hostUrl}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="text-blue-500 hover:underline text-sm"
                    >
                      {sandbox.hostUrl}
                    </a>
                  )}
                </div>
                <div className="flex items-center gap-2">
                  <Badge variant={
                    sandbox.status === 'active' ? 'success' : 'secondary'
                  }>
                    {sandbox.status}
                  </Badge>
                  <Button
                    size="sm"
                    variant="destructive"
                    onClick={() => stopSandbox(sandbox.id)}
                  >
                    Stop
                  </Button>
                </div>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
```

#### VibeKit Context
Create `packages/dashboard/src/contexts/VibeKitContext.tsx`:

- [ ] Create context provider
- [ ] Manage VibeKit client instance
- [ ] Handle authentication state

```tsx
import React, { createContext, useContext, useState, useEffect } from 'react';

interface VibeKitContextValue {
  isAuthenticated: boolean;
  authStatus: Record<string, any>;
  execute: (code: string, provider: string) => Promise<any>;
  createSandbox: (template?: string) => Promise<string>;
}

const VibeKitContext = createContext<VibeKitContextValue | null>(null);

export function VibeKitProvider({ children }: { children: React.ReactNode }) {
  const [authStatus, setAuthStatus] = useState({});
  const [isAuthenticated, setIsAuthenticated] = useState(false);

  useEffect(() => {
    checkAuthStatus();
  }, []);

  const checkAuthStatus = async () => {
    const response = await fetch('/api/auth/status');
    const status = await response.json();
    setAuthStatus(status);

    // Check if any provider is authenticated
    const hasAuth = Object.values(status).some(
      (s: any) => s.authenticated
    );
    setIsAuthenticated(hasAuth);
  };

  const execute = async (code: string, provider: string = 'claude') => {
    const response = await fetch('/api/execute', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ code, provider }),
    });

    return response.json();
  };

  const createSandbox = async (template: string = 'node:18') => {
    const response = await fetch('/api/sandbox', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ template }),
    });

    const data = await response.json();
    return data.id;
  };

  return (
    <VibeKitContext.Provider value={{
      isAuthenticated,
      authStatus,
      execute,
      createSandbox,
    }}>
      {children}
    </VibeKitContext.Provider>
  );
}

export const useVibeKit = () => {
  const context = useContext(VibeKitContext);
  if (!context) {
    throw new Error('useVibeKit must be used within VibeKitProvider');
  }
  return context;
};
```

### Update Existing Pages

#### Settings Page Integration
Update `packages/dashboard/src/pages/Settings/index.tsx`:

- [ ] Add OAuth tab
- [ ] Include OAuth settings component

#### Ideate Page Enhancement
Update `packages/dashboard/src/pages/Ideate/index.tsx`:

- [ ] Add "Preview in Sandbox" button
- [ ] Show sandbox iframe

---

## Phase 7: Dagger Integration (Week 5)

### Phase 7 Status: Not Started ⏳
**Completion:** 0/9 tasks

### Phase 7 Overview
Implement Dagger for local sandbox execution.

### Phase 7 Tasks

#### 7.1 Prerequisites
- [ ] Install Docker (required by Dagger)
- [ ] Install Dagger CLI (optional, for debugging)

#### 7.2 Dagger Implementation
- [ ] Complete `packages/vibekit/src/dagger.ts` implementation
- [ ] Add container lifecycle management
- [ ] Implement port forwarding
- [ ] Add log streaming

#### 7.3 Database Integration
- [ ] Update sandbox session storage
- [ ] Implement auto-cleanup task
- [ ] Add background cleanup scheduler

#### Implementation Details

Update `packages/vibekit/src/dagger.ts`:

- [ ] Complete Dagger client implementation
- [ ] Add container lifecycle management
- [ ] Implement port forwarding
- [ ] Add log streaming

```typescript
import Client, { connect, Container } from "@dagger.io/dagger";
import { EventEmitter } from 'events';

export class DaggerSandboxManager extends EventEmitter {
  private client?: Client;
  private containers: Map<string, Container> = new Map();

  async initialize(): Promise<void> {
    // Connect to Dagger engine
    this.client = await connect({
      LogOutput: process.stderr, // For debugging
    });
  }

  async createSandbox(options: {
    template?: string;
    projectPath?: string;
    ports?: number[];
  }): Promise<{
    id: string;
    ports: Map<number, number>;
    url: string;
  }> {
    if (!this.client) await this.initialize();

    // Create container
    let container = this.client!
      .container()
      .from(options.template || 'node:18-alpine')
      .withWorkdir('/app');

    // Mount project directory if provided
    if (options.projectPath) {
      const projectDir = this.client!.host().directory(options.projectPath);
      container = container.withMountedDirectory('/app', projectDir);
    }

    // Expose ports
    const portMappings = new Map<number, number>();
    for (const port of options.ports || [3000]) {
      container = container.withExposedPort(port);
      // Dagger will assign a random host port
      const hostPort = 8000 + Math.floor(Math.random() * 1000);
      portMappings.set(port, hostPort);
    }

    // Start container
    const id = await container.id();
    this.containers.set(id, container);

    // Get the first port for URL
    const primaryPort = portMappings.values().next().value || 8080;

    return {
      id,
      ports: portMappings,
      url: `http://localhost:${primaryPort}`,
    };
  }

  async stopSandbox(id: string): Promise<void> {
    const container = this.containers.get(id);
    if (!container) {
      throw new Error(`Sandbox ${id} not found`);
    }

    // Dagger containers are ephemeral, just remove from map
    this.containers.delete(id);
  }

  async getLogs(id: string): AsyncIterator<string> {
    const container = this.containers.get(id);
    if (!container) {
      throw new Error(`Sandbox ${id} not found`);
    }

    return container.stdout();
  }

  async cleanup(): Promise<void> {
    // Clean up all containers
    this.containers.clear();

    // Disconnect from Dagger
    if (this.client) {
      await this.client.close();
      this.client = undefined;
    }
  }
}
```

### Database Integration

Update sandbox session storage:

- [ ] Store Dagger container IDs
- [ ] Track port mappings
- [ ] Implement auto-cleanup

```rust
// packages/projects/src/storage/sandbox_sessions.rs

impl SandboxSessionStorage {
    pub async fn create_session(&self,
        user_id: &str,
        container_id: &str,
        port_mappings: HashMap<u16, u16>
    ) -> Result<String> {
        let id = generate_id();
        let metadata = serde_json::json!({
            "ports": port_mappings,
            "provider": "dagger",
        });

        sqlx::query!(
            r#"
            INSERT INTO sandbox_sessions
                (id, user_id, provider, container_id, status, metadata, expires_at)
            VALUES (?, ?, 'dagger', ?, 'active', ?, ?)
            "#,
            id, user_id, container_id,
            serde_json::to_string(&metadata)?,
            Utc::now().timestamp() + 3600 // 1 hour expiry
        )
        .execute(&self.pool)
        .await?;

        Ok(id)
    }

    pub async fn cleanup_expired(&self) -> Result<u32> {
        let result = sqlx::query!(
            r#"
            UPDATE sandbox_sessions
            SET status = 'terminated'
            WHERE status = 'active'
              AND expires_at < unixepoch()
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() as u32)
    }
}
```

### Background Cleanup Task

Add to `packages/cli/src/tasks/mod.rs`:

- [ ] Create cleanup task
- [ ] Schedule every 5 minutes
- [ ] Stop expired containers

```rust
use tokio::time::{interval, Duration};

pub async fn start_sandbox_cleanup_task(storage: SandboxSessionStorage) {
    let mut interval = interval(Duration::from_secs(300)); // 5 minutes

    loop {
        interval.tick().await;

        match storage.cleanup_expired().await {
            Ok(count) if count > 0 => {
                tracing::info!("Cleaned up {} expired sandboxes", count);
            }
            Err(e) => {
                tracing::error!("Failed to cleanup sandboxes: {}", e);
            }
            _ => {}
        }
    }
}
```

---

## Phase 8: Testing & Documentation (Week 6)

### Phase 8 Status: Not Started ⏳
**Completion:** 0/20 tasks

### Phase 8 Overview
Comprehensive testing and documentation updates.

### Phase 8 Tasks

#### 8.1 Unit Tests - Rust
- [ ] Create `packages/cli/src/vibekit/tests.rs`
- [ ] Create `packages/api/src/vibekit_handlers_test.rs`
- [ ] Create `packages/projects/src/storage/oauth_tokens_test.rs`
- [ ] Write tests for OAuth token storage
- [ ] Write tests for sandbox session management

Example test:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_vibekit_bridge_initialization() {
        let bridge = VibeKitBridge::new();
        assert!(bridge.is_ok());
    }

    #[tokio::test]
    async fn test_oauth_token_storage() {
        let storage = setup_test_storage().await;

        let token = OAuthToken {
            access_token: "test-token".to_string(),
            expires_at: 1234567890,
            // ...
        };

        storage.store_token("user-1", "claude", token).await.unwrap();

        let retrieved = storage.get_token("user-1", "claude").await.unwrap();
        assert!(retrieved.is_some());
    }
}
```

#### 8.2 Unit Tests - TypeScript
- [ ] Create `packages/vibekit/src/auth.test.ts`
- [ ] Create `packages/vibekit/src/dagger.test.ts`
- [ ] Create `packages/dashboard/src/services/vibekit.test.ts`
- [ ] Write tests for OAuth token detection
- [ ] Write tests for Dagger sandbox creation

Example test:
```typescript
import { describe, it, expect } from 'bun:test';
import { VibeKitAuth } from './auth';

describe('VibeKitAuth', () => {
  it('should detect token file', async () => {
    const auth = new VibeKitAuth();
    const token = await auth.getToken();

    // This will pass if token exists, fail if not
    // In CI, we'd mock this
    expect(token).toBeDefined();
  });

  it('should detect expired tokens', async () => {
    const auth = new VibeKitAuth();
    const expiredToken = {
      created_at: Date.now() - 86400000, // Yesterday
      expires_in: 3600, // 1 hour
    };

    const isExpired = await auth.isExpired(expiredToken);
    expect(isExpired).toBe(true);
  });
});
```

#### 8.3 Integration Tests
- [ ] Create `packages/cli/tests/integration/vibekit_integration_test.rs`
- [ ] Test full OAuth flow
- [ ] Test sandbox creation and deletion
- [ ] Test execute endpoint end-to-end
- [ ] Test all migrated legacy endpoints

```rust
#[tokio::test]
async fn test_oauth_flow_integration() {
    // Start test server
    let app = create_test_app().await;

    // Check initial status
    let response = app
        .oneshot(Request::get("/api/auth/status").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    // Parse response
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let status: serde_json::Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(status["claude"]["authenticated"], false);
}
```

#### Manual Testing Checklist

##### Legacy AIService Removal
- [ ] All 10 endpoints work with new proxy implementation
- [ ] No `AIService` imports remain
- [ ] Performance is comparable or better

##### OAuth Authentication
- [ ] `orkee login claude` successfully imports token
- [ ] `orkee login status` shows correct status
- [ ] `orkee logout claude` removes token
- [ ] Token refresh works when expired
- [ ] Dashboard shows OAuth status correctly

##### Sandbox Functionality
- [ ] `orkee sandbox create` creates Dagger container
- [ ] Can access sandbox via localhost URL
- [ ] `orkee sandbox logs` shows container output
- [ ] `orkee sandbox stop` terminates container
- [ ] Auto-cleanup works after 1 hour

##### API Endpoints
- [ ] POST `/api/execute` executes code
- [ ] POST `/api/sandbox` creates sandbox
- [ ] GET `/api/sandbox/:id` returns status
- [ ] DELETE `/api/sandbox/:id` stops sandbox
- [ ] GET `/api/auth/status` returns all provider status

#### 8.4 Documentation Updates
- [ ] Update `README.md` - Remove AIService references
- [ ] Update `CLAUDE.md` - Add VibeKit integration info
- [ ] Create `docs/VIBEKIT_INTEGRATION.md`
- [ ] Create `docs/AI_PROVIDERS.md`
- [ ] Create `docs/SANDBOX_EXECUTION.md`
- [ ] Update API documentation with new endpoints
- [ ] Add migration guide from legacy AIService
- [ ] Update deployment documentation

### Performance Benchmarks

Create benchmarks:
- [ ] Legacy AIService vs new proxy
- [ ] OAuth token refresh performance
- [ ] Sandbox creation time
- [ ] Concurrent sandbox handling

```rust
#[bench]
fn bench_proxy_vs_legacy(b: &mut Bencher) {
    // Benchmark comparison
}
```

### Security Audit

Security checklist:
- [ ] OAuth tokens encrypted in database
- [ ] No tokens logged or exposed
- [ ] Sandbox isolation verified
- [ ] Port exposure limited
- [ ] Auto-cleanup prevents resource leaks

### Deployment Verification

Test deployment:
- [ ] Docker build succeeds
- [ ] All tests pass in CI
- [ ] Documentation builds
- [ ] Migration runs cleanly

---

---

## Project Summary & Tracking

### Overall Progress
**Total Tasks:** 95
**Completed:** 0
**In Progress:** 0
**Remaining:** 95

### Phase Summary

| Phase | Status | Tasks | Completion | Week |
|-------|--------|-------|------------|------|
| **Phase 1** - Legacy AIService Removal | Not Started ⏳ | 19 | 0% | Weeks 1-2 |
| **Phase 2** - Database Schema | Not Started ⏳ | 8 | 0% | Week 2 |
| **Phase 3** - VibeKit Package | Not Started ⏳ | 11 | 0% | Week 3 |
| **Phase 4** - Rust Integration | Not Started ⏳ | 14 | 0% | Weeks 3-4 |
| **Phase 5** - CLI Commands | Not Started ⏳ | 12 | 0% | Week 4 |
| **Phase 6** - Dashboard Integration | Not Started ⏳ | 10 | 0% | Weeks 4-5 |
| **Phase 7** - Dagger Integration | Not Started ⏳ | 9 | 0% | Week 5 |
| **Phase 8** - Testing & Documentation | Not Started ⏳ | 12 | 0% | Week 6 |

### Critical Path Dependencies

```
Phase 1 (Legacy Removal) ─┐
                          ├─> Phase 4 (Rust Integration) ─> Phase 5 (CLI)
Phase 2 (Database) ───────┤                              │
                          │                              └─> Phase 6 (Dashboard)
Phase 3 (VibeKit Package) ┘                              │
                                                         └─> Phase 7 (Dagger)
                                                             │
                                                             v
                                                         Phase 8 (Testing)
```

### Key Milestones

- [ ] **Week 2 Milestone:** Legacy AIService completely removed
- [ ] **Week 3 Milestone:** VibeKit package functional
- [ ] **Week 4 Milestone:** CLI commands working
- [ ] **Week 5 Milestone:** Dashboard integration complete
- [ ] **Week 6 Milestone:** All tests passing, ready for production

### Success Criteria

#### Technical Requirements
- [ ] Zero references to legacy AIService in codebase
- [ ] All 10 legacy endpoints migrated to AI SDK proxy
- [ ] OAuth authentication working for all providers
- [ ] Dagger sandbox execution functional
- [ ] No additional ports required (only 4001 and 5173)

#### Quality Requirements
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Manual testing checklist complete
- [ ] Documentation fully updated
- [ ] Performance benchmarks meet or exceed legacy system

#### User Experience Requirements
- [ ] Backward compatibility maintained for API key users
- [ ] OAuth setup requires < 5 steps
- [ ] Sandbox creation < 10 seconds
- [ ] Clear migration path documented

### Risk Register

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| OAuth token refresh fails | High | Medium | Fallback to API keys, retry logic |
| Dagger performance issues | Medium | Low | Pre-built templates, caching |
| Breaking changes in migration | High | Low | Feature flags, gradual rollout |
| VibeKit SDK changes | Medium | Medium | Pin versions, test before upgrade |

### Architecture Decisions Record (ADR)

1. **ADR-001:** Use single port (no separate VibeKit service)
   - **Decision:** Integrate directly into CLI server
   - **Rationale:** Simpler deployment, fewer ports to manage

2. **ADR-002:** Node.js child process for VibeKit
   - **Decision:** Spawn Node.js process from Rust
   - **Rationale:** Simple, reliable, no complex bindings

3. **ADR-003:** Dagger for initial sandboxing
   - **Decision:** Start with Dagger, expand later
   - **Rationale:** Local-first, no cloud costs initially

4. **ADR-004:** Support both OAuth and API keys
   - **Decision:** Hybrid authentication approach
   - **Rationale:** Flexibility for different use cases

### Next Steps

**Immediate Actions:**
1. Start Phase 1 - Begin migrating first API handler
2. Set up project structure for VibeKit package
3. Create database migration file

**Week 1 Goals:**
- [ ] Complete 5 API handler migrations
- [ ] Database schema finalized
- [ ] VibeKit package initialized

**Communication:**
- Weekly progress updates
- Blocker escalation process
- Testing coordination

---

## Notes

- Use checkboxes to track progress as tasks are completed
- Update phase status when starting/completing phases
- Document any deviations from the plan
- Keep risk register updated with new findings