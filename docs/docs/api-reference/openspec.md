---
sidebar_position: 5
---

# OpenSpec API Reference

REST API endpoints for managing OpenSpec changes, specifications, and workflows.

## Base URL

```
http://localhost:4001/api
```

## Response Format

All endpoints return JSON with this structure:

```json
{
  "success": boolean,
  "data": any | null,
  "error": string | null
}
```

---

## Changes

Manage OpenSpec change proposals.

### List Changes

Get all changes for a project.

**Endpoint:** `GET /changes`

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `project_id` | string | Yes | Project ID to filter changes |
| `status` | string | No | Filter by status (draft, review, approved, implementing, completed, archived) |

**Example Request:**

```bash
curl http://localhost:4001/api/changes?project_id=my-project
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "add-user-auth-1",
      "project_id": "my-project",
      "prd_id": "prd-123",
      "proposal_markdown": "## Proposal\nAdd user authentication...",
      "tasks_markdown": "## Tasks\n1. Create user model...",
      "design_markdown": null,
      "status": "draft",
      "created_by": "user-456",
      "created_at": "2025-01-20T10:30:15Z",
      "updated_at": "2025-01-20T10:30:15Z",
      "archived_at": null,
      "validation_status": "valid",
      "validation_errors": null,
      "tasks_completion_percentage": 0,
      "tasks_total_count": 12,
      "tasks_completed_count": 0
    }
  ],
  "error": null
}
```

---

### Get Change

Get details of a specific change.

**Endpoint:** `GET /changes/:id`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Example Request:**

```bash
curl http://localhost:4001/api/changes/add-user-auth-1
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "add-user-auth-1",
    "project_id": "my-project",
    "prd_id": "prd-123",
    "proposal_markdown": "## Proposal\nAdd user authentication system...",
    "tasks_markdown": "## Tasks\n1. Create user model\n2. Add JWT tokens...",
    "design_markdown": "## Design\n### Architecture\n...",
    "status": "draft",
    "created_by": "user-456",
    "created_at": "2025-01-20T10:30:15Z",
    "updated_at": "2025-01-20T10:30:15Z"
  },
  "error": null
}
```

---

### Get Change Deltas

Get all spec deltas for a change.

**Endpoint:** `GET /changes/:id/deltas`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Example Request:**

```bash
curl http://localhost:4001/api/changes/add-user-auth-1/deltas
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "delta-789",
      "change_id": "add-user-auth-1",
      "capability_id": null,
      "capability_name": "user-authentication",
      "delta_type": "added",
      "delta_markdown": "## ADDED Requirements\n\n### Requirement: User Registration\n...",
      "created_at": "2025-01-20T10:30:15Z"
    }
  ],
  "error": null
}
```

---

### Update Change Status

Update the status of a change.

**Endpoint:** `PUT /changes/:id/status`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Request Body:**

```json
{
  "status": "review",
  "notes": "Ready for review"
}
```

**Status Values:**
- `draft` - Initial state
- `review` - Under review
- `approved` - Approved for implementation
- `implementing` - Work in progress
- `completed` - Implementation complete
- `archived` - Archived and deltas applied

**Example Request:**

```bash
curl -X PUT http://localhost:4001/api/changes/add-user-auth-1/status \
  -H "Content-Type: application/json" \
  -d '{"status": "review", "notes": "Ready for review"}'
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "add-user-auth-1",
    "status": "review",
    "updated_at": "2025-01-20T14:30:00Z"
  },
  "error": null
}
```

---

### Validate Change

Validate a change against OpenSpec format.

**Endpoint:** `POST /changes/:id/validate`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Request Body:**

```json
{
  "strict": false
}
```

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/changes/add-user-auth-1/validate \
  -H "Content-Type: application/json" \
  -d '{"strict": true}'
```

**Example Response (Success):**

```json
{
  "success": true,
  "data": {
    "valid": true,
    "errors": []
  },
  "error": null
}
```

**Example Response (Errors):**

```json
{
  "success": true,
  "data": {
    "valid": false,
    "errors": [
      {
        "line": 15,
        "error_type": "MissingScenarioHeader",
        "message": "Scenarios must use '#### Scenario: [Name]' format"
      },
      {
        "line": 23,
        "error_type": "InvalidScenarioFormat",
        "message": "Scenarios must use '- **WHEN** ...' and '- **THEN** ...' format"
      }
    ]
  },
  "error": null
}
```

---

### Archive Change

Archive a change and apply its deltas.

**Endpoint:** `POST /changes/:id/archive`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Request Body:**

```json
{
  "apply_specs": true
}
```

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/changes/add-user-auth-1/archive \
  -H "Content-Type: application/json" \
  -d '{"apply_specs": true}'
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "id": "add-user-auth-1",
    "status": "archived",
    "archived_at": "2025-01-20T16:00:00Z",
    "capabilities_created": ["cap-user-auth"],
    "requirements_created": 4,
    "scenarios_created": 8
  },
  "error": null
}
```

---

## Tasks

Manage tasks within changes.

### Get Change Tasks

Get all tasks for a change with completion status.

**Endpoint:** `GET /changes/:id/tasks`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Example Request:**

```bash
curl http://localhost:4001/api/changes/add-user-auth-1/tasks
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "task-001",
      "change_id": "add-user-auth-1",
      "task_number": "1",
      "task_text": "Create user model with encrypted password field",
      "is_completed": false,
      "completed_by": null,
      "completed_at": null,
      "display_order": 0,
      "parent_number": null,
      "created_at": "2025-01-20T10:30:15Z",
      "updated_at": "2025-01-20T10:30:15Z"
    },
    {
      "id": "task-002",
      "change_id": "add-user-auth-1",
      "task_number": "1.1",
      "task_text": "Add password hashing with bcrypt",
      "is_completed": false,
      "completed_by": null,
      "completed_at": null,
      "display_order": 1,
      "parent_number": "1",
      "created_at": "2025-01-20T10:30:15Z",
      "updated_at": "2025-01-20T10:30:15Z"
    }
  ],
  "error": null
}
```

---

### Toggle Task Completion

Mark a task as completed or not completed.

**Endpoint:** `POST /changes/:change_id/tasks/:task_id/toggle`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `change_id` | string | Change ID |
| `task_id` | string | Task ID |

**Request Body:**

```json
{
  "completed_by": "user-456"
}
```

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/changes/add-user-auth-1/tasks/task-001/toggle \
  -H "Content-Type: application/json" \
  -d '{"completed_by": "user-456"}'
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "task_id": "task-001",
    "is_completed": true,
    "completed_by": "user-456",
    "completed_at": "2025-01-20T15:30:00Z",
    "change_completion_percentage": 25
  },
  "error": null
}
```

---

### Refresh Task List

Re-parse tasks from the change's tasks_markdown field.

**Endpoint:** `POST /changes/:id/tasks/refresh`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/changes/add-user-auth-1/tasks/refresh
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "tasks_parsed": 12,
    "tasks_total": 12,
    "tasks_completed": 3,
    "completion_percentage": 25,
    "parsed_at": "2025-01-20T16:00:00Z"
  },
  "error": null
}
```

---

## PRDs

Manage Product Requirements Documents.

### Analyze PRD

Analyze a PRD and create a change proposal.

**Endpoint:** `POST /prds/:id/analyze`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | PRD ID |

**Request Body:**

```json
{
  "create_change": true
}
```

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/prds/prd-123/analyze \
  -H "Content-Type: application/json" \
  -d '{"create_change": true}'
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "analysis": {
      "summary": "Add user authentication system with JWT tokens...",
      "capabilities": [
        {
          "name": "user-authentication",
          "purpose": "Secure user authentication and session management",
          "complexity": 7,
          "priority": "high"
        }
      ],
      "suggested_tasks": [
        "Create user model with encrypted password field",
        "Implement JWT token generation and validation"
      ]
    },
    "change_id": "add-user-auth-1",
    "validation_status": "valid",
    "validation_errors": []
  },
  "error": null
}
```

---

## Specifications

Manage capabilities and requirements.

### Export Specs

Export OpenSpec structure to filesystem.

**Endpoint:** `POST /specs/export`

**Request Body:**

```json
{
  "project_id": "my-project",
  "path": "./openspec"
}
```

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/specs/export \
  -H "Content-Type: application/json" \
  -d '{"project_id": "my-project", "path": "./openspec"}'
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "exported_specs": 3,
    "exported_changes": 2,
    "exported_archive": 1,
    "path": "./openspec"
  },
  "error": null
}
```

---

### Import Specs

Import OpenSpec structure from filesystem.

**Endpoint:** `POST /specs/import`

**Request Body:**

```json
{
  "project_id": "my-project",
  "path": "./openspec",
  "strategy": "PreferRemote"
}
```

**Strategy Values:**
- `PreferLocal` - Keep existing database values
- `PreferRemote` - Overwrite with file values
- `Manual` - Fail on conflicts

**Example Request:**

```bash
curl -X POST http://localhost:4001/api/specs/import \
  -H "Content-Type: application/json" \
  -d '{"project_id": "my-project", "path": "./openspec", "strategy": "PreferRemote"}'
```

**Example Response:**

```json
{
  "success": true,
  "data": {
    "imported_specs": 3,
    "imported_changes": 2,
    "imported_archive": 1,
    "conflicts_resolved": 0
  },
  "error": null
}
```

---

## Approval History

Track approval workflow changes.

### Get Approval History

Get all status transitions and approvals for a change.

**Endpoint:** `GET /changes/:id/approvals`

**Path Parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | string | Change ID |

**Example Request:**

```bash
curl http://localhost:4001/api/changes/add-user-auth-1/approvals
```

**Example Response:**

```json
{
  "success": true,
  "data": [
    {
      "id": "approval-001",
      "change_id": "add-user-auth-1",
      "action": "status_change",
      "from_status": "draft",
      "to_status": "review",
      "actor": "user-456",
      "notes": "Ready for review",
      "created_at": "2025-01-20T14:00:00Z"
    },
    {
      "id": "approval-002",
      "change_id": "add-user-auth-1",
      "action": "status_change",
      "from_status": "review",
      "to_status": "approved",
      "actor": "user-789",
      "notes": "Looks good, approved",
      "created_at": "2025-01-20T15:00:00Z"
    }
  ],
  "error": null
}
```

---

## Error Codes

| Status Code | Meaning |
|-------------|---------|
| 200 | Success |
| 400 | Bad Request - Invalid parameters or validation failed |
| 404 | Not Found - Change, PRD, or resource not found |
| 409 | Conflict - Cannot perform operation (e.g., already archived) |
| 500 | Internal Server Error |

### Error Response Format

```json
{
  "success": false,
  "data": null,
  "error": "Validation failed: Scenarios must use '#### Scenario:' format"
}
```

---

## Rate Limiting

OpenSpec endpoints are subject to rate limiting:

- **Projects API**: 30 requests per minute
- **Preview Operations**: 10 requests per minute

When rate limit is exceeded, the API returns:

```json
{
  "success": false,
  "data": null,
  "error": "Rate limit exceeded. Please retry after 60 seconds."
}
```

---

## Authentication

Currently, the API does not require authentication when running locally. For production deployments, see the [Security Documentation](../security/overview.md).

---

## WebSocket Events (Future)

Real-time updates for change status and task completion will be available via WebSocket in a future release.

---

## See Also

- [CLI Reference](../openspec/cli-reference.md) - Command-line interface
- [OpenSpec Overview](../openspec/overview.md) - Core concepts
- [Workflows](../openspec/workflows.md) - End-to-end workflows
- [Troubleshooting](../openspec/troubleshooting.md) - Common issues
