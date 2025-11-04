# AI Architecture Rework: Migrate to Frontend AI SDK Pattern

## Project Overview

This document tracks the migration of all AI operations from legacy Rust AIService to the modern Frontend AI SDK pattern (Chat Mode). This is a pure architectural cleanup that improves code quality and maintainability.

### Goal

**Apply the Chat Mode pattern to ALL features**: Backend Rust = Pure CRUD only. Frontend TypeScript = All AI calls via AI SDK.

### Status: In Progress 🚧
**Completion:** 80% complete

**Timeline:** 2 weeks (Weeks 1-2 of overall VibeKit project)

## Correct Architecture (Chat Mode Pattern)

- ✅ **Frontend** - Makes all AI calls using Vercel AI SDK (streamText, generateObject)
- ✅ **AI Proxy** - Frontend AI SDK routes through `/api/ai/{provider}/*` for API key management
- ✅ **Backend** - Pure CRUD only (save messages, save PRD, etc.)
- ❌ **NO** Rust→AI proxy HTTP calls (what was incorrectly implemented)

## Reference Implementation: Chat Mode ✅

Chat mode **already implements the correct pattern:**

**Backend** (`packages/api/src/ideate_chat_handlers.rs`):
- `send_message()` - Saves user message to DB, returns immediately
- `get_history()` - Returns chat history from DB
- Pure CRUD, NO AI calls

**Frontend** (`packages/dashboard/src/services/chat-ai.ts`):
- `streamChatResponse()` - Uses AI SDK `streamText()` to stream AI response
- `extractInsights()` - Uses AI SDK `generateObject()` for structured extraction
- All AI calls go through AI SDK → AI proxy → Anthropic/OpenAI/etc

**This is the pattern to replicate for ALL features.**

## 📊 Audit Results Summary

**See `ARCHITECTURE_AUDIT.md` for detailed feature-by-feature analysis.**

**Features Using Correct Pattern (2):**
- ✅ Chat Mode Discovery - Reference implementation
- ✅ Spec Workflow - PRD/Task AI already uses AI SDK

**Features Needing Migration (6):**
- ✅ PRD Generation (6 Rust AI functions) - **COMPLETE**
- ✅ Insight Extraction (1 function) - **COMPLETE**
- ✅ Research Analysis (5 functions) - **COMPLETE**
- ✅ Expert Roundtable (3 AI functions) - **COMPLETE**
- ✅ Dependency Analysis (1 function) - **COMPLETE**
- ❌ Generic AI Handlers (5 handlers, likely duplicates)

**Total Work**: Remove ~21 Rust AI functions, create ~5 TypeScript AI service files

## Progress Tracker

### Task Breakdown
- [x] **Priority 1.1:** Audit Complete
  - [x] 1.1 Audit Features vs Chat Mode Pattern
  - [x] Create comprehensive audit document (`ARCHITECTURE_AUDIT.md`)

- [x] **Priority 1.2:** Simple Migrations (Dependency Analysis + Insight Extraction) - **COMPLETE**
  - [x] Dependency Analysis Backend (529→153 lines)
  - [x] Dependency Analysis Frontend (+326 lines)
  - [x] Insight Extraction Backend (178→31 lines)
  - [x] Insight Extraction Frontend (already done in chat-ai.ts)

- [x] **Priority 1.3:** Medium Complexity - **COMPLETE**
  - [x] PRD Generation Backend Complete (991→599 lines, removed 6 AI functions)
  - [x] PRD Generation Frontend Complete (+766 lines)
  - [x] Research Analysis Backend Complete (558→333 lines, removed 5 AI functions)
  - [x] Research Analysis Frontend Complete (+389 lines)

- [ ] **Priority 1.4:** Complex Migrations
  - [ ] Expert Roundtable Backend
  - [ ] Expert Roundtable Frontend
  - [ ] Generic AI Handlers (verify duplicates)

- [ ] **Priority 1.5:** Cleanup & Verification
  - [ ] Delete legacy AIService
  - [ ] Run full test suite
  - [ ] Update documentation

### Current Work (Priority 1.4)

**✅ COMPLETE**: Priority 1.3 - Medium Complexity Migrations

**PRD Generation Backend Migration**
- File: `packages/ideate/src/prd_generator.rs` (991→599 lines, -392 lines)
- ✅ Removed 6 AI functions:
  - `generate_complete_prd_with_model()`
  - `generate_section()`
  - `generate_from_session()`
  - `fill_skipped_sections()`
  - `generate_section_with_context()`
  - `regenerate_with_template()` + streaming version
- ✅ Kept helper functions: `format_prd_markdown()`, `merge_content()`, `build_context_from_aggregated()`

**✅ COMPLETE**: PRD Generation Frontend Migration
- File: `packages/dashboard/src/services/prd-ai.ts` (766 lines)
- ✅ Implemented all 6 AI functions using Vercel AI SDK:
  - `generateCompletePRD()` - Full PRD generation with AI SDK `generateObject()`
  - `generateSection()` - Individual sections with schema validation
  - `generateFromSession()` - From aggregated data with context builder
  - `fillSkippedSections()` - Fill skipped sections with AI
  - `generateSectionWithContext()` - Section with full context
  - `regenerateWithTemplateStream()` - Template regeneration with streaming support
- ✅ Public helper function: `buildContextFromAggregated()` for context building
- ✅ Full telemetry tracking with cost calculation
- ✅ Streaming support for template regeneration
- ⚠️ **Note**: 7+ HTTP handlers in `ideate_generation_handlers.rs` and `ideate_handlers.rs` call removed Rust functions and will need updating to use frontend pattern

**✅ COMPLETE**: Research Analysis Migration
- Backend: `packages/ideate/src/research_analyzer.rs` (558→333 lines, -225 lines)
- ✅ Removed 5 AI functions:
  - `analyze_competitor()` - Competitor analysis
  - `analyze_gaps()` - Gap analysis
  - `extract_ui_patterns()` - UI pattern extraction
  - `extract_lessons()` - Lesson extraction
  - `synthesize_research()` - Research synthesis
- ✅ Kept helper functions: `scrape_url()`, `extract_text_from_html()`, caching, CRUD operations
- ✅ Added public helper: `scrape_competitor_url()` for frontend use

- Frontend: `packages/dashboard/src/services/research-ai.ts` (389 lines)
- ✅ Implemented all 5 AI functions using Vercel AI SDK:
  - `analyzeCompetitor()` - Full competitor analysis with `generateObject()`
  - `analyzeGaps()` - Gap analysis across competitors
  - `extractUIPatterns()` - UI/UX pattern extraction
  - `extractLessons()` - Lessons from similar projects
  - `synthesizeResearch()` - Research synthesis
- ✅ Full telemetry tracking with cost calculation
- ✅ Comprehensive Zod schemas for all result types

**📋 NEXT**: Priority 1.4 - Expert Roundtable migration
- Backend: Remove 3 AI functions from `expert_moderator.rs`
- Frontend: Create `roundtable-ai.ts` with AI SDK implementations

## Detailed Migration Tasks

### ✅ Priority 1.1: Audit Features vs Chat Mode Pattern - COMPLETE

- [x] Document which features already use chat mode pattern (correct)
- [x] Document which features use Rust AI calls (incorrect, needs migration)
- [x] For each incorrect feature, identify:
  - [x] Rust functions making AI calls
  - [x] Frontend components that need AI SDK integration
  - [x] Data flow changes needed
- [x] Create comprehensive audit document (`ARCHITECTURE_AUDIT.md`)

### ✅ Priority 1.2: Simple Migrations - COMPLETE

#### Dependency Analysis ✅
**Backend** (`packages/ideate/src/dependency_analyzer.rs`):
- [x] Remove `analyze_dependencies()` AI function (529→153 lines)
- [x] Keep CRUD: `get_dependencies()`, `create_dependency()`, `delete_dependency()`
- [x] Update handler in `packages/api/src/ideate_dependency_handlers.rs` (252→131 lines)

**Frontend**:
- [x] Create `packages/dashboard/src/services/dependency-ai.ts` (326 lines)
- [x] Implement `analyzeDependencies()` using AI SDK `generateObject()`
- [x] Implement `suggestBuildOrder()` using AI SDK
- [x] Implement `suggestQuickWins()` using AI SDK

**Git Commits**:
- `5ecbb82` - "Priority 1.1 Frontend: Create AI SDK service for dependency analysis"
- Part of dependency analysis migration

#### Insight Extraction ✅
**Backend** (`packages/ideate/src/insight_extractor.rs`):
- [x] Verify `chat-ai.ts:extractInsights()` already handles this
- [x] Remove Rust `extract_insights_with_ai()` function (178→31 lines)
- [x] Keep CRUD: `save_insight()`, `get_insights()`, `link_insight_to_feature()`
- [x] Remove `reanalyze_insights()` handler from `ideate_chat_handlers.rs`

**Frontend**:
- [x] Already implemented in `packages/dashboard/src/services/chat-ai.ts:extractInsights()`
- [x] Uses AI SDK `generateObject()` with proper schema
- [x] Saves results via CRUD API

**Git Commits**:
- `6fac3ff` - "Priority 1.2 Backend: Remove Insight Extraction AI logic from Rust"

### ✅ Priority 1.3: Medium Complexity - COMPLETE

#### PRD Generation ✅
**Backend** (`packages/ideate/src/prd_generator.rs`):
- [x] Complete file analysis (991 lines initially)
- [x] Remove 6 AI functions:
  - [x] `generate_complete_prd_with_model()` - Full PRD generation
  - [x] `generate_section()` - Individual section generation
  - [x] `generate_from_session()` - PRD from aggregated session data
  - [x] `fill_skipped_sections()` - AI-fill for skipped sections
  - [x] `generate_section_with_context()` - Section generation with full context
  - [x] `regenerate_with_template()` + streaming version - Template-based regeneration
- [x] Keep CRUD helpers: `format_prd_markdown()`, `merge_content()`, `build_context_from_aggregated()`
- [x] Remove unused prompt builders: `build_session_prd_prompt()`, `build_section_prompt_with_context()`
- [x] Final size: 599 lines (392 lines removed, ~40% reduction)
- [ ] **TODO**: Update handlers in `packages/api/src/ideate_generation_handlers.rs` and `ideate_handlers.rs`

**Frontend** (`packages/dashboard/src/services/prd-ai.ts`):
- [x] Create `packages/dashboard/src/services/prd-ai.ts` (766 lines)
- [x] Implement AI SDK calls with streaming support
- [x] Match all 6 removed Rust functions with TypeScript equivalents:
  - [x] `generateCompletePRD()` - Full PRD generation with `generateObject()`
  - [x] `generateSection()` - Individual sections with schema validation
  - [x] `generateFromSession()` - From aggregated data
  - [x] `fillSkippedSections()` - Fill skipped sections
  - [x] `generateSectionWithContext()` - Section with context
  - [x] `regenerateWithTemplateStream()` - Template regeneration with streaming
- [x] Public helper: `buildContextFromAggregated()` for context building
- [x] Telemetry tracking with cost calculation for all functions
- [ ] **TODO**: Update HTTP handlers to use frontend pattern

#### Research Analysis ✅
**Backend** (`packages/ideate/src/research_analyzer.rs`):
- [x] Remove 5 AI functions (558→333 lines, -225 lines, ~40% reduction):
  - [x] `analyze_competitor()` - Competitor analysis
  - [x] `analyze_gaps()` - Gap analysis
  - [x] `extract_ui_patterns()` - UI pattern extraction
  - [x] `extract_lessons()` - Lesson extraction
  - [x] `synthesize_research()` - Research synthesis
- [x] Keep CRUD: `save_competitor()`, `get_competitors()`, `add_similar_project()`, `get_similar_projects()`
- [x] Keep scraping helpers: `scrape_url()`, `extract_text_from_html()`, caching functions
- [x] Added public helper: `scrape_competitor_url()` for frontend use
- [ ] **TODO**: Update handler in `packages/api/src/ideate_research_handlers.rs`

**Frontend** (`packages/dashboard/src/services/research-ai.ts`):
- [x] Create `packages/dashboard/src/services/research-ai.ts` (389 lines)
- [x] Implement AI SDK calls using `generateObject()`
- [x] Match all 5 removed Rust functions with TypeScript equivalents:
  - [x] `analyzeCompetitor()` - Competitor analysis with schema validation
  - [x] `analyzeGaps()` - Gap analysis across competitors
  - [x] `extractUIPatterns()` - UI/UX pattern extraction
  - [x] `extractLessons()` - Lessons from similar projects
  - [x] `synthesizeResearch()` - Research synthesis
- [x] Telemetry tracking with cost calculation for all functions

### Priority 1.4: Complex Migrations ✅ COMPLETE

#### Expert Roundtable ✅ COMPLETE
**Backend** (`packages/ideate/src/expert_moderator.rs`):
- [x] Removed 4 AI functions (313 lines removed, ~57% reduction):
  - [x] `run_discussion()` - Multi-turn discussion orchestration
  - [x] `suggest_experts()` - Expert persona suggestions
  - [x] `generate_expert_response()` - Individual expert responses
  - [x] `extract_insights()` - Insight extraction from discussion
- [x] Removed AIService dependency and unused imports
- [x] Converted helper methods to public functions for frontend use:
  - [x] `format_conversation_history()` - Format messages for AI context
  - [x] `build_moderator_opening()` - Generate opening statement
  - [x] `build_expert_suggestion_prompt()` - Build suggestion prompt
  - [x] `build_insight_extraction_prompt()` - Build insight prompt
- [x] Exported system prompts for frontend AI:
  - [x] `EXPERT_SUGGESTION_SYSTEM_PROMPT`
  - [x] `INSIGHT_EXTRACTION_SYSTEM_PROMPT`
  - [x] `EXPERT_RESPONSE_SYSTEM_PROMPT_PREFIX`
- [x] Kept CRUD operations: `handle_interjection()`, `get_messages_for_ai()`, `get_participants_for_ai()`
- [x] File reduced from 553 → 240 lines

**Frontend**:
- [x] Created `packages/dashboard/src/services/roundtable-ai.ts` (413 lines)
- [x] Implemented 3 main AI SDK functions with `generateObject()`:
  - [x] `suggestExperts()` - Expert persona suggestions with schema validation
  - [x] `generateExpertResponse()` - Individual expert responses
  - [x] `extractInsights()` - Insight extraction with categories
- [x] Added streaming support: `streamExpertResponse()` using `streamText()`
- [x] Helper functions for frontend discussion orchestration:
  - [x] `formatConversationHistory()` - Format messages for AI
  - [x] `buildModeratorOpening()` - Generate opening
  - [x] `selectNextExpert()` - Round-robin expert selection
  - [x] `shouldEndDiscussion()` - Discussion termination logic
- [x] Comprehensive Zod schemas: ExpertSuggestionSchema, ExpertResponseSchema, InsightSchema
- [x] Full telemetry tracking for all AI operations

#### Generic AI Handlers ✅ COMPLETE
**Investigation** (`packages/api/src/ai_handlers.rs`):
- [x] Investigated file - contained 5 AI handlers (1,248 lines):
  - `analyze_prd()` - Analyze PRD and extract capabilities
  - `generate_spec()` - Generate specification from requirements
  - `suggest_tasks()` - Suggest tasks from spec
  - `refine_spec()` - Refine spec with user feedback
  - `validate_completion()` - Validate task completion
- [x] Verified these were **dead code**:
  - Routes already commented out in lib.rs
  - No frontend references found
  - Superseded by prd-ai.ts frontend AI SDK implementation
  - Used legacy AIService pattern (incorrect)
- [x] Deleted entire file (1,248 lines removed)
- [x] Removed module declaration from lib.rs
- [x] Removed commented route creation function

**Note**: Separate compilation issues exist in `ideate_generation_handlers.rs` from earlier PRD migration. These are handler updates (separate task) not related to Generic AI Handlers deletion.

### Priority 1.5: Handler Updates & Cleanup ✅ COMPLETE

**HTTP Handler Routes** ✅ COMPLETE (commit b9f1d90):
- [x] Removed 16 AI handler routes (all routes calling removed AI methods)
- [x] PRD generation handlers: 5 routes removed
- [x] Quick Mode handlers: 2 routes removed
- [x] Research handlers: 5 routes removed
- [x] Roundtable handlers: 3 routes removed
- [x] Export handler: 1 route removed
- [x] Removed `get_participants` route (handler never existed)

**Handler Function Cleanup** ✅ COMPLETE (commits ea5c025, 977ebc1):
- [x] `analyze_competitor()` - DELETED (commit ea5c025, 69 lines)
- [x] `analyze_gaps()` - DELETED (commit 977ebc1, 66 lines)
- [x] `extract_patterns()` - DELETED (commit 977ebc1, 68 lines)
- [x] `extract_lessons()` - DELETED (commit 977ebc1, 97 lines)
- [x] `synthesize_research()` - DELETED (commit 977ebc1, 59 lines)
- [x] `suggest_experts()` - Already deleted in earlier session
- [x] `start_discussion()` - Already deleted in earlier session
- [x] `extract_insights()` - DELETED (commit 977ebc1, 17 lines)

**Result**: All dead handler code removed. Build succeeds with 0 errors. Total: ~375 lines of dead code deleted.

**Legacy AIService Cleanup**:
- [ ] Delete `packages/ai/src/service.rs` (entire legacy AIService - 487 lines)
- [ ] Remove AIService exports from `packages/ai/src/lib.rs`
- [ ] Remove `async-trait` from `packages/ai/Cargo.toml` (if not needed elsewhere)
- [ ] Remove direct `reqwest` from `packages/ai/Cargo.toml` (if not needed elsewhere)
- [ ] Keep only: AI proxy endpoints + usage logging + telemetry

**Verification**:
- [ ] Run full test suite
- [ ] Verify all features work end-to-end
- [ ] Update documentation

## Example Migration Pattern

### Before (Legacy AIService - INCORRECT)
```rust
// ❌ INCORRECT - Rust making AI calls
use orkee_ai::{AIService, AIServiceError};

pub async fn analyze_dependencies(...) -> Result<...> {
    let ai_service = AIService::with_api_key_and_model(api_key, model);
    let result = ai_service.generate_structured(...).await?;
    Ok(result)
}
```

### After (Chat Mode Pattern - CORRECT)
```typescript
// ✅ CORRECT - Frontend makes AI calls via AI SDK
import { generateObject } from 'ai';
import { trackAIOperationWithCost } from '@/lib/ai/cost-tracking';

export async function analyzeDependencies(
  sessionId: string,
  features: IdeateFeature[],
  modelPreferences?: ReturnType<typeof getModelForTask>,
  projectId?: string | null
): Promise<DependencyAnalysis> {
  const modelConfig = modelPreferences || getModelForTask('ideate');
  const model = getModelInstance(modelConfig.provider, modelConfig.model);

  const result = await trackAIOperationWithCost(
    'analyze_dependencies',
    projectId || null,
    modelConfig.model,
    modelConfig.provider,
    (inputTokens, outputTokens) => calculateCost(...),
    () => generateObject({
      model,
      schema: DependencyAnalysisSchema,
      prompt: buildPrompt(features),
      temperature: 0.3,
    })
  );

  // Save results to backend via CRUD API
  for (const dep of result.object.detected_dependencies) {
    await ideateService.createFeatureDependency(sessionId, dep);
  }

  return result.object;
}
```

**Key Principle**: Backend Rust = Pure CRUD only. Frontend TypeScript = All AI calls via AI SDK.

## PR Strategy

### Single Comprehensive PR
**PR Title**: "Migrate all AI operations from Rust to Frontend AI SDK"

**Scope**: ALL migration work in one PR
- Complete architectural change story
- ~2,500 lines removed (Rust AI logic)
- ~1,500 lines added (TypeScript AI SDK services)
- ~15-20 files modified
- 3-4 new TypeScript service files created

**Benefits**:
1. Complete story: "We migrated ALL AI operations to the correct architecture"
2. No partial states: Everything works correctly or nothing is merged
3. Comprehensive testing: Can test entire migration as a unit
4. Clear before/after: Easy to understand the full architectural change
5. Easier rollback: If issues arise, rollback everything at once

**Review Approach**:
- Each priority is a separate commit for easier review
- Can review commit-by-commit instead of all at once
- Clear commit messages with "Review focus" sections

## Testing Strategy

### Per-Feature Testing
- [ ] Dependency Analysis: Test analysis, build order, quick wins
- [ ] Insight Extraction: Test extraction from chat messages
- [ ] PRD Generation: Test all 6 functions, including streaming
- [ ] Research Analysis: Test all 5 analysis functions
- [ ] Expert Roundtable: Test multi-turn discussions, interjections

### Integration Testing
- [ ] All existing tests pass
- [ ] Manual testing of each migrated feature
- [ ] No references to `AIService` remain in codebase
- [ ] Verify no functionality loss
- [ ] Verify CRUD operations still work

### Verification Commands
```bash
# Search for remaining AIService references
rg "AIService" --type rust

# Search for generate_structured calls (legacy pattern)
rg "generate_structured" --type rust

# Verify all tests pass
cargo test --workspace
turbo test

# Verify builds succeed
cargo build --workspace
turbo build
```

## Success Criteria

- [ ] Zero references to legacy AIService in codebase
- [ ] All AI operations use Frontend AI SDK pattern
- [ ] All existing tests pass
- [ ] Manual testing confirms no functionality loss
- [ ] CRUD operations preserved and working
- [ ] Documentation updated
- [ ] Code review approved
- [ ] PR merged to main

## Git Commits

### Completed Commits
1. `5ecbb82` - "Priority 1.1 Frontend: Create AI SDK service for dependency analysis"
2. `6fac3ff` - "Priority 1.2 Backend: Remove Insight Extraction AI logic from Rust"
3. `6ce1c00` - "Priority 1.3 Backend: Remove PRD Generation AI logic from Rust (991→599 lines)"
4. `a5fff8e` - "Priority 1.3 Frontend: Create AI SDK service for PRD generation (766 lines)"
5. `b79bf99` - "Priority 1.3 Complete: Research Analysis migration (backend 558→333, frontend +389)"
6. `d4cdbe2` - "Priority 1.4 Complete: Expert Roundtable migration (backend 553→240, frontend +413)"
7. `58e2d36` - "Priority 1.5 Complete: Generic AI Handlers cleanup (deleted 1,248 lines of dead code)"

### Planned Commits
8. Priority 1.5: Handler Updates - Fix ideate_generation_handlers.rs compilation errors
9. Priority 1.5: Legacy AIService Cleanup - Delete service.rs and verify migrations

## Notes

- This is a pure architectural cleanup - no new features
- No functionality should be lost in the migration
- All CRUD operations must be preserved
- This work is independent of VibeKit OAuth integration (Phases 2-8)
- Can be completed and merged before OAuth work begins
