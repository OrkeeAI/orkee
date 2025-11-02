# Error Handling Improvements - Technical Debt

## Overview

This document tracks the remaining `.unwrap()` and `.expect()` calls in the ideate package that should be replaced with proper error propagation. These are technical debt items identified during PR review.

## Current State

As of 2025-01-02, the following source files contain `.unwrap()` or `.expect()` calls that should be addressed:

### High Priority (Production Code)

1. **task_decomposer.rs** - Task generation core logic
2. **complexity_analyzer.rs** - Complexity calculation engine
3. **approach_generator.rs** - Technical approach generation
4. **discovery_manager.rs** - Discovery question management
5. **manager.rs** - Ideate session management
6. **validation.rs** - PRD validation logic

### Medium Priority (Supporting Modules)

7. **prompts.rs** - Prompt template handling
8. **dependency_analyzer.rs** - Dependency detection
9. **chat_manager.rs** - Chat interaction handling
10. **prd_aggregator.rs** - PRD section aggregation
11. **prd_generator.rs** - PRD generation logic
12. **execution_tracker.rs** - Task execution tracking

### Lower Priority (Specialized Features)

13. **github_sync.rs** - GitHub integration
14. **research_prompts.rs** - Research tool prompts
15. **research_analyzer.rs** - Research analysis
16. **roundtable.rs** - Expert roundtable feature
17. **roundtable_manager.rs** - Roundtable session management
18. **expert_moderator.rs** - Expert moderation logic
19. **build_optimizer.rs** - Build order optimization
20. **export_service.rs** - PRD export functionality

## Replacement Strategy

### Pattern 1: JSON Parsing
**Current (Unsafe)**:
```rust
let context: CodebaseContext = serde_json::from_str(&json_str).unwrap();
```

**Improved (Safe)**:
```rust
let context: CodebaseContext = serde_json::from_str(&json_str)
    .map_err(|e| IdeateError::InvalidJson(format!("Failed to parse codebase context: {}", e)))?;
```

### Pattern 2: Database Row Extraction
**Current (Unsafe)**:
```rust
let id: String = row.get("id").unwrap();
```

**Improved (Safe)**:
```rust
let id: String = row.try_get("id")
    .map_err(|e| StorageError::Database(format!("Missing id column: {}", e)))?;
```

### Pattern 3: Optional Value Handling
**Current (Unsafe)**:
```rust
let value = optional_value.unwrap();
```

**Improved (Safe)**:
```rust
let value = optional_value
    .ok_or_else(|| IdeateError::MissingData("Expected value not found".to_string()))?;
```

### Pattern 4: Collection Operations
**Current (Unsafe)**:
```rust
let first = items.first().unwrap();
```

**Improved (Safe)**:
```rust
let first = items.first()
    .ok_or_else(|| IdeateError::EmptyCollection("Expected at least one item".to_string()))?;
```

## Error Type Additions Needed

Add these variants to `IdeateError` in `packages/ideate/src/lib.rs`:

```rust
pub enum IdeateError {
    // ... existing variants ...

    /// Invalid JSON data
    InvalidJson(String),

    /// Missing required data
    MissingData(String),

    /// Empty collection when items expected
    EmptyCollection(String),

    /// Database column missing or wrong type
    DatabaseColumn(String),
}
```

## Implementation Plan

### Phase 1: Critical Path (Week 1)
- [ ] task_decomposer.rs - Core task generation logic
- [ ] complexity_analyzer.rs - Expensive computation, failures are critical
- [ ] validation.rs - Quality assurance, errors must be reported

### Phase 2: User-Facing (Week 2)
- [ ] manager.rs - Session management, user-visible errors
- [ ] approach_generator.rs - Technical approach generation
- [ ] discovery_manager.rs - Discovery question flow
- [ ] chat_manager.rs - Chat interactions

### Phase 3: Supporting Features (Week 3)
- [ ] prompts.rs - Template handling
- [ ] dependency_analyzer.rs - Dependency detection
- [ ] prd_aggregator.rs - PRD assembly
- [ ] prd_generator.rs - PRD generation

### Phase 4: Specialized Features (Week 4)
- [ ] github_sync.rs - External integration
- [ ] research_*.rs - Research tools
- [ ] roundtable*.rs - Expert roundtable
- [ ] build_optimizer.rs - Build optimization
- [ ] export_service.rs - Export functionality

## Testing Strategy

For each file:
1. Identify all `.unwrap()` and `.expect()` calls
2. Replace with proper error propagation
3. Add unit test that triggers the error path
4. Verify error message is actionable

Example test:
```rust
#[tokio::test]
async fn test_complexity_analyzer_handles_invalid_json() {
    let invalid_json = "{ invalid json }";
    let result = parse_complexity_report(invalid_json);

    assert!(result.is_err());
    match result.unwrap_err() {
        IdeateError::InvalidJson(msg) => {
            assert!(msg.contains("Failed to parse"));
        }
        _ => panic!("Expected InvalidJson error"),
    }
}
```

## Acceptance Criteria

- [ ] Zero `.unwrap()` calls in production code paths (tests are OK)
- [ ] All `.expect()` calls have clear error context
- [ ] All error paths have corresponding tests
- [ ] Error messages are actionable (tell user what went wrong and what to do)
- [ ] No panics in production except for truly unrecoverable errors

## Notes

- Test files can continue using `.unwrap()` as failures stop the test
- `.expect()` with clear messages is acceptable for "should never happen" cases
- Focus on user-facing paths first (task generation, validation, PRD creation)
- Background/async tasks need especially careful error handling (no panics!)

## References

- Rust Error Handling Best Practices: https://rust-lang.github.io/api-guidelines/error-handling.html
- SQLx Error Handling: https://docs.rs/sqlx/latest/sqlx/error/index.html
- Serde Error Context: https://serde.rs/error-handling.html

---

**Created**: 2025-01-02
**Status**: Documented - Implementation pending
**Priority**: MEDIUM - Technical debt, not blocking release
**Estimated Effort**: 4 weeks (2-3 files per day, with tests)
