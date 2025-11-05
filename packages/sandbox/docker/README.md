# Orkee Sandbox Docker Images

This directory contains Docker configurations for Orkee sandbox environments that provide isolated execution contexts for AI agents.

## Images

### Base Image (`orkee/sandbox:base`)
- Ubuntu 24.04 base with common development tools
- Python 3, Node.js 20, Rust toolchain pre-installed
- Essential utilities (git, curl, wget, build-essential)
- Common package managers (pip, npm, cargo)

### Claude Code Image (`orkee/sandbox:claude-code`)
- Based on the base image
- Additional tools for Claude Code agent
- Includes ripgrep, fd, bat for enhanced file operations
- Docker and docker-compose for container management
- Multiple language frameworks (React, Vue, Angular, Next.js)
- Go and Deno runtimes

### Aider Image (`orkee/sandbox:aider`)
- Based on the base image
- Aider CLI pre-installed
- Language servers for code understanding
- Database clients (PostgreSQL, MySQL, SQLite, Redis)
- Enhanced terminal tools (tmux, screen)

## Building Images

### Quick Build
```bash
# Build all images
./build.sh

# Build with version tag
./build.sh v1.0.0
```

### Manual Build
```bash
# Build base image
docker build -t orkee/sandbox:base -f Dockerfile.base .

# Build specific agent image
docker build -t orkee/sandbox:claude-code -f Dockerfile.claude-code .
docker build -t orkee/sandbox:aider -f Dockerfile.aider .
```

### Using Docker Compose
```bash
# Build all images
docker-compose build

# Build specific service
docker-compose build claude-code
```

## Running Sandboxes

### Direct Docker Run
```bash
# Run Claude Code sandbox
docker run -it --rm \
    -v $(pwd):/workspace \
    -p 8080:8080 \
    orkee/sandbox:claude-code

# Run Aider sandbox
docker run -it --rm \
    -v $(pwd):/workspace \
    -p 8081:8080 \
    orkee/sandbox:aider
```

### Using Docker Compose
```bash
# Start Claude Code sandbox
docker-compose up claude-code

# Start Aider sandbox
docker-compose up aider

# Start in background
docker-compose up -d claude-code

# View logs
docker-compose logs -f claude-code

# Stop services
docker-compose down
```

## Volume Mounts

- `/workspace` - Main working directory for code and projects
- `/var/run/docker.sock` - Docker socket for container management (Claude Code only)

## Environment Variables

### Claude Code
- `CLAUDE_CODE_WORKSPACE` - Working directory path (default: `/workspace`)
- `CLAUDE_CODE_PORT` - Agent server port (default: `8080`)
- `CLAUDE_CODE_HOST` - Agent server host (default: `0.0.0.0`)

### Aider
- `AIDER_WORKSPACE` - Working directory path (default: `/workspace`)
- `AIDER_PORT` - Agent server port (default: `8080`)
- `AIDER_HOST` - Agent server host (default: `0.0.0.0`)
- `AIDER_AUTO_COMMITS` - Auto-commit changes (default: `false`)

## Network

All sandboxes are connected to the `orkee-sandbox` bridge network with subnet `172.28.0.0/16`.

## Development

To add a new agent sandbox:

1. Create a new Dockerfile: `Dockerfile.agent-name`
2. Base it on `orkee/sandbox:base`
3. Install agent-specific dependencies
4. Add service to `docker-compose.yml`
5. Update `build.sh` to include the new image

## Security Notes

- Containers run with limited privileges by default
- Network isolation between sandboxes
- Resource limits can be configured in docker-compose.yml
- Consider using read-only volumes where appropriate
- Docker socket mount should be used cautiously