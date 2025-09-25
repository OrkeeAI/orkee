---
sidebar_position: 3
---

# Projects API

The Projects API provides full CRUD operations for managing AI agent projects in Orkee. Projects are stored in SQLite with optional cloud sync capabilities.

## Base Endpoints

All project endpoints are prefixed with `/api/projects`.

## List Projects

**GET** `/api/projects`

Returns all projects with pagination support.

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit` | number | 50 | Maximum number of projects to return |
| `offset` | number | 0 | Number of projects to skip |
| `sort` | string | `updated_at` | Sort field (`name`, `created_at`, `updated_at`) |
| `order` | string | `desc` | Sort order (`asc`, `desc`) |
| `search` | string | - | Full-text search across name, description, path |

### Example Request

```bash
curl "http://localhost:4001/api/projects?limit=10&search=react"
```

### Response

```json
{
  "success": true,
  "data": {
    "projects": [
      {
        "id": 1,
        "name": "AI Chat App",
        "description": "React-based AI chat interface",
        "path": "/Users/john/projects/ai-chat",
        "created_at": "2024-01-15T10:00:00Z",
        "updated_at": "2024-01-15T12:30:00Z",
        "last_accessed": "2024-01-15T12:30:00Z",
        "git_info": {
          "is_git_repo": true,
          "current_branch": "main",
          "remote_url": "https://github.com/user/ai-chat.git",
          "is_dirty": false,
          "ahead": 2,
          "behind": 0
        },
        "tags": ["react", "ai", "typescript"],
        "scripts": {
          "dev": "npm run dev",
          "build": "npm run build",
          "test": "npm test"
        },
        "metadata": {
          "framework": "react",
          "language": "typescript"
        }
      }
    ],
    "total": 1,
    "limit": 10,
    "offset": 0
  }
}
```

## Get Project by ID

**GET** `/api/projects/:id`

Returns a specific project by its ID.

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | number | Project ID |

### Example Request

```bash
curl "http://localhost:4001/api/projects/1"
```

### Response

```json
{
  "success": true,
  "data": {
    "id": 1,
    "name": "AI Chat App",
    "description": "React-based AI chat interface",
    "path": "/Users/john/projects/ai-chat",
    "created_at": "2024-01-15T10:00:00Z",
    "updated_at": "2024-01-15T12:30:00Z",
    "last_accessed": "2024-01-15T12:30:00Z",
    "git_info": {
      "is_git_repo": true,
      "current_branch": "main",
      "remote_url": "https://github.com/user/ai-chat.git",
      "is_dirty": false,
      "ahead": 2,
      "behind": 0
    },
    "tags": ["react", "ai", "typescript"],
    "scripts": {
      "dev": "npm run dev",
      "build": "npm run build",
      "test": "npm test"
    },
    "metadata": {
      "framework": "react",
      "language": "typescript"
    }
  }
}
```

## Get Project by Name

**GET** `/api/projects/by-name/:name`

Returns a project by its name (case-insensitive).

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | string | Project name (URL encoded) |

### Example Request

```bash
curl "http://localhost:4001/api/projects/by-name/AI%20Chat%20App"
```

## Get Project by Path

**POST** `/api/projects/by-path`

Returns a project that matches the given file system path.

### Request Body

```json
{
  "projectRoot": "/Users/john/projects/ai-chat"
}
```

### Example Request

```bash
curl -X POST "http://localhost:4001/api/projects/by-path" \
  -H "Content-Type: application/json" \
  -d '{"projectRoot": "/Users/john/projects/ai-chat"}'
```

## Create Project

**POST** `/api/projects`

Creates a new project. Git information is automatically detected if the path is a Git repository.

### Request Body

```json
{
  "name": "New AI Project",
  "description": "A new AI project for testing",
  "path": "/Users/john/projects/new-ai-project",
  "tags": ["ai", "experimental"],
  "scripts": {
    "dev": "npm run dev",
    "build": "npm run build"
  },
  "metadata": {
    "framework": "next",
    "language": "typescript"
  }
}
```

### Required Fields

- `name`: Project name (must be unique)
- `path`: Absolute path to project directory

### Optional Fields

- `description`: Project description
- `tags`: Array of tag strings
- `scripts`: Object with script name/command pairs
- `metadata`: Object with custom metadata

### Example Request

```bash
curl -X POST "http://localhost:4001/api/projects" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "New AI Project",
    "path": "/Users/john/projects/new-ai-project",
    "description": "A new project for AI experiments"
  }'
```

### Response

```json
{
  "success": true,
  "data": {
    "id": 2,
    "name": "New AI Project",
    "description": "A new project for AI experiments",
    "path": "/Users/john/projects/new-ai-project",
    "created_at": "2024-01-15T14:00:00Z",
    "updated_at": "2024-01-15T14:00:00Z",
    "last_accessed": "2024-01-15T14:00:00Z",
    "git_info": {
      "is_git_repo": false
    },
    "tags": [],
    "scripts": {},
    "metadata": {}
  }
}
```

## Update Project

**PUT** `/api/projects/:id`

Updates an existing project. Only provided fields will be updated.

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | number | Project ID |

### Request Body

```json
{
  "name": "Updated Project Name",
  "description": "Updated description",
  "tags": ["updated", "ai", "react"],
  "scripts": {
    "dev": "pnpm dev",
    "build": "pnpm build",
    "test": "pnpm test"
  }
}
```

### Example Request

```bash
curl -X PUT "http://localhost:4001/api/projects/1" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Updated project description",
    "tags": ["react", "ai", "updated"]
  }'
```

### Response

Returns the updated project object with the same structure as GET `/api/projects/:id`.

## Delete Project

**DELETE** `/api/projects/:id`

Deletes a project from the database. This only removes the project record, not the actual files.

### Path Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | number | Project ID |

### Example Request

```bash
curl -X DELETE "http://localhost:4001/api/projects/1"
```

### Response

```json
{
  "success": true,
  "data": {
    "message": "Project deleted successfully",
    "deleted_id": 1
  }
}
```

## Data Types

### Project Object

| Field | Type | Description |
|-------|------|-------------|
| `id` | number | Unique project identifier |
| `name` | string | Project name (unique) |
| `description` | string \| null | Project description |
| `path` | string | Absolute file system path |
| `created_at` | string | ISO timestamp of creation |
| `updated_at` | string | ISO timestamp of last update |
| `last_accessed` | string | ISO timestamp of last access |
| `git_info` | GitInfo | Git repository information |
| `tags` | string[] | Array of tag strings |
| `scripts` | object | Script name/command pairs |
| `metadata` | object | Custom metadata |

### GitInfo Object

| Field | Type | Description |
|-------|------|-------------|
| `is_git_repo` | boolean | Whether directory is a Git repository |
| `current_branch` | string \| null | Current Git branch name |
| `remote_url` | string \| null | Remote origin URL |
| `is_dirty` | boolean | Whether working directory has changes |
| `ahead` | number | Commits ahead of remote |
| `behind` | number | Commits behind remote |

## Error Responses

### Project Not Found

**Status**: 404 Not Found

```json
{
  "success": false,
  "error": {
    "code": "PROJECT_NOT_FOUND",
    "message": "Project with ID 999 not found"
  }
}
```

### Validation Error

**Status**: 422 Unprocessable Entity

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid project data",
    "details": {
      "name": "Project name is required",
      "path": "Path must be an absolute path"
    }
  }
}
```

### Duplicate Project

**Status**: 409 Conflict

```json
{
  "success": false,
  "error": {
    "code": "DUPLICATE_PROJECT",
    "message": "Project with name 'AI Chat App' already exists"
  }
}
```

### Path Not Found

**Status**: 400 Bad Request

```json
{
  "success": false,
  "error": {
    "code": "PATH_NOT_FOUND",
    "message": "Project path '/invalid/path' does not exist"
  }
}
```

## Full-Text Search

The projects API supports full-text search using SQLite's FTS5 extension.

### Search Syntax

- **Simple**: `search=react` - Finds projects containing "react"
- **Multiple terms**: `search=react typescript` - Finds projects with both terms
- **Phrase search**: `search="ai chat"` - Finds exact phrase
- **Prefix search**: `search=react*` - Finds terms starting with "react"
- **Boolean operators**: `search=react AND typescript` - Boolean logic

### Search Fields

- Project name
- Project description
- File system path
- Tags (joined as text)

### Example Searches

```bash
# Find React projects
curl "http://localhost:4001/api/projects?search=react"

# Find TypeScript AI projects
curl "http://localhost:4001/api/projects?search=typescript%20AND%20ai"

# Find projects in specific directory
curl "http://localhost:4001/api/projects?search=Users/john/work*"
```

## Rate Limiting

Projects API endpoints have the following rate limits:

- GET requests: 30 requests per minute
- POST/PUT requests: 10 requests per minute
- DELETE requests: 5 requests per minute

Rate limit headers are included in all responses:

```http
X-RateLimit-Limit: 30
X-RateLimit-Remaining: 29
X-RateLimit-Reset: 1642248600
```

## Pagination

List endpoints support cursor-based pagination:

```json
{
  "success": true,
  "data": {
    "projects": [...],
    "total": 100,
    "limit": 20,
    "offset": 40,
    "has_more": true,
    "next_offset": 60
  }
}
```

Use the `next_offset` value for subsequent requests to get the next page.