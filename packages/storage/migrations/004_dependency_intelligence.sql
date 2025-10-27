-- ABOUTME: Phase 4 - Dependency Chain Intelligence schema
-- ABOUTME: Adds AI analysis caching, build order optimization, and enhanced dependency tracking

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- FEATURE DEPENDENCY GRAPH
-- ============================================================================

-- Detailed feature-to-feature dependencies (extends ideate_features)
CREATE TABLE feature_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    from_feature_id TEXT NOT NULL, -- Feature that depends on another
    to_feature_id TEXT NOT NULL, -- Feature that is depended upon
    dependency_type TEXT NOT NULL CHECK(dependency_type IN ('technical', 'logical', 'business')),
    strength TEXT DEFAULT 'required' CHECK(strength IN ('required', 'recommended', 'optional')),
    reason TEXT, -- Why this dependency exists
    auto_detected INTEGER DEFAULT 0, -- Boolean: was this detected by AI or manually added?
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (from_feature_id) REFERENCES ideate_features(id) ON DELETE CASCADE,
    FOREIGN KEY (to_feature_id) REFERENCES ideate_features(id) ON DELETE CASCADE,

    -- Prevent self-dependencies
    CHECK (from_feature_id != to_feature_id),

    -- Prevent duplicate dependencies
    UNIQUE(from_feature_id, to_feature_id)
);

CREATE INDEX idx_feature_dependencies_session ON feature_dependencies(session_id);
CREATE INDEX idx_feature_dependencies_from ON feature_dependencies(from_feature_id);
CREATE INDEX idx_feature_dependencies_to ON feature_dependencies(to_feature_id);
CREATE INDEX idx_feature_dependencies_type ON feature_dependencies(dependency_type);
CREATE INDEX idx_feature_dependencies_auto ON feature_dependencies(auto_detected);

-- ============================================================================
-- AI DEPENDENCY ANALYSIS CACHE
-- ============================================================================

-- Cache AI analysis results for performance
CREATE TABLE dependency_analysis_cache (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    features_hash TEXT NOT NULL, -- Hash of feature descriptions to detect changes
    analysis_type TEXT NOT NULL CHECK(analysis_type IN ('dependencies', 'build_order', 'visibility')),
    analysis_result TEXT NOT NULL, -- JSON object with analysis results
    confidence_score REAL, -- 0.0-1.0 confidence in analysis
    model_version TEXT, -- Which AI model was used
    analyzed_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    expires_at TEXT, -- Optional cache expiration
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(analysis_result)),
    CHECK (confidence_score IS NULL OR (confidence_score >= 0.0 AND confidence_score <= 1.0))
);

CREATE INDEX idx_dependency_analysis_session ON dependency_analysis_cache(session_id);
CREATE INDEX idx_dependency_analysis_type ON dependency_analysis_cache(analysis_type);
CREATE INDEX idx_dependency_analysis_hash ON dependency_analysis_cache(features_hash);
CREATE INDEX idx_dependency_analysis_expires ON dependency_analysis_cache(expires_at);

-- ============================================================================
-- BUILD ORDER OPTIMIZATION
-- ============================================================================

-- Store computed build sequences
CREATE TABLE build_order_optimization (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    build_sequence TEXT NOT NULL, -- JSON array of feature IDs in build order
    parallel_groups TEXT, -- JSON array of arrays: features that can be built in parallel
    critical_path TEXT, -- JSON array of feature IDs on critical path
    estimated_phases INTEGER, -- Number of sequential phases needed
    optimization_strategy TEXT CHECK(optimization_strategy IN ('fastest', 'balanced', 'safest')),
    computed_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    is_valid INTEGER DEFAULT 1, -- Boolean: becomes invalid when features/dependencies change
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(build_sequence)),
    CHECK (json_valid(parallel_groups) OR parallel_groups IS NULL),
    CHECK (json_valid(critical_path) OR critical_path IS NULL)
);

CREATE INDEX idx_build_order_session ON build_order_optimization(session_id);
CREATE INDEX idx_build_order_valid ON build_order_optimization(is_valid);
CREATE INDEX idx_build_order_strategy ON build_order_optimization(optimization_strategy);

-- ============================================================================
-- QUICK-WIN FEATURES ANALYSIS
-- ============================================================================

-- Track features identified as quick wins (high value, low dependency)
CREATE TABLE quick_win_features (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    feature_id TEXT NOT NULL,
    visibility_score REAL, -- How quickly users see value (0.0-1.0)
    dependency_count INTEGER DEFAULT 0, -- Number of dependencies
    complexity_score REAL, -- Estimated complexity (0.0-1.0, lower is simpler)
    value_score REAL, -- User value score (0.0-1.0, higher is better)
    overall_score REAL, -- Combined quick-win score
    reasoning TEXT, -- Why this is a quick win
    analyzed_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (feature_id) REFERENCES ideate_features(id) ON DELETE CASCADE,
    CHECK (visibility_score IS NULL OR (visibility_score >= 0.0 AND visibility_score <= 1.0)),
    CHECK (complexity_score IS NULL OR (complexity_score >= 0.0 AND complexity_score <= 1.0)),
    CHECK (value_score IS NULL OR (value_score >= 0.0 AND value_score <= 1.0)),
    CHECK (overall_score IS NULL OR (overall_score >= 0.0 AND overall_score <= 1.0)),

    UNIQUE(session_id, feature_id)
);

CREATE INDEX idx_quick_win_session ON quick_win_features(session_id);
CREATE INDEX idx_quick_win_feature ON quick_win_features(feature_id);
CREATE INDEX idx_quick_win_score ON quick_win_features(overall_score DESC);
CREATE INDEX idx_quick_win_visibility ON quick_win_features(visibility_score DESC);

-- ============================================================================
-- CIRCULAR DEPENDENCY DETECTION
-- ============================================================================

-- Track detected circular dependencies for warnings
CREATE TABLE circular_dependencies (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    cycle_path TEXT NOT NULL, -- JSON array of feature IDs forming the cycle
    severity TEXT DEFAULT 'error' CHECK(severity IN ('warning', 'error', 'critical')),
    detected_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    resolved INTEGER DEFAULT 0, -- Boolean: has this cycle been resolved?
    resolution_note TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(cycle_path))
);

CREATE INDEX idx_circular_dependencies_session ON circular_dependencies(session_id);
CREATE INDEX idx_circular_dependencies_resolved ON circular_dependencies(resolved);
CREATE INDEX idx_circular_dependencies_severity ON circular_dependencies(severity);

-- Note: current_section tracking for guided mode is handled by migration 003
