# PRD Ideation Feature - Implementation Plan

## Overview
Add a flexible PRD generation system supporting three modes:
- **Quick Mode**: One-liner → Complete PRD
- **Guided Mode**: Step-by-step with optional sections
- **Comprehensive Mode**: Full ideation with expert roundtables, competitor analysis

## Target PRD Structure
1. Overview (Problem, Target, Value)
2. Core Features (What, Why, How)
3. User Experience (Personas, Flows, UI/UX)
4. Technical Architecture (Components, Data, APIs)
5. Development Roadmap (MVP, Future phases - scope only, NO timelines)
6. Logical Dependency Chain (Foundation → Visible → Enhancement)
7. Risks and Mitigations (Technical, MVP, Resources)
8. Appendix (Research, Specs)

---

## Phase 1: Core Infrastructure & Database (Week 1) ✅ COMPLETED

### Database Schema ✅
- [x] Create migration file `002_ideate_schema.sql`
- [x] `ideate_sessions` table with mode tracking
- [x] `ideate_overview` table (optional)
- [x] `ideate_features` table with dependency fields
- [x] `ideate_ux` table (optional)
- [x] `ideate_technical` table (optional)
- [x] `ideate_roadmap` table (optional, NO timeline fields)
- [x] `ideate_dependencies` table (foundation/visible/enhancement)
- [x] `ideate_risks` table (optional)
- [x] `ideate_research` table (optional)
- [x] `roundtable_sessions` table (for comprehensive mode)
- [x] `prd_quickstart_templates` table
- [x] Add indexes for performance
- [x] Test migration up and down

### Backend - Session Management ✅
- [x] Create `packages/ideate` package with types, error handling, and manager
- [x] Create `packages/api/src/ideate_handlers.rs`
- [x] POST `/api/ideate/start` - Create session with mode
- [x] GET `/api/ideate/{id}/status` - Get completion status
- [x] POST `/api/ideate/{id}/skip-section` - Mark section as skipped
- [x] GET `/api/ideate/{id}` - Get full session data
- [x] PUT `/api/ideate/{id}` - Update session
- [x] DELETE `/api/ideate/{id}` - Delete session
- [x] GET `/api/{project_id}/ideate/sessions` - List sessions by project
- [x] Register routes in `packages/api/src/lib.rs`
- [x] Mount router in `packages/cli/src/api/mod.rs`
- [x] Fix enum serialization (snake_case for IdeateStatus)
- [x] Test all endpoints end-to-end

### Frontend - Entry Point ✅ COMPLETED
- [x] Create `packages/dashboard/src/components/ideate/` directory
- [x] Create `CreatePRDFlow.tsx` - Main entry component
- [x] Create `ModeSelector.tsx` - Choose Quick/Guided/Comprehensive
- [x] Add "Create PRD" button to `PRDView.tsx`
- [x] Create service `packages/dashboard/src/services/ideate.ts`
- [x] Create hooks `packages/dashboard/src/hooks/useIdeate.ts`
- [x] Test end-to-end session creation

---

## Phase 2: Quick Mode (One-liner → PRD) (Week 2) ✅ COMPLETED

### Backend - Quick Mode ✅ COMPLETED
- [x] POST `/api/ideate/{id}/quick-generate` - Generate PRD from one-liner
- [x] POST `/api/ideate/{id}/quick-expand` - Expand specific sections
- [x] GET `/api/ideate/{id}/preview` - Preview PRD before saving
- [x] POST `/api/ideate/{id}/save-as-prd` - Save PRD to OpenSpec system
- [x] Integrate AI service for PRD generation (Claude via stored API key)
- [x] Create `prd_generator.rs` service with database settings integration
- [x] Create `prompts.rs` with structured prompts for each PRD section
- [x] Implement database-based configuration (ideate.* settings)
- [x] Add error handling with proper error types

### Backend - Settings Integration ✅
- [x] Add AI configuration to system_settings table:
  - `ideate.max_tokens` (default: 8000)
  - `ideate.temperature` (default: 0.7)
  - `ideate.model` (default: claude-3-opus-20240229)
  - `ideate.timeout_seconds` (default: 120)
  - `ideate.retry_attempts` (default: 3)
- [x] Use encrypted user API keys from database
- [x] Settings accessible via Settings → Advanced UI

### Frontend - Step 1: API Service Layer Extension ✅
**File**: `packages/dashboard/src/services/ideate.ts`
- [x] Add `quickGenerate(sessionId, sections?)` method → POST `/api/ideate/{id}/quick-generate`
- [x] Add `quickExpand(sessionId, sections)` method → POST `/api/ideate/{id}/quick-expand`
- [x] Add `previewPRD(sessionId)` method → GET `/api/ideate/{id}/preview`
- [x] Add `saveAsPRD(sessionId)` method → POST `/api/ideate/{id}/save-as-prd`

**File**: `packages/dashboard/src/hooks/useIdeate.ts`
- [x] Add `useQuickGenerate(projectId, sessionId)` React Query mutation
- [x] Add `useQuickExpand(projectId, sessionId)` React Query mutation
- [x] Add `usePreviewPRD(sessionId)` React Query query
- [x] Add `useSaveAsPRD(projectId, sessionId)` React Query mutation

### Frontend - Step 2: Session List View
**File**: `packages/dashboard/src/components/ideate/SessionsList.tsx`
- [x] Create component with Card grid layout (Shadcn Card)
- [x] Fetch sessions using `useIdeateSessions(projectId)` hook
- [x] Display mode badges (Quick/Guided/Comprehensive) with Shadcn Badge
- [x] Display status badges with color coding (Draft/In Progress/Ready/Completed)
- [x] Add search input (Shadcn Input) with client-side filtering
- [x] Add mode filter dropdown (Shadcn Select)
- [x] Add status filter dropdown (Shadcn Select)
- [x] Format timestamps using `date-fns` `formatDistanceToNow()` with tooltips
- [x] Add "Resume session" button → opens appropriate mode dialog
- [x] Add "Delete session" button → `useDeleteIdeateSession()` with confirmation (Shadcn AlertDialog)
- [x] Show linked PRD badge if status is 'completed'
- [x] Integrate into PRDView.tsx or SpecsTab.tsx

### Frontend - Step 3: Quick Mode UI Components
**Directory**: `packages/dashboard/src/components/ideate/QuickMode/`

**File**: `QuickMode/index.ts`
- [x] Create barrel export file

**File**: `QuickMode/QuickModeFlow.tsx` (Main Orchestrator)
- [x] Create component orchestrating 4 steps: Input → Generating → Review/Edit → Save
- [x] Manage state: sessionId, generatedPRD, selectedSections, currentStep
- [x] Use Dialog component (full-screen or large)
- [x] Implement step navigation

**File**: `QuickMode/OneLineInput.tsx`
- [x] Create Textarea (Shadcn) with mode-specific placeholder
- [x] Add character counter (show if > 500 chars)
- [x] Add validation (required, min 10 chars)
- [x] Add clear button
- [x] Add "Generate PRD" button → triggers `useQuickGenerate()`
- [x] Add loading state (disable during generation)

**File**: `QuickMode/SectionSelector.tsx` (Optional Expansion)
- [x] Create checkbox list (Shadcn Checkbox) for 8 PRD sections:
  - [x] Overview, Core Features, User Experience, Technical Architecture
  - [x] Development Roadmap, Logical Dependency Chain, Risks and Mitigations, Appendix
- [x] Add "Select All" / "Deselect All" buttons
- [x] Trigger `useQuickExpand()` mutation on confirmation

**File**: `QuickMode/GeneratingState.tsx`
- [x] Create skeleton loaders (Shadcn Skeleton) for each section
- [x] Add progress indicator
- [x] Display "Generating PRD..." message
- [x] Add cancel button (if backend supports)

**File**: `QuickMode/PRDEditor.tsx`
- [x] Display generated PRD using react-markdown with remarkGfm, rehypeHighlight, rehypeSanitize
- [x] Implement collapsible sections (Shadcn Collapsible) for each PRD section
- [x] Add edit mode toggle per section (View: rendered markdown, Edit: Textarea)
- [x] Add "Regenerate section" button → calls `useQuickExpand()` with single section
- [x] Add "Save as PRD" button → triggers save flow
- [x] Add back to editing functionality

**File**: `QuickMode/SavePreview.tsx`
- [x] Create modal/drawer (Shadcn Dialog) showing final PRD
- [x] Display read-only markdown view
- [x] Add project name field (editable)
- [x] Add "Confirm Save" button → calls `useSaveAsPRD()`
- [x] Add "Cancel" button → back to editor
- [x] On success: show toast (sonner) and navigate to PRD view

### Frontend - Step 4: Integration & Error Handling
**Toast Notifications** (using sonner)
- [x] Add success toast: "PRD generated successfully!"
- [x] Add error toast with description for each error type:
  - [x] AI Service Unavailable
  - [x] Invalid API Key
  - [x] Token Limit Exceeded
  - [x] Network Error
- [x] Add info toast: "Generating PRD..."
- [x] Add retry button in error toasts

**PRD Save Flow Integration**
- [x] Wire up `SavePreview.tsx` to `useSaveAsPRD()` mutation
- [x] Update session status to "completed" on success
- [x] Close Quick Mode dialog on success
- [x] Navigate to PRD view or show PRD in list
- [x] Handle save errors with retry option

**Error Handling**
- [x] Wrap all API calls in try-catch blocks
- [x] Display user-friendly error messages (not raw API errors)
- [x] Log errors to console for debugging
- [x] Preserve UI state on error (allow retry)

### Frontend - Step 5: Update Existing Components
**File**: `packages/dashboard/src/components/specs/PRDView.tsx`
- [x] Add SessionsList component as tab or section
- [x] Wire up `onSessionCreated` to open QuickModeFlow dialog

**File**: `packages/dashboard/src/components/ideate/CreatePRDFlow.tsx`
- [x] Update Quick Mode branch to call `onSessionCreated(session.id)`
- [x] Parent component opens QuickModeFlow dialog

### Frontend - Step 6: Testing
**Test Cases**
- [x] Very short description (< 10 chars) - validation error
- [x] Normal description (50-200 chars) - success
- [x] Long description (> 500 chars) - success with warning
- [x] Special characters in description - sanitized
- [x] Network failure during generation - retry
- [x] Cancel during generation - cleanup
- [x] Edit section after generation - saves changes
- [x] Regenerate single section - updates only that section
- [x] Save with empty PRD name - validation error
- [x] Save success - PRD appears in list
- [x] Complete flow: Input → Generate → Edit → Save
- [x] Resume session from SessionsList
- [x] Delete session with confirmation

### Technical Decisions (Implemented)
- ✅ Markdown: react-markdown (already installed with remarkGfm, rehypeHighlight, rehypeSanitize)
- ✅ Toast: sonner (already installed and configured in App.tsx)
- ✅ Forms: Simple controlled components (useState pattern, no form library)
- ✅ UI Components: Shadcn UI (Dialog, Card, Button, Input, Textarea, Badge, Skeleton, Checkbox, Select, etc.)
- ✅ Date Formatting: date-fns for timestamp display
- ✅ Navigation: Dialog-based (following existing CreatePRDFlow pattern)

### Future Enhancements
- [ ] Add SSE streaming support for real-time generation
- [ ] Implement token limit handling with chunking
- [ ] Add retry logic with exponential backoff

---

## Phase 3: Guided Mode - Core Sections (Week 3) ✅ COMPLETED

### Backend - Section Endpoints ✅
- [x] Database schema with section tables and current_section tracking
- [x] POST/GET/DELETE `/api/ideate/{id}/overview`
- [x] POST/GET/DELETE `/api/ideate/{id}/ux`
- [x] POST/GET/DELETE `/api/ideate/{id}/technical`
- [x] POST/GET/DELETE `/api/ideate/{id}/roadmap`
- [x] POST/GET/DELETE `/api/ideate/{id}/dependencies`
- [x] POST/GET/DELETE `/api/ideate/{id}/risks`
- [x] POST/GET/DELETE `/api/ideate/{id}/research`
- [x] GET `/api/ideate/{id}/next-section` - Navigation helper
- [x] POST `/api/ideate/{id}/navigate` - Navigate to specific section
- [x] Service layer methods with CRUD operations
- [ ] Add AI suggestion endpoints for each section (deferred)
- [ ] Implement skip with AI fill functionality (deferred)

### Frontend - Service Layer ✅
- [x] Add section CRUD methods to `ideate.ts` service
- [x] Add React Query hooks for all sections in `useIdeate.ts`
- [x] Add navigation hooks (useGetNextSection, useNavigateToSection)
- [x] Add saveAsPRD integration for guided mode

### Frontend - Guided Mode Structure ✅
- [x] Create `packages/dashboard/src/components/ideate/GuidedMode/`
- [x] Create `GuidedModeFlow.tsx` - Main orchestrator component (~240 lines)
- [x] Create `SectionNavigator.tsx` - Sidebar navigation with completion indicators
- [x] Create `SkipDialog.tsx` - Confirm skip with AI fill option
- [x] Create `SectionProgress.tsx` - Visual progress indicator with percentage

### Frontend - Individual Section Forms ✅
- [x] Create `sections/OverviewSection.tsx` - 4 fields (problem, audience, value, pitch)
- [x] Create `sections/UXSection.tsx` - UI considerations and UX principles (simplified)
- [x] Create `sections/TechnicalSection.tsx` - Tech stack input (simplified)
- [x] Create `sections/RoadmapSection.tsx` - MVP scope as newline-separated list
- [x] Create `sections/DependencyChainSection.tsx` - Foundation/Visible/Enhancement features
- [x] Create `sections/RisksSection.tsx` - Technical/Scoping/Resource risks
- [x] Create `sections/AppendixSection.tsx` - Research findings and technical specs
- [x] All sections use React Hook Form with validation
- [x] All sections integrate with React Query hooks
- [x] All sections show loading states and error handling

### Frontend - Integration ✅
- [x] Update `CreatePRDFlow.tsx` to pass mode to parent
- [x] Update `PRDView.tsx` to handle guided mode sessions
- [x] Update `SessionsList.tsx` to show current section for guided sessions
- [x] Add "Resume" button functionality for guided sessions
- [x] Install react-hook-form dependency
- [x] Create GuidedMode index.ts barrel export
- [x] Fix TypeScript compilation errors

### Implementation Notes
- **Total Code**: ~2,550 lines across 23 files (11 new components + modified files)
- **Approach**: Simplified forms with basic textareas/inputs for MVP speed
- **Pattern**: Follows existing QuickMode integration pattern exactly
- **State**: Backend tracks current_section, frontend uses React Query for data sync
- **Navigation**: Bi-directional (Previous/Next) + direct section selection via sidebar
- **Progress**: Visual indicators show completed/current/skipped sections

### Deferred to Future Enhancement
- [ ] Advanced persona builder with drag-and-drop
- [ ] Interactive user flow mapper
- [ ] Component architecture diagram builder
- [ ] Data model visual editor
- [x] Visual dependency graph (Phase 4) ✅
- [ ] AI auto-fill for skipped sections
- [ ] Live PRD preview panel

---

## Phase 4: Dependency Chain Focus (Week 4) ✅ COMPLETED

### Database Schema ✅
- [x] Create migration file `003_dependency_intelligence.sql` (175 lines)
- [x] `feature_dependencies` table with type (technical/logical/business) and strength (required/recommended/optional)
- [x] `dependency_analysis_cache` table for AI analysis caching with hash-based invalidation
- [x] `build_order_optimization` table for storing optimized build sequences
- [x] `quick_win_features` table for tracking low-dependency high-value features
- [x] `circular_dependencies` table for cycle detection
- [x] Add indexes for performance
- [x] Test migration up and down

### Backend - Dependency Analyzer ✅
**File**: `packages/ideate/src/dependency_analyzer.rs` (522 lines)
- [x] AI-powered dependency detection using Claude
- [x] Automatic classification (technical/logical/business)
- [x] Dependency strength levels (required/recommended/optional)
- [x] Analysis caching with hash-based invalidation
- [x] CRUD operations for manual dependencies
- [x] Integration with encrypted user API keys

### Backend - Build Optimizer ✅
**File**: `packages/ideate/src/build_optimizer.rs` (600+ lines)
- [x] Graph algorithms using petgraph library
- [x] Topological sort for build order
- [x] Circular dependency detection using DFS
- [x] Critical path analysis
- [x] Parallel work identification
- [x] Three optimization strategies: fastest/balanced/safest
- [x] Estimated time per phase calculation

### Backend - API Endpoints ✅
**File**: `packages/api/src/ideate_dependency_handlers.rs` (253 lines)
- [x] GET `/api/ideate/{id}/features/dependencies` - Get all dependencies
- [x] POST `/api/ideate/{id}/features/dependencies` - Create manual dependency
- [x] DELETE `/api/ideate/{id}/features/dependencies/{dep_id}` - Delete dependency
- [x] POST `/api/ideate/{id}/dependencies/analyze` - AI-powered dependency analysis
- [x] POST `/api/ideate/{id}/dependencies/optimize` - Optimize build order with strategy selection
- [x] GET `/api/ideate/{id}/dependencies/build-order` - Get current build order
- [x] GET `/api/ideate/{id}/dependencies/circular` - Detect circular dependencies
- [x] GET `/api/ideate/{id}/features/suggest-visible` - Suggest quick-win features
- [x] Wire endpoints into API router (`packages/api/src/lib.rs`)

### Frontend - Service Layer ✅
**File**: `packages/dashboard/src/services/ideate.ts`
- [x] TypeScript interfaces for all Phase 4 types (FeatureDependency, BuildOrderResult, etc.)
- [x] 8 service methods matching API endpoints with proper error handling
- [x] Type-safe request/response handling

**File**: `packages/dashboard/src/hooks/useIdeate.ts`
- [x] `useFeatureDependencies(sessionId)` - Fetch dependencies with 1min cache
- [x] `useCreateFeatureDependency(sessionId)` - Create with cache invalidation
- [x] `useDeleteFeatureDependency(sessionId)` - Delete with cache invalidation
- [x] `useAnalyzeDependencies(sessionId)` - AI analysis mutation
- [x] `useOptimizeBuildOrder(sessionId)` - Optimization mutation with strategy
- [x] `useBuildOrder(sessionId)` - Fetch build order with 2min cache
- [x] `useCircularDependencies(sessionId)` - Fetch circular deps with 1min cache
- [x] `useQuickWins(sessionId)` - Fetch quick-win suggestions with 2min cache

**File**: `packages/dashboard/src/lib/queryClient.ts`
- [x] Query key factories for proper cache invalidation
- [x] `ideateFeatureDependencies`, `ideateBuildOrder`, `ideateCircularDeps`, `ideateQuickWins`

### Frontend - UI Components ✅
**File**: `packages/dashboard/src/components/ideate/GuidedMode/sections/DependencyMapper.tsx` (199 lines)
- [x] Interactive React Flow graph visualization
- [x] Auto-layout nodes based on feature list
- [x] Color-coded edges (blue=required, purple=recommended, gray=optional, red=circular)
- [x] Animated edges for required dependencies
- [x] Click-to-connect nodes for manual dependency creation
- [x] Click edge to delete dependency
- [x] AI analysis button with loading state
- [x] Circular dependency highlighting
- [x] Legend explaining edge colors

**File**: `packages/dashboard/src/components/ideate/GuidedMode/sections/BuildOrderVisualizer.tsx` (159 lines)
- [x] Timeline view with numbered phases
- [x] Strategy selector (⚡ Fastest / 📊 Balanced / 🛡️ Safest)
- [x] Parallel work groups visualization
- [x] Critical path highlighting with star icons
- [x] Estimated time per phase display
- [x] Re-optimization with different strategies
- [x] Optimization notes display
- [x] Empty state with strategy explanation

**File**: `packages/dashboard/src/components/ideate/GuidedMode/sections/FeaturePicker.tsx` (238 lines)
- [x] Smart feature selection with dual lists (Foundation vs Visible)
- [x] AI-suggested quick-win recommendations card with one-click apply
- [x] Circular dependency warnings with severity and suggestions
- [x] Color-coded badges (quick win = green, circular = red)
- [x] Checkbox interface with hover states
- [x] Unassigned features tracking
- [x] Apply button to save selections

**File**: `packages/dashboard/src/components/ideate/GuidedMode/sections/DependencyChainSection.tsx` (Enhanced - 113 lines)
- [x] Three-tab interface (Feature Selection / Dependency Graph / Build Timeline)
- [x] Integration of all Phase 4 components
- [x] State management for foundation/visible/enhancement features
- [x] Save button with loading state
- [x] Empty state handling

### Integration ✅
- [x] All components integrated into Guided Mode flow
- [x] Real-time dependency graph updates via React Query
- [x] Cache invalidation strategy ensures consistency
- [x] AI analysis with user API key integration
- [x] Circular dependency warnings surface automatically
- [x] Quick-win suggestions update based on dependency changes

---

## Phase 5: Comprehensive Mode - Research Tools (Week 5) ✅ COMPLETED

### Backend - Research & Analysis ✅
**File**: `packages/ideate/src/research_analyzer.rs` (526 lines)
- [x] Web scraping with `reqwest` + `scraper` crates
- [x] Competitor analysis with AI-powered feature extraction
- [x] Gap analysis comparing features across competitors
- [x] UI/UX pattern extraction from websites
- [x] Similar project tracking with lessons extraction
- [x] Research synthesis aggregating all findings
- [x] 24-hour caching to reduce redundant scraping

**File**: `packages/ideate/src/research_prompts.rs` (220 lines)
- [x] 7 specialized AI prompts for research tasks
- [x] System prompt for research expertise

**File**: `packages/storage/migrations/005_research_analysis_cache.sql`
- [x] Cache table with 24-hour TTL
- [x] Indexes for performance

**File**: `packages/api/src/ideate_research_handlers.rs` (440 lines)
- [x] POST `/api/ideate/{id}/research/competitors/analyze` - Analyze competitor URL
- [x] GET `/api/ideate/{id}/research/competitors` - List analyzed competitors
- [x] POST `/api/ideate/{id}/research/gaps/analyze` - Compare your features vs competitors
- [x] POST `/api/ideate/{id}/research/patterns/extract` - Extract UI patterns from URL
- [x] POST `/api/ideate/{id}/research/similar-projects` - Add similar project
- [x] GET `/api/ideate/{id}/research/similar-projects` - List similar projects
- [x] POST `/api/ideate/{id}/research/lessons/extract` - Extract lessons from similar project
- [x] POST `/api/ideate/{id}/research/synthesize` - Synthesize all research findings

### Frontend - Service Layer ✅
**File**: `packages/dashboard/src/services/ideate.ts`
- [x] TypeScript interfaces (UIPattern, Opportunity, GapAnalysis, Lesson, ResearchSynthesis)
- [x] 8 service methods matching API endpoints

**File**: `packages/dashboard/src/hooks/useIdeate.ts`
- [x] `useAnalyzeCompetitor`, `useCompetitors` - Competitor analysis hooks
- [x] `useAnalyzeGaps`, `useExtractPatterns` - Analysis mutation hooks
- [x] `useAddSimilarProject`, `useSimilarProjects` - Similar project hooks
- [x] `useExtractLessons`, `useSynthesizeResearch` - Research synthesis hooks

**File**: `packages/dashboard/src/lib/queryClient.ts`
- [x] Query keys for cache management (ideateCompetitors, ideateSimilarProjects)

### Frontend - Research UI ✅
**File**: `packages/dashboard/src/components/ideate/ComprehensiveMode/ComprehensiveModeFlow.tsx` (140 lines)
- [x] Main orchestrator with tabbed research interface
- [x] Integrates with existing GuidedMode sections
- [x] Research section with Competitor Analysis and Similar Projects tabs
- [x] Full navigation, progress tracking, save as PRD

**File**: `packages/dashboard/src/components/ideate/ComprehensiveMode/sections/CompetitorAnalysisSection.tsx` (460 lines)
- [x] Competitor Scanner - URL input, scraping, AI analysis
- [x] Feature Comparison - Display strengths, gaps, features
- [x] Gap Analysis - Compare your features vs competitors
- [x] UI Pattern Extraction - Extract and categorize UI patterns

**File**: `packages/dashboard/src/components/ideate/ComprehensiveMode/sections/SimilarProjectsSection.tsx` (440 lines)
- [x] Project Manager - Add/track similar projects manually
- [x] Lesson Extraction - AI-powered insights from projects
- [x] Research Synthesis - Aggregate all research findings
- [x] Priority-based lesson categorization

**File**: `packages/dashboard/src/components/ideate/ComprehensiveMode/index.ts`
- [x] Barrel export for all ComprehensiveMode components

### Integration ✅
**File**: `packages/dashboard/src/components/specs/PRDView.tsx`
- [x] Import ComprehensiveModeFlow
- [x] Add state management for comprehensive mode
- [x] Update handleResumeSession for comprehensive mode
- [x] Update handleSessionCreated for comprehensive mode
- [x] Add handleComprehensiveModeComplete handler
- [x] Render ComprehensiveModeFlow component

### Implementation Summary
- **Backend**: ~1,186 lines (Rust)
- **Frontend**: ~1,040 lines (TypeScript/React)
- **Total**: ~2,226 lines of production code
- **Files Created**: 11 new files
- **Files Modified**: 5 existing files
- **API Endpoints**: 8 new REST endpoints

### Features Delivered
- ✅ Competitor URL scraping and analysis
- ✅ Feature extraction and comparison
- ✅ Gap analysis identifying opportunities
- ✅ UI/UX pattern extraction from websites
- ✅ Similar project tracking
- ✅ AI-powered lesson extraction
- ✅ Research synthesis with market positioning
- ✅ 24-hour intelligent caching
- ✅ Rate limiting (2s delay, max 3 concurrent)
- ✅ Type-safe API with error handling
- ✅ React Query integration for optimal UX

### Deferred to Future Enhancement
- [ ] Screenshot analysis with AI vision (not implemented - focused on text analysis)
- [ ] Advanced visual comparison tools

---

## Phase 6: Comprehensive Mode - Expert Roundtable (Week 6)

### Backend - Roundtable System
- [ ] POST `/api/ideate/{id}/roundtable/start` - Start discussion
- [ ] GET `/api/ideate/{id}/roundtable/stream` - SSE stream endpoint
- [ ] POST `/api/ideate/{id}/roundtable/question` - User interjection
- [ ] POST `/api/ideate/{id}/experts/suggest` - AI suggest experts
- [ ] Create expert persona system
- [ ] Implement streaming chat with multiple experts
- [ ] Add moderator AI to orchestrate discussion
- [ ] Handle user interjections mid-discussion

### Frontend - Roundtable UI
- [ ] Create `ComprehensiveMode/ExpertRoundtable/`
- [ ] Create `ExpertSelector.tsx` - Choose/create experts
- [ ] Create `LiveDiscussion.tsx` - Real-time chat interface
- [ ] Create `ExpertCard.tsx` - Expert profile display
- [ ] Create `InsightsExtractor.tsx` - Extract key insights
- [ ] Add streaming message display
- [ ] Add interjection input
- [ ] Add "End discussion" functionality

---

## Phase 7: PRD Generation & Export (Week 7)

### Backend - PRD Generation
- [ ] POST `/api/ideate/{id}/generate-prd` - Generate complete PRD
- [ ] POST `/api/ideate/{id}/generate-section/{section}` - Generate one section
- [ ] GET `/api/ideate/{id}/prd/preview` - Preview before save
- [ ] POST `/api/ideate/{id}/prd/save` - Save as PRD in system
- [ ] Implement section-by-section generation
- [ ] Handle skipped sections with AI fill
- [ ] Add PRD validation logic
- [ ] Create markdown export

### Frontend - PRD Generation UI
- [ ] Create `PRDGenerator/`
- [ ] Create `PreviewPane.tsx` - Live PRD preview
- [ ] Create `SectionEditor.tsx` - Edit generated sections
- [ ] Create `ExportOptions.tsx` - Choose export format
- [ ] Add "Generate PRD" button
- [ ] Add section regeneration
- [ ] Add manual editing before save
- [ ] Add export formats (Markdown, PDF, HTML)

### Integration
- [ ] Connect to existing PRD system
- [ ] Test full Quick Mode flow
- [ ] Test full Guided Mode flow
- [ ] Test full Comprehensive Mode flow
- [ ] Add comprehensive error handling
- [ ] Add loading states throughout

---

## Phase 8: Polish & Optimization (Week 8)

### Templates & Intelligence
- [ ] Create default quickstart templates
- [ ] Add template selection UI
- [ ] Implement smart section suggestions
- [ ] Add auto-save functionality
- [x] Session list & resume (moved to Phase 2 - COMPLETED)

### UX Improvements
- [ ] Add keyboard shortcuts
- [ ] Add tooltips and help text
- [ ] Improve mobile responsiveness
- [ ] Add dark mode support
- [ ] Polish animations and transitions

### Performance
- [ ] Optimize database queries
- [ ] Add caching where appropriate
- [ ] Implement request debouncing
- [ ] Optimize bundle size
- [ ] Add loading skeletons

### Testing & Documentation
- [ ] Write unit tests for backend handlers
- [ ] Write integration tests for flows
- [ ] Write component tests for UI
- [ ] Create user documentation
- [ ] Create developer documentation
- [ ] Add inline code comments

### Final Review
- [ ] Test all three modes end-to-end
- [ ] Test skip functionality
- [ ] Test AI fill functionality
- [ ] Test dependency chain builder
- [ ] Test roundtable discussions
- [ ] Test competitor analysis
- [ ] Fix any bugs found
- [ ] Performance profiling
- [ ] Security audit
- [ ] Accessibility check

---

## Success Criteria

### Quick Mode ✅ COMPLETED
- [x] User can enter one-liner and get complete PRD in < 30 seconds
- [x] Generated PRD includes all 8 sections
- [x] User can edit and save PRD

### Guided Mode ✅ COMPLETED
- [x] User can navigate between 7 sections (Overview, UX, Technical, Roadmap, Dependencies, Risks, Research)
- [x] Navigation works smoothly with Previous/Next buttons and sidebar
- [x] Progress is tracked with current_section in database
- [x] Can resume sessions from SessionsList
- [x] Visual progress indicator shows completion percentage
- [x] Form data persists across navigation (React Query cache)
- [x] Can save as PRD when ready
- [ ] User can skip any section (UI ready, backend needs AI fill implementation)
- [ ] Skipped sections can be AI-filled (deferred to future enhancement)

### Comprehensive Mode
- [ ] Roundtable works with 3+ experts
- [ ] Competitor analysis extracts features
- [ ] Similar projects add value
- [ ] All insights feed into PRD

### Dependency Chain
- [ ] Visual dependency graph works
- [ ] Auto-detection finds dependencies
- [ ] Build order is logical
- [ ] Foundation/Visible/Enhancement phases clear

### PRD Quality
- [ ] Generated PRDs match template structure
- [ ] Content is coherent and actionable
- [ ] No timeline pressure (scope only)
- [ ] Logical dependency chain included

---

## Notes
- All sections are optional except initial description
- Focus on logical build order, not timelines
- Dependency chain is critical for development planning
- Get to "visible/usable" features quickly
- Features should be atomic but extensible
