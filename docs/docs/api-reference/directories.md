---
sidebar_position: 4
---

# Directories API

The Directories API provides secure file system browsing capabilities with configurable path validation and sandboxing for project exploration and management.

## Base Endpoint

All directory endpoints are prefixed with `/api/directories`.

## List Directory Contents

**GET** `/api/directories/list`

Lists the contents of a specified directory with security controls and filtering options.

### Query Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `path` | string | **Required** | Absolute path to directory |
| `show_hidden` | boolean | `false` | Include hidden files (starting with `.`) |
| `limit` | number | 1000 | Maximum number of entries to return |
| `sort` | string | `name` | Sort field (`name`, `modified`, `size`, `type`) |
| `order` | string | `asc` | Sort order (`asc`, `desc`) |
| `filter` | string | - | Filter entries by name (case-insensitive) |

### Example Request

```bash
curl "http://localhost:4001/api/directories/list?path=/Users/john/projects&show_hidden=false&sort=modified&order=desc"
```

### Response

```json
{
  "success": true,
  "data": {
    "path": "/Users/john/projects",
    "entries": [
      {
        "name": "ai-chat",
        "path": "/Users/john/projects/ai-chat",
        "type": "directory",
        "size": 4096,
        "modified": "2024-01-15T12:30:00Z",
        "permissions": "drwxr-xr-x",
        "is_hidden": false,
        "is_git_repo": true,
        "children_count": 15
      },
      {
        "name": "package.json",
        "path": "/Users/john/projects/ai-chat/package.json",
        "type": "file",
        "size": 1524,
        "modified": "2024-01-15T10:15:00Z",
        "permissions": "-rw-r--r--",
        "is_hidden": false,
        "mime_type": "application/json"
      },
      {
        "name": ".env.example",
        "path": "/Users/john/projects/ai-chat/.env.example",
        "type": "file",
        "size": 245,
        "modified": "2024-01-14T16:20:00Z",
        "permissions": "-rw-r--r--",
        "is_hidden": true,
        "mime_type": "text/plain"
      }
    ],
    "total": 3,
    "parent": "/Users/john",
    "is_root": false,
    "permissions": {
      "readable": true,
      "writable": true,
      "executable": true
    }
  }
}
```

### Entry Types

| Type | Description |
|------|-------------|
| `file` | Regular file |
| `directory` | Directory/folder |
| `symlink` | Symbolic link |
| `socket` | Unix socket |
| `pipe` | Named pipe (FIFO) |
| `device` | Block or character device |

### File Size Formatting

File sizes are returned in bytes. For display purposes, consider formatting:

```javascript
function formatFileSize(bytes) {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
}
```

## Path Security & Sandboxing

The Directories API implements multiple security modes to prevent unauthorized file system access.

### Security Modes

Configure via `BROWSE_SANDBOX_MODE` environment variable:

#### Strict Mode (`strict`)

- Only explicitly allowed paths in `ALLOWED_BROWSE_PATHS`
- No path traversal (`../`) permitted
- Most secure, limited functionality

```bash
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/Users/john/Documents,/Users/john/Projects"
```

#### Relaxed Mode (`relaxed`) - Default

- Home directory and allowed paths accessible
- System directories blocked (`/etc`, `/usr`, `/sys`)
- Sensitive directories blocked (`.ssh`, `.aws`, `.gnupg`)
- Path traversal restricted

```bash
BROWSE_SANDBOX_MODE=relaxed
ALLOWED_BROWSE_PATHS="/Users/john/Documents,/Users/john/Projects,/Users/john/Desktop"
```

#### Disabled Mode (`disabled`)

- No path restrictions
- **Not recommended for production**
- Use only in trusted environments

```bash
BROWSE_SANDBOX_MODE=disabled
```

### Default Allowed Paths

When `ALLOWED_BROWSE_PATHS` is not set:

- `~/Documents`
- `~/Projects` 
- `~/Desktop`
- `~/Downloads`

### Blocked System Paths

These paths are always blocked in `strict` and `relaxed` modes:

**System Directories:**
- `/etc`, `/usr/bin`, `/usr/sbin`, `/bin`, `/sbin`
- `/var/log`, `/var/lib`, `/var/run`
- `/sys`, `/proc`, `/dev`
- `/boot`, `/lib`, `/lib64`

**Sensitive User Directories:**
- `~/.ssh` - SSH keys and configuration
- `~/.aws` - AWS credentials
- `~/.gnupg` - GPG keys
- `~/.docker` - Docker credentials
- Browser credential directories

## Error Responses

### Path Not Found

**Status**: 404 Not Found

```json
{
  "success": false,
  "error": {
    "code": "PATH_NOT_FOUND",
    "message": "Directory '/nonexistent/path' does not exist"
  }
}
```

### Access Denied

**Status**: 403 Forbidden

```json
{
  "success": false,
  "error": {
    "code": "ACCESS_DENIED",
    "message": "Access to '/etc/passwd' is not allowed"
  }
}
```

### Permission Denied

**Status**: 403 Forbidden

```json
{
  "success": false,
  "error": {
    "code": "PERMISSION_DENIED",
    "message": "Insufficient permissions to read '/private/admin'"
  }
}
```

### Path Traversal Blocked

**Status**: 400 Bad Request

```json
{
  "success": false,
  "error": {
    "code": "PATH_TRAVERSAL_BLOCKED",
    "message": "Path traversal detected: '../../../etc/passwd'"
  }
}
```

### Invalid Path Format

**Status**: 400 Bad Request

```json
{
  "success": false,
  "error": {
    "code": "INVALID_PATH",
    "message": "Path must be absolute (start with '/')"
  }
}
```

## Rate Limiting

Directory listing has conservative rate limits due to potential system impact:

- **Default**: 20 requests per minute
- **Burst**: Up to 5 requests in quick succession
- **Configurable** via `RATE_LIMIT_BROWSE_RPM`

Rate limit headers:

```http
X-RateLimit-Limit: 20
X-RateLimit-Remaining: 19
X-RateLimit-Reset: 1642248600
```

## Performance Considerations

### Large Directories

- Default limit of 1000 entries prevents overwhelming responses
- Use `limit` parameter to reduce payload size
- Consider pagination for very large directories

### Network Drives

- Network-mounted directories may have slower response times
- Timeouts apply to prevent hanging requests
- Consider caching for frequently accessed network paths

### Hidden Files

- Use `show_hidden=false` (default) for better performance
- Hidden files often include system files that slow enumeration

## Common Use Cases

### Project Browser

List project directories for a development dashboard:

```bash
curl "http://localhost:4001/api/directories/list?path=/Users/john/projects&filter=.git&sort=modified&order=desc"
```

### File Explorer Navigation

Navigate file system with breadcrumb support:

```bash
# Get current directory
curl "http://localhost:4001/api/directories/list?path=/Users/john/documents"

# Navigate to parent (from response.data.parent)
curl "http://localhost:4001/api/directories/list?path=/Users/john"

# Navigate to child directory
curl "http://localhost:4001/api/directories/list?path=/Users/john/documents/projects"
```

### Git Repository Discovery

Find Git repositories in a directory tree:

```bash
curl "http://localhost:4001/api/directories/list?path=/Users/john/projects" | \
jq '.data.entries[] | select(.is_git_repo == true) | .path'
```

### Configuration File Search

Look for configuration files:

```bash
curl "http://localhost:4001/api/directories/list?path=/Users/john/project&show_hidden=true&filter=config"
```

## Integration Examples

### React File Browser Component

```typescript
interface DirectoryEntry {
  name: string;
  path: string;
  type: 'file' | 'directory' | 'symlink';
  size: number;
  modified: string;
  is_hidden: boolean;
  is_git_repo?: boolean;
}

async function fetchDirectory(path: string): Promise<DirectoryEntry[]> {
  const response = await fetch(
    `/api/directories/list?path=${encodeURIComponent(path)}&sort=type&order=asc`
  );
  const data = await response.json();
  return data.success ? data.data.entries : [];
}
```

### CLI Directory Listing

```bash
#!/bin/bash
# Simple directory browser using Orkee API

API_BASE="http://localhost:4001"
CURRENT_PATH="${1:-$HOME}"

while true; do
  echo "Current: $CURRENT_PATH"
  echo "----------------------------------------"
  
  RESPONSE=$(curl -s "$API_BASE/api/directories/list?path=$CURRENT_PATH")
  ENTRIES=$(echo "$RESPONSE" | jq -r '.data.entries[] | "\(.type)\t\(.name)"')
  
  echo "$ENTRIES" | while read -r type name; do
    case $type in
      directory) echo "=Á $name" ;;
      file) echo "=Ä $name" ;;
      *) echo "S $name" ;;
    esac
  done
  
  echo "----------------------------------------"
  read -p "Enter directory name or 'q' to quit: " choice
  
  if [ "$choice" = "q" ]; then
    break
  elif [ -n "$choice" ]; then
    CURRENT_PATH="$CURRENT_PATH/$choice"
  fi
done
```

## Security Best Practices

### Production Configuration

```bash
# Recommended production settings
BROWSE_SANDBOX_MODE=strict
ALLOWED_BROWSE_PATHS="/opt/projects,/var/www/html"
RATE_LIMIT_BROWSE_RPM=10
```

### Development Configuration

```bash
# Development settings (more permissive)
BROWSE_SANDBOX_MODE=relaxed
ALLOWED_BROWSE_PATHS="$HOME/Documents,$HOME/Projects,$HOME/Desktop"
RATE_LIMIT_BROWSE_RPM=30
```

### Monitoring & Logging

Enable request logging to monitor directory access patterns:

```bash
RUST_LOG=orkee::api::directories=info cargo run
```

Log entries include:
- Requested paths
- Access denials
- Rate limit violations
- Performance metrics