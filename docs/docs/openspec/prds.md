---
sidebar_position: 3
---

# Working with PRDs

Product Requirements Documents (PRDs) are the foundation of the OpenSpec workflow. This guide covers everything you need to know about creating, managing, and analyzing PRDs in Orkee.

## What is a PRD?

A PRD (Product Requirements Document) is a high-level document that describes:

- **What** you're building (features, functionality)
- **Why** you're building it (business goals, user needs)
- **Who** it's for (target users, stakeholders)
- **How** it should work (user flows, acceptance criteria)

In Orkee's OpenSpec implementation, PRDs are:
- Stored as markdown in the SQLite database
- Versioned automatically on updates
- Analyzed by AI to extract capabilities
- Regenerated from spec changes

## PRD Data Model

### Database Schema

```sql
CREATE TABLE prds (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    content_markdown TEXT NOT NULL,
    version INTEGER DEFAULT 1,
    status TEXT DEFAULT 'draft',  -- draft, approved, superseded
    source TEXT DEFAULT 'manual', -- manual, generated, synced
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    created_by TEXT,
    FOREIGN KEY (project_id) REFERENCES projects(id)
);
```

### Status Lifecycle

- **draft** - Initial state, editable
- **approved** - Locked for implementation
- **superseded** - Replaced by newer version

### Source Types

- **manual** - User-created PRD
- **generated** - AI-regenerated from specs
- **synced** - Updated via sync operation

## Creating PRDs

### Method 1: Upload via Dashboard

The easiest way to create a PRD:

1. Navigate to project **Specs** tab
2. Click **"Upload PRD"**
3. Choose method:
   - **Paste Markdown** - Copy/paste from your editor
   - **Upload File** - Select `.md` or `.markdown` file

**Supported formats:**
- `.md` (Markdown)
- `.markdown` (Markdown)
- `.txt` (Plain text, treated as markdown)

### Method 2: API Endpoint

Create PRD programmatically:

```bash
curl -X POST http://localhost:4001/api/projects/PROJECT_ID/prds \
  -H "Content-Type: application/json" \
  -d '{
    "title": "User Authentication System",
    "content_markdown": "# Overview\nBuild secure authentication...",
    "status": "draft"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "prd_abc123",
    "project_id": "proj_xyz",
    "title": "User Authentication System",
    "version": 1,
    "status": "draft",
    "created_at": "2025-01-20T10:00:00Z"
  }
}
```

## PRD Structure Best Practices

### Template Structure

```markdown
# [Feature/System Name]

## Overview
Brief description of what you're building and why.

## Goals
- Primary business objective
- Secondary objectives
- Success metrics

## Target Users
- User persona 1: Description
- User persona 2: Description

## Requirements

### Functional Requirements

#### [Capability 1 Name]
Description of capability

**User Stories:**
- As a [user], I want to [action] so that [benefit]
- As a [user], I want to [action] so that [benefit]

**Acceptance Criteria:**
- Given [context], when [action], then [outcome]
- Given [context], when [action], then [outcome]

#### [Capability 2 Name]
...

### Non-Functional Requirements
- Performance: [specific metrics]
- Security: [requirements]
- Scalability: [requirements]
- Accessibility: [requirements]

## Out of Scope
Explicitly list what this PRD does NOT cover.

## Dependencies
- External services needed
- Third-party libraries
- Team dependencies

## Timeline
- Phase 1: [date range] - [deliverables]
- Phase 2: [date range] - [deliverables]

## Success Metrics
- Metric 1: [target value]
- Metric 2: [target value]
```

### Real Example

```markdown
# User Profile Management System

## Overview
Build a comprehensive user profile system that allows users to create,
update, and customize their personal information and preferences.

## Goals
- Reduce customer support tickets related to account settings (target: -40%)
- Increase user engagement through personalization (target: +25%)
- Provide GDPR-compliant data management

## Target Users
- End Users: Need to manage their personal information
- Admin Users: Need to view and moderate user profiles
- Support Staff: Need to assist users with profile issues

## Requirements

### Functional Requirements

#### Profile Creation
Users can create and customize their profile information.

**User Stories:**
- As a new user, I want to create my profile during onboarding
- As an existing user, I want to add optional information later
- As a user, I want to choose what information is public vs private

**Acceptance Criteria:**
- WHEN user completes registration THEN basic profile is created
- WHEN user visits profile settings THEN all fields are editable
- WHEN user saves changes THEN updates are reflected immediately
- WHEN required fields are missing THEN validation errors are shown

#### Profile Search
Users can search for and discover other user profiles.

**User Stories:**
- As a user, I want to search for other users by name or username
- As a user, I want to filter search results by location or interests
- As a user, I want to see profile preview cards in search results

**Acceptance Criteria:**
- WHEN user enters search query THEN results appear in <500ms
- WHEN user applies filters THEN results update automatically
- WHEN user clicks profile THEN full profile page opens
- WHEN profile is private THEN only public info is shown in search

### Non-Functional Requirements

**Performance:**
- Profile page load time: <2 seconds (p95)
- Search results: <500ms (p95)
- Profile updates: <1 second

**Security:**
- All profile data encrypted at rest
- Privacy controls per field
- Audit log for profile changes
- Rate limiting on profile views (prevent scraping)

**Scalability:**
- Support 1M+ profiles
- Handle 10k concurrent profile edits
- Search index updated in real-time

**Accessibility:**
- WCAG 2.1 AA compliance
- Screen reader friendly
- Keyboard navigation support

## Out of Scope
- Social features (following, messaging) - separate PRD
- Profile analytics/insights - Phase 2
- Integration with third-party identity providers - separate PRD

## Dependencies
- Authentication system (already implemented)
- File upload service for profile photos
- Search infrastructure (Elasticsearch)

## Timeline
- Phase 1 (Weeks 1-4): Basic profile CRUD
- Phase 2 (Weeks 5-6): Search functionality
- Phase 3 (Weeks 7-8): Privacy controls and admin features

## Success Metrics
- Profile completion rate: 70%
- Average profile views per user: 5/month
- Support tickets for profile issues: <10/week
```

## Analyzing PRDs with AI

### Starting Analysis

Once a PRD is uploaded:

1. Click **"Analyze with AI"** in the PRD view
2. Wait for AI processing (typically 10-30 seconds)
3. Review extracted capabilities and requirements

### What AI Extracts

The AI analysis produces:

**Capabilities** - Major functional areas:
```json
{
  "id": "profile-creation",
  "name": "Profile Creation",
  "purpose": "Allow users to create and customize profiles",
  "requirements": [...]
}
```

**Requirements** - Specific needs with scenarios:
```json
{
  "name": "User Registration",
  "content": "Users can create profile during onboarding",
  "scenarios": [
    {
      "name": "Valid registration",
      "when": "user completes registration",
      "then": "basic profile is created",
      "and": ["confirmation email is sent", "user is redirected to dashboard"]
    }
  ]
}
```

**Task Suggestions** - Implementation items:
```json
{
  "title": "Implement profile creation API endpoint",
  "description": "POST /api/profiles with validation",
  "capabilityId": "profile-creation",
  "requirementName": "User Registration",
  "complexity": 5,
  "estimatedHours": 8
}
```

### AI Analysis Configuration

Configure AI behavior:

```bash
# ~/.orkee/.env

# Provider selection (anthropic or openai)
VITE_AI_PROVIDER=anthropic

# Model selection
VITE_AI_MODEL=claude-3-5-sonnet-20241022

# Temperature (0.0-1.0, higher = more creative)
AI_TEMPERATURE=0.7

# Max tokens for response
AI_MAX_TOKENS=4096
```

## Managing PRD Versions

### Viewing Version History

PRDs are versioned automatically:

```bash
# API: Get all versions of a PRD
curl http://localhost:4001/api/projects/PROJECT_ID/prds
```

Each update increments the version number:
- v1: Initial creation
- v2: First update
- v3: Second update, etc.

### Comparing Versions

Use the SpecDiffViewer component to compare versions:

1. Select two versions in the PRD view
2. Click **"Compare Versions"**
3. See side-by-side diff with changes highlighted

### Reverting to Previous Version

To revert:

1. View previous version in PRD list
2. Click **"Restore This Version"**
3. Confirm restoration

:::warning
Restoring a previous version creates a new version (doesn't delete history). For example, reverting v3 to v1 creates v4 with v1's content.
:::

## PRD Regeneration

When specs change, the PRD can be regenerated to reflect current state.

### When to Regenerate

Regenerate PRD when:
- Multiple spec capabilities have been added
- Requirements have significantly changed
- You want to document current implementation state

### How to Regenerate

**Via Dashboard:**
1. Navigate to **Specs > Coverage** tab
2. Click **"Sync PRD"** in PRD Sync section
3. Choose regeneration mode:
   - **Replace** - Completely replace PRD with generated version
   - **Append** - Add new sections to existing PRD
   - **Merge** - Intelligent merge of changes

**Via API:**
```bash
curl -X POST http://localhost:4001/api/projects/PROJECT_ID/prds/PRD_ID/sync \
  -H "Content-Type: application/json" \
  -d '{"mode": "replace"}'
```

### Generated PRD Format

Regenerated PRDs follow this structure:

```markdown
# [Project Name] - Requirements

*Auto-generated from current specifications*

## Overview
[Summary of capabilities]

## Capabilities

### [Capability 1]
[Purpose]

#### Requirements

##### [Requirement 1]
[Description]

**Scenarios:**
- WHEN [condition] THEN [outcome]
- WHEN [condition] THEN [outcome]

**Linked Tasks:**
- [x] Task 1 (completed)
- [ ] Task 2 (in progress)

##### [Requirement 2]
...

### [Capability 2]
...

## Statistics
- Total Capabilities: X
- Total Requirements: Y
- Total Scenarios: Z
- Total Tasks: A
- Completed Tasks: B
- Coverage: C%
```

## API Reference

### List PRDs

```bash
GET /api/projects/:project_id/prds
```

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "id": "prd_1",
      "title": "User Authentication",
      "version": 2,
      "status": "approved",
      "created_at": "2025-01-20T10:00:00Z"
    }
  ]
}
```

### Get PRD

```bash
GET /api/projects/:project_id/prds/:prd_id
```

**Response:**
```json
{
  "success": true,
  "data": {
    "id": "prd_1",
    "title": "User Authentication",
    "content_markdown": "# Overview\n...",
    "version": 2,
    "status": "approved"
  }
}
```

### Create PRD

```bash
POST /api/projects/:project_id/prds
Content-Type: application/json

{
  "title": "New Feature",
  "content_markdown": "# Overview\n...",
  "status": "draft"
}
```

### Update PRD

```bash
PUT /api/projects/:project_id/prds/:prd_id
Content-Type: application/json

{
  "title": "Updated Title",
  "content_markdown": "# New content\n...",
  "status": "approved"
}
```

### Delete PRD

```bash
DELETE /api/projects/:project_id/prds/:prd_id
```

### Analyze PRD

```bash
POST /api/projects/:project_id/prds/:prd_id/analyze
```

**Response:**
```json
{
  "success": true,
  "data": {
    "summary": "Authentication system with login/logout",
    "capabilities": [...],
    "suggestedTasks": [...],
    "dependencies": [...]
  }
}
```

## Best Practices

### Writing Effective PRDs

✅ **Be Specific**
- Use concrete examples
- Include acceptance criteria
- Define success metrics

✅ **Stay High-Level**
- Focus on "what", not "how"
- Avoid implementation details
- Let specs define technical approach

✅ **Include Context**
- Explain why feature is needed
- Describe target users
- Note dependencies and constraints

### PRD Maintenance

**Regular Updates**: Keep PRDs current as project evolves

**Version Control**: Always increment version, never delete history

**Team Review**: Have stakeholders review before marking "approved"

**Documentation**: Link to related design docs, user research

### AI Analysis Tips

**Better PRD Structure = Better AI Results**

The AI performs best when PRDs:
- Have clear section headers
- Use consistent formatting
- Include acceptance criteria
- Separate concerns into sections

**Review AI Output**: Always review and edit AI-extracted capabilities before creating specs.

## Troubleshooting

### PRD Upload Fails

**Problem**: "Failed to upload PRD" error

**Solutions:**
- Check file format (.md, .markdown, .txt only)
- Verify file size (<10MB)
- Ensure valid UTF-8 encoding
- Try pasting content instead of uploading

### AI Analysis Errors

**Problem**: "AI analysis failed"

**Solutions:**
- Verify `ANTHROPIC_API_KEY` is configured
- Check AI usage dashboard for error details
- Try analyzing smaller sections
- Simplify markdown formatting

### Version Conflicts

**Problem**: "Version conflict" when updating

**Solutions:**
- Refresh to get latest version
- Resolve conflicts manually
- Create new PRD if drastically different

## Next Steps

- [Create and manage specs](./specs.md)
- [Generate tasks from specs](./tasks.md)
- [Understand full workflows](./workflows.md)
- [Monitor AI costs](./cost-tracking.md)

---

**Related**: [OpenSpec Overview](./overview.md) | [Getting Started](./getting-started.md)
