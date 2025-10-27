-- ABOUTME: Ideation schema for flexible PRD creation
-- ABOUTME: Supports Quick, Guided, and Comprehensive modes with optional sections

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- BRAINSTORMING SESSIONS
-- ============================================================================

-- Main ideateing session
CREATE TABLE ideate_sessions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,

    -- Minimal required info
    initial_description TEXT NOT NULL,

    -- Session metadata
    mode TEXT NOT NULL CHECK(mode IN ('quick', 'guided', 'comprehensive')),
    status TEXT NOT NULL DEFAULT 'draft' CHECK(status IN ('draft', 'in_progress', 'ready_for_prd', 'completed')),

    -- Track what user chose to skip (JSON array of section names)
    skipped_sections TEXT,

    -- Link to generated PRD (if completed)
    generated_prd_id TEXT,

    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (generated_prd_id) REFERENCES prds(id) ON DELETE SET NULL,
    CHECK (json_valid(skipped_sections) OR skipped_sections IS NULL)
);

CREATE INDEX idx_ideate_sessions_project ON ideate_sessions(project_id);
CREATE INDEX idx_ideate_sessions_status ON ideate_sessions(status);
CREATE INDEX idx_ideate_sessions_mode ON ideate_sessions(mode);

CREATE TRIGGER ideate_sessions_updated_at AFTER UPDATE ON ideate_sessions
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE ideate_sessions SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;

-- ============================================================================
-- PRD SECTIONS (All Optional)
-- ============================================================================

-- Overview Section
CREATE TABLE ideate_overview (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    problem_statement TEXT,
    target_audience TEXT,
    value_proposition TEXT,
    one_line_pitch TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_ideate_overview_session ON ideate_overview(session_id);

-- Core Features
CREATE TABLE ideate_features (
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

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(depends_on) OR depends_on IS NULL),
    CHECK (json_valid(enables) OR enables IS NULL),
    CHECK (build_phase IN (1, 2, 3))
);

CREATE INDEX idx_ideate_features_session ON ideate_features(session_id);
CREATE INDEX idx_ideate_features_phase ON ideate_features(build_phase);
CREATE INDEX idx_ideate_features_visible ON ideate_features(is_visible);

-- User Experience
CREATE TABLE ideate_ux (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    personas TEXT, -- JSON array of personas
    user_flows TEXT, -- JSON array of user flows
    ui_considerations TEXT,
    ux_principles TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(personas) OR personas IS NULL),
    CHECK (json_valid(user_flows) OR user_flows IS NULL)
);

CREATE INDEX idx_ideate_ux_session ON ideate_ux(session_id);

-- Technical Architecture
CREATE TABLE ideate_technical (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    components TEXT, -- JSON array
    data_models TEXT, -- JSON array
    apis TEXT, -- JSON array
    infrastructure TEXT, -- JSON object
    tech_stack_quick TEXT, -- For quick mode: "React + Node + PostgreSQL"
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(components) OR components IS NULL),
    CHECK (json_valid(data_models) OR data_models IS NULL),
    CHECK (json_valid(apis) OR apis IS NULL),
    CHECK (json_valid(infrastructure) OR infrastructure IS NULL)
);

CREATE INDEX idx_ideate_technical_session ON ideate_technical(session_id);

-- Development Roadmap (NO timelines, just scope)
CREATE TABLE ideate_roadmap (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    mvp_scope TEXT, -- JSON array of features in MVP
    future_phases TEXT, -- JSON array of post-MVP phases
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(mvp_scope) OR mvp_scope IS NULL),
    CHECK (json_valid(future_phases) OR future_phases IS NULL)
);

CREATE INDEX idx_ideate_roadmap_session ON ideate_roadmap(session_id);

-- Logical Dependency Chain
CREATE TABLE ideate_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    foundation_features TEXT, -- JSON array of feature IDs: must build first
    visible_features TEXT, -- JSON array of feature IDs: get something usable quickly
    enhancement_features TEXT, -- JSON array of feature IDs: build upon foundation
    dependency_graph TEXT, -- JSON object for visual representation
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(foundation_features) OR foundation_features IS NULL),
    CHECK (json_valid(visible_features) OR visible_features IS NULL),
    CHECK (json_valid(enhancement_features) OR enhancement_features IS NULL),
    CHECK (json_valid(dependency_graph) OR dependency_graph IS NULL)
);

CREATE INDEX idx_ideate_dependencies_session ON ideate_dependencies(session_id);

-- Risks and Mitigations
CREATE TABLE ideate_risks (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    technical_risks TEXT, -- JSON array
    mvp_scoping_risks TEXT, -- JSON array
    resource_risks TEXT, -- JSON array
    mitigations TEXT, -- JSON array
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(technical_risks) OR technical_risks IS NULL),
    CHECK (json_valid(mvp_scoping_risks) OR mvp_scoping_risks IS NULL),
    CHECK (json_valid(resource_risks) OR resource_risks IS NULL),
    CHECK (json_valid(mitigations) OR mitigations IS NULL)
);

CREATE INDEX idx_ideate_risks_session ON ideate_risks(session_id);

-- Research & Appendix
CREATE TABLE ideate_research (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    competitors TEXT, -- JSON array
    similar_projects TEXT, -- JSON array
    research_findings TEXT,
    technical_specs TEXT,
    reference_links TEXT, -- JSON array (renamed from 'references' to avoid SQL keyword)
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(competitors) OR competitors IS NULL),
    CHECK (json_valid(similar_projects) OR similar_projects IS NULL),
    CHECK (json_valid(reference_links) OR reference_links IS NULL)
);

CREATE INDEX idx_ideate_research_session ON ideate_research(session_id);

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
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(experts) OR experts IS NULL),
    CHECK (json_valid(key_insights) OR key_insights IS NULL),
    CHECK (json_valid(recommendations) OR recommendations IS NULL)
);

CREATE INDEX idx_roundtable_sessions_session ON roundtable_sessions(session_id);

-- ============================================================================
-- GENERATION TRACKING (Quick Mode)
-- ============================================================================

-- Track PRD generation progress for Quick Mode
CREATE TABLE ideate_generations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    status TEXT NOT NULL CHECK(status IN ('generating', 'completed', 'failed')),
    generated_sections TEXT, -- JSON object: {"overview": "...", "features": "..."}
    current_section TEXT, -- Which section is currently being generated
    error_message TEXT,
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(generated_sections) OR generated_sections IS NULL)
);

CREATE INDEX idx_ideate_generations_session ON ideate_generations(session_id);
CREATE INDEX idx_ideate_generations_status ON ideate_generations(status);

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
