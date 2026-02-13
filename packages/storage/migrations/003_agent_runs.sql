-- Agent Runs: tracks autonomous agent loop executions against a PRD
-- Each run drives one or more iterations, each targeting a user story

CREATE TABLE IF NOT EXISTS agent_runs (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    prd_id TEXT,
    prd_json TEXT NOT NULL,
    system_prompt TEXT,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    max_iterations INTEGER NOT NULL DEFAULT 10,
    current_iteration INTEGER NOT NULL DEFAULT 0,
    stories_total INTEGER NOT NULL DEFAULT 0,
    stories_completed INTEGER NOT NULL DEFAULT 0,
    total_cost REAL NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
    error TEXT,
    runner_pid INTEGER,
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_agent_runs_project ON agent_runs(project_id);
CREATE INDEX IF NOT EXISTS idx_agent_runs_status ON agent_runs(status);

-- Link individual iterations to their parent run
ALTER TABLE agent_executions ADD COLUMN run_id TEXT REFERENCES agent_runs(id) ON DELETE SET NULL;
ALTER TABLE agent_executions ADD COLUMN iteration_number INTEGER;
ALTER TABLE agent_executions ADD COLUMN story_id TEXT;

CREATE INDEX IF NOT EXISTS idx_agent_executions_run ON agent_executions(run_id);

-- Auto-update updated_at on agent_runs changes
CREATE TRIGGER IF NOT EXISTS update_agent_runs_updated_at
AFTER UPDATE ON agent_runs
BEGIN
    UPDATE agent_runs SET updated_at = datetime('now') WHERE id = NEW.id;
END;
