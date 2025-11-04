# AI Architecture Rework: Migrate to Frontend AI SDK Pattern

## Project Overview

This document tracks the migration of all AI operations from legacy Rust AIService to the modern Frontend AI SDK pattern (Chat Mode). This is a pure architectural cleanup that improves code quality and maintainability.

### Goal

**Apply the Chat Mode pattern to ALL features**: Backend Rust = Pure CRUD only. Frontend TypeScript = All AI calls via AI SDK.

### Status: In Progress 🚧
**Completion:** 60% complete

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
- ❌ Research Analysis (5 functions)
- ❌ Expert Roundtable (3 functions)
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
  - [ ] Research Analysis Backend
  - [ ] Research Analysis Frontend

- [ ] **Priority 1.4:** Complex Migrations
  - [ ] Expert Roundtable Backend
  - [ ] Expert Roundtable Frontend
  - [ ] Generic AI Handlers (verify duplicates)

- [ ] **Priority 1.5:** Cleanup & Verification
  - [ ] Delete legacy AIService
  - [ ] Run full test suite
  - [ ] Update documentation

### Current Work (Priority 1.3) - ✅ COMPLETE

**✅ COMPLETE**: PRD Generation Backend Migration
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

**📋 NEXT**: Priority 1.4 - Research Analysis migration
- Backend: Remove 5 AI functions from `research_analyzer.rs`
- Frontend: Create `research-ai.ts` with AI SDK implementations

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

#### Research Analysis
**Backend** (`packages/ideate/src/research_analyzer.rs`):
- [ ] Remove 5 AI functions:
  - [ ] `analyze_competitor()` - Competitor analysis
  - [ ] `analyze_gaps()` - Gap analysis
  - [ ] `extract_ui_patterns()` - UI pattern extraction
  - [ ] `extract_lessons()` - Lesson extraction
  - [ ] `synthesize_research()` - Research synthesis
- [ ] Keep CRUD: `save_competitor()`, `get_competitors()`, `add_similar_project()`, `get_similar_projects()`
- [ ] Update handler in `packages/api/src/ideate_research_handlers.rs`

**Frontend**:
- [ ] Create `packages/dashboard/src/services/research-ai.ts` (~300-400 lines)
- [ ] Implement AI SDK calls using `generateObject()`
- [ ] Match all 5 removed Rust functions

### Priority 1.4: Complex Migrations

#### Expert Roundtable
**Backend** (`packages/ideate/src/expert_moderator.rs`):
- [ ] Remove 3 AI functions:
  - [ ] `run_discussion()` - Multi-turn discussion
  - [ ] `handle_interjection()` - User interjection handling
  - [ ] `extract_insights()` - Insight extraction
- [ ] Keep CRUD: `create_roundtable()`, `add_participants()`, `save_message()`, `get_messages()`, `save_insight()`, `get_insights()`
- [ ] Update handler in `packages/api/src/ideate_roundtable_handlers.rs`
- [ ] Redesign SSE streaming endpoint for frontend-driven discussion

**Frontend**:
- [ ] Create `packages/dashboard/src/services/roundtable-ai.ts` (~400-500 lines)
- [ ] Implement multi-turn discussion loop in frontend
- [ ] Handle streaming responses from multiple "experts"
- [ ] Support user interjections

#### Generic AI Handlers
**Investigation** (`packages/api/src/ai_handlers.rs`):
- [ ] Verify these duplicate Spec Workflow functionality
- [ ] If yes: Delete entire file and route callers to Spec Workflow
- [ ] If no: Identify unique functionality and migrate to frontend

### Priority 1.5: Cleanup & Verification

- [ ] Delete `packages/ai/src/service.rs` (entire legacy AIService - 487 lines)
- [ ] Remove AIService exports from `packages/ai/src/lib.rs`
- [ ] Remove `async-trait` from `packages/ai/Cargo.toml` (if not needed elsewhere)
- [ ] Remove direct `reqwest` from `packages/ai/Cargo.toml` (if not needed elsewhere)
- [ ] Revert incorrect commits (selective revert of Rust→proxy HTTP calls)
- [ ] Keep only: AI proxy endpoints + usage logging + telemetry
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
4. **PENDING** - "Priority 1.3 Frontend: Create AI SDK service for PRD generation (766 lines)"

### Planned Commits
5. Priority 1.3 Handlers: Update HTTP handlers for frontend AI pattern
6. Priority 1.4 Backend: Remove Research Analysis AI logic from Rust
7. Priority 1.4 Frontend: Create AI SDK service for research analysis
8. Priority 1.5 Backend: Remove Expert Roundtable AI logic from Rust
9. Priority 1.5 Frontend: Create AI SDK service for Expert Roundtable
10. Priority 1.6: Handle Generic AI Handlers (verify/delete/migrate)
11. Cleanup: Delete legacy AIService and verify all migrations

## Notes

- This is a pure architectural cleanup - no new features
- No functionality should be lost in the migration
- All CRUD operations must be preserved
- This work is independent of VibeKit OAuth integration (Phases 2-8)
- Can be completed and merged before OAuth work begins
