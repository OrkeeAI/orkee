---
sidebar_position: 1
---

# OpenSpec Overview

OpenSpec integration in Orkee provides a complete spec-driven development methodology, enabling teams to move from Product Requirements Documents (PRDs) to validated task execution with AI-powered assistance.

## What is OpenSpec?

OpenSpec is a structured approach to software development that emphasizes:

- **Specification-First Design** - Define what you're building before you build it
- **Testable Requirements** - Every requirement has WHEN/THEN scenarios for validation
- **Bidirectional Sync** - Tasks, specs, and PRDs stay in sync automatically
- **AI-Powered Analysis** - Extract capabilities and generate tasks from documents
- **Change Management** - Formal approval workflow for spec modifications

## Architecture Flow

```
Product Requirements Document (PRD)
    ↓ AI Analysis & Extraction
Capabilities (Functional Areas)
    ↓ Break Down Into
Requirements & Scenarios (WHEN/THEN/AND)
    ↓ Generate Implementation
Tasks (Linked to Requirements)
    ↓ Developer Creates Additional Tasks
Orphan Tasks → AI Suggests Spec Links
    ↓ Sync Back to Update
PRD (Regenerated from Current State)
```

## Core Concepts

### Product Requirements Documents (PRDs)

PRDs are high-level documents describing what you want to build. Orkee's OpenSpec implementation:

- Stores PRDs as markdown in the database
- Versions PRDs automatically
- Analyzes PRDs with AI to extract capabilities
- Regenerates PRDs from spec changes

### Capabilities

Capabilities are major functional areas of your application:

- Represent distinct features or modules (e.g., "Authentication", "User Profile")
- Contain multiple requirements
- Map to top-level sections in your architecture
- Tracked with status (active, deprecated, archived)

### Requirements

Requirements are specific functional needs within a capability:

- Each has a clear description
- Includes multiple test scenarios
- Links to implementation tasks
- Validates task completion against scenarios

### Scenarios

Scenarios are testable conditions written in WHEN/THEN/AND format:

```markdown
WHEN user submits valid login credentials
THEN user is authenticated and redirected to dashboard
AND session token is stored securely
AND login event is logged
```

### Tasks

Tasks are implementation items that:

- Link to spec requirements (or exist as orphans)
- Are validated against WHEN/THEN scenarios
- Can suggest new spec requirements when orphaned
- Track completion status

## Database Schema

OpenSpec adds 9 tables to Orkee's SQLite database:

| Table | Purpose |
|-------|---------|
| `prds` | Product requirements documents with versioning |
| `spec_capabilities` | High-level functional capabilities |
| `spec_requirements` | Individual requirements within capabilities |
| `spec_scenarios` | WHEN/THEN/AND test scenarios |
| `spec_changes` | Change proposals with approval workflow |
| `spec_deltas` | Changes to capabilities (added/modified/removed) |
| `task_spec_links` | Links between tasks and requirements |
| `prd_spec_sync_history` | Audit trail for all sync operations |
| `ai_usage_logs` | AI cost tracking and usage monitoring |

## Key Features

### AI-Powered Workflows

- **PRD Analysis** - Extract capabilities and requirements from documents
- **Task Generation** - Generate implementation tasks from specs
- **Orphan Detection** - Identify tasks without spec links
- **Spec Suggestions** - AI recommends spec requirements for orphans
- **Validation** - Verify task completion against scenarios

### Bidirectional Sync

Changes flow in both directions:

- **PRD → Spec → Task** - Traditional top-down approach
- **Task → Spec → PRD** - Bottom-up validation and updates

### Cost Tracking

Monitor AI usage across all operations:

- Per-operation cost tracking
- Token usage metrics
- Model and provider breakdown
- Historical usage trends

### Change Management

Formal workflow for spec modifications:

1. Create change proposal
2. Define spec deltas (additions, modifications, removals)
3. Review and approve
4. Apply changes to specs
5. Update PRD automatically

## Implementation Status

✅ **Production Ready** - All core features implemented and tested:

- **Database**: 9 tables with indexes and foreign keys
- **Rust Core**: 46 unit tests passing (parser, validator, sync)
- **API**: 28 REST endpoints across 5 categories
- **Frontend**: 11 React components for complete UI
- **AI Integration**: Vercel AI SDK with cost tracking
- **Workflows**: PRD→Spec→Task and Task→Spec→PRD flows

## Quick Start

Get started with OpenSpec in three steps:

1. **Upload a PRD** - Use the PRDUploadDialog to upload your requirements
2. **Review Analysis** - AI extracts capabilities and suggests tasks
3. **Start Building** - Generated tasks link to spec requirements for validation

Continue to [Getting Started](./getting-started.md) for detailed instructions.

## Related Documentation

- [PRD Management](./prds.md) - Working with Product Requirements Documents
- [Specs & Capabilities](./specs.md) - Creating and managing specifications
- [Task Integration](./tasks.md) - Linking tasks to spec requirements
- [AI Features](./ai-features.md) - AI-powered analysis and generation
- [Workflows](./workflows.md) - End-to-end development workflows
- [Cost Tracking](./cost-tracking.md) - Monitor AI usage and costs

---

**Next**: Learn how to [get started with OpenSpec](./getting-started.md)
