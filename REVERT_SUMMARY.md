# Revert Summary: Rust→Proxy HTTP Calls Migration

## What Was Reverted

Reverted all code changes from commits `efcb6b3` through `9f012b3` (10 commits total) that incorrectly implemented a "Rust makes HTTP calls to AI proxy" pattern.

### Commits Scope (Reverted Code Only)

1. `efcb6b3` - "Uncomment AI handler routes in preparation for proxy migration"
2. `3ccc18d` - "Migrate all 5 AI handlers from AIService to AI proxy"
3. `1dc70e6` - "Update vibekit.md: Mark Phase 1.1 (API handlers) as completed" (doc only)
4. `01a1564` - "Migrate Ideate package AI calls (partial)"
5. `0df1a4e` - "Complete Ideate package AI migration for research, expert, and dependency analyzers"
6. `8c36517` - "Update vibekit.md: Mark Phase 1.2 (Ideate package) as completed" (doc only)
7. `abb8d78` - "Phase 1 status update: Document blocker for cleanup task" (doc only)
8. `9cf5f87` - "Extend Phase 1 scope to include additional API handlers"
9. `80b890f` - "Add temporary AIService import for streaming functions"
10. `9f012b3` - "Migrate remaining API handlers to use updated library signatures"

### Files Restored to Pre-Migration State

**API Handlers** (11 files):
- `packages/api/src/ai_handlers.rs` - Restored to use AIService directly
- `packages/api/src/ideate_chat_handlers.rs` - Restored original
- `packages/api/src/ideate_research_handlers.rs` - Restored to pass AIService instances
- `packages/api/src/ideate_roundtable_handlers.rs` - Restored original ExpertModerator usage
- `packages/api/src/lib.rs` - Re-commented AI router (line 708-720)
- `packages/cli/src/api/mod.rs` - Re-commented AI router nesting

**Ideate Library** (5 files):
- `packages/ideate/src/prd_generator.rs` - Restored to use AIService
- `packages/ideate/src/insight_extractor.rs` - Restored original
- `packages/ideate/src/research_analyzer.rs` - Restored to use AIService
- `packages/ideate/src/expert_moderator.rs` - Restored to accept AIService
- `packages/ideate/src/dependency_analyzer.rs` - Restored to accept AIService

### What Was Removed (Incorrect Pattern)

**From `ai_handlers.rs`**:
- `call_ai_proxy<T>()` helper function (~95 lines) that made HTTP calls to `/api/ai/anthropic/v1/messages`
- HTTP request logic using `reqwest::Client`
- All 5 handlers migrated to use HTTP proxy calls

**From Ideate Library**:
- HTTP client imports and usage
- `user_id` and `model` parameters (replaced AIService instances)
- HTTP request bodies matching Anthropic Messages API format
- Error handling for HTTP failures

**Total Lines Removed**: ~1,384 lines (net)

---

## What Was Kept

### Documentation (Correct Architecture)

**Preserved Files**:
- `vibekit.md` - Master plan with correct Chat Mode pattern documented
- `ARCHITECTURE_AUDIT.md` - Comprehensive feature audit
- All documentation commits (11 and 12) were NOT reverted:
  - `fea8ae3` - "Architecture pivot: Move all AI logic to frontend"
  - `00959e6` - "Clarify Phase 1 architecture: Use Chat Mode pattern for all features"
  - `5064fda` - "Add comprehensive architecture audit"
  - `02392c9` - "Update vibekit.md Phase 1 with audit results"

### Code State After Revert

**Current State**: All files restored to commit `8b7347b` (before incorrect migration started)

**AIService Usage**: Now back to using `orkee_ai::AIService` directly in:
- API handlers (`ai_handlers.rs`, `ideate_research_handlers.rs`, etc.)
- Ideate library (`prd_generator.rs`, `research_analyzer.rs`, etc.)

**Compilation Status**: ✅ Full workspace compiles successfully

---

## Why This Was Incorrect

The reverted approach had Rust backend making HTTP calls to the AI proxy:

```rust
// INCORRECT PATTERN (reverted)
let client = reqwest::Client::new();
let response = client
    .post("http://localhost:4001/api/ai/anthropic/v1/messages")
    .header("x-user-id", user_id)
    .json(&anthropic_request)
    .send()
    .await?;
```

**Problems**:
1. Backend still contained AI logic
2. Couldn't use Vercel AI SDK features (streaming, structured output, retries)
3. Mixed backend HTTP client with frontend concerns
4. Defeated purpose of having AI SDK in TypeScript

## Correct Pattern (To Be Implemented)

**Frontend makes ALL AI calls** using Vercel AI SDK:

```typescript
// CORRECT PATTERN (from chat-ai.ts)
const result = await streamText({
  model: getModel('anthropic', 'claude-sonnet-4-20250514'),
  system: systemPrompt,
  messages: chatHistory,
  temperature: 0.7,
});
```

**Backend is pure CRUD**:

```rust
// CORRECT PATTERN (from ideate_chat_handlers.rs)
pub async fn send_message(
    State(db): State<DbState>,
    Path(session_id): Path<String>,
    Json(input): Json<SendMessageInput>,
) -> impl IntoResponse {
    let manager = ChatManager::new(db.pool.clone());
    let result = manager.add_message(&session_id, input.role, input.content).await;
    ok_or_internal_error(result, "Failed to send message")
}
```

---

## Next Steps

Now that we have a clean slate, we can proceed with the correct migration:

1. Start with Priority 1 features (Dependency Analysis, Insight Extraction)
2. Remove Rust AI logic from each feature
3. Create TypeScript AI SDK services in frontend
4. Convert backend handlers to pure CRUD
5. Test end-to-end

See `ARCHITECTURE_AUDIT.md` for detailed migration plan.
