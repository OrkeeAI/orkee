# Orkee Package Refactoring Plan

This document outlines the plan to refactor the monolithic `packages/projects/` into focused, single-responsibility packages.

## Current State

The `packages/projects/` package currently contains **97 Rust files** with over **31,000 lines of code**, handling at least 15 distinct domains, which violates the Single Responsibility Principle and makes the codebase difficult to maintain, test, and understand.

## Refactoring Strategy

Extract functionality into focused packages while maintaining backward compatibility through re-exports in the original `projects` package until all consumers are updated.

## Package Extraction Checklist

### Phase 1: Foundation Packages

- [x] **`orkee_core`** - Core types and utilities
  - Core types (Project, GitRepositoryInfo, ProjectStatus, Priority, etc.)
  - Common utilities (ID generation, path operations, compression)
  - Validation functions and types
  - Constants (orkee_dir, projects_file, PROJECTS_VERSION)
  - **Status**: ✅ COMPLETED
  - **Tests**: 8/8 passing
  - **Projects tests**: 291/291 passing with core dependency

### Phase 2: Storage & Simple Utilities (No Cross-Dependencies)

- [x] **`storage`** - Data layer and persistence
  - SQLite implementation (storage/sqlite.rs - 1,393 lines)
  - Migration system (single source of truth)
  - Storage traits and factory (storage/factory.rs)
  - Legacy JSON storage (storage/legacy.rs)
  - Transaction management
  - **Status**: ✅ COMPLETED
  - **Tests**: 11/11 passing
  - **Actual effort**: 3 hours (as estimated)
  - **Dependencies**: orkee_core
  - **Note**: Sync engine (storage/sync/) excluded as dead code
  - **Key files**:
    - `storage/sqlite.rs` (1,393 lines) - SQLite implementation
    - `storage/factory.rs` - Storage factory pattern
    - `storage/legacy.rs` - Backward compatibility
    - `storage/lib.rs` - Main storage module with EncryptionMode enum
    - `migrations/001_initial_schema.sql` - Database schema (single source of truth)

- [x] **`security`** - Security and authentication (~1,400 lines)
  - Encryption service (encryption/encryption.rs - 783 lines)
  - API token management (api_tokens/ - 246 lines)
  - User authentication (users/ - 377 lines)
  - Password management
  - Permission system
  - **Status**: ✅ COMPLETED
  - **Tests**: 50/50 passing
  - **Actual effort**: 2 hours (as estimated)
  - **Dependencies**: orkee_core, storage
  - **Key files**:
    - `encryption/encryption.rs` (783 lines) - ChaCha20-Poly1305 AEAD encryption
    - `api_tokens/storage.rs` (246 lines) - Token CRUD operations
    - `users/storage.rs` (377 lines) - User management and encrypted API keys
    - `api/security_handlers.rs` (1,244 lines) - HTTP handlers (kept in projects for now)
  - **Testing**:
    - 27 encryption tests (key rotation, machine/password modes, portability)
    - 7 token tests (generation, validation, constant-time comparison)
    - 16 user tests (encryption, masking, key migration)
  - **Notes**:
    - Removed agent validation from UserStorage to avoid circular dependency
    - Validation moved to API/handler level in projects package

- [x] **`formatter`** - Output formatting
  - Project formatting (formatter.rs)
  - Table formatting
  - Detail views
  - **Status**: ✅ COMPLETED
  - **Tests**: 5/5 passing
  - **Actual effort**: 15 minutes (faster than estimated!)
  - **Dependencies**: orkee_core
  - **Key files**:
    - `formatter.rs` - Main formatting logic

- [x] **`git_utils`** - Git integration
  - Git repository info extraction
  - Git operations
  - **Status**: ✅ COMPLETED
  - **Tests**: 1/1 passing
  - **Actual effort**: 15 minutes (faster than estimated!)
  - **Dependencies**: orkee_core, git2, tracing
  - **Key files**:
    - `git_utils.rs` - Git utility functions (get_git_repository_info, parse_github_url)

### Phase 3: Domain Packages (Depend on Storage)

- [x] **`openspec`** - OpenSpec specification system (~7,140 lines)
  - PRD management (parser, validator, types)
  - Spec parsing and validation (markdown validator)
  - Task parsing from specs (task_parser)
  - Materializer for spec generation
  - Archive functionality
  - Change builder
  - Sync functionality
  - Database operations (db.rs - 1,732 lines)
  - **Status**: ✅ COMPLETED
  - **Tests**: 88/88 passing
  - **Actual effort**: 3-4 hours (as estimated)
  - **Dependencies**: orkee_core (works directly with sqlx::Pool, no storage dependency needed)
  - **Key files**:
    - `openspec/db.rs` (1,732 lines) - Main database operations
    - `openspec/materializer.rs` (852 lines) - Spec generation
    - `openspec/validator.rs` (743 lines) - Validation logic
    - `openspec/parser.rs` (644 lines) - Parsing logic
    - `openspec/sync.rs` (532 lines) - Synchronization
    - `openspec/task_parser.rs` (531 lines) - Task parsing
    - `openspec/archive.rs` (490 lines) - Archive management
    - `openspec/integration.rs` (465 lines) - Integration logic
    - `openspec/change_builder.rs` (463 lines) - Change management
    - `openspec/types.rs` (165 lines) - Type definitions
    - `openspec/cli.rs` (96 lines) - CLI integration
    - `openspec/markdown_validator.rs` (274 lines) - Markdown validation
  - **Testing considerations**:
    - OpenSpec has its own test suite
    - Integration tests with database operations
    - Materializer tests for spec generation

- [x] **`models`** - AI model and agent registry (~659 lines)
  - Model registry (registry.rs - 574 lines)
  - Model and agent types (types.rs - 77 lines)
  - JSON configuration files (models.json, agents.json)
  - **Status**: ✅ COMPLETED
  - **Tests**: N/A (pure config, no unit tests)
  - **Actual effort**: 30 minutes (as estimated!)
  - **Dependencies**: serde, serde_json, lazy_static only (NO orkee_core, NO storage)
  - **Key files**:
    - `registry.rs` (574 lines) - In-memory registry with LazyLock
    - `types.rs` (77 lines) - Model, Agent, AgentConfig types
    - `config/models.json` - Model definitions
    - `../../agents/config/agents.json` - Agent definitions (loaded from agents package)

- [x] **`agents`** - User agent configuration (~214 lines)
  - User agent preferences and settings
  - Agent-specific configurations
  - **Status**: ✅ COMPLETED
  - **Tests**: 0/0 passing (no unit tests yet, integration tests in projects TBD)
  - **Actual effort**: 1 hour (better than estimated!)
  - **Dependencies**: orkee_core, storage, models
  - **Database tables**: user_agents
  - **Key files**:
    - `storage.rs` (176 lines) - User agent CRUD
    - `types.rs` (30 lines) - UserAgent type
  - **Testing**:
    - No unit tests yet
    - Uses models::REGISTRY for validation (no DB foreign keys)
  - **Notes**:
    - Renamed AgentStorage → UserAgentStorage for clarity
    - Flattened directory structure (removed user_agents subdirectory)
    - Executions extracted to separate package for better separation of concerns

- [x] **`executions`** - Agent execution and PR review tracking (~798 lines)
  - Execution lifecycle tracking
  - PR review management
  - Code change metrics
  - **Status**: ✅ COMPLETED
  - **Tests**: 0/0 passing (no unit tests yet, integration tests in projects TBD)
  - **Actual effort**: 30 minutes (extraction from agents package)
  - **Dependencies**: orkee_core, storage, models
  - **Database tables**: agent_executions, pr_reviews
  - **Key files**:
    - `storage.rs` (616 lines) - Execution CRUD operations
    - `types.rs` (174 lines) - AgentExecution, PrReview types
  - **Testing**:
    - No unit tests yet
    - Uses nanoid for ID generation
  - **Notes**:
    - Separated from agents package for cleaner separation of concerns
    - Config (agents) vs. runtime observability (executions)
    - Zero coupling between agents and executions packages

- [x] **`ai`** - AI service and usage tracking (~809 lines)
  - AI service integration (service.rs - 282 lines)
  - AI usage logs (usage_logs/ - 527 lines)
  - **Status**: ✅ COMPLETED
  - **Tests**: 2/2 passing (ai_usage_integration_tests, ai_proxy_integration_tests)
  - **Actual effort**: 15 minutes (much faster than estimated!)
  - **Dependencies**: orkee_core, storage, reqwest
  - **Database tables**: ai_usage_logs
  - **Key files**:
    - `service.rs` (282 lines) - Anthropic API client
    - `usage_logs/storage.rs` (448 lines) - Usage log CRUD
    - `usage_logs/types.rs` (79 lines) - AiUsageLog types
  - **API handlers** (kept in projects for Phase 4):
    - `api/ai_handlers.rs` (1,254 lines) - AI endpoints
    - `api/ai_proxy_handlers.rs` (523 lines) - Proxy functionality
    - `api/ai_usage_log_handlers.rs` (89 lines) - Usage endpoints
    - `api/agents_handlers.rs` (123 lines) - Agent endpoints
    - `api/executions_handlers.rs` (341 lines) - Execution endpoints
  - **Notes**:
    - Test files successfully moved to ai package
    - No import changes needed - service.rs already self-contained

- [x] **`context`** - Code analysis and context management (~4,644 lines)
  - AST analysis (ast_analyzer.rs - 396 lines)
  - Dependency graph building (graph_builder.rs - 1,209 lines)
  - Language support (language_support.rs - 367 lines)
  - Incremental parsing (incremental_parser.rs - 311 lines)
  - History service (history_service.rs - 306 lines)
  - Batch processor (batch_processor.rs - 141 lines)
  - Formatter (formatter.rs - 347 lines)
  - OpenSpec bridge (openspec_bridge.rs - 365 lines)
  - **Status**: ✅ COMPLETED
  - **Tests**: 38/38 passing (7 ignored for incomplete AST features)
  - **Actual effort**: 2 hours (as estimated!)
  - **Dependencies**: openspec, tree-sitter, sqlx
  - **Key files**:
    - `graph_builder.rs` (1,209 lines) - Main graph building logic
    - `dependency_graph.rs` (426 lines) - Dependency graph types and operations
    - `ast_analyzer.rs` (396 lines) - AST parsing and symbol extraction
    - `language_support.rs` (367 lines) - Language-specific configurations
    - `openspec_bridge.rs` (365 lines) - OpenSpec integration
    - `formatter.rs` (347 lines) - Context formatting
    - `incremental_parser.rs` (311 lines) - Incremental parsing
    - `history_service.rs` (306 lines) - File history tracking
    - `spec_context.rs` (290 lines) - Spec context generation
    - `batch_processor.rs` (141 lines) - Batch processing
    - `types.rs` (103 lines) - Type definitions
    - `graph_types.rs` (70 lines) - Graph-specific types
  - **API handlers** (kept in projects for Phase 4):
    - `api/context_handlers.rs` (799 lines) - HTTP handlers
    - `api/graph_handlers.rs` (288 lines) - Graph API endpoints
  - **Notes**:
    - Generated SQLx query cache for offline compilation
    - Added missing dependencies: rand, num_cpus

- [x] **`tags`** - Tagging system
  - Tag management (types, CRUD operations)
  - Tag storage (SQLite with archive support)
  - **Status**: ✅ COMPLETED
  - **Tests**: 11/11 passing
  - **Actual effort**: 30 minutes (as estimated!)
  - **Dependencies**: orkee_core, storage
  - **Key files**:
    - `tags/types.rs` - Tag type definitions
    - `tags/storage.rs` - SQLite storage implementation with archive/unarchive
    - `tags/lib.rs` - Main module with re-exports

- [x] **`settings`** - Settings management
  - System configuration
  - Settings validation
  - Settings storage
  - **Status**: ✅ COMPLETED
  - **Tests**: 27/27 passing (11 integration tests + 16 validation unit tests)
  - **Actual effort**: 45 minutes (faster than estimated!)
  - **Dependencies**: storage (no orkee_core dependency needed)
  - **Key files**:
    - `settings/types.rs` - Type definitions (SystemSetting, SettingUpdate, SettingCategory)
    - `settings/validation.rs` (399 lines) - Input validation with comprehensive tests
    - `settings/storage.rs` (242 lines) - Database CRUD operations
    - `settings/storage_tests.rs` (331 lines) - Integration tests
    - `settings/lib.rs` - Main module with re-exports

- [x] **`tasks`** - Task management system
  - Task CRUD operations (tasks/)
  - Task execution tracking
  - Task status management
  - Manual task creation
  - Task-spec integration
  - **Status**: ✅ COMPLETED
  - **Tests**: 0/0 passing (no unit tests yet, integration tests in projects: 9/9)
  - **Actual effort**: 1.5 hours (as estimated)
  - **Dependencies**: orkee_core, storage, openspec
  - **Key files**:
    - `tasks/types.rs` (131 lines) - Task type definitions
    - `tasks/storage.rs` (465 lines) - SQLite storage implementation
    - `tasks/lib.rs` - Main module with re-exports
  - **API handlers** (kept in projects for Phase 4):
    - `api/tasks_handlers.rs` (213 lines) - HTTP handlers
    - `api/task_spec_handlers.rs` (209 lines) - Spec integration
  - **Note**: Removed Agent dependency - Task only stores assigned_agent_id

### Phase 4: Integration Layer (Depends on Everything)

- [x] **`api`** - HTTP layer and routing
  - All HTTP handlers (api/)
  - Request/response types
  - Routing configuration
  - Middleware
  - **Status**: ✅ COMPLETED
  - **Tests**: 45/45 passing (API package) + All workspace tests passing
  - **Actual effort**: 4 hours (estimated 1-2 hours, but required Axum 0.7→0.8 migration)
  - **Dependencies**: All other packages (this is the integration layer)
  - **Priority**: Extract LAST - depends on all other packages
  - **Key files**:
    - `api/handlers.rs` (1,113 lines) - Main project handlers
    - `api/ai_handlers.rs` (1,302 lines) - AI endpoints
    - `api/security_handlers.rs` (1,244 lines) - Security endpoints
    - `api/context_handlers.rs` (799 lines) - Context endpoints
    - Other handler files
  - **Notable challenges**:
    - Upgraded Axum from 0.7 to 0.8 (breaking changes in route syntax)
    - Converted route parameters from `:param` to `{param}` syntax
    - Converted wildcard routes from `*param` to `{*param}` syntax
    - Removed `#[async_trait]` requirement for `FromRequestParts` trait
    - Fixed orphan rule violation by converting `IntoResponse` impl to helper function
    - Added `test-utils` feature to projects package for test helper access
    - Updated test imports across CLI package integration tests

## Detailed Migration Steps for Each Package

### Step 1: Create Package Structure

```bash
# Create package directory and source folder
mkdir -p packages/<name>/src

# Create Cargo.toml
cat > packages/<name>/Cargo.toml <<EOF
[package]
name = "<name>"  # or "orkee_<name>" if conflicts with Rust std
version.workspace = true
edition.workspace = true
description = "<Description of package>"
authors.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
# Add minimal dependencies needed for this package
# Common ones:
serde = { version = "1.0", features = ["derive"] }
chrono = { version = "0.4", features = ["serde"] }
thiserror = "2.0"

# Add orkee_core if needed
orkee_core = { path = "../core" }

[dev-dependencies]
tokio = { version = "1.0", features = ["macros"] }
tempfile = "3.0"
EOF
```

```bash
# Add to workspace
# Edit Cargo.toml in project root, add to members array:
# members = [
#     "packages/core",
#     "packages/<name>",  # <-- Add this line
#     ...
# ]
```

### Step 2: Extract Code

```bash
# Copy relevant directory or files
cp -r packages/projects/src/<module_dir> packages/<name>/src/

# Create lib.rs
cat > packages/<name>/src/lib.rs <<EOF
// ABOUTME: <Brief description of what this package does>
// ABOUTME: <Second line of description>

pub mod <module1>;
pub mod <module2>;

// Re-export main types
pub use <module1>::{Type1, Type2};
EOF
```

### Step 3: Update Dependencies

```bash
# Add dependency in projects/Cargo.toml
# Under [dependencies], add:
# <name> = { path = "../<name>" }

# Update imports in projects/src/lib.rs
# Remove the old module declaration:
# pub mod <old_module>;  # Remove this line

# Add re-exports from new package:
# pub use <name>::{Type1, Type2, function1, function2};
```

### Step 4: Bulk Import Updates

```bash
# Replace all internal imports in projects package
# Example for openspec:
find packages/projects/src -name "*.rs" -type f -exec sed -i '' 's/use crate::openspec::/use openspec::/g' {} \;

# Replace qualified paths:
find packages/projects/src -name "*.rs" -type f -exec sed -i '' 's/crate::openspec::/openspec::/g' {} \;

# Common patterns to replace:
# - crate::<module>:: → <package>::
# - use crate::<module> → use <package>

# For orkee_core specifically (already completed):
# - crate::types:: → orkee_core::types::
# - crate::constants:: → orkee_core::constants::
# - crate::validator:: → orkee_core::validation::
```

### Step 5: Fix Compilation Errors

```bash
# Build the new package
cargo build --package <name>

# Common errors to fix:
# 1. Missing imports - add them to the file
# 2. Circular dependencies - restructure if needed
# 3. Missing feature flags - add to Cargo.toml
# 4. Type mismatches - ensure correct re-exports

# Build projects package
cargo build --package orkee-projects

# Common errors:
# 1. Unresolved imports - check re-exports in lib.rs
# 2. Missing types - add to re-export list
# 3. Module not found - ensure mod declaration exists
```

### Step 6: Test Everything

```bash
# Run new package tests
cargo test --package <name>

# Run projects package tests
cargo test --package orkee-projects

# Run integration tests
cargo test --workspace

# If tests fail, check:
# 1. Module paths in test imports
# 2. Test utilities availability
# 3. Feature flags for test-only code
```

### Step 7: Update Consumers (If Needed)

```bash
# Check if CLI uses this module
grep -r "<module>" packages/cli/src/

# Check if TUI uses this module
grep -r "<module>" packages/tui/src/

# If direct usage found, add dependency:
# In packages/cli/Cargo.toml or packages/tui/Cargo.toml:
# <name> = { path = "../<name>" }

# Update imports in those packages:
# use orkee_projects::<module>:: → use <name>::
```

## Common Issues and Solutions

### Issue: Name collision with Rust std library
**Solution**: Use `orkee_` prefix for the crate name
```toml
[package]
name = "orkee_<name>"  # Instead of just "<name>"
```

### Issue: Circular dependencies
**Solution**: Extract to a lower-level package first, or restructure dependencies

### Issue: Feature flags needed
**Solution**: Add feature flags to Cargo.toml
```toml
[features]
default = []
<feature> = ["dep:<dependency>"]
```

### Issue: Macro imports not working
**Solution**: Re-export macros explicitly
```rust
pub use module_name::macro_name;
```

### Issue: Tests can't find types
**Solution**: Check test imports and ensure types are pub
```rust
#[cfg(test)]
mod tests {
    use super::*;  // Import from parent module
    use crate::Type;  // Import from crate root
}
```

## Benefits After Refactoring

1. **Clear Responsibilities** - Each package has a single, well-defined purpose
2. **Better Testing** - Can test OpenSpec without AI dependencies, for example
3. **Faster Compilation** - Changes to OpenSpec don't recompile AI code
4. **Easier Onboarding** - New developers can understand one package at a time
5. **Flexible Deployment** - Could potentially deploy some packages as separate services
6. **Better Dependency Management** - Clear dependency graph, no circular dependencies
7. **Parallel Development** - Teams can work on different packages without conflicts

## Naming Convention

- Package directory: `packages/<name>/` (e.g., `packages/openspec/`)
- Crate name: `<name>` or `orkee_<name>` if there's a naming conflict with standard Rust crates
- Module path: Use the crate name directly

**Note**: Avoid using bare names that conflict with Rust's standard library (e.g., `core`, `std`, `alloc`). Use `orkee_` prefix when necessary.

## Dependency Order (Extract in this order)

**Phase 1: Foundation** (✅ COMPLETED)
1. ✅ **orkee_core** - No dependencies

**Phase 2: Storage & Simple Utilities** (No cross-dependencies - can be done in parallel)
2. ✅ **storage** - Depends on orkee_core (COMPLETED - blocks many other packages)
3. ✅ **security** - Depends on orkee_core (COMPLETED)
4. ✅ **formatter** - Depends on orkee_core (COMPLETED)
5. ✅ **git_utils** - Depends on orkee_core (COMPLETED)
6. ✅ **models** - Pure config, no dependencies (COMPLETED)

**Phase 3: Domain Packages** (Depend on storage - must wait for Phase 2)
7. ✅ **openspec** - Depends on orkee_core (COMPLETED - works directly with sqlx::Pool)
8. ✅ **tags** - Depends on orkee_core, storage (COMPLETED)
9. ✅ **settings** - Depends on orkee_core, storage (COMPLETED)
10. ✅ **tasks** - Depends on orkee_core, storage, openspec (COMPLETED)
11. ✅ **agents** - Depends on orkee_core, storage, models (COMPLETED)
12. ✅ **executions** - Depends on orkee_core, storage, models (COMPLETED)
13. ✅ **ai** - Depends on orkee_core, storage (COMPLETED)
14. ✅ **context** - Depends on openspec, tree-sitter, sqlx (COMPLETED)

**Phase 4: Integration Layer** (Depends on everything - extract LAST)
15. ✅ **api** - Depends on all other packages (COMPLETED)

## Current Progress

- ✅ Phase 1: Foundation (orkee_core) - COMPLETED
- ✅ Phase 2: Storage & Simple Utilities (storage ✅, security ✅, formatter ✅, git_utils ✅, models ✅) - COMPLETED
- ✅ Phase 3: Domain Packages (openspec ✅, tags ✅, settings ✅, tasks ✅, agents ✅, executions ✅, ai ✅, context ✅) - COMPLETED (8/8 - 100%)
- ✅ Phase 4: Integration Layer (api ✅) - COMPLETED

### Status

✅ **ALL PHASES COMPLETE** - Package refactoring fully finished!

## Notes

- **No backward compatibility needed**: Since the app isn't in production yet, we don't maintain facade re-exports in `packages/projects/`
- Consumers (cli, tui, dashboard) import directly from extracted packages (models, agents, ai, etc.)
- `packages/projects/` remains as an integration layer for API handlers and business logic
- Each extraction is a separate commit for easy review and rollback if needed
- All tests pass after each extraction

## Time Estimate

- **Total estimated time**: 15-20 hours
- **Total actual time**: 22.5 hours
- **Breakdown by phase**:
  - orkee_core: 2 hours
  - openspec: 4 hours
  - storage: 3 hours
  - security: 2 hours
  - formatter: 0.25 hours
  - git_utils: 0.25 hours
  - tags: 0.5 hours
  - settings: 0.75 hours
  - tasks: 1.5 hours
  - models: 0.5 hours
  - agents: 1 hour
  - executions: 0.5 hours
  - ai: 0.25 hours
  - context: 2 hours
  - api: 4 hours (estimated 1-2, but required Axum 0.7→0.8 upgrade)
- **Phase 1 complete**: Foundation package (orkee_core)
- **Phase 2 complete**: All storage and utility packages
- **Phase 3 complete**: All domain packages (100%)
- **Phase 4 complete**: Integration layer (api) ✅

This refactoring can be done incrementally, with each package extraction being independently valuable.
