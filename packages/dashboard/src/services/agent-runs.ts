// ABOUTME: Frontend service for agent run lifecycle operations
// ABOUTME: Handles CRUD, start/stop control, and SSE event subscriptions for autonomous agent runs

import { apiRequest } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';

// ── Types ──────────────────────────────────────────────────────────────────

export type AgentRunStatus = 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';

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

export interface AgentRun {
  id: string;
  projectId: string;
  prdId: string | null;
  prdJson: PrdJson;
  systemPrompt: string | null;
  status: AgentRunStatus;
  maxIterations: number;
  currentIteration: number;
  storiesTotal: number;
  storiesCompleted: number;
  totalCost: number;
  totalTokens: number;
  startedAt: string | null;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
  error: string | null;
  runnerPid: number | null;
}

export interface StartRunInput {
  project_id: string;
  prd_id?: string;
  prd_json: PrdJson;
  max_iterations?: number;
  system_prompt?: string;
}

// ── NDJSON Event Types (from agent runner) ─────────────────────────────────

export type RunEventType =
  | 'run_started' | 'run_completed' | 'run_failed'
  | 'iteration_started' | 'iteration_completed' | 'iteration_failed'
  | 'agent_text' | 'agent_tool'
  | 'branch_created' | 'pr_created' | 'pr_merged'
  | 'story_completed';

export interface RunEvent {
  type: RunEventType;
  [key: string]: unknown;
}

// ── API Response ───────────────────────────────────────────────────────────

interface ApiResponse<T> {
  success: boolean;
  data: T;
  error?: string;
}

// ── Service ────────────────────────────────────────────────────────────────

export async function startRun(input: StartRunInput): Promise<AgentRun> {
  const response = await apiRequest<ApiResponse<AgentRun>>('/api/agent-runs', {
    method: 'POST',
    body: JSON.stringify(input),
  });

  if (!response.success || !response.data?.success) {
    throw new Error(response.data?.error || response.error || 'Failed to start agent run');
  }

  return response.data.data;
}

export async function listRuns(
  projectId?: string,
  status?: AgentRunStatus,
  pagination?: PaginationParams,
): Promise<PaginatedResponse<AgentRun>> {
  let query = pagination ? buildPaginationQuery(pagination) : '';

  const params = new URLSearchParams(query ? query.replace('?', '') : '');
  if (projectId) params.set('project_id', projectId);
  if (status) params.set('status', status);

  const qs = params.toString();
  const url = `/api/agent-runs${qs ? `?${qs}` : ''}`;

  const response = await apiRequest<ApiResponse<PaginatedResponse<AgentRun>>>(url);

  if (!response.success || !response.data?.success) {
    throw new Error(response.data?.error || response.error || 'Failed to list agent runs');
  }

  return response.data.data;
}

export async function getRun(runId: string): Promise<AgentRun> {
  const response = await apiRequest<ApiResponse<AgentRun>>(`/api/agent-runs/${runId}`);

  if (!response.success || !response.data?.success) {
    throw new Error(response.data?.error || response.error || 'Failed to get agent run');
  }

  return response.data.data;
}

export async function stopRun(runId: string): Promise<void> {
  const response = await apiRequest<ApiResponse<null>>(`/api/agent-runs/${runId}/stop`, {
    method: 'POST',
  });

  if (!response.success || !response.data?.success) {
    throw new Error(response.data?.error || response.error || 'Failed to stop agent run');
  }
}

export async function deleteRun(runId: string): Promise<void> {
  const response = await apiRequest<ApiResponse<null>>(`/api/agent-runs/${runId}`, {
    method: 'DELETE',
  });

  if (!response.success || !response.data?.success) {
    throw new Error(response.data?.error || response.error || 'Failed to delete agent run');
  }
}
