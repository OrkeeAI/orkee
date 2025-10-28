-- ABOUTME: PRD output template management
-- ABOUTME: Stores markdown templates for formatting generated PRD content

-- Enable foreign keys
PRAGMA foreign_keys = ON;

-- ============================================================================
-- PRD OUTPUT TEMPLATES
-- ============================================================================

-- Store user-defined markdown templates for PRD formatting
CREATE TABLE prd_output_templates (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),

    -- Template metadata
    name TEXT NOT NULL,
    description TEXT,

    -- Template content (markdown format)
    content TEXT NOT NULL,

    -- Template settings
    is_default INTEGER NOT NULL DEFAULT 0, -- Boolean: is this the default template?

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (datetime('now', 'utc')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now', 'utc'))
);

-- Indexes for performance
CREATE INDEX idx_prd_output_templates_name ON prd_output_templates(name);
CREATE INDEX idx_prd_output_templates_default ON prd_output_templates(is_default);
CREATE INDEX idx_prd_output_templates_created ON prd_output_templates(created_at DESC);

-- Auto-update updated_at timestamp
CREATE TRIGGER prd_output_templates_updated_at AFTER UPDATE ON prd_output_templates
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE prd_output_templates SET updated_at = datetime('now', 'utc') WHERE id = NEW.id;
END;

-- ============================================================================
-- SEED DATA: Default Template
-- ============================================================================

-- Standard PRD template (idempotent insert)
INSERT OR IGNORE INTO prd_output_templates (id, name, description, content, is_default)
VALUES (
    'standard',
    'Standard PRD',
    'Default template for general product requirements',
    '# Product Requirements Document

## Overview

**Problem Statement**: [Describe the problem]

**Target Audience**: [Who are the users?]

**Value Proposition**: [Why is this solution better?]

## Core Features

### Feature 1
- **What**: [Description]
- **Why**: [Importance]
- **How**: [Implementation approach]

## Technical Architecture

[Technical details]

## User Experience

### Personas
[User personas]

### User Flows
[Key user journeys]

## Roadmap

### MVP Scope
[Minimum viable features]

### Future Phases
[Post-MVP features]

## Risks & Mitigations

[Potential risks and how to address them]
',
    1
);
