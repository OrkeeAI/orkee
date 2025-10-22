-- Context-OpenSpec Integration - Phase 3
-- Link context configurations to spec capabilities and enable spec-driven context

-- Link context configurations to spec capabilities
ALTER TABLE context_configurations
ADD COLUMN spec_capability_id TEXT REFERENCES spec_capabilities(id);

-- Map AST symbols to spec requirements
CREATE TABLE ast_spec_mappings (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    project_id TEXT NOT NULL REFERENCES projects(id),
    file_path TEXT NOT NULL,
    symbol_name TEXT NOT NULL,
    symbol_type TEXT NOT NULL, -- function, class, interface, etc.
    line_number INTEGER,
    requirement_id TEXT REFERENCES spec_requirements(id),
    confidence REAL DEFAULT 0.0, -- AI confidence in mapping
    verified INTEGER DEFAULT 0,  -- SQLite uses INTEGER for booleans (0/1)
    created_at TEXT DEFAULT (datetime('now')),
    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE
);

-- Context templates for different spec scenarios
CREATE TABLE context_templates (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    description TEXT,
    template_type TEXT NOT NULL, -- 'prd', 'capability', 'task', 'validation'
    include_patterns TEXT DEFAULT '[]',  -- JSON array
    exclude_patterns TEXT DEFAULT '[]',  -- JSON array
    ast_filters TEXT,  -- JSON object: {"include_types": ["function", "class"]}
    created_at TEXT DEFAULT (datetime('now'))
);

-- Track which context was used for each AI operation (enhancement)
ALTER TABLE ai_usage_logs
ADD COLUMN context_snapshot_id TEXT REFERENCES context_snapshots(id);

-- Indexes for performance
CREATE INDEX idx_ast_spec_mappings_project ON ast_spec_mappings(project_id);
CREATE INDEX idx_ast_spec_mappings_requirement ON ast_spec_mappings(requirement_id);
CREATE INDEX idx_context_configs_spec ON context_configurations(spec_capability_id);
CREATE INDEX idx_ai_usage_logs_context ON ai_usage_logs(context_snapshot_id);
