-- ABOUTME: PRD Generation tracking and export history
-- ABOUTME: Tracks PRD generation state, versions, validation, and export records

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- PRD GENERATION TRACKING
-- ============================================================================

-- Track PRD generation state and versions
CREATE TABLE ideate_prd_generations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,

    -- Generated content
    generated_content TEXT, -- Full PRD JSON (GeneratedPRD structure)
    markdown_content TEXT,  -- Formatted markdown

    -- Generation metadata
    generation_method TEXT NOT NULL CHECK(generation_method IN ('full', 'sections', 'merged', 'ai_filled')),
    filled_sections TEXT, -- JSON array of sections that were AI-filled

    -- Validation status
    validation_status TEXT NOT NULL DEFAULT 'pending' CHECK(validation_status IN ('pending', 'valid', 'warnings', 'errors')),
    validation_details TEXT, -- JSON validation results

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(generated_content) OR generated_content IS NULL),
    CHECK (json_valid(filled_sections) OR filled_sections IS NULL),
    CHECK (json_valid(validation_details) OR validation_details IS NULL)
);

CREATE INDEX idx_prd_generations_session ON ideate_prd_generations(session_id);
CREATE INDEX idx_prd_generations_version ON ideate_prd_generations(session_id, version DESC);
CREATE INDEX idx_prd_generations_status ON ideate_prd_generations(validation_status);
CREATE INDEX idx_prd_generations_method ON ideate_prd_generations(generation_method);

CREATE TRIGGER ideate_prd_generations_updated_at AFTER UPDATE ON ideate_prd_generations
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ideate_prd_generations SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;

-- ============================================================================
-- EXPORT HISTORY
-- ============================================================================

-- Track PRD exports in various formats
CREATE TABLE ideate_exports (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    generation_id TEXT, -- Optional: links to specific generation version

    -- Export metadata
    format TEXT NOT NULL CHECK(format IN ('markdown', 'html', 'pdf', 'docx')),
    file_path TEXT, -- Relative or absolute path to exported file
    file_size_bytes INTEGER,

    -- Export options (JSON)
    export_options TEXT, -- JSON of ExportOptions

    -- Timestamps
    exported_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (generation_id) REFERENCES ideate_prd_generations(id) ON DELETE SET NULL,
    CHECK (json_valid(export_options) OR export_options IS NULL)
);

CREATE INDEX idx_exports_session ON ideate_exports(session_id);
CREATE INDEX idx_exports_generation ON ideate_exports(generation_id);
CREATE INDEX idx_exports_format ON ideate_exports(format);
CREATE INDEX idx_exports_date ON ideate_exports(exported_at DESC);

-- ============================================================================
-- SECTION GENERATION HISTORY
-- ============================================================================

-- Track individual section generation/regeneration events
CREATE TABLE ideate_section_generations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    generation_id TEXT, -- Links to main generation record

    -- Section info
    section_name TEXT NOT NULL CHECK(section_name IN ('overview', 'features', 'ux', 'technical', 'roadmap', 'dependencies', 'risks', 'research')),
    section_content TEXT, -- JSON content for the section

    -- Generation context
    was_skipped INTEGER NOT NULL DEFAULT 0, -- Boolean: was this section originally skipped?
    was_ai_filled INTEGER NOT NULL DEFAULT 0, -- Boolean: was this filled by AI?
    context_used TEXT, -- Context string used for generation

    -- AI usage
    tokens_used INTEGER, -- Number of tokens used for generation
    model TEXT, -- AI model used

    -- Timestamps
    generated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (generation_id) REFERENCES ideate_prd_generations(id) ON DELETE SET NULL,
    CHECK (json_valid(section_content) OR section_content IS NULL)
);

CREATE INDEX idx_section_generations_session ON ideate_section_generations(session_id);
CREATE INDEX idx_section_generations_generation ON ideate_section_generations(generation_id);
CREATE INDEX idx_section_generations_section ON ideate_section_generations(section_name);
CREATE INDEX idx_section_generations_ai_filled ON ideate_section_generations(was_ai_filled);
CREATE INDEX idx_section_generations_date ON ideate_section_generations(generated_at DESC);

-- ============================================================================
-- VALIDATION RULES
-- ============================================================================

-- Store PRD validation rules and results
CREATE TABLE ideate_validation_rules (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),

    -- Rule definition
    rule_name TEXT NOT NULL UNIQUE,
    rule_type TEXT NOT NULL CHECK(rule_type IN ('required_field', 'min_length', 'max_length', 'format', 'consistency', 'completeness')),
    section TEXT, -- Which section this applies to (NULL = all sections)
    field_path TEXT, -- JSON path to field (e.g., "overview.problem_statement")

    -- Rule parameters (JSON)
    rule_params TEXT, -- e.g., {"min_length": 50, "max_length": 500}

    -- Rule metadata
    severity TEXT NOT NULL DEFAULT 'warning' CHECK(severity IN ('error', 'warning', 'info')),
    error_message TEXT NOT NULL,
    is_active INTEGER NOT NULL DEFAULT 1, -- Boolean: is this rule currently enforced?

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    CHECK (json_valid(rule_params) OR rule_params IS NULL)
);

CREATE INDEX idx_validation_rules_section ON ideate_validation_rules(section);
CREATE INDEX idx_validation_rules_type ON ideate_validation_rules(rule_type);
CREATE INDEX idx_validation_rules_severity ON ideate_validation_rules(severity);
CREATE INDEX idx_validation_rules_active ON ideate_validation_rules(is_active);

-- Seed default validation rules
INSERT OR IGNORE INTO ideate_validation_rules (id, rule_name, rule_type, section, field_path, rule_params, severity, error_message)
VALUES
    ('val_rule_1', 'overview_problem_required', 'required_field', 'overview', 'problem_statement', NULL, 'error', 'Problem statement is required'),
    ('val_rule_2', 'overview_audience_required', 'required_field', 'overview', 'target_audience', NULL, 'error', 'Target audience is required'),
    ('val_rule_3', 'overview_value_required', 'required_field', 'overview', 'value_proposition', NULL, 'error', 'Value proposition is required'),
    ('val_rule_4', 'problem_min_length', 'min_length', 'overview', 'problem_statement', '{"min_length": 50}', 'warning', 'Problem statement should be at least 50 characters'),
    ('val_rule_5', 'features_min_count', 'completeness', 'features', NULL, '{"min_count": 3}', 'warning', 'PRD should have at least 3 core features'),
    ('val_rule_6', 'roadmap_mvp_required', 'required_field', 'roadmap', 'mvp_scope', NULL, 'warning', 'MVP scope should be defined'),
    ('val_rule_7', 'dependencies_foundation_required', 'completeness', 'dependencies', 'foundation_features', '{"min_count": 1}', 'warning', 'At least one foundation feature should be identified');

-- ============================================================================
-- GENERATION STATISTICS
-- ============================================================================

-- Track generation statistics for analytics
CREATE TABLE ideate_generation_stats (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    generation_id TEXT NOT NULL,

    -- Token usage
    total_tokens_used INTEGER NOT NULL DEFAULT 0,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,

    -- Timing
    generation_duration_ms INTEGER, -- Time taken to generate in milliseconds

    -- Sections
    sections_generated INTEGER NOT NULL DEFAULT 0,
    sections_skipped INTEGER NOT NULL DEFAULT 0,
    sections_ai_filled INTEGER NOT NULL DEFAULT 0,

    -- Quality metrics
    completeness_score REAL, -- 0.0 to 1.0
    validation_score REAL, -- 0.0 to 1.0

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (generation_id) REFERENCES ideate_prd_generations(id) ON DELETE CASCADE
);

CREATE INDEX idx_generation_stats_session ON ideate_generation_stats(session_id);
CREATE INDEX idx_generation_stats_generation ON ideate_generation_stats(generation_id);
CREATE INDEX idx_generation_stats_date ON ideate_generation_stats(created_at DESC);
CREATE INDEX idx_generation_stats_completeness ON ideate_generation_stats(completeness_score DESC);
