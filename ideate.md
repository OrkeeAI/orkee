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

## Phase 2: Quick Mode (One-liner → PRD) (Week 2) 🚧 IN PROGRESS

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

### Frontend - Session List View
- [ ] Create `packages/dashboard/src/components/ideate/SessionsList.tsx`
- [ ] Display all ideate sessions for current project
- [ ] Show session mode (Quick/Guided/Comprehensive)
- [ ] Show session status (Draft/In Progress/Ready/Completed)
- [ ] Add search/filter by mode and status
- [ ] Add "Resume session" functionality
- [ ] Add "Delete session" with confirmation
- [ ] Add created/updated timestamps
- [ ] Show linked PRD if completed

### Frontend - Quick Mode UI
- [ ] Create `packages/dashboard/src/components/ideate/QuickMode/`
- [ ] Create `OneLineInput.tsx` - Simple description input
- [ ] Create `QuickExpansion.tsx` - Section selection
- [ ] Create `InstantPRD.tsx` - Show generated PRD
- [ ] Add loading states (non-streaming for now)
- [ ] Add edit capabilities after generation
- [ ] Add "Save as PRD" functionality
- [ ] Test with various one-liner descriptions

### Integration
- [ ] Wire up Quick Mode to PRD save flow
- [ ] Add PRD preview before saving
- [ ] Handle generation errors gracefully
- [ ] Add success/error notifications

### Future Enhancements
- [ ] Add SSE streaming support for real-time generation
- [ ] Implement token limit handling with chunking
- [ ] Add retry logic with exponential backoff

---

## Phase 3: Guided Mode - Core Sections (Week 3)

### Backend - Section Endpoints
- [ ] POST/GET `/api/ideate/{id}/overview`
- [ ] POST/GET `/api/ideate/{id}/features`
- [ ] POST/GET `/api/ideate/{id}/ux`
- [ ] POST/GET `/api/ideate/{id}/technical`
- [ ] POST/GET `/api/ideate/{id}/roadmap`
- [ ] POST/GET `/api/ideate/{id}/dependencies`
- [ ] POST/GET `/api/ideate/{id}/risks`
- [ ] POST/GET `/api/ideate/{id}/research`
- [ ] Add AI suggestion endpoints for each section
- [ ] Implement skip with AI fill functionality

### Frontend - Guided Mode Structure
- [ ] Create `packages/dashboard/src/components/ideate/GuidedMode/`
- [ ] Create `SectionNavigator.tsx` - Navigation between sections
- [ ] Create `SkipDialog.tsx` - Confirm skip with AI fill option
- [ ] Create `SectionProgress.tsx` - Visual progress indicator

### Frontend - Individual Section Forms
- [ ] Create `Sections/OverviewSection.tsx`
  - [ ] Problem statement input
  - [ ] Target audience selector
  - [ ] Value proposition builder
- [ ] Create `Sections/FeaturesSection.tsx`
  - [ ] Feature list with What/Why/How
  - [ ] Dependency picker (to other features)
  - [ ] Phase selector (foundation/visible/enhancement)
- [ ] Create `Sections/UXSection.tsx`
  - [ ] Persona builder
  - [ ] User flow mapper
  - [ ] UI/UX considerations
- [ ] Create `Sections/TechnicalSection.tsx`
  - [ ] Component architecture
  - [ ] Data model builder
  - [ ] API/integration list
  - [ ] Tech stack selector
- [ ] Create `Sections/RoadmapSection.tsx`
  - [ ] MVP scope builder (NO timelines)
  - [ ] Future phase planner
  - [ ] Scope-only focus
- [ ] Create `Sections/DependencyChainSection.tsx`
  - [ ] Foundation features picker
  - [ ] Visible features picker
  - [ ] Enhancement features picker
  - [ ] Build order visualizer
- [ ] Create `Sections/RisksSection.tsx`
  - [ ] Risk identifier
  - [ ] Mitigation planner
- [ ] Create `Sections/AppendixSection.tsx`
  - [ ] Research notes
  - [ ] Technical specs
  - [ ] References

### Shared Components
- [ ] Create `DependencyGraph.tsx` - Visual dependency viewer
- [ ] Create `PhaseBuilder.tsx` - Foundation/Visible/Enhancement UI
- [ ] Create `FeatureCard.tsx` - Reusable feature display
- [ ] Create `SkipButton.tsx` - Skip section button
- [ ] Create `AIFillButton.tsx` - AI complete section button
- [ ] Create `PRDPreview.tsx` - Live PRD preview panel

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
- [ ] Session list & resume (moved to Phase 2)

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

### Quick Mode
- [ ] User can enter one-liner and get complete PRD in < 30 seconds
- [ ] Generated PRD includes all 8 sections
- [ ] User can edit and save PRD

### Guided Mode
- [ ] User can skip any section
- [ ] Skipped sections can be AI-filled
- [ ] Navigation works smoothly
- [ ] Progress is saved automatically
- [ ] Can resume sessions

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
