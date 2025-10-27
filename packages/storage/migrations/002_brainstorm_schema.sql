-- ABOUTME: Brainstorming and ideation schema for flexible PRD creation
-- ABOUTME: Supports Quick, Guided, and Comprehensive modes with optional sections

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- BRAINSTORMING SESSIONS
-- ============================================================================

-- Main brainstorming session
CREATE TABLE brainstorm_sessions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,

    -- Minimal required info
    initial_description TEXT NOT NULL,

    -- Session metadata
    mode TEXT NOT NULL CHECK(mode IN ('quick', 'guided', 'comprehensive')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'in_progress', 'ready_for_prd', 'completed')),

    -- Track what user chose to skip (JSON array of section names)
    skipped_sections TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    CHECK (json_valid(skipped_sections) OR skipped_sections IS NULL)
);

CREATE INDEX idx_brainstorm_sessions_project ON brainstorm_sessions(project_id);
CREATE INDEX idx_brainstorm_sessions_status ON brainstorm_sessions(status);
CREATE INDEX idx_brainstorm_sessions_mode ON brainstorm_sessions(mode);

CREATE TRIGGER brainstorm_sessions_updated_at AFTER UPDATE ON brainstorm_sessions
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE brainstorm_sessions SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;

-- ============================================================================
-- PRD SECTIONS (All Optional)
-- ============================================================================

-- Overview Section
CREATE TABLE brainstorm_overview (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    problem_statement TEXT,
    target_audience TEXT,
    value_proposition TEXT,
    one_line_pitch TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_brainstorm_overview_session ON brainstorm_overview(session_id);

-- Core Features
CREATE TABLE brainstorm_features (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    feature_name TEXT NOT NULL,
    what_it_does TEXT,
    why_important TEXT,
    how_it_works TEXT,

    -- Dependency chain fields
    depends_on TEXT, -- JSON array of feature IDs this depends on
    enables TEXT, -- JSON array of feature IDs this unlocks
    build_phase INTEGER DEFAULT 1, -- 1=foundation, 2=visible, 3=enhancement
    is_visible INTEGER DEFAULT 0, -- Boolean: does this give user something to see/use?

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(depends_on) OR depends_on IS NULL),
    CHECK (json_valid(enables) OR enables IS NULL),
    CHECK (build_phase IN (1, 2, 3))
);

CREATE INDEX idx_brainstorm_features_session ON brainstorm_features(session_id);
CREATE INDEX idx_brainstorm_features_phase ON brainstorm_features(build_phase);
CREATE INDEX idx_brainstorm_features_visible ON brainstorm_features(is_visible);

-- User Experience
CREATE TABLE brainstorm_ux (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    personas TEXT, -- JSON array of personas
    user_flows TEXT, -- JSON array of user flows
    ui_considerations TEXT,
    ux_principles TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(personas) OR personas IS NULL),
    CHECK (json_valid(user_flows) OR user_flows IS NULL)
);

CREATE INDEX idx_brainstorm_ux_session ON brainstorm_ux(session_id);

-- Technical Architecture
CREATE TABLE brainstorm_technical (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    components TEXT, -- JSON array
    data_models TEXT, -- JSON array
    apis TEXT, -- JSON array
    infrastructure TEXT, -- JSON object
    tech_stack_quick TEXT, -- For quick mode: "React + Node + PostgreSQL"
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(components) OR components IS NULL),
    CHECK (json_valid(data_models) OR data_models IS NULL),
    CHECK (json_valid(apis) OR apis IS NULL),
    CHECK (json_valid(infrastructure) OR infrastructure IS NULL)
);

CREATE INDEX idx_brainstorm_technical_session ON brainstorm_technical(session_id);

-- Development Roadmap (NO timelines, just scope)
CREATE TABLE brainstorm_roadmap (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    mvp_scope TEXT, -- JSON array of features in MVP
    future_phases TEXT, -- JSON array of post-MVP phases
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(mvp_scope) OR mvp_scope IS NULL),
    CHECK (json_valid(future_phases) OR future_phases IS NULL)
);

CREATE INDEX idx_brainstorm_roadmap_session ON brainstorm_roadmap(session_id);

-- Logical Dependency Chain
CREATE TABLE brainstorm_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    foundation_features TEXT, -- JSON array of feature IDs: must build first
    visible_features TEXT, -- JSON array of feature IDs: get something usable quickly
    enhancement_features TEXT, -- JSON array of feature IDs: build upon foundation
    dependency_graph TEXT, -- JSON object for visual representation
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(foundation_features) OR foundation_features IS NULL),
    CHECK (json_valid(visible_features) OR visible_features IS NULL),
    CHECK (json_valid(enhancement_features) OR enhancement_features IS NULL),
    CHECK (json_valid(dependency_graph) OR dependency_graph IS NULL)
);

CREATE INDEX idx_brainstorm_dependencies_session ON brainstorm_dependencies(session_id);

-- Risks and Mitigations
CREATE TABLE brainstorm_risks (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    technical_risks TEXT, -- JSON array
    mvp_scoping_risks TEXT, -- JSON array
    resource_risks TEXT, -- JSON array
    mitigations TEXT, -- JSON array
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(technical_risks) OR technical_risks IS NULL),
    CHECK (json_valid(mvp_scoping_risks) OR mvp_scoping_risks IS NULL),
    CHECK (json_valid(resource_risks) OR resource_risks IS NULL),
    CHECK (json_valid(mitigations) OR mitigations IS NULL)
);

CREATE INDEX idx_brainstorm_risks_session ON brainstorm_risks(session_id);

-- Research & Appendix
CREATE TABLE brainstorm_research (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    competitors TEXT, -- JSON array
    similar_projects TEXT, -- JSON array
    research_findings TEXT,
    technical_specs TEXT,
    reference_links TEXT, -- JSON array (renamed from 'references' to avoid SQL keyword)
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(competitors) OR competitors IS NULL),
    CHECK (json_valid(similar_projects) OR similar_projects IS NULL),
    CHECK (json_valid(reference_links) OR reference_links IS NULL)
);

CREATE INDEX idx_brainstorm_research_session ON brainstorm_research(session_id);

-- ============================================================================
-- COMPREHENSIVE MODE FEATURES
-- ============================================================================

-- Expert Roundtable Sessions
CREATE TABLE roundtable_sessions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    experts TEXT, -- JSON array of expert personas
    discussion_log TEXT, -- Full conversation transcript
    key_insights TEXT, -- JSON array of extracted insights
    recommendations TEXT, -- JSON array of expert recommendations
    started_at TEXT,
    ended_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES brainstorm_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(experts) OR experts IS NULL),
    CHECK (json_valid(key_insights) OR key_insights IS NULL),
    CHECK (json_valid(recommendations) OR recommendations IS NULL)
);

CREATE INDEX idx_roundtable_sessions_session ON roundtable_sessions(session_id);

-- ============================================================================
-- QUICKSTART TEMPLATES
-- ============================================================================

-- Templates for Quick Start
CREATE TABLE prd_quickstart_templates (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    name TEXT NOT NULL,
    project_type TEXT, -- 'saas', 'mobile', 'api', 'marketplace', etc.
    one_liner_prompts TEXT, -- JSON array of prompts to expand one-liner
    default_features TEXT, -- JSON array of common features for this type
    default_dependencies TEXT, -- JSON object with typical dependency chains
    is_system INTEGER DEFAULT 0, -- Boolean: system template vs user-created
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    CHECK (json_valid(one_liner_prompts) OR one_liner_prompts IS NULL),
    CHECK (json_valid(default_features) OR default_features IS NULL),
    CHECK (json_valid(default_dependencies) OR default_dependencies IS NULL)
);

CREATE INDEX idx_prd_quickstart_templates_type ON prd_quickstart_templates(project_type);
CREATE INDEX idx_prd_quickstart_templates_system ON prd_quickstart_templates(is_system);
