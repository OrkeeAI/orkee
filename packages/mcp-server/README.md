# Orkee MCP Server

An MCP (Model Context Protocol) server for Orkee project management, providing AI assistants with access to project data through standardized tools and resources.

## Features

- **Project Management**: List, search, create, update, and delete projects
- **Git Integration**: Enhanced filtering and search based on git repository information
- **MCP Protocol**: Full compatibility with Claude Desktop and other MCP clients
- **JSON-RPC**: Standards-compliant JSON-RPC 2.0 protocol implementation

## Tools Available

### 1. `projects`
List, get, or search Orkee projects with advanced filtering:
- **Actions**: `list`, `get`, `search`
- **Filters**: 
  - `status`: Filter by project status (`active`, `archived`, `all`)
  - `has_git`: Filter by git repository presence (`true`/`false`)
  - `priority`: Filter by priority (`high`, `medium`, `low`)
  - `tags`: Filter by project tags
- **Search**: Query across project names, descriptions, and git repository information

### 2. `project_manage`
Full project lifecycle management:
- **Actions**: `create`, `update`, `delete`
- **Fields**: name, projectRoot, description, tags, status, priority
- **Validation**: Ensures required fields and proper data types

## Installation & Usage

### Building the Server

```bash
# Build in release mode
cargo build --release

# The binary will be available at:
./target/release/orkee-mcp
```

### Command Line Usage

```bash
# Start MCP server (for Claude Desktop integration)
./target/release/orkee-mcp --mcp

# Show available tools
./target/release/orkee-mcp --tools

# Show available resources  
./target/release/orkee-mcp --resources

# Show available prompts
./target/release/orkee-mcp --prompts

# Show help
./target/release/orkee-mcp --help
```

## Claude Desktop Integration

### 1. Configuration Setup

Add the following to your Claude Desktop configuration file:

**On macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`

```json
{
  "mcpServers": {
    "orkee": {
      "command": "/absolute/path/to/orkee/packages/mcp-server/target/release/orkee-mcp",
      "args": ["--mcp"],
      "env": {}
    }
  }
}
```

### 2. Restart Claude Desktop

After updating the configuration, restart Claude Desktop to load the MCP server.

### 3. Verification

In Claude Desktop, you should be able to:
- List projects: "Show me all my Orkee projects"
- Search projects: "Find projects related to 'web development'"
- Filter by git: "Show me projects that have git repositories"
- Create projects: "Create a new project called 'My App' in /path/to/project"

## Testing the Server

### Manual JSON-RPC Testing

```bash
# Test initialization
echo '{"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {"protocolVersion": "2024-11-05", "capabilities": {}, "clientInfo": {"name": "test-client", "version": "1.0"}}}' | ./target/release/orkee-mcp --mcp

# Test tools listing
echo '{"jsonrpc": "2.0", "id": 2, "method": "tools/list"}' | ./target/release/orkee-mcp --mcp

# Test project listing
echo '{"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {"name": "projects", "arguments": {"action": "list", "status": "all"}}}' | ./target/release/orkee-mcp --mcp
```

## Implementation Details

### Architecture
- **Manual JSON-RPC**: Direct implementation without external RPC frameworks
- **Async/Await**: Full async support using Tokio runtime  
- **Error Handling**: Comprehensive error management with anyhow
- **Type Safety**: Strong typing with serde for all protocol messages

### Key Files
- `src/main.rs`: JSON-RPC server implementation and request routing
- `src/mcp.rs`: MCP protocol types and handlers (initialize, ping, etc.)
- `src/tools.rs`: Tool implementations (projects, project_manage)

### Git Repository Enhancement
Projects with git repositories include additional searchable metadata:
```json
{
  "gitRepository": {
    "owner": "username",
    "repo": "reponame", 
    "url": "https://github.com/username/repo.git",
    "branch": "main"
  }
}
```

## Protocol Compliance

This server implements MCP Protocol version `2024-11-05` with:
- ✅ JSON-RPC 2.0 transport
- ✅ Standard initialization handshake
- ✅ Tool discovery and execution
- ✅ Proper error handling and responses
- ✅ Resource and prompt capabilities (extensible)

## Dependencies

- **Core**: `tokio`, `serde`, `serde_json`, `anyhow`
- **CLI**: `clap` for command-line interface
- **Signal Handling**: `signal-hook` for graceful shutdown
- **Projects**: `orkee-projects` library for project management

## Development

```bash
# Check compilation
cargo check

# Run tests
cargo test

# Development mode
cargo run --bin orkee-mcp -- --mcp
```

## License

MIT License - see the main Orkee project for details.