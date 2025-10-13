# Environment Variable Centralization Recommendation

## Current State

Environment variable parsing is currently duplicated across multiple packages:
- `packages/preview/src/env.rs` - Centralized utilities (23 uses)
- `packages/dashboard/src-tauri/src/tray.rs` - Custom parsing functions (5 uses)
- `packages/dashboard/src-tauri/src/lib.rs` - Custom `parse_env_with_fallback` (5 uses)
- `packages/cli/src/api/path_validator.rs` - Direct `std::env::var` calls (8 uses)
- 11 other files with scattered env var parsing (28+ total uses)

**Total**: 69 occurrences across 15 files

## Problem

1. **Code Duplication**: Same parsing logic repeated in multiple places
2. **Inconsistent Error Handling**: Different packages handle parse failures differently
3. **No Standard Validation**: Some code validates ranges, others don't
4. **Maintenance Burden**: Changes to env var parsing require updates in multiple places
5. **Testing Gaps**: Not all env var parsing code has comprehensive tests

## Existing Solution (Preview Package)

The preview package already has excellent env var utilities (`packages/preview/src/env.rs`):

### Available Functions

```rust
// Basic parsing with default
pub fn parse_env_or_default<T>(var_name: &str, default: T) -> T

// Parsing with validation (e.g., range checks)
pub fn parse_env_or_default_with_validation<T, F>(
    var_name: &str,
    default: T,
    validator: F
) -> T

// Parsing with fallback variable support
pub fn parse_env_with_fallback<T>(
    primary_var: &str,
    fallback_var: &str,
    default: T
) -> T
```

### Benefits
- ✅ Type-safe parsing
- ✅ Automatic warning logging for invalid values
- ✅ Comprehensive test coverage
- ✅ Validation support
- ✅ Fallback variable support

## Recommended Solution

### Option 1: Create Shared `orkee-config` Crate (Recommended)

Create a new shared crate specifically for configuration:

```
packages/config/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── env.rs      # Move from preview/src/env.rs
    └── constants.rs # Centralize all env var names
```

**Benefits**:
- Clear separation of concerns
- Can include other config-related utilities
- No risk of circular dependencies
- Other packages only depend on config (not on each other)

**Implementation Steps**:
1. Create `packages/config` crate
2. Move `preview/src/env.rs` → `config/src/env.rs`
3. Add `constants.rs` with all env var names
4. Update all packages to depend on `orkee-config`
5. Migrate env parsing code package-by-package
6. Add integration tests

### Option 2: Extend `orkee-projects` Crate

Add env utilities to the existing shared `orkee-projects` crate:

```
packages/projects/src/
├── env.rs  # New: env parsing utilities
```

**Benefits**:
- No new crate needed
- `orkee-projects` is already used by CLI and TUI
- Faster to implement

**Drawbacks**:
- Mixes project management concerns with configuration
- Creates tighter coupling
- Less clear architecture

## Migration Strategy

### Phase 1: Create Shared Module (Week 1)
- [ ] Choose Option 1 or Option 2
- [ ] Create/update crate structure
- [ ] Move env parsing utilities
- [ ] Add comprehensive documentation
- [ ] Create migration guide

### Phase 2: Migrate High-Traffic Code (Week 2)
- [ ] Migrate `tray.rs` HTTP timeout parsing
- [ ] Migrate `lib.rs` port configuration
- [ ] Migrate `path_validator.rs` sandbox mode
- [ ] Update tests

### Phase 3: Migrate Remaining Code (Week 3)
- [ ] Migrate CLI package env vars
- [ ] Migrate cloud package env vars
- [ ] Migrate preview package to use shared module
- [ ] Remove duplicated code

### Phase 4: Documentation & Standards (Week 4)
- [ ] Document all environment variables in one place
- [ ] Create env var naming conventions
- [ ] Add validation guidelines
- [ ] Update CLAUDE.md

## Example Migration

### Before (tray.rs)
```rust
fn get_http_request_timeout_secs() -> u64 {
    match std::env::var("ORKEE_HTTP_REQUEST_TIMEOUT_SECS") {
        Ok(raw_value) => match raw_value.parse::<u64>() {
            Ok(parsed_value) => {
                if (1..=30).contains(&parsed_value) {
                    parsed_value
                } else {
                    warn!("Invalid value, using default");
                    DEFAULT_HTTP_REQUEST_TIMEOUT_SECS
                }
            }
            Err(_) => {
                warn!("Unparseable value, using default");
                DEFAULT_HTTP_REQUEST_TIMEOUT_SECS
            }
        },
        Err(_) => DEFAULT_HTTP_REQUEST_TIMEOUT_SECS,
    }
}
```

### After (with orkee-config)
```rust
use orkee_config::env::parse_env_or_default_with_validation;
use orkee_config::constants::ORKEE_HTTP_REQUEST_TIMEOUT_SECS;

fn get_http_request_timeout_secs() -> u64 {
    parse_env_or_default_with_validation(
        ORKEE_HTTP_REQUEST_TIMEOUT_SECS,
        5,
        |v| v >= 1 && v <= 30
    )
}
```

## Priority & Effort

- **Priority**: Low (architectural improvement, not a bug)
- **Effort**: Medium (2-4 weeks)
- **Impact**: High (reduces maintenance burden, improves consistency)
- **Risk**: Low (can be done incrementally, well-tested utilities)

## Decision

This recommendation is marked as **LOW PRIORITY** in the code review. Implementation should be considered when:
1. Adding significant new environment variables
2. Refactoring configuration management
3. During a dedicated technical debt sprint
4. When inconsistent env var behavior causes issues

## References

- Current implementation: `packages/preview/src/env.rs`
- Related files: See grep results for `std::env::var|parse_env` across codebase
- Similar patterns: Serde for serialization, tracing for logging
