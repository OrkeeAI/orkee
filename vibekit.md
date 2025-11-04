# Dagger Sandbox Integration Plan

## Project Overview

This document tracks the integration of Dagger for local development sandboxing in Orkee. This allows projects to run in isolated containerized environments for testing and development.

**Note**: OAuth authentication is tracked separately in `oauth.md`.

### High-Level Goals
- Integrate Dagger for local sandbox execution
- Support multiple container templates (Node.js, Python, etc.)
- Track sandbox sessions in database
- Provide CLI and API for sandbox management
- Auto-cleanup expired sandbox sessions

### Timeline Summary
**Total Duration:** 2-3 weeks
- **Week 1:** Phase 1 - Database Schema
- **Week 2:** Phase 2 - Dagger Integration & CLI Commands
- **Week 3:** Phase 3 - Dashboard Integration & Testing

## Phase Progress Tracker

### Phase Status Overview
- [ ] **Phase 1:** Database Schema for Sandbox Sessions _(Week 1)_
- [ ] **Phase 2:** Dagger Integration & CLI Commands _(Week 2)_
- [ ] **Phase 3:** Dashboard Integration & Testing _(Week 3)_

---

## Phase 1: Database Schema for Sandbox Sessions (Week 1)

### Phase 1 Status: Not Started ⏳
**Completion:** 0/4 tasks

### Phase 1 Overview
Add database tables to support sandbox session tracking by modifying the existing initial schema migration (no production users yet, safe to modify).

### Phase 1 Tasks

#### 1.1 Database Schema Updates
- [ ] Update `packages/storage/migrations/001_initial_schema.sql` to add sandbox tables
- [ ] Update `packages/storage/migrations/001_initial_schema.down.sql` to drop sandbox tables
- [ ] Test migration up
- [ ] Test migration down

**Note:** Since no production users exist yet, we can safely modify the initial schema migration instead of creating a new migration file.

#### 1.2 Schema Changes to Add

**Location:** Add to `packages/storage/migrations/001_initial_schema.sql` after the `api_tokens` table (line ~703)

**Sandbox sessions table:**
```sql
-- Track Dagger sandbox sessions
CREATE TABLE sandbox_sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    user_id TEXT NOT NULL,
    project_id TEXT,
    provider TEXT NOT NULL DEFAULT 'dagger' CHECK (provider IN ('dagger')),
    container_id TEXT NOT NULL, -- Dagger container ID
    status TEXT NOT NULL CHECK (status IN ('active', 'paused', 'terminated')),
    host_url TEXT, -- Local URL (e.g., localhost:8080)
    working_directory TEXT DEFAULT '/app',
    metadata TEXT, -- JSON blob for provider-specific data
    created_at INTEGER NOT NULL DEFAULT (unixepoch()),
    last_active_at INTEGER NOT NULL DEFAULT (unixepoch()),
    expires_at INTEGER, -- Auto-cleanup after this time
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE SET NULL,
    CHECK (json_valid(metadata) OR metadata IS NULL)
);

-- Indexes
CREATE INDEX idx_sandbox_sessions_user ON sandbox_sessions(user_id);
CREATE INDEX idx_sandbox_sessions_project ON sandbox_sessions(project_id);
CREATE INDEX idx_sandbox_sessions_status ON sandbox_sessions(status);
CREATE INDEX idx_sandbox_sessions_expires ON sandbox_sessions(expires_at);
```

**Update down migration:**
Add to `packages/storage/migrations/001_initial_schema.down.sql` in the SECURITY section:
```sql
DROP TABLE IF EXISTS sandbox_sessions;
```

---

## Phase 2: Dagger Integration & CLI Commands (Week 2)

### Phase 2 Status: Not Started ⏳
**Completion:** 0/12 tasks

### Phase 2 Overview
Implement Dagger for local sandbox execution and add CLI commands for management.

### Phase 2 Tasks

#### 2.1 Prerequisites
- [ ] Install Docker (required by Dagger)
- [ ] Install Dagger CLI (optional, for debugging)

#### 2.2 Dagger Implementation
- [ ] Create `packages/dagger/` TypeScript package
- [ ] Implement Dagger client wrapper
- [ ] Add container lifecycle management
- [ ] Implement port forwarding
- [ ] Add log streaming
- [ ] Create Rust-Node.js bridge for Dagger calls

#### 2.3 Rust Storage Layer
- [ ] Create `packages/projects/src/storage/sandbox_sessions.rs`
- [ ] Update `packages/projects/src/storage/mod.rs` to export sandbox module
- [ ] Implement sandbox session CRUD operations

#### 2.4 CLI Commands
- [ ] Create `packages/cli/src/bin/cli/sandbox.rs`
- [ ] Implement `orkee sandbox list` command
- [ ] Implement `orkee sandbox create` command
- [ ] Implement `orkee sandbox stop` command
- [ ] Implement `orkee sandbox logs` command
- [ ] Implement `orkee sandbox clean` command

### Dagger Package Structure

```
packages/dagger/
├── package.json
├── tsconfig.json
├── src/
│   ├── index.ts           # Main exports
│   ├── client.ts          # Dagger client wrapper
│   ├── sandbox.ts         # Sandbox lifecycle management
│   ├── bridge.ts          # Rust-Node.js bridge
│   └── types.ts           # TypeScript type definitions
└── dist/                  # Compiled JavaScript
```

### Dagger Client Implementation

```typescript
// packages/dagger/src/client.ts
import Client, { connect, Container } from "@dagger.io/dagger";
import { EventEmitter } from 'events';

export interface SandboxConfig {
  template?: string;
  workdir?: string;
  ports?: number[];
  environment?: Record<string, string>;
}

export class DaggerSandbox extends EventEmitter {
  private client?: Client;
  private containers: Map<string, Container> = new Map();

  async initialize(): Promise<void> {
    this.client = await connect({
      LogOutput: process.stderr,
    });
  }

  async createSandbox(config: SandboxConfig = {}): Promise<{
    id: string;
    ports: Map<number, number>;
    url: string;
  }> {
    if (!this.client) await this.initialize();

    let container = this.client!
      .container()
      .from(config.template || 'node:18-alpine')
      .withWorkdir(config.workdir || '/app');

    // Expose ports
    const portMappings = new Map<number, number>();
    for (const port of config.ports || [3000]) {
      container = container.withExposedPort(port);
      const hostPort = 8000 + Math.floor(Math.random() * 1000);
      portMappings.set(port, hostPort);
    }

    // Set environment variables
    for (const [key, value] of Object.entries(config.environment || {})) {
      container = container.withEnvVariable(key, value);
    }

    const id = await container.id();
    this.containers.set(id, container);

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
    this.containers.clear();
    if (this.client) {
      await this.client.close();
      this.client = undefined;
    }
  }
}
```

### Rust Bridge Implementation

```rust
// packages/cli/src/dagger/mod.rs
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

pub struct DaggerBridge {
    node_script_path: PathBuf,
}

impl DaggerBridge {
    pub fn new() -> Result<Self> {
        let node_script_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../dagger/dist/bridge.js");

        if !node_script_path.exists() {
            anyhow::bail!("Dagger bridge not built. Run: cd packages/dagger && bun run build");
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
            anyhow::bail!("Dagger bridge failed: {}", stderr);
        }

        let response: BridgeResponse = serde_json::from_slice(&output.stdout)?;

        if !response.success {
            anyhow::bail!("Dagger error: {}", response.error.unwrap_or_default());
        }

        Ok(response)
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

    pub async fn stop_sandbox(&self, id: String) -> Result<()> {
        let request = BridgeRequest {
            command: "stop_sandbox".to_string(),
            payload: serde_json::json!({ "id": id }),
        };

        self.call_bridge(request).await?;
        Ok(())
    }
}
```

### Sandbox Storage Implementation

```rust
// packages/projects/src/storage/sandbox_sessions.rs
use sqlx::SqlitePool;
use chrono::Utc;
use std::collections::HashMap;
use anyhow::Result;

pub struct SandboxSessionStorage {
    pool: SqlitePool,
}

impl SandboxSessionStorage {
    pub async fn create_session(
        &self,
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

### CLI Commands

```rust
// packages/cli/src/bin/cli/sandbox.rs
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
```

### Background Cleanup Task

```rust
// packages/cli/src/tasks/sandbox_cleanup.rs
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

## Phase 3: Dashboard Integration & Testing (Week 3)

### Phase 3 Status: Not Started ⏳
**Completion:** 0/8 tasks

### Phase 3 Overview
Add UI for sandbox management and comprehensive testing.

### Phase 3 Tasks

#### 3.1 Dashboard Components
- [ ] Create `packages/dashboard/src/components/SandboxManager.tsx`
- [ ] Implement sandbox list view
- [ ] Add create/stop controls
- [ ] Show sandbox logs in UI

#### 3.2 API Endpoints
- [ ] Create `packages/api/src/sandbox_handlers.rs`
- [ ] Implement POST `/api/sandbox` - create sandbox
- [ ] Implement GET `/api/sandbox/:id` - get sandbox status
- [ ] Implement DELETE `/api/sandbox/:id` - stop sandbox
- [ ] Add routes to main router

#### 3.3 Testing
- [ ] Write unit tests for Dagger client
- [ ] Write integration tests for sandbox lifecycle
- [ ] Test cleanup task
- [ ] Manual testing checklist

### Sandbox Manager Component

```tsx
// packages/dashboard/src/components/SandboxManager.tsx
import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Card } from '@/components/ui/card';

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
    const interval = setInterval(fetchSandboxes, 5000);
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
                <Button
                  size="sm"
                  variant="destructive"
                  onClick={() => stopSandbox(sandbox.id)}
                >
                  Stop
                </Button>
              </div>
            </Card>
          ))}
        </div>
      )}
    </div>
  );
}
```

---

## Testing Checklist

### Manual Testing
- [ ] `orkee sandbox create` creates Dagger container
- [ ] Can access sandbox via localhost URL
- [ ] `orkee sandbox logs` shows container output
- [ ] `orkee sandbox stop` terminates container
- [ ] Auto-cleanup works after 1 hour
- [ ] Dashboard shows sandbox list
- [ ] Can create/stop sandboxes from UI

### Integration Tests
- [ ] Test sandbox creation and deletion
- [ ] Test expired sandbox cleanup
- [ ] Test port mapping
- [ ] Test log streaming

---

## Success Criteria

### Technical Requirements
- [ ] Dagger sandbox execution functional
- [ ] Sandbox sessions tracked in database
- [ ] Auto-cleanup prevents resource leaks
- [ ] CLI and dashboard integration complete

### User Experience Requirements
- [ ] Sandbox creation < 10 seconds
- [ ] Clear status display
- [ ] Simple CLI commands
- [ ] Working dashboard UI

---

## Notes

- OAuth authentication is tracked separately in `oauth.md`
- Dagger provides local-first sandboxing without cloud costs
- Uses Node.js bridge from Rust (similar to OAuth approach)
- Sandbox sessions expire after 1 hour by default
- Background cleanup task runs every 5 minutes
