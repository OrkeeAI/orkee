// ABOUTME: Type definitions for the agent runner's event protocol and PRD schema.
// ABOUTME: Shared between the runner process and the Rust backend that parses its output.

// ── PRD Schema ─────────────────────────────────────────────────────────────

export interface UserStory {
  id: string;
  title: string;
  description: string;
  acceptanceCriteria: string[];
  epic: string;
  priority: number;
  passes: boolean;
  notes: string;
}

export interface PrdJson {
  project: string;
  sourcePrd: string;
  branchName: string;
  description: string;
  userStories: UserStory[];
}

// ── Run Configuration ──────────────────────────────────────────────────────

export interface RunConfig {
  runId: string;
  projectDir: string;
  prdPath: string;
  maxIterations: number;
  systemPromptPath?: string;
}

// ── NDJSON Event Protocol ──────────────────────────────────────────────────
// Each line of stdout is one JSON-serialized RunEvent.
// The Rust backend reads these line-by-line and broadcasts via SSE.

export interface RunStartedEvent {
  type: "run_started";
  run_id: string;
  total_stories: number;
  completed_stories: number;
}

export interface RunCompletedEvent {
  type: "run_completed";
  run_id: string;
  total_cost: number;
  stories_completed: number;
  duration_secs: number;
}

export interface RunFailedEvent {
  type: "run_failed";
  run_id: string;
  error: string;
}

export interface IterationStartedEvent {
  type: "iteration_started";
  iteration: number;
  story_id: string;
  story_title: string;
}

export interface IterationCompletedEvent {
  type: "iteration_completed";
  iteration: number;
  story_id: string;
  cost: number;
  duration_secs: number;
  tools: Record<string, number>;
}

export interface IterationFailedEvent {
  type: "iteration_failed";
  iteration: number;
  story_id: string;
  error: string;
}

export interface AgentTextEvent {
  type: "agent_text";
  text: string;
}

export interface AgentToolEvent {
  type: "agent_tool";
  tool: string;
  detail: string;
}

export interface BranchCreatedEvent {
  type: "branch_created";
  branch: string;
}

export interface PrCreatedEvent {
  type: "pr_created";
  pr_number: number;
  pr_url: string;
}

export interface PrMergedEvent {
  type: "pr_merged";
  pr_number: number;
}

export interface StoryCompletedEvent {
  type: "story_completed";
  story_id: string;
  passed: number;
  total: number;
}

export type RunEvent =
  | RunStartedEvent
  | RunCompletedEvent
  | RunFailedEvent
  | IterationStartedEvent
  | IterationCompletedEvent
  | IterationFailedEvent
  | AgentTextEvent
  | AgentToolEvent
  | BranchCreatedEvent
  | PrCreatedEvent
  | PrMergedEvent
  | StoryCompletedEvent;
