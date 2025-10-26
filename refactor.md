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

### Phase 2: High Priority Extractions

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

- [ ] **`ai`** - AI and agent system (~2,100+ lines)
  - AI service integration (ai_service.rs - 282 lines)
  - Agent registry and management (agents/)
  - Model registry and capabilities (models/ - 574 lines)
  - AI proxy functionality (api/ai_proxy_handlers.rs - 523 lines)
  - AI handlers (api/ai_handlers.rs - 1,302 lines)
  - Usage logging (ai_usage_logs/)
  - Agent execution tracking (executions/)
  - **Estimated effort**: 2-3 hours
  - **Dependencies**: orkee_core, storage (for persistence)
  - **Key files**:
    - `ai_service.rs` (282 lines) - Core AI service
    - `agents/` - Agent management
    - `models/registry.rs` (574 lines) - Model registry
    - `api/ai_handlers.rs` (1,302 lines) - HTTP handlers
    - `api/ai_proxy_handlers.rs` (523 lines) - Proxy functionality
    - `ai_usage_logs/` - Usage tracking
    - `executions/` - Execution tracking

- [ ] **`security`** - Security and authentication (812+ lines)
  - Encryption service (security/encryption.rs - 812 lines)
  - API token management (api_tokens/)
  - User authentication (users/)
  - Password management
  - Permission system
  - **Estimated effort**: 2 hours
  - **Dependencies**: orkee_core
  - **Key files**:
    - `security/encryption.rs` (812 lines) - Main encryption logic
    - `api_tokens/` - Token management
    - `users/` - User management
    - `api/security_handlers.rs` (1,244 lines) - HTTP handlers
  - **Testing considerations**:
    - Extensive encryption tests (20+ test cases)
    - Key rotation tests
    - Token validation tests

### Phase 3: Medium Priority Extractions

- [ ] **`storage`** - Data layer and persistence
  - SQLite implementation (storage/sqlite.rs - 1,393 lines)
  - Migration system
  - Storage traits and factory (storage/factory.rs)
  - Sync engine for cloud (storage/sync/)
  - Legacy JSON storage (storage/legacy.rs)
  - Transaction management
  - **Estimated effort**: 3 hours
  - **Dependencies**: orkee_core
  - **Key files**:
    - `storage/sqlite.rs` (1,393 lines) - SQLite implementation
    - `storage/factory.rs` - Storage factory pattern
    - `storage/sync/engine.rs` (615 lines) - Sync engine
    - `storage/legacy.rs` - Backward compatibility
    - `storage/mod.rs` - Main storage module
  - **Note**: This is foundational and should be extracted early to reduce dependencies

- [ ] **`tasks`** - Task management system
  - Task CRUD operations (tasks/)
  - Task execution tracking
  - Task status management
  - Manual task creation
  - Task-spec integration
  - **Estimated effort**: 1-2 hours
  - **Dependencies**: orkee_core, storage, openspec (for spec-derived tasks)
  - **Key files**:
    - `tasks/` directory
    - `api/tasks_handlers.rs` - HTTP handlers
    - `api/task_spec_router.rs` - Spec integration

- [ ] **`context`** - Code analysis and context management
  - AST analysis (context/ast_analyzer.rs)
  - Dependency graph building (context/graph_builder.rs - 1,209 lines)
  - Language support (context/language_support.rs)
  - Incremental parsing (context/incremental_parser.rs)
  - History service (context/history_service.rs)
  - Batch processor (context/batch_processor.rs)
  - Formatter (context/formatter.rs)
  - OpenSpec bridge (context/openspec_bridge.rs)
  - **Estimated effort**: 2-3 hours
  - **Dependencies**: orkee_core, storage
  - **Key files**:
    - `context/graph_builder.rs` (1,209 lines) - Main graph building
    - `context/ast_analyzer.rs` - AST analysis
    - `context/dependency_graph.rs` - Graph types
    - `context/language_support.rs` - Language-specific support
    - `api/context_handlers.rs` (799 lines) - HTTP handlers
    - `api/graph_handlers.rs` - Graph API endpoints

### Phase 4: Lower Priority Extractions

- [ ] **`api`** - HTTP layer and routing
  - All HTTP handlers (api/)
  - Request/response types
  - Routing configuration
  - Middleware
  - **Estimated effort**: 1-2 hours (mostly moving files)
  - **Dependencies**: All other packages (this is the integration layer)
  - **Key files**:
    - `api/handlers.rs` (1,113 lines) - Main project handlers
    - `api/ai_handlers.rs` (1,302 lines) - AI endpoints
    - `api/security_handlers.rs` (1,244 lines) - Security endpoints
    - `api/context_handlers.rs` (799 lines) - Context endpoints
    - Other handler files
  - **Note**: Should be extracted last as it depends on all other packages

- [ ] **`formatter`** - Output formatting
  - Project formatting (formatter.rs)
  - Table formatting
  - Detail views
  - **Estimated effort**: 30 minutes
  - **Dependencies**: orkee_core
  - **Key files**:
    - `formatter.rs` - Main formatting logic

- [ ] **`git_utils`** - Git integration
  - Git repository info extraction
  - Git operations
  - **Estimated effort**: 30 minutes
  - **Dependencies**: orkee_core
  - **Key files**:
    - `git_utils.rs` - Git utility functions

- [ ] **`tags`** - Tagging system
  - Tag management
  - Tag storage
  - **Estimated effort**: 30 minutes
  - **Dependencies**: orkee_core, storage
  - **Key files**:
    - `tags/` directory

- [ ] **`settings`** - Settings management
  - System configuration
  - Settings validation
  - Settings storage
  - **Estimated effort**: 1 hour
  - **Dependencies**: orkee_core, storage
  - **Key files**:
    - `settings/` directory
    - `settings/validation.rs` - Settings validation
    - `settings/storage.rs` - Settings persistence

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

1. ✅ **orkee_core** - No dependencies (COMPLETED)
2. **storage** - Depends on orkee_core
3. **security** - Depends on orkee_core
4. **openspec** - Depends on orkee_core, storage
5. **ai** - Depends on orkee_core, storage
6. **tasks** - Depends on orkee_core, storage, openspec
7. **context** - Depends on orkee_core, storage
8. **formatter**, **git_utils**, **tags**, **settings** - Minimal dependencies
9. **api** - Depends on all other packages (extract last)

## Current Progress

- ✅ Phase 1: Foundation (orkee_core) - COMPLETED
- ⏳ Phase 2: High Priority (openspec ✅, ai, security) - IN PROGRESS
- ⏸️ Phase 3: Medium Priority - PENDING
- ⏸️ Phase 4: Lower Priority - PENDING

## Notes

- The original `packages/projects/` can remain as a facade/integration package that re-exports from the new packages
- This ensures backward compatibility for existing consumers (cli, tui, dashboard)
- Each extraction should be a separate PR/commit for easy review and rollback if needed
- Tests should pass after each extraction

## Time Estimate

- **Total estimated time**: 15-20 hours
- **Already completed**: 6 hours (orkee_core: 2 hours, openspec: 4 hours)
- **Remaining**: 9-14 hours

This refactoring can be done incrementally, with each package extraction being independently valuable.
