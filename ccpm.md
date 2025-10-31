# Conversational Mode (CCPM) Implementation Plan for Orkee

## Overview

This document outlines the implementation of a Conversational Mode (based on CCPM - Claude Code PM methodology) as a fourth ideation mode in Orkee. The system enables a conversational PRD discovery process, automatic Epic generation, task decomposition, and GitHub Issues integration for transparent execution tracking.

**Key Principles:**
- All content stored in SQLite (no filesystem storage)
- Conversational Mode added as fourth option (preserving existing modes)
- PRD → Epic → Tasks workflow with full traceability
- GitHub Issues for Epics and Tasks (PRDs remain private)
- No "ccpm_" prefixes in database fields

## Progress Tracking

### Overall Status
- [x] Phase 1: Database Schema Modifications
- [x] Phase 2: Conversational Mode UI
- [ ] Phase 3: Epic Management System
- [ ] Phase 4: Task Decomposition
- [ ] Phase 5: GitHub Integration
- [ ] Phase 6: Testing & Polish

---

## Phase 1: Database Schema Modifications

### 1.1 Schema Updates (`/packages/storage/migrations/001_initial_schema.sql`)

#### Modify Existing Tables

**ideate_sessions table:**
- [x] Update mode CHECK constraint to include 'conversational'
```sql
-- Line ~1171: Update the CHECK constraint
mode TEXT NOT NULL CHECK(mode IN ('quick', 'guided', 'comprehensive', 'conversational')),
```

**prds table:**
- [x] Add new fields for conversational mode
```sql
-- Add after line ~278 (before created_at)
conversation_id TEXT,           -- Links to prd_conversations
github_epic_url TEXT,          -- GitHub epic issue URL
discovery_status TEXT DEFAULT 'draft' CHECK(discovery_status IN ('draft', 'brainstorming', 'refining', 'validating', 'finalized')),
discovery_completed_at TEXT,
quality_score INTEGER CHECK(quality_score >= 0 AND quality_score <= 100),
```

**tasks table:**
- [x] Add Epic and GitHub fields
```sql
-- Add after line ~587 (after from_prd_id)
epic_id TEXT,
github_issue_number INTEGER,
github_issue_url TEXT,
parallel_group TEXT,
depends_on TEXT,               -- JSON array of task IDs
conflicts_with TEXT,           -- JSON array of task IDs
task_type TEXT DEFAULT 'task' CHECK(task_type IN ('task', 'subtask')),
size_estimate TEXT CHECK(size_estimate IN ('XS', 'S', 'M', 'L', 'XL')),
technical_details TEXT,        -- Implementation notes
effort_hours INTEGER CHECK(effort_hours > 0),
can_parallel BOOLEAN DEFAULT FALSE,
```

**projects table:**
- [x] Add GitHub configuration fields
```sql
-- Add after line ~71 (after git_repository)
github_owner TEXT,
github_repo TEXT,
github_sync_enabled BOOLEAN DEFAULT FALSE,
github_token_encrypted TEXT CHECK(github_token_encrypted IS NULL OR length(github_token_encrypted) >= 38),
github_labels_config TEXT,     -- JSON object with label mappings
github_default_assignee TEXT,
```

#### Create New Tables

- [x] Create prd_conversations table
```sql
-- Add after ideate tables section (~line 1960)

-- ============================================================================
-- CONVERSATIONAL MODE (CCPM) TABLES
-- ============================================================================

-- PRD Conversation History
CREATE TABLE prd_conversations (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    prd_id TEXT,
    message_order INTEGER NOT NULL,
    role TEXT NOT NULL CHECK(role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    message_type TEXT CHECK(message_type IN ('discovery', 'refinement', 'validation', 'general')),
    metadata TEXT,             -- JSON for tool calls, suggestions, etc.
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE,
    UNIQUE(session_id, message_order),
    CHECK (json_valid(metadata) OR metadata IS NULL)
);

CREATE INDEX idx_prd_conversations_session ON prd_conversations(session_id);
CREATE INDEX idx_prd_conversations_prd ON prd_conversations(prd_id);
CREATE INDEX idx_prd_conversations_order ON prd_conversations(session_id, message_order);
CREATE INDEX idx_prd_conversations_type ON prd_conversations(message_type);
```

- [x] Create epics table
```sql
-- Epic Management
CREATE TABLE epics (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    prd_id TEXT NOT NULL,
    name TEXT NOT NULL,

    -- Epic content (markdown stored in DB)
    overview_markdown TEXT NOT NULL,
    architecture_decisions TEXT,    -- JSON array of decisions with rationale
    technical_approach TEXT NOT NULL,
    implementation_strategy TEXT,
    dependencies TEXT,              -- JSON array of external dependencies
    success_criteria TEXT,          -- JSON array of measurable criteria

    -- Task breakdown metadata
    task_categories TEXT,           -- JSON array of task categories
    estimated_effort TEXT CHECK(estimated_effort IN ('days', 'weeks', 'months')),
    complexity TEXT CHECK(complexity IN ('low', 'medium', 'high', 'very_high')),

    -- Status tracking
    status TEXT DEFAULT 'draft' CHECK(status IN ('draft', 'ready', 'in_progress', 'blocked', 'completed', 'cancelled')),
    progress_percentage INTEGER DEFAULT 0 CHECK(progress_percentage >= 0 AND progress_percentage <= 100),

    -- GitHub integration
    github_issue_number INTEGER,
    github_issue_url TEXT,
    github_synced_at TEXT,

    -- Timestamps
    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    started_at TEXT,
    completed_at TEXT,

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    FOREIGN KEY (prd_id) REFERENCES prds(id) ON DELETE CASCADE,
    CHECK (json_valid(architecture_decisions) OR architecture_decisions IS NULL),
    CHECK (json_valid(dependencies) OR dependencies IS NULL),
    CHECK (json_valid(success_criteria) OR success_criteria IS NULL),
    CHECK (json_valid(task_categories) OR task_categories IS NULL)
);

CREATE INDEX idx_epics_project ON epics(project_id);
CREATE INDEX idx_epics_prd ON epics(prd_id);
CREATE INDEX idx_epics_status ON epics(status);
CREATE INDEX idx_epics_progress ON epics(progress_percentage);
CREATE INDEX idx_epics_github ON epics(github_issue_number);

-- Add foreign key constraint to tasks table
CREATE INDEX idx_tasks_epic ON tasks(epic_id);

-- Epic update trigger
CREATE TRIGGER epics_updated_at AFTER UPDATE ON epics
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE epics SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;
```

- [x] Create github_sync table
```sql
-- GitHub Synchronization Tracking
CREATE TABLE github_sync (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    project_id TEXT NOT NULL,
    entity_type TEXT NOT NULL CHECK(entity_type IN ('epic', 'task', 'comment', 'status')),
    entity_id TEXT NOT NULL,
    github_issue_number INTEGER,
    github_issue_url TEXT,
    sync_status TEXT DEFAULT 'pending' CHECK(sync_status IN ('pending', 'syncing', 'synced', 'failed', 'conflict')),
    sync_direction TEXT CHECK(sync_direction IN ('local_to_github', 'github_to_local', 'bidirectional')),
    last_synced_at TEXT,
    last_sync_hash TEXT,           -- SHA256 of content for change detection
    last_sync_error TEXT,
    retry_count INTEGER DEFAULT 0,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (project_id) REFERENCES projects(id) ON DELETE CASCADE,
    UNIQUE(entity_type, entity_id)
);

CREATE INDEX idx_github_sync_project ON github_sync(project_id);
CREATE INDEX idx_github_sync_entity ON github_sync(entity_type, entity_id);
CREATE INDEX idx_github_sync_status ON github_sync(sync_status);
CREATE INDEX idx_github_sync_issue ON github_sync(github_issue_number);
CREATE INDEX idx_github_sync_pending ON github_sync(sync_status) WHERE sync_status = 'pending';

-- GitHub sync update trigger
CREATE TRIGGER github_sync_updated_at AFTER UPDATE ON github_sync
FOR EACH ROW WHEN NEW.updated_at = OLD.updated_at
BEGIN
    UPDATE github_sync SET updated_at = strftime('%Y-%m-%dT%H:%M:%SZ', 'now') WHERE id = NEW.id;
END;
```

- [x] Create work_analysis table
```sql
-- Work Stream Analysis for Parallel Execution
CREATE TABLE work_analysis (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    epic_id TEXT NOT NULL,

    -- Analysis results (all JSON)
    parallel_streams TEXT NOT NULL,    -- Array of work stream definitions
    file_patterns TEXT,                -- Object mapping streams to file patterns
    dependency_graph TEXT NOT NULL,    -- Task dependency DAG
    conflict_analysis TEXT,            -- Potential file/resource conflicts
    parallelization_strategy TEXT,     -- Recommended execution strategy

    -- Metadata
    analyzed_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    is_current BOOLEAN DEFAULT TRUE,
    analysis_version INTEGER DEFAULT 1,
    confidence_score REAL CHECK(confidence_score >= 0.0 AND confidence_score <= 1.0),

    FOREIGN KEY (epic_id) REFERENCES epics(id) ON DELETE CASCADE,
    CHECK (json_valid(parallel_streams)),
    CHECK (json_valid(file_patterns) OR file_patterns IS NULL),
    CHECK (json_valid(dependency_graph)),
    CHECK (json_valid(conflict_analysis) OR conflict_analysis IS NULL),
    CHECK (json_valid(parallelization_strategy) OR parallelization_strategy IS NULL)
);

CREATE INDEX idx_work_analysis_epic ON work_analysis(epic_id);
CREATE INDEX idx_work_analysis_current ON work_analysis(epic_id, is_current);
```

- [x] Create discovery_questions table
```sql
-- Reusable Discovery Questions for Conversational Mode
CREATE TABLE discovery_questions (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    category TEXT NOT NULL CHECK(category IN ('problem', 'users', 'features', 'technical', 'risks', 'constraints', 'success')),
    question_text TEXT NOT NULL,
    follow_up_prompts TEXT,            -- JSON array of follow-up questions
    context_keywords TEXT,             -- JSON array of keywords that trigger this question
    priority INTEGER DEFAULT 5 CHECK(priority >= 1 AND priority <= 10),
    is_required BOOLEAN DEFAULT FALSE,
    display_order INTEGER,
    is_active BOOLEAN DEFAULT TRUE,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX idx_discovery_questions_category ON discovery_questions(category, display_order);
CREATE INDEX idx_discovery_questions_active ON discovery_questions(is_active, priority DESC);
CREATE INDEX idx_discovery_questions_required ON discovery_questions(is_required) WHERE is_required = TRUE;
```

- [x] Create conversation_insights table
```sql
-- Extracted Insights from Conversations
CREATE TABLE conversation_insights (
    id TEXT PRIMARY KEY CHECK(length(id) >= 8),
    session_id TEXT NOT NULL,
    insight_type TEXT NOT NULL CHECK(insight_type IN ('requirement', 'constraint', 'risk', 'assumption', 'decision')),
    insight_text TEXT NOT NULL,
    confidence_score REAL CHECK(confidence_score >= 0.0 AND confidence_score <= 1.0),
    source_message_ids TEXT,          -- JSON array of message IDs this came from
    applied_to_prd BOOLEAN DEFAULT FALSE,

    created_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),

    FOREIGN KEY (session_id) REFERENCES ideate_sessions(id) ON DELETE CASCADE,
    CHECK (json_valid(source_message_ids) OR source_message_ids IS NULL)
);

CREATE INDEX idx_conversation_insights_session ON conversation_insights(session_id);
CREATE INDEX idx_conversation_insights_type ON conversation_insights(insight_type);
CREATE INDEX idx_conversation_insights_applied ON conversation_insights(applied_to_prd);
```

### 1.2 Seed Data

- [x] Add default discovery questions
```sql
-- Insert after line ~2000
-- Default Discovery Questions for Conversational Mode
INSERT INTO discovery_questions (id, category, question_text, priority, is_required, display_order) VALUES
('dq-prob-1', 'problem', 'What specific problem are you trying to solve?', 10, TRUE, 1),
('dq-prob-2', 'problem', 'Who experiences this problem most acutely?', 9, TRUE, 2),
('dq-prob-3', 'problem', 'What happens if this problem isn''t solved?', 7, FALSE, 3),
('dq-user-1', 'users', 'Who are your primary users or customers?', 10, TRUE, 1),
('dq-user-2', 'users', 'What are their main goals and pain points?', 9, TRUE, 2),
('dq-user-3', 'users', 'How do they currently solve this problem?', 8, FALSE, 3),
('dq-feat-1', 'features', 'What are the must-have features for MVP?', 10, TRUE, 1),
('dq-feat-2', 'features', 'What features would delight users but aren''t essential?', 6, FALSE, 2),
('dq-feat-3', 'features', 'Are there features you explicitly want to avoid?', 5, FALSE, 3),
('dq-tech-1', 'technical', 'Do you have any technical constraints or requirements?', 8, FALSE, 1),
('dq-tech-2', 'technical', 'What existing systems need to integrate with this?', 7, FALSE, 2),
('dq-tech-3', 'technical', 'Are there performance or scalability requirements?', 6, FALSE, 3),
('dq-risk-1', 'risks', 'What are the biggest risks to this project?', 8, FALSE, 1),
('dq-risk-2', 'risks', 'What would cause this project to fail?', 7, FALSE, 2),
('dq-cons-1', 'constraints', 'What is your timeline for this project?', 9, TRUE, 1),
('dq-cons-2', 'constraints', 'Do you have budget or resource constraints?', 7, FALSE, 2),
('dq-succ-1', 'success', 'How will you measure success?', 9, TRUE, 1),
('dq-succ-2', 'success', 'What does "done" look like for the MVP?', 8, TRUE, 2);
```

---

## Phase 2: Conversational Mode UI Components

**Status**: ✅ COMPLETED

### 2.1 Component Structure

- [x] Create component directories
```
/packages/dashboard/src/components/ideate/ConversationalMode/
├── ConversationalMode.tsx
├── ConversationView.tsx
├── DiscoveryAssistant.tsx
├── MessageBubble.tsx
├── SuggestedQuestions.tsx
├── PRDPreview.tsx
├── QualityIndicator.tsx
├── ConversationHistory.tsx
├── InsightsSidebar.tsx
└── hooks/
    ├── useConversation.ts
    ├── useDiscoveryQuestions.ts
    └── useStreamingResponse.ts
```

### 2.2 Main Components

- [x] ConversationalModeFlow.tsx - Main container
```typescript
interface ConversationalModeProps {
  sessionId: string;
  projectId: string;
  onComplete: (prdId: string) => void;
}

// Component manages:
// - Conversation state
// - Discovery flow
// - PRD generation
// - Quality validation
```

- [x] ConversationView.tsx - Chat interface
```typescript
interface ConversationViewProps {
  messages: ConversationMessage[];
  onSendMessage: (content: string) => void;
  isLoading: boolean;
  suggestedQuestions?: string[];
}

// Features:
// - Message display with role indicators
// - Typing indicators
// - Suggested questions
// - Auto-scroll
```

- [x] DiscoveryAssistant.tsx - AI logic (integrated into ConversationalModeFlow)
```typescript
interface DiscoveryAssistantProps {
  sessionId: string;
  onInsightDetected: (insight: Insight) => void;
  onQualityUpdate: (score: number) => void;
}

// Handles:
// - Question selection based on context
// - Response analysis
// - Insight extraction
// - Quality scoring
```

- [x] PRDPreview.tsx - Live preview (integrated into ConversationalModeFlow via QualityIndicator)
```typescript
interface PRDPreviewProps {
  sessionId: string;
  isGenerating: boolean;
  qualityScore?: number;
}

// Shows:
// - Real-time PRD sections as they're discovered
// - Missing sections highlighted
// - Quality indicators
// - Generate button when ready
```

### 2.3 API Integration

- [x] Create API client functions
```typescript
// /packages/dashboard/src/services/conversational.ts

export const conversationalAPI = {
  // Session management
  startSession: (projectId: string, description: string) => Promise<Session>,
  getSession: (sessionId: string) => Promise<Session>,

  // Messaging
  sendMessage: (sessionId: string, content: string) => Promise<Message>,
  getHistory: (sessionId: string) => Promise<Message[]>,

  // PRD operations
  generatePRD: (sessionId: string) => Promise<PRD>,
  validatePRD: (sessionId: string) => Promise<ValidationResult>,

  // Discovery
  getQuestions: (category?: string) => Promise<DiscoveryQuestion[]>,
  getInsights: (sessionId: string) => Promise<Insight[]>,
};
```

- [x] Implement streaming support
```typescript
// SSE streaming for AI responses
export const streamConversation = (
  sessionId: string,
  message: string,
  onChunk: (chunk: string) => void,
  onComplete: () => void,
  onError: (error: Error) => void
) => {
  const eventSource = new EventSource(`/api/ideate/conversational/${sessionId}/stream`);
  // ... implementation
};
```

### 2.4 State Management

- [x] React Context used (no Redux needed - using React hooks and local state)
```typescript
// /packages/dashboard/src/store/slices/conversationalSlice.ts

interface ConversationalState {
  sessions: Record<string, ConversationSession>;
  activeSessionId: string | null;
  messages: Record<string, ConversationMessage[]>;
  insights: Record<string, Insight[]>;
  isGenerating: boolean;
  qualityScores: Record<string, number>;
}
```

---

### Phase 2 Implementation Summary

**Completed**: All Phase 2 components and integrations

**Files Created**:
1. `/packages/dashboard/src/services/conversational.ts` - API service
2. `/packages/dashboard/src/components/ideate/ConversationalMode/hooks/useConversation.ts`
3. `/packages/dashboard/src/components/ideate/ConversationalMode/hooks/useDiscoveryQuestions.ts`
4. `/packages/dashboard/src/components/ideate/ConversationalMode/hooks/useStreamingResponse.ts`
5. `/packages/dashboard/src/components/ideate/ConversationalMode/components/MessageBubble.tsx`
6. `/packages/dashboard/src/components/ideate/ConversationalMode/components/SuggestedQuestions.tsx`
7. `/packages/dashboard/src/components/ideate/ConversationalMode/components/QualityIndicator.tsx`
8. `/packages/dashboard/src/components/ideate/ConversationalMode/components/ConversationView.tsx`
9. `/packages/dashboard/src/components/ideate/ConversationalMode/components/InsightsSidebar.tsx`
10. `/packages/dashboard/src/components/ideate/ConversationalMode/ConversationalModeFlow.tsx`

**Files Modified**:
1. `/packages/dashboard/src/components/ideate/ModeSelector.tsx` - Added conversational mode option
2. `/packages/dashboard/src/components/ideate/CreatePRDFlow.tsx` - Added conversational mode descriptions
3. `/packages/dashboard/src/services/ideate.ts` - Updated IdeateMode type
4. `/packages/dashboard/src/components/specs/IdeateTab.tsx` - Integrated conversational mode flow

**Features Implemented**:
- ✅ Chat-based conversation interface with auto-scroll
- ✅ Real-time streaming responses via SSE
- ✅ Discovery question suggestions based on context
- ✅ Quality metrics and coverage tracking
- ✅ Insight extraction and categorization
- ✅ PRD generation from conversation
- ✅ Full integration with existing ideate flow
- ✅ Mode selector with 4 options (Quick, Guided, Comprehensive, Conversational)

**Architecture Decisions**:
- Used React hooks for state management (no Redux)
- Implemented SSE for streaming responses
- Co-located hooks with components in feature folder
- Followed existing component patterns from QuickMode/GuidedMode
- Full-screen conversational interface (not dialog-based)

---

## Phase 3: Epic Management System

### 3.1 Epic Components

- [ ] Create Epic UI components
```
/packages/dashboard/src/components/Epics/
├── EpicsTab.tsx
├── EpicList.tsx
├── EpicDetail.tsx
├── EpicGenerator.tsx
├── TaskBreakdown.tsx
├── DependencyView.tsx
├── WorkStreamAnalysis.tsx
└── GitHubSyncStatus.tsx
```

### 3.2 Epic Generation

- [ ] EpicGenerator.tsx
```typescript
interface EpicGeneratorProps {
  prdId: string;
  onEpicCreated: (epicId: string) => void;
}

// Process:
// 1. Analyze PRD content
// 2. Extract technical requirements
// 3. Generate architecture decisions
// 4. Create implementation strategy
// 5. Save to epics table
```

- [ ] Epic generation API endpoint
```typescript
// POST /api/epics/generate
interface GenerateEpicRequest {
  prdId: string;
  includeTaskBreakdown?: boolean;
}

interface GenerateEpicResponse {
  epicId: string;
  tasksCreated?: number;
}
```

### 3.3 Epic Display

- [ ] EpicDetail.tsx
```typescript
interface EpicDetailProps {
  epicId: string;
  onTasksGenerated: () => void;
}

// Displays:
// - Overview section
// - Architecture decisions
// - Technical approach
// - Implementation strategy
// - Task breakdown preview
// - GitHub sync status
// - Progress tracking
```

### 3.4 Epic Services

- [ ] Epic service layer
```typescript
// /packages/dashboard/src/services/epics.ts

export const epicsAPI = {
  // CRUD operations
  create: (epic: CreateEpicDto) => Promise<Epic>,
  get: (epicId: string) => Promise<Epic>,
  update: (epicId: string, updates: Partial<Epic>) => Promise<Epic>,
  delete: (epicId: string) => Promise<void>,

  // Generation
  generateFromPRD: (prdId: string) => Promise<Epic>,

  // Task operations
  decomposeToTasks: (epicId: string) => Promise<Task[]>,
  getEpicTasks: (epicId: string) => Promise<Task[]>,

  // Analysis
  analyzeWorkStreams: (epicId: string) => Promise<WorkAnalysis>,
  calculateProgress: (epicId: string) => Promise<number>,
};
```

---

## Phase 4: Task Decomposition

### 4.1 Task Generation Logic

- [ ] Task decomposer service
```typescript
// /packages/api/src/services/taskDecomposer.ts

export class TaskDecomposer {
  async decomposeEpic(epicId: string): Promise<Task[]> {
    // 1. Load epic content
    // 2. Identify task categories
    // 3. Generate tasks per category
    // 4. Detect dependencies
    // 5. Assign parallel groups
    // 6. Estimate sizes
    // 7. Save to database
  }

  async detectDependencies(tasks: Task[]): Promise<DependencyGraph> {
    // Analyze task relationships
  }

  async assignParallelGroups(tasks: Task[], dependencies: DependencyGraph): Promise<void> {
    // Group tasks that can run in parallel
  }
}
```

### 4.2 Task Breakdown UI

- [ ] TaskBreakdown.tsx
```typescript
interface TaskBreakdownProps {
  epicId: string;
  onTasksGenerated: (tasks: Task[]) => void;
}

// Features:
// - Task list grouped by category
// - Dependency visualization
// - Parallel group indicators
// - Size estimates
// - Edit capabilities
```

### 4.3 Dependency Visualization

- [ ] DependencyView.tsx
```typescript
interface DependencyViewProps {
  tasks: Task[];
  dependencies: DependencyEdge[];
}

// Visualizes:
// - Task nodes
// - Dependency arrows
// - Parallel groups (colored)
// - Critical path highlighting
```

### 4.4 Work Stream Analysis

- [ ] Work stream analyzer
```typescript
// /packages/api/src/services/workStreamAnalyzer.ts

export class WorkStreamAnalyzer {
  async analyze(epicId: string): Promise<WorkAnalysis> {
    // 1. Load tasks
    // 2. Identify file patterns
    // 3. Group by work streams
    // 4. Detect conflicts
    // 5. Generate strategy
  }

  identifyStreams(tasks: Task[]): WorkStream[] {
    // Database, API, UI, Tests, etc.
  }

  detectConflicts(streams: WorkStream[]): ConflictMatrix {
    // File-level conflict detection
  }
}
```

---

## Phase 5: GitHub Integration

### 5.1 GitHub Configuration

- [ ] Project settings UI
```typescript
// /packages/dashboard/src/components/ProjectSettings/GitHubSettings.tsx

interface GitHubSettingsProps {
  projectId: string;
}

// Settings:
// - Repository owner/name
// - Personal access token (encrypted)
// - Label configuration
// - Sync preferences
// - Default assignee
```

### 5.2 GitHub Service

- [ ] GitHub sync service
```typescript
// /packages/api/src/services/githubSync.ts

export class GitHubSyncService {
  // Epic operations
  async createEpicIssue(epicId: string): Promise<number> {
    // 1. Load epic from DB
    // 2. Format as GitHub issue
    // 3. Create with 'epic' label
    // 4. Update DB with issue number
  }

  async syncEpicToGitHub(epicId: string): Promise<void> {
    // Update existing issue
  }

  // Task operations
  async createTaskIssues(epicId: string): Promise<void> {
    // 1. Load tasks for epic
    // 2. Create issues with 'task' label
    // 3. Link to epic via task list
    // 4. Update DB with issue numbers
  }

  async linkTasksToEpic(epicIssueNumber: number, taskIssueNumbers: number[]): Promise<void> {
    // Update epic body with task list
  }

  // Sync operations
  async pullUpdates(projectId: string): Promise<SyncResult> {
    // 1. Fetch issues from GitHub
    // 2. Compare with local state
    // 3. Update local DB
    // 4. Handle conflicts
  }

  async resolveConflict(entityId: string, resolution: 'local' | 'remote'): Promise<void> {
    // Apply conflict resolution
  }
}
```

### 5.3 GitHub Webhook Handler

- [ ] Webhook endpoint
```typescript
// POST /api/github/webhook
interface GitHubWebhookHandler {
  handleIssueEvent(event: IssueEvent): Promise<void>;
  handleCommentEvent(event: CommentEvent): Promise<void>;
  handlePullRequestEvent(event: PullRequestEvent): Promise<void>;
}
```

### 5.4 Sync Status UI

- [ ] GitHubSyncStatus.tsx
```typescript
interface GitHubSyncStatusProps {
  projectId: string;
  epicId?: string;
}

// Displays:
// - Sync status (pending/synced/failed)
// - Last sync time
// - Conflict indicators
// - Manual sync button
// - Error messages
```

---

## Phase 6: Testing & Polish

### 6.1 Unit Tests

- [ ] Database tests
```typescript
// /packages/storage/tests/conversational.test.ts
describe('Conversational Mode DB', () => {
  test('creates conversation records');
  test('links PRD to conversation');
  test('creates epic from PRD');
  test('decomposes epic to tasks');
  test('tracks GitHub sync status');
});
```

- [ ] Service tests
```typescript
// /packages/api/tests/services/
- conversational.test.ts
- epicGenerator.test.ts
- taskDecomposer.test.ts
- githubSync.test.ts
- workStreamAnalyzer.test.ts
```

- [ ] Component tests
```typescript
// /packages/dashboard/tests/components/
- ConversationalMode.test.tsx
- EpicDetail.test.tsx
- TaskBreakdown.test.tsx
- GitHubSyncStatus.test.tsx
```

### 6.2 Integration Tests

- [ ] API integration tests
```typescript
// /packages/api/tests/integration/
describe('Conversational Flow', () => {
  test('complete conversation to PRD flow');
  test('PRD to Epic generation');
  test('Epic to Tasks decomposition');
  test('GitHub sync workflow');
});
```

### 6.3 E2E Tests

- [ ] Playwright tests
```typescript
// /packages/e2e/tests/conversational.spec.ts
test('Complete Conversational Mode workflow', async ({ page }) => {
  // 1. Start new conversational session
  // 2. Complete discovery conversation
  // 3. Generate PRD
  // 4. Create Epic
  // 5. Decompose to Tasks
  // 6. Sync to GitHub
});
```

### 6.4 Performance Optimization

- [ ] Query optimization
  - Add missing indexes
  - Optimize JSON queries
  - Add query result caching

- [ ] UI optimization
  - Implement virtual scrolling for long conversations
  - Add pagination for task lists
  - Optimize re-renders

### 6.5 Documentation

- [ ] User documentation
  - Conversational Mode guide
  - Epic management tutorial
  - GitHub integration setup
  - Video walkthrough

- [ ] Developer documentation
  - API endpoint reference
  - Database schema docs
  - Component architecture
  - Testing guide

---

## Implementation Checklist

### Week 1: Foundation
- [ ] Database schema updates
- [ ] Create new tables
- [ ] Add seed data
- [ ] Update TypeScript types
- [ ] Basic API endpoints

### Week 2: Conversational UI
- [ ] Build conversation components
- [ ] Implement streaming responses
- [ ] Add discovery flow
- [ ] Create PRD preview
- [ ] Quality validation

### Week 3: Epic System
- [ ] Epic generation logic
- [ ] Task decomposition
- [ ] Dependency detection
- [ ] Work stream analysis
- [ ] Epic UI components

### Week 4: GitHub & Polish
- [ ] GitHub sync service
- [ ] Webhook handlers
- [ ] Sync status UI
- [ ] Conflict resolution
- [ ] Testing suite
- [ ] Documentation

---

## API Endpoints Reference

### Conversational Mode
```
POST   /api/ideate/conversational/start
POST   /api/ideate/conversational/{id}/message
GET    /api/ideate/conversational/{id}/history
GET    /api/ideate/conversational/{id}/insights
POST   /api/ideate/conversational/{id}/generate-prd
POST   /api/ideate/conversational/{id}/validate
GET    /api/ideate/conversational/{id}/stream (SSE)
```

### Epic Management
```
POST   /api/epics/generate
GET    /api/epics
GET    /api/epics/{id}
PUT    /api/epics/{id}
DELETE /api/epics/{id}
POST   /api/epics/{id}/decompose
GET    /api/epics/{id}/tasks
POST   /api/epics/{id}/analyze-work
GET    /api/epics/{id}/progress
```

### Task Operations
```
POST   /api/tasks/bulk-create
PUT    /api/tasks/{id}/dependencies
PUT    /api/tasks/{id}/parallel-group
GET    /api/tasks/epic/{epicId}
POST   /api/tasks/detect-conflicts
```

### GitHub Sync
```
POST   /api/github/sync/epic/{id}
POST   /api/github/sync/tasks/{epicId}
GET    /api/github/sync/status/{projectId}
POST   /api/github/sync/pull/{projectId}
POST   /api/github/webhook
PUT    /api/github/resolve-conflict/{id}
```

### Discovery Questions
```
GET    /api/discovery/questions
GET    /api/discovery/questions/{category}
POST   /api/discovery/questions
PUT    /api/discovery/questions/{id}
DELETE /api/discovery/questions/{id}
```

---

## Migration Notes

Since no users are in production yet:
1. Directly modify `001_initial_schema.sql`
2. No data migration needed
3. Can make breaking changes safely
4. Test with fresh database

---

## Success Metrics

### Quantitative
- PRD creation time: Target 50% reduction
- Epic generation accuracy: >80% useful tasks
- Task dependency detection: >90% accuracy
- GitHub sync reliability: >99% success rate
- Parallel execution: 30% time savings

### Qualitative
- User satisfaction with conversational flow
- Quality of generated PRDs
- Usefulness of Epic breakdowns
- Team collaboration improvement
- Developer experience

---

## Risk Mitigation

### Technical Risks
1. **AI Response Quality**
   - Mitigation: Fine-tune prompts, add validation

2. **GitHub API Rate Limits**
   - Mitigation: Implement caching, batch operations

3. **Large Conversation Storage**
   - Mitigation: Implement message pruning, compression

### User Experience Risks
1. **Conversation Gets Stuck**
   - Mitigation: Add skip options, manual overrides

2. **Poor Epic Generation**
   - Mitigation: Allow manual editing, regeneration

3. **Task Decomposition Errors**
   - Mitigation: Human review step, edit capabilities

---

## Next Steps After Implementation

1. **User Testing**
   - Internal dogfooding
   - Beta user feedback
   - A/B testing vs other modes

2. **Enhancements**
   - Multi-agent task assignment
   - Advanced conflict detection
   - Real-time collaboration
   - AI model fine-tuning

3. **Integrations**
   - Jira sync option
   - Slack notifications
   - CI/CD triggers
   - Analytics dashboard

---

## References

- CCPM Source: https://github.com/automazeio/ccpm
- CCPM Docs: https://gitdocs1.s3.amazonaws.com/digests/automazeio-ccpm/
- Orkee Codebase: /Users/danziger/code/orkee/orkee-oss/
- SQLite Schema: /packages/storage/migrations/001_initial_schema.sql

---

*Last Updated: [Current Date]*
*Implementation Status: Planning Complete, Ready for Development*