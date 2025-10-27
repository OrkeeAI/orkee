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
- [ ] Visual dependency graph (Phase 4)
- [ ] AI auto-fill for skipped sections
- [ ] Live PRD preview panel

---

## Phase 4: Dependency Chain Focus (Week 4)

### Backend - Dependency Intelligence
- [ ] POST `/api/ideate/{id}/features/dependencies` - Set dependencies
- [ ] POST `/api/ideate/{id}/dependencies/analyze` - Auto-detect dependencies
- [ ] POST `/api/ideate/{id}/dependencies/optimize` - Optimal build order
- [ ] POST `/api/ideate/{id}/features/suggest-visible` - Quick-to-visible features
- [ ] Create AI prompts for dependency detection
- [ ] Implement graph algorithms for build order
- [ ] Add validation for circular dependencies

### Frontend - Dependency Tools
- [ ] Create `DependencyMapper.tsx` - Interactive dependency builder
  - [ ] Drag-and-drop interface
  - [ ] Visual connection lines
  - [ ] Circular dependency warnings
- [ ] Create `BuildOrderVisualizer.tsx` - Show optimal build sequence
  - [ ] Timeline-style view (no dates, just order)
  - [ ] Highlight critical path
  - [ ] Show parallel-buildable features
- [ ] Create `FoundationPicker.tsx` - Select foundational features
- [ ] Create `VisibleFeatures.tsx` - Identify quick-win features
- [ ] Add "Optimize for quick visibility" button
- [ ] Add "Validate dependencies" button

### Integration
- [ ] Auto-suggest dependencies when features added
- [ ] Real-time dependency graph updates
- [ ] Export dependency graph as image
- [ ] Validate PRD completeness

---

## Phase 5: Comprehensive Mode - Research Tools (Week 5)

### Backend - Research & Analysis
- [ ] POST `/api/ideate/{id}/competitors/analyze` - Analyze competitor URL
- [ ] POST `/api/ideate/{id}/competitors/compare` - Compare features
- [ ] GET `/api/ideate/{id}/competitors` - List competitors
- [ ] POST `/api/ideate/{id}/similar/add` - Add similar project
- [ ] POST `/api/ideate/{id}/similar/extract-patterns` - Extract UI patterns
- [ ] Implement web scraping with `reqwest` + `scraper`
- [ ] Add screenshot analysis with AI vision
- [ ] Create competitor feature extraction logic

### Frontend - Research UI
- [ ] Create `ComprehensiveMode/CompetitorAnalysis/`
- [ ] Create `CompetitorScanner.tsx` - URL input and analysis
- [ ] Create `FeatureComparison.tsx` - Side-by-side comparison table
- [ ] Create `GapFinder.tsx` - Identify opportunities
- [ ] Create `ComprehensiveMode/SimilarProjects/`
- [ ] Create `ProjectFinder.tsx` - Add similar projects
- [ ] Create `PatternExtractor.tsx` - Extract UI/UX patterns
- [ ] Create `LessonsLearned.tsx` - Capture insights

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
