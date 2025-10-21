---
sidebar_position: 5
---

# Task-Spec Integration

Task-spec integration connects your implementation work to formal specifications, enabling validation, traceability, and automated sync. This guide covers linking tasks to requirements, validating completion, and managing orphan tasks.

## Understanding Task-Spec Links

### What are Task-Spec Links?

Task-spec links create bidirectional relationships between:

- **Tasks** - Implementation items (code, documentation, design)
- **Requirements** - Formal spec requirements with WHEN/THEN scenarios

Benefits of linking:
- ✅ Validate task completion against scenarios
- ✅ Track requirement coverage
- ✅ Identify orphan tasks (no spec)
- ✅ Generate tasks from specs
- ✅ Maintain traceability

### Link Data Model

```sql
CREATE TABLE task_spec_links (
    task_id TEXT NOT NULL,
    requirement_id TEXT NOT NULL,
    scenario_id TEXT,              -- Optional specific scenario
    validation_status TEXT DEFAULT 'pending',
    validation_result TEXT,         -- JSON with details
    created_at TIMESTAMP,
    updated_at TIMESTAMP,
    PRIMARY KEY (task_id, requirement_id),
    FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,
    FOREIGN KEY (requirement_id) REFERENCES spec_requirements(id) ON DELETE CASCADE
);
```

**Validation Statuses:**
- `pending` - Not yet validated
- `passed` - Validated successfully
- `failed` - Validation failed

## Linking Tasks to Requirements

### Method 1: TaskSpecLinker Component

The easiest way to link tasks:

1. Click task in task list
2. Click **"Link to Spec"** button (or TaskSpecIndicator)
3. Search for requirement:
   - Search by name
   - Filter by capability
   - Browse all requirements
4. Click **"Link"** next to requirement
5. Task is now linked

**TaskSpecLinker Features:**
- 🔍 Full-text search across all requirements
- 🗂️ Filter by capability
- 📋 Preview requirement details and scenarios
- ✅ Show existing links
- 🔗 One-click linking/unlinking

### Method 2: During Task Creation

When generating tasks from PRD analysis:

1. Upload and analyze PRD
2. Review suggested tasks
3. Click **"Generate Tasks"**
4. Tasks are automatically linked to source requirements

### Method 3: API

Link tasks programmatically:

```bash
curl -X POST http://localhost:4001/api/tasks/TASK_ID/link-spec \
  -H "Content-Type: application/json" \
  -d '{
    "requirement_id": "req_abc123",
    "scenario_id": "scenario_xyz"
  }'
```

**Response:**
```json
{
  "success": true,
  "data": {
    "task_id": "task_001",
    "requirement_id": "req_abc123",
    "scenario_id": "scenario_xyz",
    "validation_status": "pending",
    "created_at": "2025-01-20T10:00:00Z"
  }
}
```

## Validating Tasks Against Scenarios

### Manual Validation

Validate task completion against linked scenarios:

1. Click task with spec link
2. Click **"Validate"** in TaskSpecIndicator
3. Review each scenario:
   - ✅ WHEN condition met
   - ✅ THEN outcome achieved
   - ✅ AND clauses satisfied
4. Mark validation status

### AI-Powered Validation

Use AI to validate task implementation:

1. Open ValidationResultsPanel
2. Click **"Validate with AI"**
3. AI checks:
   - Code implementation vs scenarios
   - Test coverage
   - Edge case handling
4. Review validation report

**AI Validation Process:**

The AI receives:
- Requirement name and description
- All WHEN/THEN/AND scenarios
- Task description and completion notes
- Optional: Code diff or implementation details

The AI returns:
```json
{
  "overallStatus": "passed",
  "scenarioResults": [
    {
      "scenarioName": "Valid login",
      "status": "passed",
      "details": "Implementation correctly handles valid credentials",
      "suggestions": []
    },
    {
      "scenarioName": "Invalid credentials",
      "status": "failed",
      "details": "Missing rate limiting for failed attempts",
      "suggestions": [
        "Add rate limiting middleware",
        "Implement exponential backoff"
      ]
    }
  ],
  "coverage": 0.85,
  "recommendations": [
    "Add tests for edge cases",
    "Document error handling"
  ]
}
```

### Validation States

**Pending**
- Default state after linking
- Task not yet complete
- Validation not run

**Passed**
- All scenarios validated
- Requirements met
- Task can be closed

**Failed**
- One or more scenarios failed
- Requirements not fully met
- Task needs revision

### Validation API

```bash
# Validate task against linked requirements
POST /api/tasks/:task_id/validate-spec

# Get validation results
GET /api/tasks/:task_id/spec-links
```

**Response includes:**
```json
{
  "success": true,
  "data": {
    "links": [
      {
        "requirement_id": "req_1",
        "requirement_name": "User Login",
        "validation_status": "passed",
        "validation_result": {
          "scenarioResults": [...],
          "coverage": 1.0,
          "validated_at": "2025-01-20T11:00:00Z"
        }
      }
    ]
  }
}
```

## Generating Tasks from Specs

### From PRD Analysis

Automatically generate tasks when analyzing PRD:

1. Upload PRD
2. Click **"Analyze with AI"**
3. Review suggested tasks
4. Click **"Generate Tasks"**

Each task includes:
- Title and description
- Linked requirement
- Complexity score (1-10)
- Estimated hours (optional)

**Example Generated Tasks:**

From requirement "User Login":
```
Task 1: Implement login API endpoint
- Description: POST /api/auth/login with email/password validation
- Requirement: User Login
- Complexity: 5/10
- Estimated: 4 hours

Task 2: Create login form component
- Description: React form with validation and error handling
- Requirement: User Login
- Complexity: 4/10
- Estimated: 3 hours

Task 3: Add login integration tests
- Description: Test all login scenarios including errors
- Requirement: User Login
- Complexity: 6/10
- Estimated: 5 hours
```

### From Existing Spec

Generate tasks from capability:

1. Open SpecDetailsView
2. Select requirements
3. Click **"Generate Tasks"**
4. AI suggests implementation tasks

### API Endpoint

```bash
POST /api/:project_id/tasks/generate-from-spec
Content-Type: application/json

{
  "spec_id": "spec_abc",
  "requirement_ids": ["req_1", "req_2"],
  "options": {
    "includeTests": true,
    "includeDocumentation": true,
    "complexityFilter": "medium"  // low, medium, high, all
  }
}
```

**Response:**
```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "id": "task_new_1",
        "title": "Implement user registration API",
        "description": "...",
        "requirement_id": "req_1",
        "complexity": 5,
        "estimated_hours": 8
      }
    ],
    "count": 5
  }
}
```

## Managing Orphan Tasks

### What are Orphan Tasks?

Orphan tasks are tasks without spec links:
- Created manually by developers
- Not part of formal specification
- Need spec assignment for traceability

### Detecting Orphans

**Via SyncDashboard:**
1. Navigate to **Specs > Coverage** tab
2. Click **"Orphan Tasks"** section
3. View list of unlinked tasks

**Via API:**
```bash
GET /api/:project_id/tasks/orphans
```

**Response:**
```json
{
  "success": true,
  "data": {
    "orphans": [
      {
        "id": "task_orphan_1",
        "title": "Add OAuth2 support",
        "created_at": "2025-01-20T09:00:00Z",
        "age_days": 5
      }
    ],
    "count": 3,
    "total_tasks": 50,
    "orphan_percentage": 6.0
  }
}
```

### Handling Orphans

**Option 1: Link to Existing Requirement**

1. Click **"Link to Spec"** on orphan task
2. Search for appropriate requirement
3. Link task

**Option 2: AI Suggest New Requirement**

1. Click **"Suggest Spec"** on orphan task
2. AI analyzes task and suggests:
   - Add to existing capability
   - Create new capability
   - Merge with similar tasks
3. Review suggestion
4. Create spec or link as suggested

**Option 3: Create Change Proposal**

For significant features:

1. Click **"Create Change Proposal"**
2. AI generates proposal with:
   - New capability or requirement
   - WHEN/THEN scenarios
   - Related tasks
3. Submit for review
4. Approve to create spec

### AI Spec Suggestions

**API Endpoint:**
```bash
POST /api/tasks/:task_id/suggest-spec
```

**AI Analysis:**
The AI examines:
- Task title and description
- Existing capabilities in project
- Similar requirements
- Task metadata (labels, assignees, etc.)

**AI Response:**
```json
{
  "success": true,
  "data": {
    "suggestions": [
      {
        "type": "add_to_existing",
        "confidence": 0.85,
        "capability_id": "user-auth",
        "capability_name": "User Authentication",
        "new_requirement": {
          "name": "OAuth Provider Integration",
          "content": "Users can authenticate via OAuth providers",
          "scenarios": [
            {
              "name": "Google OAuth login",
              "when": "user clicks 'Login with Google'",
              "then": "OAuth flow initiates",
              "and": ["user is authenticated", "account is linked"]
            }
          ]
        }
      },
      {
        "type": "create_new_capability",
        "confidence": 0.65,
        "new_capability": {
          "name": "social-authentication",
          "purpose": "Third-party authentication providers",
          "requirements": [...]
        }
      }
    ]
  }
}
```

## Task Coverage Metrics

### Coverage Dashboard

The SyncDashboard shows:

**Spec Coverage:**
- Requirements with linked tasks: 45/50 (90%)
- Requirements with completed tasks: 30/50 (60%)
- Average tasks per requirement: 2.5

**Task Coverage:**
- Tasks with spec links: 112/120 (93%)
- Orphan tasks: 8/120 (7%)
- Validated tasks: 85/112 (76%)

### Coverage API

```bash
GET /api/:project_id/specs/coverage
```

**Response:**
```json
{
  "success": true,
  "data": {
    "overall": {
      "capabilities": 10,
      "requirements": 50,
      "scenarios": 150,
      "tasks": 120,
      "linked_tasks": 112,
      "orphan_tasks": 8
    },
    "requirements": [
      {
        "id": "req_1",
        "name": "User Login",
        "linked_tasks": 3,
        "completed_tasks": 2,
        "coverage": 0.67
      }
    ],
    "capabilities": [
      {
        "id": "cap_1",
        "name": "User Authentication",
        "requirements": 8,
        "linked_tasks": 24,
        "coverage": 1.0
      }
    ]
  }
}
```

## Best Practices

### Linking Strategy

✅ **Do:**
- Link tasks during creation (not after)
- One task per scenario (when possible)
- Link implementation AND test tasks
- Update links when requirements change

❌ **Don't:**
- Link one task to multiple requirements
- Skip linking test/documentation tasks
- Leave orphans unresolved
- Link tasks to wrong requirements

### Validation Workflow

**Recommended Process:**

1. **Before Starting Task**
   - Review linked requirement
   - Understand all scenarios
   - Note edge cases

2. **During Implementation**
   - Check scenarios regularly
   - Update task with progress notes
   - Flag unclear scenarios

3. **After Completion**
   - Run AI validation
   - Review each scenario result
   - Fix any failures
   - Mark task complete

### Orphan Management

**Weekly Review:**
- Check orphan count
- Categorize by type:
  - Missing specs (need new capability)
  - Mis-categorized (link to existing)
  - Technical debt (create change proposal)
- Set orphan target (e.g., <5%)

**Team Process:**
- Require spec link before task approval
- Review orphans in sprint planning
- AI-suggest specs for common patterns

## Troubleshooting

### Link Not Appearing

**Problem**: Task shows as linked but link not visible

**Solutions:**
- Refresh task view
- Check `task_spec_links` table
- Verify requirement still exists
- Re-create link

### Validation Always Fails

**Problem**: Validation fails despite meeting scenarios

**Solutions:**
- Review scenario wording carefully
- Check if scenarios are too strict
- Provide more context to AI validator
- Manually override validation

### Cannot Link Task

**Problem**: "Cannot link task" error

**Solutions:**
- Verify task exists
- Ensure requirement exists
- Check for existing link
- Verify permissions (future feature)

## API Reference

### Link Task to Requirement

```bash
POST /api/tasks/:task_id/link-spec
```

### Get Task Links

```bash
GET /api/tasks/:task_id/spec-links
```

### Validate Task

```bash
POST /api/tasks/:task_id/validate-spec
```

### Suggest Spec for Task

```bash
POST /api/tasks/:task_id/suggest-spec
```

### Generate Tasks from Spec

```bash
POST /api/:project_id/tasks/generate-from-spec
```

### Find Orphan Tasks

```bash
GET /api/:project_id/tasks/orphans
```

## Next Steps

- [Understand full workflows](./workflows.md)
- [Manage changes with proposals](./workflows.md#change-proposals)
- [Monitor AI usage and costs](./cost-tracking.md)
- [Explore AI features in depth](./ai-features.md)

---

**Related**: [Specs & Requirements](./specs.md) | [Workflows](./workflows.md) | [AI Features](./ai-features.md)
