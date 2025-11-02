# JSON Field Query Patterns - Performance Guide

## Overview

This document describes the performance characteristics and best practices for querying JSON fields in Orkee's ideation system. Based on performance testing with 100+ epics, we document safe query patterns and potential bottlenecks.

## JSON Fields in Database Schema

The following tables use JSON TEXT columns for flexible data storage:

### Epics Table
- `codebase_context` - Patterns, similar features, reusable components (~400-600 bytes typical)
- `parent_tasks` - High-level task breakdown before expansion (~700-1000 bytes typical)
- `simplification_analysis` - Opportunities to leverage existing code
- `quality_validation` - Validation checklist results
- `alternative_approaches` - Technical approach options with trade-offs

### Tasks Table
- `execution_steps` - TDD cycle steps with commands (~700-900 bytes typical)
- `relevant_files` - Files to create/modify with operations
- `codebase_references` - Patterns to follow
- `validation_history` - Append-only progress updates

### Ideate Sessions Table
- `alternative_approaches` - 2-3 technical approaches with pros/cons
- `validation_checkpoints` - Section-by-section validation state
- `codebase_context` - Codebase analysis results

## Performance Test Results

From `packages/ideate/tests/performance_tests.rs`:

### Query Performance Benchmarks (100 epics)

| Operation | Duration | Threshold | Status |
|-----------|----------|-----------|--------|
| Insert 100 epics with JSON | 8.3ms | <5000ms | ✅ PASS |
| Query all epics (no JSON parsing) | 294µs | <100ms | ✅ PASS |
| Filter by status (indexed) | 107µs | <50ms | ✅ PASS |
| Query with JSON field retrieval | 167µs | <200ms | ✅ PASS |
| Complex multi-filter query | 111µs | <150ms | ✅ PASS |
| Parse large JSON (706 bytes) | 98µs | <50ms | ✅ PASS |

## Safe Query Patterns

### Pattern 1: Retrieve JSON Without Parsing
**When to use**: Passing JSON directly to client without server-side processing

```sql
-- FAST: No JSON parsing, just string retrieval
SELECT id, name, codebase_context
FROM epics
WHERE status = 'in_progress'
ORDER BY created_at DESC;
```

**Performance**: 100-200µs for 50 rows

### Pattern 2: Filter by Non-JSON Columns First
**When to use**: Filtering large datasets before JSON retrieval

```sql
-- GOOD: Use indexes first, then retrieve JSON
SELECT id, parent_tasks, quality_validation
FROM epics
WHERE status = 'in_progress'      -- Indexed column first
  AND complexity IN ('high', 'very_high')  -- Additional indexed filter
  AND codebase_context IS NOT NULL -- NULL check is cheap
ORDER BY created_at DESC
LIMIT 50;
```

**Performance**: 100-150µs for complex filters with JSON retrieval

### Pattern 3: Application-Layer JSON Parsing
**When to use**: Always (SQLite doesn't have native JSON functions in our schema)

```rust
// Retrieve JSON as string from database
let epic: (String, Option<String>) = sqlx::query_as(
    "SELECT id, codebase_context FROM epics WHERE id = ?"
)
.bind(&epic_id)
.fetch_one(&pool)
.await?;

// Parse JSON in application layer (Rust)
if let Some(context_json) = epic.1 {
    let context: CodebaseContext = serde_json::from_str(&context_json)?;
    // Use parsed data
}
```

**Performance**: JSON parsing adds ~50-100µs per row, acceptable for <100 rows

### Pattern 4: Use WITH Clauses for Complex Queries
**When to use**: Querying parent/subtask hierarchies or complex multi-step queries

```sql
-- GOOD: Use CTE to separate parent and subtask queries
WITH parent_tasks AS (
    SELECT id, title, epic_id, parent_task_id, complexity_score
    FROM tasks
    WHERE epic_id = ?
      AND parent_task_id IS NULL
),
subtasks AS (
    SELECT id, title, epic_id, parent_task_id, complexity_score
    FROM tasks
    WHERE epic_id = ?
      AND parent_task_id IS NOT NULL
)
SELECT
    p.id AS parent_id,
    p.title AS parent_title,
    COUNT(s.id) AS subtask_count,
    AVG(s.complexity_score) AS avg_complexity
FROM parent_tasks p
LEFT JOIN subtasks s ON s.parent_task_id = p.id
GROUP BY p.id, p.title
ORDER BY p.title;
```

**Benefits**:
- Clearer query structure and intent
- Better query plan optimization
- Easier to debug and maintain
- ~20-30% faster than subqueries for complex queries

**Performance**: WITH clauses are compiled into efficient query plans, typically <200µs for 50-100 tasks

### Pattern 5: Batch Operations with Transactions
**When to use**: Inserting multiple tasks/epics/records at once

```rust
// GOOD: Use transaction with batch insert
let mut tx = pool.begin().await?;

for task in tasks {
    sqlx::query(
        "INSERT INTO tasks (id, project_id, title, ...) VALUES (?, ?, ?, ...)"
    )
    .bind(&task.id)
    .bind(&task.project_id)
    .bind(&task.title)
    .execute(&mut *tx)
    .await?;
}

tx.commit().await?;
```

**Benefits**:
- Atomic operation (all-or-nothing)
- ~10x faster than individual inserts for 10+ records
- Reduced database round-trips
- Better error recovery

**Performance**: Transaction overhead ~1-2ms, but saves ~50-100µs per insert for batches >10

**Example Use Cases**:
- Generating 5-10 subtasks from a parent task
- Creating 10-20 tasks from epic decomposition
- Batch updating task dependencies after analysis

## Avoid These Anti-Patterns

### ❌ Anti-Pattern 1: SELECT * on Large Datasets
```sql
-- BAD: Retrieves all JSON fields even if not needed
SELECT * FROM epics;  -- Returns unnecessary JSON blobs
```

**Problem**: Wastes memory and bandwidth retrieving unused JSON
**Fix**: Select only needed columns

### ❌ Anti-Pattern 2: No WHERE Clause on Large Tables
```sql
-- BAD: Full table scan with JSON retrieval
SELECT id, parent_tasks FROM epics;  -- No filtering
```

**Problem**: Retrieves all rows including large JSON fields
**Fix**: Add WHERE clause to filter first

### ❌ Anti-Pattern 3: Parsing JSON in Loop
```rust
// BAD: Parse same JSON multiple times
for task in tasks {
    let steps = serde_json::from_str(&task.execution_steps)?; // Repeated parsing
    // Use steps
}
```

**Problem**: Redundant parsing if same data used multiple times
**Fix**: Parse once, cache result if reused

## Indexing Strategy

### Indexes Already in Place
```sql
CREATE INDEX idx_epics_status ON epics(status);
CREATE INDEX idx_epics_project ON epics(project_id);
```

### Why No JSON Indexes?
SQLite doesn't support JSON path indexes without JSON1 extension. Our approach:
1. **Filter by indexed columns first** (status, project_id, complexity)
2. **Retrieve JSON fields** for filtered result set (typically <100 rows)
3. **Parse in application layer** where we have better type safety and performance control

This is **more efficient** than SQL JSON path queries for our use case.

## Memory Considerations

### Typical JSON Field Sizes
- Small: 200-400 bytes (codebase references, file lists)
- Medium: 400-800 bytes (parent tasks, execution steps)
- Large: 800-1500 bytes (full codebase context with multiple arrays)

### Safe Batch Sizes
Based on testing:
- **<50 epics with JSON**: Safe for single query, <200µs
- **50-100 epics**: Consider pagination if JSON fields are large
- **100+ epics**: Use LIMIT/OFFSET pagination

### Memory Budget Example
```
100 epics × 1KB average JSON per field × 3 fields = ~300KB
```
This is acceptable for modern systems, but consider pagination for mobile clients.

## Best Practices Summary

### ✅ DO
1. Filter by indexed columns before retrieving JSON
2. Select only the JSON fields you need
3. Use pagination (LIMIT/OFFSET) for large result sets
4. Parse JSON once in application layer
5. Validate JSON schema on write to ensure parseable on read

### ❌ DON'T
1. Use `SELECT *` when you only need specific fields
2. Retrieve JSON fields without filtering first
3. Parse JSON repeatedly in loops
4. Store JSON larger than 5KB in these fields (consider normalization)
5. Attempt SQL JSON path queries (not supported without JSON1 extension)

## Monitoring & Alerts

Add performance monitoring for queries that:
1. Return >100 rows with JSON fields
2. Take >500ms to complete
3. Parse JSON >10KB in size

These thresholds indicate potential optimization opportunities.

## When to Normalize

Consider moving JSON data to separate tables when:
1. JSON field consistently >5KB
2. Need to filter by nested JSON properties frequently
3. JSON array has >20 items that need individual querying
4. Performance tests show >500ms query times

Example: If `execution_steps` grows to >50 steps per task, create a separate `task_steps` table.

## Future Optimizations

Potential improvements if performance becomes an issue:
1. **SQLite JSON1 Extension**: Enable for JSON path queries
2. **Separate Tables**: Normalize frequently-queried JSON fields
3. **Caching Layer**: Cache parsed JSON for hot paths
4. **Compression**: Use BLOB with zlib for very large JSON (>10KB)

Currently, none of these are needed based on performance testing results.

## References

- Performance test implementation: `packages/ideate/tests/performance_tests.rs`
- Integration test patterns: `packages/ideate/tests/integration_tests.rs`
- Schema definition: `packages/storage/migrations/001_initial_schema.sql`

---

**Last Updated**: 2025-01-02
**Performance Baseline**: SQLite 3.x, 100-epic dataset, in-memory database
