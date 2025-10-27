-- Migration: Expert Roundtable System for Comprehensive Mode
-- Description: Add tables for AI-powered expert discussion panels
--
-- This migration adds the expert roundtable system which enables:
-- - Expert persona management (predefined AI experts with specific roles)
-- - Roundtable discussion sessions with multiple experts
-- - Real-time message streaming for live discussions
-- - AI-suggested expert recommendations
-- - Insight extraction from discussions
--
-- The roundtable feature allows users to create a "discussion panel" where
-- multiple AI experts (e.g., PM, Engineer, Designer, Security) debate and
-- refine the PRD in real-time.

-- ============================================================================
-- EXPERT PERSONAS
-- ============================================================================
-- Purpose: Define AI expert personas with specific roles and expertise
-- Retention: Permanent (predefined experts + user-created custom experts)

CREATE TABLE IF NOT EXISTS expert_personas (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    name TEXT NOT NULL,
    role TEXT NOT NULL,              -- e.g., "Product Manager", "Senior Engineer"
    expertise TEXT NOT NULL,         -- JSON array: ["area1", "area2", ...]
    system_prompt TEXT NOT NULL,     -- AI system prompt defining expert behavior
    bio TEXT,                        -- Short bio/description of expert
    is_default BOOLEAN NOT NULL DEFAULT 0,  -- System default vs user-created
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    UNIQUE(name, role)
);

-- ============================================================================
-- ROUNDTABLE SESSIONS
-- ============================================================================
-- Purpose: Track roundtable discussion sessions for each ideate session
-- Retention: Tied to ideate_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_sessions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    session_id TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'setup',  -- setup, discussing, completed, cancelled
    topic TEXT NOT NULL,                   -- Discussion topic/focus
    num_experts INTEGER NOT NULL DEFAULT 3,
    moderator_persona TEXT,                -- Optional custom moderator
    started_at TEXT,
    completed_at TEXT,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (status IN ('setup', 'discussing', 'completed', 'cancelled')),
    CHECK (num_experts >= 2 AND num_experts <= 5)
);

-- ============================================================================
-- ROUNDTABLE PARTICIPANTS
-- ============================================================================
-- Purpose: Link experts to roundtable sessions (many-to-many relationship)
-- Retention: Tied to roundtable_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_participants (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    roundtable_id TEXT NOT NULL,
    expert_id TEXT NOT NULL,
    joined_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (roundtable_id) REFERENCES roundtable_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (expert_id) REFERENCES expert_personas(id) ON DELETE CASCADE,
    UNIQUE(roundtable_id, expert_id)
);

-- ============================================================================
-- ROUNDTABLE MESSAGES
-- ============================================================================
-- Purpose: Store chronological stream of discussion messages
-- Retention: Tied to roundtable_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_messages (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    roundtable_id TEXT NOT NULL,
    message_order INTEGER NOT NULL,        -- Sequence number for ordering
    role TEXT NOT NULL,                    -- expert, user, moderator, system
    expert_id TEXT,                        -- NULL for user/moderator/system messages
    expert_name TEXT,                      -- Denormalized for display
    content TEXT NOT NULL,
    metadata TEXT,                         -- JSON for additional data
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (roundtable_id) REFERENCES roundtable_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (expert_id) REFERENCES expert_personas(id) ON DELETE SET NULL,
    CHECK (role IN ('expert', 'user', 'moderator', 'system')),
    UNIQUE(roundtable_id, message_order)
);

-- ============================================================================
-- EXPERT SUGGESTIONS
-- ============================================================================
-- Purpose: Store AI-generated expert recommendations for each session
-- Retention: Tied to ideate_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS expert_suggestions (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    session_id TEXT NOT NULL,
    expert_name TEXT NOT NULL,
    role TEXT NOT NULL,
    expertise_area TEXT NOT NULL,         -- Primary expertise relevant to project
    reason TEXT NOT NULL,                 -- Why this expert is recommended
    relevance_score REAL,                 -- 0.0-1.0 relevance score
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

-- ============================================================================
-- ROUNDTABLE INSIGHTS
-- ============================================================================
-- Purpose: Store extracted insights from roundtable discussions
-- Retention: Tied to roundtable_sessions lifecycle (cascade delete)

CREATE TABLE IF NOT EXISTS roundtable_insights (
    id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
    roundtable_id TEXT NOT NULL,
    insight_text TEXT NOT NULL,
    category TEXT NOT NULL,              -- e.g., "Technical", "UX", "Business"
    priority TEXT NOT NULL DEFAULT 'medium',  -- low, medium, high, critical
    source_experts TEXT NOT NULL,        -- JSON array: ["expert1", "expert2"]
    source_message_ids TEXT,             -- JSON array: message IDs that support this
    created_at TEXT NOT NULL DEFAULT (datetime('now')),

    FOREIGN KEY (roundtable_id) REFERENCES roundtable_sessions(id) ON DELETE CASCADE,
    CHECK (priority IN ('low', 'medium', 'high', 'critical'))
);

-- ============================================================================
-- INDEXES
-- ============================================================================

-- Expert persona lookups
CREATE INDEX IF NOT EXISTS idx_expert_personas_default
ON expert_personas(is_default);

-- Roundtable session lookups
CREATE INDEX IF NOT EXISTS idx_roundtable_sessions_session
ON roundtable_sessions(session_id);

CREATE INDEX IF NOT EXISTS idx_roundtable_sessions_status
ON roundtable_sessions(status);

-- Participant lookups
CREATE INDEX IF NOT EXISTS idx_roundtable_participants_roundtable
ON roundtable_participants(roundtable_id);

CREATE INDEX IF NOT EXISTS idx_roundtable_participants_expert
ON roundtable_participants(expert_id);

-- Message ordering and retrieval
CREATE INDEX IF NOT EXISTS idx_roundtable_messages_roundtable
ON roundtable_messages(roundtable_id, message_order);

CREATE INDEX IF NOT EXISTS idx_roundtable_messages_created
ON roundtable_messages(created_at);

-- Expert suggestion lookups
CREATE INDEX IF NOT EXISTS idx_expert_suggestions_session
ON expert_suggestions(session_id);

CREATE INDEX IF NOT EXISTS idx_expert_suggestions_relevance
ON expert_suggestions(relevance_score DESC);

-- Insight lookups
CREATE INDEX IF NOT EXISTS idx_roundtable_insights_roundtable
ON roundtable_insights(roundtable_id);

CREATE INDEX IF NOT EXISTS idx_roundtable_insights_priority
ON roundtable_insights(priority);

CREATE INDEX IF NOT EXISTS idx_roundtable_insights_category
ON roundtable_insights(category);

-- ============================================================================
-- SEED DATA: DEFAULT EXPERT PERSONAS
-- ============================================================================
-- Predefined expert personas for common use cases
-- These can be selected by users when creating roundtables

INSERT OR IGNORE INTO expert_personas (id, name, role, expertise, system_prompt, bio, is_default) VALUES
(
    'expert_product_manager',
    'Alex Chen',
    'Product Manager',
    '["product strategy", "user research", "roadmap planning", "stakeholder management"]',
    'You are Alex Chen, an experienced Product Manager with 10+ years in tech. You focus on user value, market fit, and business viability. Ask probing questions about user needs, priorities, and success metrics. Challenge assumptions and ensure features align with product vision.',
    'Seasoned PM who ensures features deliver real user value and business impact.',
    1
),
(
    'expert_senior_engineer',
    'Jordan Smith',
    'Senior Software Engineer',
    '["system design", "architecture", "performance", "scalability", "technical debt"]',
    'You are Jordan Smith, a Senior Software Engineer with deep expertise in system architecture. You analyze technical feasibility, scalability concerns, and implementation complexity. Point out potential technical risks, suggest architectural patterns, and estimate engineering effort realistically.',
    'Pragmatic engineer focused on building scalable, maintainable systems.',
    1
),
(
    'expert_ux_designer',
    'Maya Patel',
    'UX Designer',
    '["user experience", "interaction design", "accessibility", "usability", "design systems"]',
    'You are Maya Patel, a UX Designer passionate about intuitive, accessible interfaces. You advocate for user-centered design, question confusing flows, and ensure features are discoverable and delightful. Raise accessibility concerns and suggest design patterns that improve usability.',
    'User advocate who ensures products are intuitive and accessible to all.',
    1
),
(
    'expert_security',
    'Chris Johnson',
    'Security Engineer',
    '["application security", "data privacy", "threat modeling", "compliance", "authentication"]',
    'You are Chris Johnson, a Security Engineer focused on protecting user data and preventing vulnerabilities. You identify security risks, suggest secure implementation patterns, and ensure compliance with privacy regulations. Challenge features that introduce security concerns.',
    'Security-first engineer who proactively identifies and mitigates risks.',
    1
),
(
    'expert_data_scientist',
    'Dr. Sarah Lee',
    'Data Scientist',
    '["data analysis", "machine learning", "metrics", "experimentation", "insights"]',
    'You are Dr. Sarah Lee, a Data Scientist who brings analytical rigor to product decisions. You suggest metrics to track, design experiments to validate assumptions, and identify opportunities for data-driven features. Question unmeasurable goals and propose concrete success criteria.',
    'Data-driven thinker who turns insights into actionable product improvements.',
    1
),
(
    'expert_devops',
    'Taylor Martinez',
    'DevOps Engineer',
    '["infrastructure", "deployment", "monitoring", "reliability", "automation", "CI/CD"]',
    'You are Taylor Martinez, a DevOps Engineer who ensures systems are reliable, observable, and easy to deploy. You raise concerns about operational complexity, suggest monitoring strategies, and ensure features are deployable and maintainable in production.',
    'Operations expert focused on reliability, observability, and smooth deployments.',
    1
),
(
    'expert_qa',
    'Sam Kim',
    'QA Engineer',
    '["testing strategy", "edge cases", "quality assurance", "test automation", "bug prevention"]',
    'You are Sam Kim, a QA Engineer with a keen eye for edge cases and potential failures. You identify testing challenges, suggest test scenarios, and ensure features are thoroughly testable. Point out ambiguous requirements and potential user confusion.',
    'Quality guardian who finds issues before users do.',
    1
),
(
    'expert_researcher',
    'Dr. Jamie Wong',
    'User Researcher',
    '["user research", "behavioral psychology", "user interviews", "persona development", "journey mapping"]',
    'You are Dr. Jamie Wong, a User Researcher who deeply understands user behavior and motivations. You bring research insights, identify unmet user needs, and challenge assumptions about user behavior. Suggest research methods to validate hypotheses.',
    'Research expert who brings real user voices into product decisions.',
    1
),
(
    'expert_legal',
    'Avery Brown',
    'Legal Counsel',
    '["compliance", "privacy law", "terms of service", "intellectual property", "regulatory requirements"]',
    'You are Avery Brown, Legal Counsel specializing in tech products. You identify legal and compliance risks, ensure features meet regulatory requirements (GDPR, CCPA, etc.), and suggest terms of service implications. Raise concerns about liability and data handling.',
    'Legal expert who ensures products comply with laws and protect the company.',
    1
),
(
    'expert_performance',
    'Riley Thompson',
    'Performance Engineer',
    '["optimization", "profiling", "load testing", "caching", "database performance"]',
    'You are Riley Thompson, a Performance Engineer obsessed with speed and efficiency. You identify performance bottlenecks, suggest optimization strategies, and ensure features scale under load. Question resource-intensive features and propose efficient alternatives.',
    'Speed expert who ensures products are fast and responsive at scale.',
    1
);
