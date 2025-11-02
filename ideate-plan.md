# Orkee Ideation → PRD → Epic → Task Flow Optimization Plan

**Date**: 2025-11-02
**Status**: Planning Phase → Implementation
**Document**: Strategic plan for improving Orkee's ideation and task generation workflow

---

## Executive Summary

This document outlines a comprehensive plan to optimize Orkee's ideation → PRD → Epic → Task flow by incorporating best practices from CCPM, ai-dev-tasks, Superpowers, and claude-task-master while maintaining Orkee's unique database-first architecture.

## Critical Context & Requirements

### References Analyzed
- **CCPM**: https://github.com/automazeio/ccpm - Parallel agent execution, GitHub-native workflow
- **ai-dev-tasks**: https://github.com/snarktank/ai-dev-tasks - Two-phase task generation, codebase awareness
- **Superpowers**: https://github.com/obra/superpowers - One-question-at-a-time, bite-sized steps
- **claude-task-master**: https://github.com/eyaltoledano/claude-task-master - TDD enforcement, complexity analysis

### Key Design Constraints
1. **No production users yet** - Can modify 001_initial_schema.sql directly
2. **Everything in SQLite** - All data must be stored in database for cloud sync
3. **TDD integration required** - Every task must have test strategy
4. **Task count limits** - Max 20 tasks per epic (user configurable)
5. **PRD template compatibility** - Must store all sections to support any prd_output_template

### Current Orkee Architecture
- **Backend**: Rust with Axum, SQLx runtime queries
- **Frontend**: React with TypeScript, React Query
- **Database**: SQLite at ~/.orkee/orkee.db
- **Prompts**: Located in packages/prompts/src/ideate/
- **API Pattern**: JSON responses {success, data, error}

## Implementation Progress Tracker

### Overall Phase Status
- [x] **Phase 1**: Database Schema Updates (Day 1) - COMPLETED
- [x] **Phase 2**: Ideation & Discovery Improvements (Day 2-3) - COMPLETED
- [x] **Phase 3**: PRD Generation Enhancements (Day 4-5) - COMPLETED
- [x] **Phase 4**: Epic & Task Decomposition (Day 6-8) - COMPLETED (core backend functionality)
- [x] **Phase 5**: Execution & Progress Tracking (Day 9-10) - COMPLETED (core backend functionality)
- [ ] **Phase 6**: Integration & Polish - **EXPANDED INTO 7 SUB-PHASES (6A-6G)**
  - [ ] **Phase 6A**: API Endpoint Completion (Priority 1)
  - [ ] **Phase 6B**: Prompt Enhancements (Priority 2)
  - [ ] **Phase 6C**: UI Integration - Chat Mode (Priority 3)
  - [ ] **Phase 6D**: UI Integration - Quick & Guided Modes (Priority 4)
  - [ ] **Phase 6E**: UI Integration - Epic & Task Views (Priority 5)
  - [ ] **Phase 6F**: Testing (Priority 6)
  - [ ] **Phase 6G**: Documentation & Release (Priority 7)

**Key Question**: How do these improvements integrate with Orkee's existing Quick, Guided, and Chat modes?

---

## Mode Integration Strategy

### Current Orkee Modes

1. **Quick Mode**: One-liner → AI generates full PRD instantly
2. **Guided Mode**: Step-by-step wizard with skippable sections
3. **Chat Mode**: Conversational discovery through dialogue

### How Improvements Apply to Each Mode

#### Quick Mode
**Current Flow**: One-liner → Generate all sections → Display PRD

**Enhanced Flow with Improvements**:
- ✅ **Non-Goals section** - Generated automatically from one-liner
- ✅ **Open Questions** - AI identifies ambiguities in one-liner
- ✅ **Codebase context** - Analyzed before generation
- ✅ **Success metrics** - AI generates measurable criteria
- ⚠️ **Incremental validation** - Optional "review mode" after generation
- ❌ **One-question-at-a-time** - Not applicable (one-shot generation)
- ✅ **Two-phase task generation** - Applies to Epic→Task regardless of PRD source
- ✅ **TDD enforcement** - All tasks get test strategies

**Summary**: Quick Mode benefits from better PRD structure and task generation, but keeps its one-shot nature.

#### Guided Mode
**Current Flow**: Step-by-step wizard → Fill sections → Generate PRD

**Enhanced Flow with Improvements**:
- ✅ **Non-Goals section** - New wizard step
- ✅ **Open Questions** - New wizard step
- ✅ **Codebase context** - Auto-populated in technical section
- ✅ **Success metrics** - Enhanced in existing step
- ✅ **Incremental validation** - After each wizard step
- ⚠️ **One-question-at-a-time** - Wizard already does this
- ✅ **Alternative approaches** - New step for technical approach selection
- ✅ **Two-phase task generation** - Applies to Epic→Task
- ✅ **TDD enforcement** - All tasks get test strategies

**Summary**: Guided Mode gets new wizard steps and better validation at each step.

#### Chat Mode (Conversational)
**Current Flow**: Free-form conversation → AI extracts PRD sections → Generate PRD

**Enhanced Flow with Improvements**:
- ✅ **Non-Goals section** - AI asks "What should we NOT build?"
- ✅ **Open Questions** - AI captures uncertainties during chat
- ✅ **Codebase context** - Analyzed and discussed in conversation
- ✅ **Success metrics** - AI prompts for measurable outcomes
- ✅ **Incremental validation** - "Does this summary look right?" checkpoints
- ✅ **One-question-at-a-time** - PRIMARY IMPROVEMENT for this mode
- ✅ **Alternative approaches** - AI presents options during conversation
- ✅ **Two-phase task generation** - Applies to Epic→Task
- ✅ **TDD enforcement** - All tasks get test strategies

**Summary**: Chat Mode benefits most from one-question-at-a-time and incremental validation.

### Unified Improvements (All Modes)

These improvements apply **after PRD generation**, regardless of which mode created the PRD:

1. **Epic Generation**:
   - Alternative approach exploration (2-3 options)
   - Simplification analysis (<20 tasks target)
   - Codebase context integration

2. **Task Decomposition**:
   - Two-phase generation (parent tasks → review → subtasks)
   - Complexity-based task count
   - TDD-first with required test strategies
   - File-level references
   - Bite-sized execution steps (2-5 min each)

3. **Progress Tracking**:
   - Checkpoint system
   - Append-only updates
   - Validation history

---

## Implementation Plan

### Core Philosophy
Take the best ideas from CCPM, ai-dev-tasks, Superpowers, and task-master while maintaining Orkee's unique database-first architecture and multi-mode ideation system.

---

## Phase 1: Database Schema Updates (Day 1)
**Since no users yet, we'll modify 001_initial_schema.sql directly**

### Phase 1 Checklist
- [x] Update 001_initial_schema.sql with new fields
- [x] Update 001_initial_schema.down.sql with drop statements
- [x] Test schema migration locally
- [x] Update Rust structs to match new schema
- [ ] Update TypeScript types to match new schema (deferred - not critical for Phase 1)

### 1.1 Enhanced PRD Storage

Location: `/packages/storage/migrations/001_initial_schema.sql`

**IMPORTANT**: The ideate_sessions table is at line 1926. Add these columns BEFORE the closing );

```sql
-- Add to ideate_sessions table (find at line 1926, add before line 1961)
-- These fields ensure we can recreate PRDs with any template
ALTER TABLE ideate_sessions ADD COLUMN non_goals TEXT;         -- What we're NOT building
ALTER TABLE ideate_sessions ADD COLUMN open_questions TEXT;     -- Known unknowns
ALTER TABLE ideate_sessions ADD COLUMN constraints_assumptions TEXT; -- Limitations
ALTER TABLE ideate_sessions ADD COLUMN success_metrics TEXT;    -- Measurable criteria
ALTER TABLE ideate_sessions ADD COLUMN alternative_approaches TEXT; -- JSON: 2-3 options with trade-offs
ALTER TABLE ideate_sessions ADD COLUMN validation_checkpoints TEXT; -- JSON: section-by-section validation
ALTER TABLE ideate_sessions ADD COLUMN codebase_context TEXT;   -- JSON: existing patterns found
```

### 1.2 Enhanced Epic Storage

Location: `/packages/storage/migrations/001_initial_schema.sql`

```sql
-- Add to epics table (around line 120)
ALTER TABLE epics ADD COLUMN codebase_context TEXT;      -- JSON: existing patterns found
ALTER TABLE epics ADD COLUMN simplification_analysis TEXT; -- Opportunities to leverage existing code
ALTER TABLE epics ADD COLUMN task_count_limit INTEGER DEFAULT 20; -- User-configurable max
ALTER TABLE epics ADD COLUMN decomposition_phase TEXT CHECK(decomposition_phase IN ('parent_planning', 'subtask_generation', 'completed'));
ALTER TABLE epics ADD COLUMN parent_tasks TEXT;          -- JSON: high-level tasks before expansion
ALTER TABLE epics ADD COLUMN quality_validation TEXT;    -- JSON: validation checklist results
```

### 1.3 Enhanced Task Storage with TDD

Location: `/packages/storage/migrations/001_initial_schema.sql`

```sql
-- Update tasks table (around line 580)
ALTER TABLE tasks ADD COLUMN test_strategy TEXT NOT NULL DEFAULT ''; -- Required TDD approach
ALTER TABLE tasks ADD COLUMN acceptance_criteria TEXT;   -- Clear completion definition
ALTER TABLE tasks ADD COLUMN relevant_files TEXT;        -- JSON: files to create/modify
ALTER TABLE tasks ADD COLUMN similar_implementations TEXT; -- References to existing code
ALTER TABLE tasks ADD COLUMN complexity_score INTEGER CHECK(complexity_score >= 1 AND complexity_score <= 10);
ALTER TABLE tasks ADD COLUMN execution_steps TEXT;       -- JSON: bite-sized 2-5 min steps
ALTER TABLE tasks ADD COLUMN validation_history TEXT;    -- JSON: append-only progress updates
ALTER TABLE tasks ADD COLUMN codebase_references TEXT;   -- JSON: patterns to follow
ALTER TABLE tasks ADD COLUMN parent_task_id TEXT;        -- For two-phase generation
```

### 1.4 New Tables for Better Organization

```sql
-- Task Complexity Analysis (inspired by task-master)
CREATE TABLE task_complexity_reports (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    epic_id TEXT NOT NULL,
    task_id TEXT,
    complexity_score INTEGER CHECK(complexity_score >= 1 AND complexity_score <= 10),
    recommended_subtasks INTEGER,
    expansion_prompt TEXT,
    reasoning TEXT,
    analyzed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE
);

CREATE INDEX idx_complexity_reports_epic ON task_complexity_reports(epic_id);
CREATE INDEX idx_complexity_reports_task ON task_complexity_reports(task_id);

-- Discovery Sessions (from Superpowers one-question-at-a-time)
CREATE TABLE discovery_sessions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    question_number INTEGER NOT NULL,
    question_text TEXT NOT NULL,
    question_type TEXT CHECK(question_type IN ('open', 'multiple_choice', 'yes_no')),
    options TEXT, -- JSON array for multiple choice
    user_answer TEXT,
    asked_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    answered_at TEXT,
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    UNIQUE(session_id, question_number)
);

CREATE INDEX idx_discovery_sessions_session ON discovery_sessions(session_id);
CREATE INDEX idx_discovery_sessions_order ON discovery_sessions(session_id, question_number);

-- PRD Validation History
CREATE TABLE prd_validation_history (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    section_name TEXT NOT NULL,
    validation_status TEXT CHECK(validation_status IN ('approved', 'rejected', 'regenerated')),
    user_feedback TEXT,
    quality_score INTEGER CHECK(quality_score >= 0 AND quality_score <= 100),
    validated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE
);

CREATE INDEX idx_prd_validation_session ON prd_validation_history(session_id);
CREATE INDEX idx_prd_validation_section ON prd_validation_history(section_name);
```

### 1.5 Update Down Migration

Location: `/packages/storage/migrations/001_initial_schema.down.sql`

Add drops for new tables:
```sql
DROP TABLE IF EXISTS prd_validation_history;
DROP TABLE IF EXISTS discovery_sessions;
DROP TABLE IF EXISTS task_complexity_reports;
```

---

## Phase 2: Ideation & Discovery Improvements (Day 2-3)

### Phase 2 Checklist
- [x] **2.1 One-Question-at-a-Time Discovery**
  - [x] Create discovery_manager.rs
  - [ ] Update ConversationalModeFlow.tsx (deferred to Phase 6 - UI work)
  - [x] Add discovery_sessions table usage
  - [x] Implement multiple choice formatting
  - [ ] Add progress indicator UI (deferred to Phase 6 - UI work)
- [x] **2.2 Codebase Context Analysis**
  - [x] Create codebase_analyzer.rs
  - [x] Implement pattern detection
  - [x] Find similar features logic
  - [ ] Integration with all three modes (deferred to Phase 6 - API integration)
- [x] **2.3 Alternative Approach Exploration**
  - [x] Create approach_generator.rs
  - [x] Generate 2-3 approaches with trade-offs
  - [ ] Add comparison UI component (deferred to Phase 6 - UI work)
  - [ ] Store in alternative_approaches field (requires API endpoints)

### 2.1 One-Question-at-a-Time Discovery (Chat Mode Primary)

**Implementation**: `packages/ideate/src/discovery_manager.rs`

```rust
pub struct DiscoveryManager {
    session_id: String,
    question_count: usize,
}

impl DiscoveryManager {
    pub async fn get_next_question(&self, context: &SessionContext) -> Result<Question> {
        // Analyze what we know so far
        // Determine what we need to know next
        // Return ONE question
        // Prefer multiple choice when possible

        let question = match self.question_count {
            0 => Question::open("What problem are you trying to solve?"),
            1 => Question::multiple_choice(
                "Who is the primary user?",
                vec!["Internal team", "External customers", "Both", "Other"]
            ),
            _ => self.generate_contextual_question(context).await?
        };

        // Store in discovery_sessions table
        self.save_question(&question).await?;

        Ok(question)
    }
}
```

**Frontend**: Update `ConversationalModeFlow.tsx`
- Display one question at a time
- Show progress indicator (question 3 of ~10)
- Allow "Skip" for non-critical questions

### 2.2 Codebase Context Analysis

**Implementation**: `packages/ideate/src/codebase_analyzer.rs`

```rust
pub struct CodebaseAnalyzer {
    project_path: PathBuf,
}

impl CodebaseAnalyzer {
    pub async fn analyze_for_session(&self, session: &IdeateSession) -> Result<CodebaseContext> {
        let mut context = CodebaseContext::default();

        // 1. Scan for existing patterns
        context.patterns = self.identify_patterns().await?;

        // 2. Find similar features
        context.similar_features = self.find_similar_features(&session.description).await?;

        // 3. Identify reusable components
        context.reusable_components = self.find_reusable_components().await?;

        // 4. Detect architecture style
        context.architecture_style = self.detect_architecture().await?;

        Ok(context)
    }

    async fn identify_patterns(&self) -> Result<Vec<Pattern>> {
        // Scan for:
        // - Database patterns (SQLx runtime queries)
        // - API patterns (Axum handlers)
        // - Frontend patterns (React components)
        // - Testing patterns
    }
}
```

**Integration Points**:
- Quick Mode: Run before PRD generation
- Guided Mode: Display in technical section
- Chat Mode: Discuss findings in conversation

### 2.3 Alternative Approach Exploration

**Implementation**: `packages/ideate/src/approach_generator.rs`

```rust
pub struct ApproachGenerator {
    epic: Epic,
    codebase_context: CodebaseContext,
}

impl ApproachGenerator {
    pub async fn generate_alternatives(&self) -> Result<Vec<TechnicalApproach>> {
        // Generate 2-3 approaches with trade-offs
        let approaches = vec![
            TechnicalApproach {
                name: "Leverage Existing System".to_string(),
                description: "Extend current architecture".to_string(),
                pros: vec!["Faster", "Less risk", "Familiar patterns"],
                cons: vec!["May hit scaling limits", "Technical debt"],
                estimated_days: 8,
                complexity: "Medium",
                recommended: true,
                reasoning: "Best balance of speed and maintainability",
            },
            TechnicalApproach {
                name: "Clean Slate Implementation".to_string(),
                description: "Build from scratch with modern patterns".to_string(),
                pros: vec!["Latest best practices", "No legacy constraints"],
                cons: vec!["Longer timeline", "Higher risk"],
                estimated_days: 15,
                complexity: "High",
                recommended: false,
                reasoning: "Only if current system can't support requirements",
            },
        ];

        Ok(approaches)
    }
}
```

---

## Phase 3: PRD Generation Enhancements (Day 4-5)

### Phase 3 Checklist
- [x] **3.1 Enhanced PRD Sections**
  - [x] Add Non-Goals section to prompts (non-goals.json)
  - [x] Add Open Questions section to prompts (open-questions.json)
  - [x] Add Success Metrics section to prompts (success-metrics.json)
  - [x] Update complete.json prompt with new sections
  - [x] Add prompt functions to prompts.rs
  - [ ] Update all mode prompts (Quick, Guided, Chat) - DEFERRED TO PHASE 6
- [ ] **3.2 Incremental Section Validation**
  - [ ] Quick Mode: Optional review toggle - DEFERRED TO PHASE 6 (UI work)
  - [ ] Guided Mode: Step validation - DEFERRED TO PHASE 6 (UI work)
  - [ ] Chat Mode: Natural checkpoints - DEFERRED TO PHASE 6 (UI work)
  - [ ] API endpoint for section validation - DEFERRED (requires API integration)
- [x] **3.3 PRD Quality Validation**
  - [x] Create PRDValidator struct
  - [x] Implement quality scoring
  - [x] Add comprehensive validation tests
  - [ ] Add validation UI component - DEFERRED TO PHASE 6 (UI work)
  - [ ] Pre-save checklist implementation - DEFERRED (requires API integration)

### 3.1 Enhanced PRD Sections

**Update prompts in**: `packages/prompts/src/ideate/*.md`

Add new sections to all PRD generation prompts:
```markdown
## Non-Goals (Out of Scope)
What are we explicitly NOT building? This prevents scope creep.
- Not implementing X because...
- Deferring Y to future phase...
- Excluding Z due to constraints...

## Open Questions
What needs clarification before implementation?
- [ ] How should we handle edge case A?
- [ ] What's the preferred approach for B?
- [ ] Need input on C from stakeholder

## Success Metrics
How do we measure success? (Must be quantifiable)
- Metric 1: Reduce X by Y% within Z days
- Metric 2: Increase A to B per month
- Metric 3: Achieve C score of D or higher
```

### 3.2 Incremental Section Validation

**Quick Mode**: Optional post-generation review
```typescript
// After PRD generation
if (user.prefers_validation) {
    for (const section of prd.sections) {
        const approved = await validateSection(section);
        if (!approved) {
            const regenerated = await regenerateSection(section, feedback);
            prd.sections[section.name] = regenerated;
        }
    }
}
```

**Guided Mode**: Validation after each step
```typescript
// In wizard flow
async function completeStep(step: WizardStep) {
    const content = await generateStepContent(step);
    const validation = await showValidation(content);

    if (validation.approved) {
        saveStepContent(step, content);
        proceedToNextStep();
    } else {
        const revised = await reviseContent(content, validation.feedback);
        saveStepContent(step, revised);
    }
}
```

**Chat Mode**: Natural checkpoints
```typescript
// During conversation
if (sectionsCompleted % 3 === 0) {
    await showSummary("Here's what we have so far. Does this look right?");
    const feedback = await getUserFeedback();
    if (feedback.needs_revision) {
        await discussRevisions(feedback);
    }
}
```

### 3.3 PRD Quality Validation

**Implementation**: `packages/ideate/src/validation.rs`

```rust
pub struct PRDValidator;

impl PRDValidator {
    pub fn validate(&self, prd: &GeneratedPRD) -> ValidationResult {
        let mut issues = Vec::new();
        let mut score = 100;

        // Check for placeholder text
        if prd.overview.problem_statement.contains("TODO") ||
           prd.overview.problem_statement.contains("[") {
            issues.push("Overview contains placeholder text");
            score -= 10;
        }

        // Check for Non-Goals section
        if prd.non_goals.is_none() || prd.non_goals.as_ref().unwrap().is_empty() {
            issues.push("Missing Non-Goals section (prevents scope creep)");
            score -= 15;
        }

        // Check for measurable success metrics
        if let Some(metrics) = &prd.success_metrics {
            if !metrics.iter().any(|m| m.contains(char::is_numeric)) {
                issues.push("Success metrics lack quantifiable targets");
                score -= 10;
            }
        }

        // Check acceptance criteria
        for feature in &prd.features {
            if feature.acceptance_criteria.is_none() {
                issues.push(&format!("Feature '{}' missing acceptance criteria", feature.name));
                score -= 5;
            }
        }

        ValidationResult {
            passed: score >= 70,
            score,
            issues,
            suggestions: self.generate_suggestions(&issues),
        }
    }
}
```

---

## Phase 4: Epic & Task Decomposition (Day 6-8)

### Phase 4 Checklist
- [x] **4.1 Two-Phase Task Generation**
  - [x] Implement generate_parent_tasks()
  - [x] Implement expand_to_subtasks()
  - [ ] Add parent task review UI (deferred to Phase 6 - UI work)
  - [ ] Add "Generate Detailed Tasks" button (deferred to Phase 6 - UI work)
  - [x] Store parent_tasks in epic field
- [x] **4.2 TDD-First Task Structure**
  - [x] Add test_strategy field (required in TaskTemplate)
  - [x] Add acceptance_criteria field
  - [x] Add execution_steps field (with TaskStep type)
  - [x] Generate TDD cycle steps (7-step TDD workflow)
  - [x] Validation for test strategy (required field)
- [x] **4.3 Complexity-Based Estimation**
  - [x] Create ComplexityAnalyzer
  - [x] Implement complexity scoring (1-10 scale)
  - [x] Dynamic task count calculation (based on complexity)
  - [x] Store complexity reports (task_complexity_reports table)
- [x] **4.4 Task Simplification**
  - [x] Add leverage analysis (via CodebaseContext integration)
  - [x] Identify reusable components (file references)
  - [x] Task combination suggestions (parent task placeholders)
  - [x] Enforce task count limits (respects epic.task_count_limit)

### 4.1 Two-Phase Task Generation

**Implementation**: `packages/ideate/src/task_decomposer.rs`

```rust
pub struct TaskDecomposer {
    epic: Epic,
    codebase_context: CodebaseContext,
}

impl TaskDecomposer {
    /// Phase 1: Generate high-level parent tasks
    pub async fn generate_parent_tasks(&self) -> Result<Vec<ParentTask>> {
        let complexity = self.analyze_complexity().await?;
        let task_count = self.calculate_task_count(complexity);

        // Generate 5-10 parent tasks based on complexity
        let parent_tasks = self.ai_generate_parents(task_count).await?;

        // Store in epic.parent_tasks field
        self.save_parent_tasks(&parent_tasks).await?;

        Ok(parent_tasks)
    }

    /// Phase 2: Expand parent tasks into detailed subtasks
    pub async fn expand_to_subtasks(&self, parent_tasks: &[ParentTask]) -> Result<Vec<Task>> {
        let mut all_tasks = Vec::new();

        for parent in parent_tasks {
            // Analyze codebase for this specific task
            let context = self.analyze_for_task(parent).await?;

            // Generate subtasks with all required fields
            let subtasks = self.generate_subtasks(parent, &context).await?;

            // Ensure TDD approach
            for task in &subtasks {
                self.validate_tdd_approach(task)?;
            }

            all_tasks.extend(subtasks);
        }

        // Analyze dependencies and parallel groups
        self.assign_parallel_groups(&mut all_tasks).await?;

        Ok(all_tasks)
    }

    fn calculate_task_count(&self, complexity: u8) -> usize {
        let user_limit = self.epic.task_count_limit.unwrap_or(20);

        match complexity {
            1..=3 => min(5, user_limit),   // Simple
            4..=6 => min(10, user_limit),  // Medium
            7..=8 => min(15, user_limit),  // Complex
            9..=10 => min(20, user_limit), // Very Complex
            _ => user_limit
        }
    }
}
```

**UI Flow**:
1. User triggers Epic → Task decomposition
2. System generates 5-10 parent tasks
3. UI displays parent tasks with "Review and Continue" button
4. User can edit/reorder/remove parent tasks
5. User clicks "Generate Detailed Tasks"
6. System expands into full task list with subtasks

### 4.2 TDD-First Task Structure

**Every task MUST have**:

```rust
#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub title: String,
    pub description: String,

    // REQUIRED TDD fields
    pub test_strategy: String,        // "Write integration test for auth endpoint"
    pub acceptance_criteria: Vec<String>, // ["Returns 200 for valid credentials", "Returns 401 for invalid"]

    // File references
    pub relevant_files: Vec<FileReference>,
    pub similar_implementations: Vec<String>,

    // Execution steps with TDD cycle
    pub execution_steps: Vec<TaskStep>,

    // Dependencies and parallelization
    pub depends_on: Vec<String>,
    pub parallel_group: Option<String>,
    pub conflicts_with: Vec<String>,

    // Tracking
    pub complexity_score: u8,
    pub estimated_hours: f32,
    pub validation_history: Vec<ValidationEntry>,
}

#[derive(Serialize, Deserialize)]
pub struct TaskStep {
    pub step_number: usize,
    pub action: String,              // "Write failing test for login endpoint"
    pub test_command: Option<String>, // "cargo test test_login_endpoint"
    pub expected_output: String,     // "FAIL: function not implemented"
    pub estimated_minutes: u8,       // 2-5 minutes per step
}

#[derive(Serialize, Deserialize)]
pub struct FileReference {
    pub path: String,
    pub operation: FileOperation,    // Create, Modify, Delete
    pub reason: String,              // Why this file?
}
```

### 4.3 Complexity-Based Estimation

**Implementation**: `packages/ideate/src/complexity_analyzer.rs`

```rust
pub struct ComplexityAnalyzer;

impl ComplexityAnalyzer {
    pub async fn analyze_epic(&self, epic: &Epic) -> ComplexityReport {
        let mut score = 0;

        // Factors that increase complexity
        if epic.technical_approach.contains("distributed") { score += 2; }
        if epic.technical_approach.contains("migration") { score += 2; }
        if epic.dependencies.len() > 5 { score += 1; }
        if epic.success_criteria.len() > 10 { score += 1; }

        // Factors that decrease complexity
        if epic.codebase_context.contains("similar_features") { score -= 1; }
        if epic.technical_approach.contains("existing") { score -= 1; }

        ComplexityReport {
            score: score.clamp(1, 10),
            reasoning: self.explain_score(score),
            recommended_tasks: self.recommend_task_count(score),
            expansion_strategy: self.suggest_strategy(score),
        }
    }
}
```

---

## Phase 5: Execution & Progress Tracking (Day 9-10)

### Phase 5 Checklist
- [x] **5.1 Bite-Sized Execution Steps**
  - [x] Generate 5-10 steps per task (implemented in task_decomposer.rs:234-290)
  - [x] TDD cycle enforcement (7-step TDD workflow)
  - [x] 2-5 minute step sizing (estimated_minutes field per step)
  - [x] Test commands in steps (test_command field with cargo test commands)
- [x] **5.2 Checkpoint System**
  - [x] Create ExecutionCheckpoint struct (execution_tracker.rs:20-29)
  - [x] Generate logical checkpoints (generate_checkpoints() method)
  - [ ] Add checkpoint UI modals (deferred to Phase 6 - UI work)
  - [x] Checkpoint validation logic (required_validation field)
  - [x] Database table: execution_checkpoints (001_initial_schema.sql:2274-2290)
- [x] **5.3 Append-Only Progress Tracking**
  - [x] Create ValidationEntry struct (execution_tracker.rs:52-60)
  - [x] Implement append_progress() (append_progress() method)
  - [x] Never overwrite history (append-only implementation)
  - [ ] Add progress UI display (deferred to Phase 6 - UI work)
  - [x] Database table: validation_entries (001_initial_schema.sql:2293-2305)

### 5.1 Bite-Sized Execution Steps

Each task must have 5-10 steps following TDD cycle:

```rust
fn generate_tdd_steps(task: &Task) -> Vec<TaskStep> {
    vec![
        TaskStep {
            step_number: 1,
            action: format!("Write failing test for {}", task.feature),
            test_command: Some("cargo test test_feature_name".to_string()),
            expected_output: "FAIL: function 'feature_name' not found".to_string(),
            estimated_minutes: 5,
        },
        TaskStep {
            step_number: 2,
            action: "Create minimal implementation stub".to_string(),
            test_command: None,
            expected_output: "File created with function signature".to_string(),
            estimated_minutes: 3,
        },
        TaskStep {
            step_number: 3,
            action: "Run test to verify it still fails correctly".to_string(),
            test_command: Some("cargo test test_feature_name".to_string()),
            expected_output: "FAIL: assertion failed (not implemented)".to_string(),
            estimated_minutes: 2,
        },
        TaskStep {
            step_number: 4,
            action: "Implement core functionality".to_string(),
            test_command: None,
            expected_output: "Implementation complete".to_string(),
            estimated_minutes: 15,
        },
        TaskStep {
            step_number: 5,
            action: "Run test to verify success".to_string(),
            test_command: Some("cargo test test_feature_name".to_string()),
            expected_output: "PASS: 1 test passed".to_string(),
            estimated_minutes: 2,
        },
        TaskStep {
            step_number: 6,
            action: "Refactor if needed".to_string(),
            test_command: Some("cargo test test_feature_name".to_string()),
            expected_output: "PASS: still passing after refactor".to_string(),
            estimated_minutes: 5,
        },
        TaskStep {
            step_number: 7,
            action: "Commit changes".to_string(),
            test_command: Some("git add . && git commit -m 'Add feature_name with tests'".to_string()),
            expected_output: "Committed to branch".to_string(),
            estimated_minutes: 2,
        },
    ]
}
```

### 5.2 Checkpoint System

```rust
#[derive(Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub after_task_id: String,
    pub checkpoint_type: CheckpointType,
    pub message: String,
    pub required_validation: Vec<String>,
}

pub enum CheckpointType {
    Review,      // User reviews completed work
    Test,        // Run test suite
    Integration, // Verify integration points
    Approval,    // Stakeholder approval needed
}

impl Epic {
    pub fn generate_checkpoints(&self) -> Vec<ExecutionCheckpoint> {
        let mut checkpoints = Vec::new();

        // After every 3 tasks or at logical boundaries
        checkpoints.push(ExecutionCheckpoint {
            after_task_id: "task_003".to_string(),
            checkpoint_type: CheckpointType::Review,
            message: "Database layer complete. Review schema before continuing?".to_string(),
            required_validation: vec!["Schema matches requirements", "Migrations run cleanly"],
        });

        checkpoints
    }
}
```

### 5.3 Append-Only Progress Tracking

```rust
#[derive(Serialize, Deserialize)]
pub struct ValidationEntry {
    pub timestamp: DateTime<Utc>,
    pub entry_type: ValidationEntryType,
    pub content: String,
    pub author: String,
}

pub enum ValidationEntryType {
    Progress,     // Work completed
    Issue,        // Problem encountered
    Decision,     // Technical decision made
    Checkpoint,   // Checkpoint reached
}

impl Task {
    /// Never overwrites, only appends
    pub fn add_progress(&mut self, entry: ValidationEntry) {
        self.validation_history.push(entry);
        // Never modify existing entries
        // Preserve all history
    }
}
```

---

## Phase 6: Integration & Polish - EXPANDED INTO SUB-PHASES

**Discovery (2025-11-02)**: After comprehensive codebase exploration, found that most Phase 1-5 backend functionality is COMPLETE. The following sub-phases focus on integration, new endpoints, UI work, and polish.

---

## Phase 6A: API Endpoint Completion (Priority 1)

### Status: **COMPLETED** (6 of 6 sections complete)

### Checklist
- [x] **6A.1 Discovery & Codebase Analysis Endpoints** (commit b2e0e88)
  - [x] POST `/api/ideate/sessions/{id}/analyze-codebase` - Trigger codebase analysis
  - [x] GET `/api/ideate/sessions/{id}/codebase-context` - Get analysis results
  - [x] POST `/api/ideate/sessions/{id}/next-question` - Chat mode discovery
  - [x] GET `/api/ideate/sessions/{id}/discovery-progress` - Discovery status
- [x] **6A.2 PRD Validation Endpoints** (commit 110d765)
  - [x] POST `/api/ideate/sessions/{id}/validate-section/{section}` - Validate individual section
  - [x] GET `/api/ideate/sessions/{id}/quality-score` - Get overall quality score
  - [x] POST `/api/ideate/sessions/{id}/validation-history` - Store validation feedback
- [x] **6A.3 Epic Alternative Approaches Endpoints** (commit 8639442)
  - [x] POST `/api/epics/{id}/generate-alternatives` - Generate 2-3 technical approaches
  - [x] GET `/api/epics/{id}/alternatives` - Get all alternatives
  - [x] PUT `/api/epics/{id}/select-approach` - Select preferred approach
- [x] **6A.4 Epic Complexity & Simplification Endpoints** (commit 27192a5)
  - [x] POST `/api/epics/{id}/analyze-complexity` - Complexity analysis
  - [x] POST `/api/epics/{id}/simplify` - Get simplification suggestions
  - [x] GET `/api/epics/{id}/leverage-analysis` - What existing code can be reused
- [x] **6A.5 Two-Phase Task Generation Endpoints** (HIGH PRIORITY) - **COMPLETED** (commit 6cb3612)
  - [x] POST `/api/projects/:project_id/epics/:epic_id/decompose-phase1` - Generate parent tasks only
  - [x] GET `/api/projects/:project_id/epics/:epic_id/parent-tasks` - Get parent tasks for review
  - [x] PUT `/api/projects/:project_id/epics/:epic_id/parent-tasks` - Update parent tasks before expansion
  - [x] POST `/api/projects/:project_id/epics/:epic_id/decompose-phase2` - Expand parent tasks to subtasks
- [x] **6A.6 Task Execution Tracking Endpoints** (commit 516bfc5)
  - [x] POST `/api/tasks/{id}/generate-steps` - Generate TDD execution steps
  - [x] POST `/api/tasks/{id}/append-progress` - Append-only progress update
  - [x] GET `/api/tasks/{id}/validation-history` - Get progress history
  - [x] GET `/api/tasks/{id}/checkpoints` - Get execution checkpoints
  - [x] POST `/api/epics/{id}/checkpoints` - Generate epic-level checkpoints

### Implementation Notes
- Most backend managers already exist (TaskDecomposer, ComplexityAnalyzer, etc.)
- Focus on creating thin handler wrappers
- Follow existing `ok_or_internal_error` pattern from other handlers
- Add to `packages/api/src/lib.rs` router configuration

---

## Phase 6B: Prompt Enhancements (Priority 2)

### Status: **NOT STARTED**

### Checklist
- [ ] **6B.1 Add Context Awareness to All Prompts**
  - [ ] Update `complete.json` with codebase context instructions
  - [ ] Update `features.json` to reference similar implementations
  - [ ] Update `technical.json` to leverage existing patterns
  - [ ] Add codebase_context parameter to prompt functions
- [ ] **6B.2 Add TDD Requirements**
  - [ ] Update all prompts to require test strategies
  - [ ] Add acceptance criteria requirements
  - [ ] Include test command examples
  - [ ] Emphasize test-first approach
- [ ] **6B.3 Add Simplification Pressure**
  - [ ] Add "Can we leverage existing code?" questions
  - [ ] Add "Can we combine requirements?" prompts
  - [ ] Add target task count reminders
  - [ ] Emphasize MINIMUM viable approach
- [ ] **6B.4 Add File Specificity**
  - [ ] Require exact file paths in all prompts
  - [ ] Request similar implementation references
  - [ ] Include "files to create/modify" in responses

### Implementation Notes
- Prompts are in `packages/prompts/prd/*.json`
- Update the `template` field with new instructions
- Increment `version` in metadata
- Update `lastModified` date
- Test with `packages/ideate/src/prompts.rs` functions

---

## Phase 6C: UI Integration - Chat Mode (Priority 3)

### Status: **NOT STARTED**

### Checklist
- [ ] **6C.1 One-Question-at-a-Time Enhancement**
  - [ ] Update `ChatModeFlow.tsx` to use discovery_sessions API
  - [ ] Add progress indicator ("Question 3 of ~10")
  - [ ] Show question type (open/multiple choice/yes_no)
  - [ ] Add "Skip" button for non-critical questions
- [ ] **6C.2 Codebase Context Display**
  - [ ] Add codebase analysis trigger button
  - [ ] Display found patterns in sidebar
  - [ ] Show similar features with links
  - [ ] Display reusable components
- [ ] **6C.3 Natural Validation Checkpoints**
  - [ ] Add summary modal every 3-5 questions
  - [ ] Show "Does this look right?" prompts
  - [ ] Allow inline editing of captured info
  - [ ] Store validation feedback

### Location
`packages/dashboard/src/components/ideate/ChatMode/`

---

## Phase 6D: UI Integration - Quick & Guided Modes (Priority 4)

### Status: **NOT STARTED**

### Checklist
- [ ] **6D.1 Quick Mode Review Toggle**
  - [ ] Add "Review sections before saving" checkbox
  - [ ] Post-generation section-by-section review
  - [ ] Allow regeneration of individual sections
  - [ ] Show quality score
- [ ] **6D.2 Guided Mode Step Validation**
  - [ ] Add validation after each wizard step
  - [ ] Show quality score per section
  - [ ] Allow section regeneration
  - [ ] Add "Continue" / "Regenerate" buttons
- [ ] **6D.3 Alternative Approaches UI**
  - [ ] Create comparison table component
  - [ ] Show pros/cons/estimated days
  - [ ] Highlight recommended approach
  - [ ] Allow approach selection

### Locations
- Quick Mode: `packages/dashboard/src/components/ideate/QuickMode/`
- Guided Mode: `packages/dashboard/src/components/ideate/GuidedMode/`

---

## Phase 6E: UI Integration - Epic & Task Views (Priority 5)

### Status: **NOT STARTED**

### Checklist
- [ ] **6E.1 Parent Task Review UI**
  - [ ] Display parent tasks before expansion
  - [ ] Allow reordering parent tasks
  - [ ] Allow editing/deleting parent tasks
  - [ ] Add "Generate Detailed Tasks" button
  - [ ] Show estimated task count
- [ ] **6E.2 Complexity Display**
  - [ ] Show complexity score (1-10)
  - [ ] Display complexity reasoning
  - [ ] Show recommended task count
  - [ ] Task count vs. limit indicator
- [ ] **6E.3 TDD Task View**
  - [ ] Display test strategy prominently
  - [ ] Show execution steps with timing
  - [ ] Make test commands copyable
  - [ ] Link relevant files
  - [ ] Show similar implementations
- [ ] **6E.4 Checkpoint UI Modals**
  - [ ] Checkpoint trigger modal
  - [ ] Required validation checklist
  - [ ] Progress history timeline
  - [ ] Append-only progress display

### Locations
- Epic views: `packages/dashboard/src/components/epics/` (may need creation)
- Task views: `packages/dashboard/src/components/tasks/` (may need creation)

---

## Phase 6F: Testing (Priority 6)

### Status: **NOT STARTED**

### Checklist
- [ ] **6F.1 Backend Unit Tests**
  - [ ] Test new API endpoints
  - [ ] Test prompt parameter substitution
  - [ ] Test codebase analyzer logic
  - [ ] Test complexity scoring
  - [ ] Test task decomposition flows
- [ ] **6F.2 Integration Tests**
  - [ ] Test full ideate → PRD → Epic → Tasks flow
  - [ ] Test two-phase task generation
  - [ ] Test checkpoint system
  - [ ] Test validation history
- [ ] **6F.3 Frontend Tests**
  - [ ] Test Chat Mode discovery flow
  - [ ] Test Quick Mode review toggle
  - [ ] Test Guided Mode validation
  - [ ] Test Epic parent task review
  - [ ] Test Task execution steps UI

---

## Phase 6G: Documentation & Release (Priority 7)

### Status: **NOT STARTED**

### Checklist
- [ ] **6G.1 API Documentation**
  - [ ] Document all new endpoints
  - [ ] Add request/response examples
  - [ ] Document error codes
  - [ ] Update OpenAPI spec if exists
- [ ] **6G.2 User Documentation**
  - [ ] Update mode selection guide
  - [ ] Two-phase task generation tutorial
  - [ ] TDD workflow guide
  - [ ] Checkpoint system explanation
- [ ] **6G.3 Developer Documentation**
  - [ ] Database schema changes summary
  - [ ] Prompt engineering guidelines
  - [ ] Testing requirements
  - [ ] Architecture decision records
- [ ] **6G.4 Release Preparation**
  - [ ] Write release notes
  - [ ] Create migration guide
  - [ ] Update CHANGELOG
  - [ ] Version bump

---

---

## Success Metrics

### Quantitative Goals

| Metric | Current | Target | Measurement |
|--------|---------|---------|------------|
| PRD Quality Score | ~70% | >85% | Validation scoring |
| Average Tasks per Epic | 20-30 | 10-15 | Database query |
| Tasks with Test Strategy | ~30% | 100% | Required field |
| Tasks with File References | ~20% | 100% | Database query |
| User Checkpoints per Flow | 0-1 | 3-5 | Checkpoint records |
| PRDs with Non-Goals | 0% | 100% | Database query |
| Two-phase Generation Usage | 0% | 80% | API analytics |
| TDD Step Coverage | 0% | 100% | Step analysis |

### Qualitative Goals

- **Discovery Experience**: Less overwhelming through one-question-at-a-time
- **Task Clarity**: Developers know exactly which files to work on
- **Progress Visibility**: Clear checkpoints and validation history
- **Simplification Culture**: Pressure to reuse, not rebuild
- **Quality Confidence**: Pre-execution validation catches issues early

---

## Risk Mitigation

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Schema migration fails | High | Test thoroughly, backup database |
| Two-phase generation confuses users | Medium | Clear UI, education, optional bypass |
| TDD enforcement too rigid | Medium | Allow override with warning |
| Complexity analysis inaccurate | Low | Adjustable, learning system |

### User Experience Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Longer PRD generation time | Medium | Show progress, allow skipping |
| Too many checkpoints | Medium | Configurable, smart defaults |
| One-question-at-a-time too slow | Low | Allow bulk questions option |

---

## Implementation Timeline

### Week 1: Foundation (Days 1-5)
- **Day 1**: Database schema updates
- **Day 2**: Non-Goals, Open Questions, Success Metrics sections
- **Day 3**: Codebase context analyzer
- **Day 4-5**: Two-phase task generation backend

### Week 2: Core Features (Days 6-10)
- **Day 6**: One-question-at-a-time (Chat mode)
- **Day 7**: TDD enforcement in tasks
- **Day 8**: Complexity-based task sizing
- **Day 9**: Checkpoint system
- **Day 10**: Validation and quality scoring

### Week 3: UI & Polish (Days 11-15)
- **Day 11-12**: Frontend updates for all modes
- **Day 13**: API endpoint integration
- **Day 14**: Testing and bug fixes
- **Day 15**: Documentation and release

---

## Migration Strategy

Since there are no production users yet:

1. **Direct schema modification** in `001_initial_schema.sql`
2. **No data migration needed**
3. **Clean slate advantages**
4. **Backward compatibility not required**

For future users who adopt before these changes:
- Migration scripts will be provided
- Existing PRDs will get default values for new fields
- Optional upgrade wizard to enhance existing content

---

## Validation Plan

### Unit Tests
- Codebase analyzer accuracy
- Complexity scoring logic
- TDD step generation
- Validation scoring

### Integration Tests
- Full flow: Ideation → PRD → Epic → Tasks
- Two-phase generation workflow
- Checkpoint system
- Database operations

### User Testing
- Each mode with improvements
- Task clarity and actionability
- Checkpoint interruption flow
- Quality validation accuracy

---

## Documentation Updates

### User Documentation
- Mode selection guide (which mode when?)
- Two-phase task generation explanation
- TDD workflow tutorial
- Checkpoint system guide

### Developer Documentation
- Database schema changes
- API endpoint reference
- Prompt engineering guidelines
- Testing requirements

---

## Future Enhancements (Post-Release)

1. **Machine Learning**:
   - Learn from user's validation choices
   - Improve complexity estimation over time
   - Better codebase pattern recognition

2. **Team Features**:
   - Different checkpoint rules per team
   - Shared PRD templates
   - Task assignment integration

3. **Advanced Execution**:
   - Parallel agent orchestration
   - Git worktree integration
   - Automated test running

4. **Analytics**:
   - Task estimation accuracy
   - Checkpoint effectiveness
   - Quality score correlation with success

---

## Critical Implementation Notes

### File Locations & Line Numbers
- **Schema**: `/packages/storage/migrations/001_initial_schema.sql`
  - ideate_sessions table: line 1926
  - epics table: line 116
  - tasks table: line 587
  - Add new tables after line 2210 (after existing CCPM tables)

- **Rust Types**:
  - `/packages/ideate/src/types.rs` - IdeateSession, Epic structs
  - `/packages/tasks/src/types.rs` - Task struct
  - `/packages/ideate/src/lib.rs` - Export new modules

- **TypeScript Types**:
  - `/packages/dashboard/src/services/ideate.ts` - IdeateSession, Epic interfaces
  - `/packages/dashboard/src/services/tasks.ts` - Task interface

- **API Handlers**:
  - `/packages/api/src/ideate_handlers.rs` - PRD generation endpoints
  - `/packages/api/src/ideate_conversational_handlers.rs` - Chat mode endpoints
  - `/packages/api/src/epic_handlers.rs` - Epic management
  - `/packages/api/src/lib.rs` - Route registration

- **Frontend Components**:
  - `/packages/dashboard/src/components/ideate/ConversationalMode/` - Chat mode
  - `/packages/dashboard/src/components/ideate/GuidedMode/` - Guided mode
  - `/packages/dashboard/src/components/ideate/QuickMode/` - Quick mode
  - `/packages/dashboard/src/components/epics/` - Epic management UI

### Key Implementation Gotchas

1. **SQLx Runtime Queries**: Orkee uses runtime queries (`sqlx::query()`), NOT compile-time macros (`sqlx::query!()`). This means:
   - Dynamic query construction is allowed
   - No `.sqlx/` cache needed
   - Schema validation happens at runtime

2. **API Response Format**: All endpoints must return:
   ```rust
   ApiResponse {
       success: bool,
       data: Option<T>,
       error: Option<String>,
   }
   ```

3. **React Query Hooks**: Follow existing pattern in `/packages/dashboard/src/components/epics/hooks/useEpics.ts`:
   - Use `useQuery` for fetching
   - Use `useMutation` for updates
   - Toast notifications on success/error
   - Query invalidation after mutations

4. **Prompt Updates**: Prompts are in `/packages/prompts/src/ideate/` as separate .md files:
   - `quick-mode.md`
   - `guided-mode.md`
   - `conversational-mode.md`
   - Each mode needs Non-Goals and Open Questions sections added

5. **Database Migrations**: Since no production users:
   - Directly modify `001_initial_schema.sql`
   - Also update `001_initial_schema.down.sql` with DROP statements
   - Test with: `rm ~/.orkee/orkee.db && cargo run`

## Detailed Task Breakdown for Implementation

### Phase 1 Tasks (Database Schema) - Day 1
- [ ] Backup existing database
- [ ] Modify `/packages/storage/migrations/001_initial_schema.sql`
  - [ ] Add fields to ideate_sessions table
  - [ ] Add fields to epics table
  - [ ] Add fields to tasks table
  - [ ] Create task_complexity_reports table
  - [ ] Create discovery_sessions table
  - [ ] Create prd_validation_history table
- [ ] Update `/packages/storage/migrations/001_initial_schema.down.sql`
- [ ] Test migration locally
- [ ] Update Rust types in `/packages/ideate/src/types.rs`
- [ ] Update TypeScript types in `/packages/dashboard/src/services/ideate.ts`

### Phase 2 Tasks (Discovery & Ideation) - Days 2-3
- [ ] Create `/packages/ideate/src/discovery_manager.rs`
- [ ] Create `/packages/ideate/src/codebase_analyzer.rs`
- [ ] Create `/packages/ideate/src/approach_generator.rs`
- [ ] Update `/packages/dashboard/src/components/ideate/ConversationalMode/ConversationalModeFlow.tsx`
- [ ] Add one-question-at-a-time logic
- [ ] Add multiple choice formatting
- [ ] Add progress indicator component
- [ ] Test Chat Mode improvements
- [ ] Test codebase analysis integration

### Phase 3 Tasks (PRD Generation) - Days 4-5
- [ ] Update prompts in `/packages/prompts/src/ideate/`
  - [ ] Add Non-Goals section
  - [ ] Add Open Questions section
  - [ ] Add Success Metrics section
- [ ] Create `/packages/ideate/src/validation.rs`
- [ ] Add incremental validation for each mode
- [ ] Create quality checklist UI component
- [ ] Add validation API endpoints
- [ ] Test PRD generation with new sections

### Phase 4 Tasks (Epic & Task Decomposition) - Days 6-8
- [ ] Update `/packages/ideate/src/task_decomposer.rs`
  - [ ] Add generate_parent_tasks()
  - [ ] Add expand_to_subtasks()
  - [ ] Add TDD step generation
- [ ] Create `/packages/ideate/src/complexity_analyzer.rs`
- [ ] Add parent task review UI
- [ ] Add task simplification logic
- [ ] Test two-phase generation flow
- [ ] Test complexity-based sizing

### Phase 5 Tasks (Execution & Progress) - Days 9-10
- [ ] Add TDD execution steps generator
- [ ] Create checkpoint system
- [ ] Implement append-only progress tracking
- [ ] Add checkpoint UI modals
- [ ] Add progress history display
- [ ] Test checkpoint interruptions

### Phase 6 Tasks (Integration & Polish) - Days 11-12
- [ ] Update all prompts with new requirements
- [ ] Complete UI enhancements for all modes
- [ ] Implement all new API endpoints
- [ ] Write unit tests
- [ ] Write integration tests
- [ ] Update documentation
- [ ] Prepare release notes
- [ ] Final testing and bug fixes

## Conclusion

This plan optimizes Orkee's ideation → PRD → Epic → Task flow by incorporating proven patterns from leading systems while maintaining Orkee's unique strengths:

- **Database-first architecture** (cloud-syncable)
- **Multi-mode flexibility** (Quick, Guided, Chat)
- **Rich UI** (not CLI-only)
- **Template system** (customizable PRD output)

The improvements focus on:
1. **Better discovery** through one-question-at-a-time and validation
2. **Richer PRDs** with Non-Goals and success metrics
3. **Smarter task generation** with two-phase approach and TDD
4. **Actionable tasks** with file references and execution steps
5. **Progress tracking** with checkpoints and append-only history

All improvements store data in SQLite for cloud sync capability and work across all three modes with mode-appropriate adaptations.

---

## Appendix: Key Code Examples

### Example: Adding Non-Goals to Rust Struct
```rust
// packages/ideate/src/types.rs
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct IdeateSession {
    // ... existing fields ...
    pub non_goals: Option<String>,
    pub open_questions: Option<String>,
    pub constraints_assumptions: Option<String>,
    pub success_metrics: Option<String>,
    pub alternative_approaches: Option<String>, // JSON
    pub validation_checkpoints: Option<String>, // JSON
    pub codebase_context: Option<String>, // JSON
}
```

### Example: Two-Phase Task API
```rust
// packages/api/src/ideate_handlers.rs
pub async fn decompose_phase1(
    Path(epic_id): Path<String>,
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<ParentTask>>> {
    let decomposer = TaskDecomposer::new(&state.pool);
    let parent_tasks = decomposer.generate_parent_tasks(&epic_id).await?;

    Ok(ApiResponse {
        success: true,
        data: Some(parent_tasks),
        error: None,
    })
}

pub async fn decompose_phase2(
    Path(epic_id): Path<String>,
    Json(parent_tasks): Json<Vec<ParentTask>>,
    State(state): State<AppState>,
) -> Result<ApiResponse<Vec<Task>>> {
    let decomposer = TaskDecomposer::new(&state.pool);
    let tasks = decomposer.expand_to_subtasks(&epic_id, &parent_tasks).await?;

    Ok(ApiResponse {
        success: true,
        data: Some(tasks),
        error: None,
    })
}
```

### Example: One-Question-at-a-Time Frontend
```typescript
// packages/dashboard/src/components/ideate/ConversationalMode/ConversationalModeFlow.tsx
const [currentQuestion, setCurrentQuestion] = useState(0);
const [answers, setAnswers] = useState<Record<number, string>>({});

const questions = [
  { type: 'open', text: 'What problem are you solving?' },
  {
    type: 'multiple',
    text: 'Who is the primary user?',
    options: ['Internal team', 'External customers', 'Both', 'Other']
  },
  // ...
];

const handleAnswer = (answer: string) => {
  setAnswers({ ...answers, [currentQuestion]: answer });

  // Store in discovery_sessions table
  await saveDiscoveryAnswer(sessionId, currentQuestion, answer);

  // Move to next question
  if (currentQuestion < questions.length - 1) {
    setCurrentQuestion(currentQuestion + 1);
  } else {
    // Generate PRD with all answers
    await generatePRD(sessionId, answers);
  }
};
```

### Example: TDD Task Structure
```typescript
// packages/dashboard/src/services/tasks.ts
interface Task {
  id: string;
  title: string;
  description: string;
  test_strategy: string; // REQUIRED
  acceptance_criteria: string[];
  relevant_files: FileReference[];
  execution_steps: TaskStep[];
  complexity_score: number;
  parent_task_id?: string;
}

interface TaskStep {
  step_number: number;
  action: string; // "Write failing test for login"
  test_command?: string; // "cargo test test_login"
  expected_output: string; // "FAIL: not implemented"
  estimated_minutes: number; // 2-5
}
```

### Example: Quality Validation
```rust
// packages/ideate/src/validation.rs
impl PRDValidator {
    pub fn validate(&self, prd: &GeneratedPRD) -> ValidationResult {
        let mut score = 100;
        let mut issues = Vec::new();

        // Check for Non-Goals
        if prd.non_goals.is_none() {
            issues.push("Missing Non-Goals section");
            score -= 15;
        }

        // Check for measurable success metrics
        if let Some(metrics) = &prd.success_metrics {
            let has_numbers = metrics.iter().any(|m|
                m.chars().any(|c| c.is_numeric())
            );
            if !has_numbers {
                issues.push("Success metrics lack quantifiable targets");
                score -= 10;
            }
        }

        ValidationResult {
            passed: score >= 70,
            score,
            issues
        }
    }
}
```

*End of Plan Document*