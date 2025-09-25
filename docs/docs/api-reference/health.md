---
sidebar_position: 2
---

# Health Check API

The health check endpoints provide system status monitoring and diagnostic information for the Orkee API server.

## Endpoints

### Basic Health Check

**GET** `/api/health`

Returns basic server health status.

#### Response

```json
{
  "success": true,
  "data": {
    "status": "healthy"
  }
}
```

#### Status Values

- `healthy` - Server is running normally
- `degraded` - Server is running but with issues
- `unhealthy` - Server has critical problems

### Detailed Status

**GET** `/api/status`

Returns comprehensive server status information.

#### Response

```json
{
  "success": true,
  "data": {
    "status": "healthy",
    "version": "0.1.0",
    "uptime": 3600,
    "database": {
      "status": "connected",
      "path": "/Users/user/.orkee/orkee.db",
      "size": 1048576,
      "projects_count": 12,
      "last_backup": "2024-01-15T10:30:00Z"
    },
    "system": {
      "memory_usage": 52428800,
      "disk_space": {
        "total": 1000000000000,
        "free": 500000000000,
        "used": 500000000000
      },
      "cpu_usage": 2.5
    },
    "api": {
      "requests_total": 1500,
      "requests_per_minute": 45,
      "average_response_time": 125,
      "error_rate": 0.02
    }
  }
}
```

#### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `status` | string | Overall health status |
| `version` | string | Orkee version number |
| `uptime` | number | Server uptime in seconds |
| `database.status` | string | Database connection status |
| `database.path` | string | SQLite database file path |
| `database.size` | number | Database file size in bytes |
| `database.projects_count` | number | Total number of projects |
| `database.last_backup` | string | ISO timestamp of last backup |
| `system.memory_usage` | number | Memory usage in bytes |
| `system.disk_space` | object | Disk usage statistics |
| `system.cpu_usage` | number | CPU usage percentage |
| `api.requests_total` | number | Total API requests since startup |
| `api.requests_per_minute` | number | Current request rate |
| `api.average_response_time` | number | Average response time in ms |
| `api.error_rate` | number | Error rate as decimal (0.02 = 2%) |

## Use Cases

### Monitoring Integration

Use health checks for monitoring systems like Prometheus:

```bash
# Kubernetes liveness probe
curl -f http://localhost:4001/api/health || exit 1

# Prometheus scraping endpoint
curl http://localhost:4001/api/status | jq '.data.api.requests_per_minute'
```

### Load Balancer Health Checks

Configure load balancers to check health status:

```nginx
upstream orkee_servers {
    server localhost:4001;
    server localhost:4002 backup;
}

# Health check configuration
location /health {
    access_log off;
    return 200 "healthy\n";
    add_header Content-Type text/plain;
}
```

### Development Debugging

Check server status during development:

```bash
# Quick health check
curl -s http://localhost:4001/api/health | jq '.data.status'

# Full diagnostic information
curl -s http://localhost:4001/api/status | jq '.data'

# Monitor continuously
watch -n 5 'curl -s http://localhost:4001/api/status | jq ".data.system.cpu_usage"'
```

## Error Responses

### Server Unavailable

**Status**: 503 Service Unavailable

```json
{
  "success": false,
  "error": {
    "code": "SERVICE_UNAVAILABLE",
    "message": "Database connection failed"
  }
}
```

### Database Issues

**Status**: 500 Internal Server Error

```json
{
  "success": false,
  "error": {
    "code": "DATABASE_ERROR",
    "message": "SQLite database is locked or corrupted"
  }
}
```

## Response Headers

Health endpoints include additional headers for monitoring:

```http
HTTP/1.1 200 OK
Content-Type: application/json
X-Request-ID: req_abc123def456
X-Response-Time: 15ms
X-Server-Version: 0.1.0
X-Uptime: 3600
Cache-Control: no-cache, no-store, must-revalidate
```

## Rate Limiting

Health endpoints have generous rate limits:

- `/api/health`: 60 requests per minute
- `/api/status`: 30 requests per minute

Rate limit headers are included in responses:

```http
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59
X-RateLimit-Reset: 1642248600
```

## Security Considerations

- Health endpoints don't require authentication
- No sensitive information is exposed in responses
- Database paths are sanitized in production
- System metrics are aggregated to prevent fingerprinting

## Integration Examples

### Docker Healthcheck

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:4001/api/health || exit 1
```

### systemd Service Monitor

```ini
[Unit]
Description=Orkee Health Monitor
After=orkee.service

[Service]
Type=oneshot
ExecStart=/usr/bin/curl -f http://localhost:4001/api/health
```

### GitHub Actions Health Check

```yaml
- name: Health Check
  run: |
    curl -f http://localhost:4001/api/health
    STATUS=$(curl -s http://localhost:4001/api/status | jq -r '.data.status')
    if [ "$STATUS" != "healthy" ]; then
      exit 1
    fi
```