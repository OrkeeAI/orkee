# Architecture Audit: Chat Mode Pattern vs Rust AI Calls

## Executive Summary

**Goal**: Migrate ALL features from "Rust makes AI calls" to "Frontend makes AI calls via AI SDK" (Chat Mode pattern).

**Current State**:
- ✅ **Chat Mode**: Already implements correct pattern (Frontend AI SDK + Backend CRUD)
- ✅ **Spec Workflow**: Already uses AI SDK for PRD/task generation
- ❌ **All Other Features**: Use incorrect pattern (Rust makes AI calls or HTTP calls to proxy)
- ❌ **Previous Migration Work**: 12 commits of Rust→proxy HTTP calls need to be reverted

## Pattern Comparison

### ✅ Correct Pattern (Chat Mode)

**Data Flow**:
1. User action in UI → Frontend makes AI SDK call (`streamText`, `generateObject`)
2. AI SDK routes through `/api/ai/{provider}/*` proxy (adds API keys)
3. AI proxy forwards to provider (Anthropic, OpenAI, etc.)
4. Response streams back to frontend
5. Frontend saves result to backend via CRUD API
6. Backend stores data in database (NO AI logic)

**Files**:
- **Frontend**: `packages/dashboard/src/services/chat-ai.ts` - AI SDK calls
- **Backend**: `packages/api/src/ideate_chat_handlers.rs` - Pure CRUD

**Key Characteristics**:
- Backend never makes AI calls
- Backend only: `save_message()`, `get_history()`, etc.
- All intelligence in frontend TypeScript
- AI SDK handles streaming, retries, error handling

### ❌ Incorrect Pattern (Most Features)

**Data Flow (Current)**:
1. User action in UI → API call to Rust backend
2. Rust backend makes HTTP request to AI proxy or uses AIService
3. Rust parses response and saves to database
4. Returns result to frontend

**Problems**:
- AI logic mixed with backend CRUD
- Can't use AI SDK features (streaming, structured output, etc.)
- Harder to test and maintain
- Defeats purpose of having AI SDK

---

## Feature Audit

### ✅ Features Using CORRECT Pattern

#### 1. Chat Mode Discovery (`packages/api/src/ideate_chat_handlers.rs`)
**Backend Functions** (CRUD only):
- `create_session()` - Create chat session
- `send_message()` - Save user/assistant message
- `get_history()` - Load chat history
- `update_session()` - Update session metadata
- `list_sessions()` - List all sessions

**Frontend AI SDK** (`packages/dashboard/src/services/chat-ai.ts`):
- `streamChatResponse()` - Uses `streamText()` for chat streaming
- `extractInsights()` - Uses `generateObject()` for insight extraction
- `calculateQualityScore()` - Uses `generateObject()` for quality metrics
- `generatePRD()` - Uses `generateText()` for PRD generation

**Status**: ✅ Reference implementation - no changes needed

#### 2. Spec Workflow (`packages/dashboard/src/lib/workflows/spec-workflow.ts`)
**Backend Functions** (NOT CHECKED YET - likely CRUD):
- Spec storage/retrieval
- Task management
- PRD storage

**Frontend AI SDK** (`packages/dashboard/src/lib/ai/services.ts`):
- `analyzePRD()` - Uses `generateObject()` with PRD analysis schema
- `generateSpec()` - Uses `generateObject()` for capability generation
- `suggestTasks()` - Uses `generateObject()` for task suggestions
- `analyzeOrphanTask()` - Uses `generateObject()` for task analysis
- `validateTaskCompletion()` - Uses `generateObject()` for validation
- `refineSpec()` - Uses `generateObject()` for refinement
- `generateSpecMarkdown()` - Uses `generateText()` for markdown generation
- `regeneratePRD()` - Uses `generateText()` for PRD regeneration

**Status**: ✅ Already correct - no changes needed

---

### ❌ Features Using INCORRECT Pattern

#### 3. PRD Generation & Management (`packages/ideate/src/prd_generator.rs`)

**Current Rust AI Functions** (NEED TO REMOVE):
- `generate_prd()` - Makes HTTP call to AI proxy for PRD generation
- `refine_prd()` - Makes HTTP call to AI proxy for refinement
- `expand_section()` - Makes HTTP call to AI proxy for expansion
- `validate_prd()` - Makes HTTP call to AI proxy for validation
- `stream_prd_generation()` - Uses legacy AIService for streaming (TEMPORARY)
- `stream_section_expansion()` - Uses legacy AIService for streaming (TEMPORARY)

**Backend CRUD Functions** (KEEP):
- `save_prd()` - Store PRD in database
- `get_prd()` - Load PRD from database
- `update_prd()` - Update PRD metadata
- `list_prds()` - List all PRDs

**Migration Plan**:
1. Remove all 6 AI functions from `prd_generator.rs`
2. Keep only CRUD functions
3. Create `packages/dashboard/src/services/prd-ai.ts` with AI SDK calls:
   - `generatePRD()` using `generateText()`
   - `refinePRD()` using `generateText()`
   - `expandSection()` using `generateText()`
   - `validatePRD()` using `generateObject()`
   - All with streaming support via `streamText()`

**Affected API Handlers**:
- `packages/api/src/ideate_generation_handlers.rs` - Remove AI logic, add CRUD endpoints

---

#### 4. Insight Extraction (`packages/ideate/src/insight_extractor.rs`)

**Current Rust AI Functions** (NEED TO REMOVE):
- `extract_insights()` - Makes HTTP call to AI proxy

**Backend CRUD Functions** (KEEP):
- `save_insight()` - Store insight
- `get_insights()` - Load insights
- `link_insight_to_feature()` - Link insight to feature

**Migration Plan**:
1. Remove `extract_insights()` AI logic
2. Frontend already has this: `chat-ai.ts:extractInsights()` using AI SDK
3. Verify all callers use frontend version

**Affected API Handlers**:
- `packages/api/src/ideate_chat_handlers.rs` - Comment at line 62-63 says "Insight extraction is now handled by the frontend"

**Status**: May already be partially migrated - needs verification

---

#### 5. Research & Competitor Analysis (`packages/ideate/src/research_analyzer.rs`)

**Current Rust AI Functions** (NEED TO REMOVE):
- `analyze_competitor()` - Makes HTTP call to AI proxy
- `analyze_gaps()` - Makes HTTP call to AI proxy
- `extract_ui_patterns()` - Makes HTTP call to AI proxy
- `extract_lessons()` - Makes HTTP call to AI proxy
- `synthesize_research()` - Makes HTTP call to AI proxy

**Backend CRUD Functions** (KEEP):
- `save_competitor()` - Store competitor data
- `get_competitors()` - Load competitors
- `add_similar_project()` - Store similar project
- `get_similar_projects()` - Load similar projects

**Migration Plan**:
1. Remove all 5 AI functions
2. Create `packages/dashboard/src/services/research-ai.ts` with AI SDK calls:
   - `analyzeCompetitor()` using `generateObject()`
   - `analyzeGaps()` using `generateObject()`
   - `extractUIPatterns()` using `generateObject()`
   - `extractLessons()` using `generateObject()`
   - `synthesizeResearch()` using `generateObject()`

**Affected API Handlers**:
- `packages/api/src/ideate_research_handlers.rs` - All 6 handlers need migration

---

#### 6. Expert Roundtable (`packages/ideate/src/expert_moderator.rs`)

**Current Rust AI Functions** (NEED TO REMOVE):
- `run_discussion()` - Makes HTTP call to AI proxy for each expert turn
- `handle_interjection()` - Makes HTTP call to AI proxy for moderator response
- `extract_insights()` - Makes HTTP call to AI proxy for insight extraction

**Backend CRUD Functions** (KEEP):
- `create_roundtable()` - Create roundtable session
- `add_participants()` - Add experts to roundtable
- `save_message()` - Store roundtable message
- `get_messages()` - Load roundtable messages
- `save_insight()` - Store extracted insight
- `get_insights()` - Load insights

**Migration Plan**:
1. Remove all 3 AI functions
2. Create `packages/dashboard/src/services/roundtable-ai.ts` with AI SDK calls:
   - `generateExpertResponse()` using `streamText()` for each expert turn
   - `generateModeratorResponse()` using `streamText()` for interjections
   - `extractInsights()` using `generateObject()` for insight extraction
3. Frontend manages discussion loop and UI updates

**Affected API Handlers**:
- `packages/api/src/ideate_roundtable_handlers.rs` - Several handlers need migration
- Note: SSE streaming endpoint (`stream_discussion`) may need redesign

**Special Consideration**:
- Current implementation runs discussion in background task on backend
- New implementation: Frontend drives discussion loop, makes AI SDK call for each turn
- May need new API design for user interjections during streaming

---

#### 7. Dependency Analysis (`packages/ideate/src/dependency_analyzer.rs`)

**Current Rust AI Functions** (NEED TO REMOVE):
- `analyze_dependencies()` - Makes HTTP call to AI proxy

**Backend CRUD Functions** (KEEP):
- `get_dependencies()` - Load dependencies
- `create_dependency()` - Store dependency
- `delete_dependency()` - Remove dependency

**Migration Plan**:
1. Remove `analyze_dependencies()` AI logic
2. Create `packages/dashboard/src/services/dependency-ai.ts`:
   - `analyzeDependencies()` using `generateObject()` for dependency extraction

**Affected API Handlers**:
- `packages/api/src/ideate_dependency_handlers.rs` - Handler at line 108-119

---

#### 8. Generic AI Handlers (`packages/api/src/ai_handlers.rs`)

**Current Rust AI Functions** (ALL INCORRECT - NEED TO REMOVE):
- `analyze_prd()` - Makes HTTP call via `call_ai_proxy()` helper
- `generate_spec()` - Makes HTTP call via `call_ai_proxy()` helper
- `suggest_tasks()` - Makes HTTP call via `call_ai_proxy()` helper
- `refine_spec()` - Makes HTTP call via `call_ai_proxy()` helper
- `validate_completion()` - Makes HTTP call via `call_ai_proxy()` helper
- `call_ai_proxy<T>()` - Helper function for HTTP calls (lines 514-608)

**Status**:
- These handlers were migrated in previous (incorrect) work
- **Spec Workflow already has frontend AI SDK replacements** (see Feature #2 above)
- These handlers should be DELETED or converted to CRUD-only

**Migration Plan**:
1. Verify all calls to these handlers come from Spec Workflow
2. If yes: Delete entire file
3. If no: Identify other callers and migrate them to use Spec Workflow AI SDK services

---

## Summary Statistics

### Features by Status

**✅ Correct (2 features)**:
- Chat Mode Discovery
- Spec Workflow (PRD/Task AI)

**❌ Incorrect - Needs Migration (6 features)**:
- PRD Generation (6 functions)
- Insight Extraction (1 function, may be partially done)
- Research Analysis (5 functions)
- Expert Roundtable (3 functions)
- Dependency Analysis (1 function)
- Generic AI Handlers (5 handlers + helper, likely duplicates of Spec Workflow)

**Total AI Functions to Remove**: ~21 Rust functions making AI calls

**Total Frontend AI Services to Create**: ~5 new TypeScript files with AI SDK integration

---

## Migration Priority

### Priority 1: High Value, Low Risk
1. **Dependency Analysis** (1 function) - Simple, single function
2. **Insight Extraction** (1 function) - May already be done, needs verification

### Priority 2: Medium Complexity
3. **PRD Generation** (6 functions) - Well-defined, but streaming needs attention
4. **Research Analysis** (5 functions) - Straightforward structured generation

### Priority 3: High Complexity
5. **Expert Roundtable** (3 functions) - Complex: multi-turn discussion, background task, SSE streaming
6. **Generic AI Handlers** (5 handlers) - Verify vs Spec Workflow, may be duplicates

---

## Revert Plan

### Commits to Review/Revert (12 total)

1. `efcb6b3` - "Uncomment AI handler routes in preparation for proxy migration"
2. `3ccc18d` - "Migrate all 5 AI handlers from AIService to AI proxy"
3. `1dc70e6` - "Update vibekit.md: Mark Phase 1.1 (API handlers) as completed"
4. `01a1564` - "Migrate Ideate package AI calls (partial)"
5. `0df1a4e` - "Complete Ideate package AI migration for research, expert, and dependency analyzers"
6. `8c36517` - "Update vibekit.md: Mark Phase 1.2 (Ideate package) as completed"
7. `abb8d78` - "Phase 1 status update: Document blocker for cleanup task"
8. `9cf5f87` - "Extend Phase 1 scope to include additional API handlers"
9. `80b890f` - "Add temporary AIService import for streaming functions"
10. `9f012b3` - "Migrate remaining API handlers to use updated library signatures"
11. `fea8ae3` - "Architecture pivot: Move all AI logic to frontend"
12. `00959e6` - "Clarify Phase 1 architecture: Use Chat Mode pattern for all features"

**Revert Strategy**:
- Commits 11-12 (architecture clarification in vibekit.md): **KEEP** - These document correct approach
- Commits 1-10 (Rust→proxy code changes): **SELECTIVE REVERT** - Revert AI logic, keep any CRUD improvements

**Note**: Uncommitted change in `ideate_dependency_handlers.rs` also needs review

---

## Next Steps

1. ✅ Complete this audit
2. Get Joe's approval on migration priority order
3. Start with Priority 1 features (Dependency Analysis, Insight Extraction)
4. For each feature:
   - Remove Rust AI logic
   - Create frontend AI SDK service
   - Update API handlers to CRUD-only
   - Test end-to-end
   - Commit incrementally
5. Revert incorrect commits after migration complete
