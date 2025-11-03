# Caching Strategy - Performance Optimization Guide

## Overview

This document describes the caching infrastructure for expensive operations in Orkee's ideation system. Three dedicated cache tables optimize performance for complexity analysis, codebase context retrieval, and PRD validation.

## Cache Tables

### 1. Complexity Analysis Cache

**Table**: `complexity_analysis_cache`
**Purpose**: Cache expensive complexity calculation results for epics
**TTL**: Optional expiration (recommended: 24 hours)

```sql
CREATE TABLE complexity_analysis_cache (
    epic_id TEXT PRIMARY KEY,
    complexity_report TEXT NOT NULL, -- JSON: ComplexityReport
    computed_at TEXT NOT NULL,
    expires_at TEXT, -- Optional TTL
    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE
);
```

**When to Use**:
- Epic complexity analysis involves multiple heuristics and calculations
- Results rarely change unless epic details are modified
- Cache hit saves ~500-2000ms depending on epic size

**Cache Key**: `epic_id`
**Invalidation**: When epic description, architecture, or technical approach changes

**Usage Pattern**:
```rust
// Check cache first
let cached = sqlx::query_as::<_, (String,)>(
    "SELECT complexity_report FROM complexity_analysis_cache
     WHERE epic_id = ? AND (expires_at IS NULL OR expires_at > datetime('now'))"
)
.bind(&epic_id)
.fetch_optional(&pool)
.await?;

if let Some((report_json,)) = cached {
    return Ok(serde_json::from_str(&report_json)?);
}

// Cache miss - compute and store
let report = analyze_complexity(&epic).await?;
let report_json = serde_json::to_string(&report)?;

sqlx::query(
    "INSERT INTO complexity_analysis_cache (epic_id, complexity_report, expires_at)
     VALUES (?, ?, datetime('now', '+24 hours'))
     ON CONFLICT(epic_id) DO UPDATE SET
         complexity_report = excluded.complexity_report,
         computed_at = datetime('now'),
         expires_at = excluded.expires_at"
)
.bind(&epic_id)
.bind(&report_json)
.execute(&pool)
.await?;
```

### 2. Codebase Context Cache

**Table**: `codebase_context_cache`
**Purpose**: Cache codebase analysis results (patterns, similar features, reusable components)
**TTL**: Based on codebase file changes (recommended: check git commit hash)

```sql
CREATE TABLE codebase_context_cache (
    project_id TEXT PRIMARY KEY,
    context_data TEXT NOT NULL, -- JSON: CodebaseContext
    file_count INTEGER,
    patterns_found INTEGER,
    analyzed_at TEXT NOT NULL,
    expires_at TEXT
);
```

**When to Use**:
- Codebase analysis scans hundreds of files
- Results change infrequently (only when code patterns change)
- Cache hit saves ~2-10 seconds depending on project size

**Cache Key**: `project_id`
**Invalidation**: When git commit hash changes or after 7 days

**Usage Pattern**:
```rust
// Check cache with git hash validation
let git_hash = get_current_git_hash(&project_path)?;
let cached = sqlx::query_as::<_, (String, String)>(
    "SELECT context_data, git_hash FROM codebase_context_cache
     WHERE project_id = ?"
)
.bind(&project_id)
.fetch_optional(&pool)
.await?;

if let Some((context_json, cached_hash)) = cached {
    if cached_hash == git_hash {
        return Ok(serde_json::from_str(&context_json)?);
    }
}

// Cache miss or stale - analyze and store
let context = analyze_codebase(&project_path).await?;
let context_json = serde_json::to_string(&context)?;

sqlx::query(
    "INSERT INTO codebase_context_cache
     (project_id, context_data, file_count, patterns_found, git_hash, expires_at)
     VALUES (?, ?, ?, ?, ?, datetime('now', '+7 days'))
     ON CONFLICT(project_id) DO UPDATE SET
         context_data = excluded.context_data,
         file_count = excluded.file_count,
         patterns_found = excluded.patterns_found,
         git_hash = excluded.git_hash,
         analyzed_at = datetime('now'),
         expires_at = excluded.expires_at"
)
.bind(&project_id)
.bind(&context_json)
.bind(context.file_count)
.bind(context.patterns.len())
.bind(&git_hash)
.execute(&pool)
.await?;
```

### 3. Validation Score Cache

**Table**: `validation_score_cache`
**Purpose**: Cache PRD section validation results
**TTL**: Content-based invalidation (SHA256 hash of section content)

```sql
CREATE TABLE validation_score_cache (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    section_name TEXT NOT NULL,
    content_hash TEXT NOT NULL, -- SHA256 of section content
    validation_result TEXT NOT NULL, -- JSON: PRDValidationResult
    computed_at TEXT NOT NULL,
    UNIQUE(session_id, section_name, content_hash)
);
```

**When to Use**:
- Validation involves LLM calls or complex rule evaluation
- Same content should produce same validation result
- Cache hit saves ~200-500ms per section

**Cache Key**: `(session_id, section_name, content_hash)`
**Invalidation**: Automatic when content changes (different hash)

**Usage Pattern**:
```rust
use sha2::{Sha256, Digest};

// Compute content hash
let mut hasher = Sha256::new();
hasher.update(section_content.as_bytes());
let content_hash = format!("{:x}", hasher.finalize());

// Check cache
let cached = sqlx::query_as::<_, (String,)>(
    "SELECT validation_result FROM validation_score_cache
     WHERE session_id = ? AND section_name = ? AND content_hash = ?"
)
.bind(&session_id)
.bind(&section_name)
.bind(&content_hash)
.fetch_optional(&pool)
.await?;

if let Some((result_json,)) = cached {
    return Ok(serde_json::from_str(&result_json)?);
}

// Cache miss - validate and store
let result = validate_section(&section_content).await?;
let result_json = serde_json::to_string(&result)?;

sqlx::query(
    "INSERT INTO validation_score_cache
     (id, session_id, section_name, content_hash, validation_result)
     VALUES (?, ?, ?, ?, ?)
     ON CONFLICT(session_id, section_name, content_hash) DO NOTHING"
)
.bind(generate_id())
.bind(&session_id)
.bind(&section_name)
.bind(&content_hash)
.bind(&result_json)
.execute(&pool)
.await?;
```

## Cache Maintenance

### Expiration Cleanup

Run periodic cleanup task (recommended: daily):

```sql
-- Delete expired complexity analysis cache
DELETE FROM complexity_analysis_cache
WHERE expires_at IS NOT NULL AND expires_at < datetime('now');

-- Delete expired codebase context cache
DELETE FROM codebase_context_cache
WHERE expires_at IS NOT NULL AND expires_at < datetime('now');

-- Delete old validation scores (keep last 30 days)
DELETE FROM validation_score_cache
WHERE computed_at < datetime('now', '-30 days');
```

### Cache Statistics

Monitor cache effectiveness:

```sql
-- Complexity cache hit rate (requires instrumentation)
SELECT
    COUNT(*) as cached_epics,
    AVG(julianday('now') - julianday(computed_at)) as avg_age_days
FROM complexity_analysis_cache
WHERE expires_at IS NULL OR expires_at > datetime('now');

-- Codebase context freshness
SELECT
    project_id,
    file_count,
    patterns_found,
    julianday('now') - julianday(analyzed_at) as age_days
FROM codebase_context_cache
ORDER BY age_days DESC;

-- Validation cache efficiency
SELECT
    session_id,
    section_name,
    COUNT(*) as cache_entries,
    MAX(computed_at) as last_validated
FROM validation_score_cache
GROUP BY session_id, section_name;
```

## Performance Impact

### Before Caching
- Complexity analysis: 500-2000ms per epic
- Codebase analysis: 2-10 seconds per project
- Section validation: 200-500ms per section

### After Caching (Cache Hit)
- Complexity analysis: <5ms (400x faster)
- Codebase analysis: <10ms (200-1000x faster)
- Section validation: <5ms (40-100x faster)

### Expected Hit Rates
- Complexity cache: 70-80% (epics change infrequently)
- Codebase cache: 90-95% (code changes less often than PRDs)
- Validation cache: 50-60% (sections get edited during refinement)

## Best Practices

### DO
✅ Set reasonable TTLs (24 hours for complexity, 7 days for codebase)
✅ Use content hashing for validation cache
✅ Invalidate on epic/section changes
✅ Monitor cache hit rates
✅ Clean up expired entries regularly

### DON'T
❌ Cache without expiration (data grows unbounded)
❌ Use cache for rapidly changing data
❌ Skip cache invalidation logic
❌ Cache error results (only cache successful computations)
❌ Store sensitive data in cache without encryption

## Future Optimizations

1. **In-Memory Layer**: Add Redis/Memcached for hot data
2. **Partial Cache**: Cache intermediate computation steps
3. **Proactive Refresh**: Refresh cache before expiration
4. **Distributed Cache**: Share cache across multiple instances

---

**Last Updated**: 2025-01-02
**Schema Version**: 001_initial_schema.sql with performance cache tables
