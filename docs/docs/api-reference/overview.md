---
sidebar_position: 1
---

# API Reference Overview

Orkee provides a comprehensive REST API that enables programmatic access to all core functionality. The API follows RESTful conventions and returns JSON responses with consistent error handling.

## Base URL

```
http://localhost:4001/api
```

The default API server runs on port `4001`, configurable via `ORKEE_API_PORT` environment variable.

## Authentication

Currently, the API operates without authentication for local development. Future versions will include:
- API key authentication
- JWT token support
- Role-based access control

## Request Format

### Headers

All requests should include:

```http
Content-Type: application/json
Accept: application/json
```

### Request Body

POST and PUT requests expect JSON payloads:

```json
{
  "name": "My Project",
  "description": "Project description"
}
```

## Response Format

All API responses follow a consistent structure:

### Success Response

```json
{
  "success": true,
  "data": {
    // Response data here
  }
}
```

### Error Response

```json
{
  "success": false,
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid request parameters",
    "details": {
      "field": "name",
      "issue": "required"
    }
  }
}
```

## HTTP Status Codes

| Code | Meaning | Description |
|------|---------|-------------|
| 200 | OK | Request successful |
| 201 | Created | Resource created successfully |
| 400 | Bad Request | Invalid request parameters |
| 404 | Not Found | Resource not found |
| 422 | Unprocessable Entity | Validation failed |
| 500 | Internal Server Error | Server error occurred |

## Rate Limiting

Development servers have no rate limiting. Production deployments include:
- 1000 requests per minute per IP
- Burst allowance of 100 requests
- Rate limit headers in responses

## CORS Support

The API includes automatic CORS configuration:
- Development: Allows all localhost origins (3000-8999)
- Production: Configurable via `ORKEE_CORS_ORIGIN`

## API Endpoints

### Core Resources

| Endpoint | Description |
|----------|-------------|
| [Health Check](health) | Server status and health monitoring |
| [Projects](projects) | Project management and CRUD operations |
| [Directories](directories) | File system directory operations |

### Quick Start Example

```bash
# Check API health
curl http://localhost:4001/api/health

# List all projects
curl http://localhost:4001/api/projects

# Create a new project
curl -X POST http://localhost:4001/api/projects \
  -H "Content-Type: application/json" \
  -d '{"name": "My Project", "path": "/path/to/project"}'
```

## SDK Support

Official SDKs are planned for:
- JavaScript/TypeScript (npm package)
- Python (pip package)  
- Go (Go module)
- Rust (crate)

## OpenAPI Specification

The complete API specification is available at:
```
http://localhost:4001/api/spec
```

This provides machine-readable documentation for code generation and testing tools.