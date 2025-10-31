// ABOUTME: Epic management service layer for API integration
// ABOUTME: Handles CRUD operations, generation, task decomposition, and progress tracking for Epics

import { apiClient } from './api';

export type EpicStatus = 'draft' | 'ready' | 'in_progress' | 'blocked' | 'completed' | 'cancelled';
export type EpicComplexity = 'low' | 'medium' | 'high' | 'very_high';
export type EstimatedEffort = 'days' | 'weeks' | 'months';

export interface ArchitectureDecision {
  decision: string;
  rationale: string;
  alternatives?: string[];
  tradeoffs?: string;
}

export interface ExternalDependency {
  name: string;
  type: string; // 'library', 'service', 'api', etc.
  version?: string;
  reason: string;
}

export interface SuccessCriterion {
  criterion: string;
  measurable: boolean;
  target?: string;
}

export interface Epic {
  id: string;
  projectId: string;
  prdId: string;
  name: string;

  // Epic content
  overviewMarkdown: string;
  architectureDecisions?: ArchitectureDecision[];
  technicalApproach: string;
  implementationStrategy?: string;
  dependencies?: ExternalDependency[];
  successCriteria?: SuccessCriterion[];

  // Task breakdown metadata
  taskCategories?: string[];
  estimatedEffort?: EstimatedEffort;
  complexity?: EpicComplexity;

  // Status tracking
  status: EpicStatus;
  progressPercentage: number;

  // GitHub integration
  githubIssueNumber?: number;
  githubIssueUrl?: string;
  githubSyncedAt?: string;

  // Timestamps
  createdAt: string;
  updatedAt: string;
  startedAt?: string;
  completedAt?: string;
}

export interface CreateEpicInput {
  prdId: string;
  name: string;
  overviewMarkdown: string;
  architectureDecisions?: ArchitectureDecision[];
  technicalApproach: string;
  implementationStrategy?: string;
  dependencies?: ExternalDependency[];
  successCriteria?: SuccessCriterion[];
  taskCategories?: string[];
  estimatedEffort?: EstimatedEffort;
  complexity?: EpicComplexity;
}

export interface UpdateEpicInput {
  name?: string;
  overviewMarkdown?: string;
  architectureDecisions?: ArchitectureDecision[];
  technicalApproach?: string;
  implementationStrategy?: string;
  dependencies?: ExternalDependency[];
  successCriteria?: SuccessCriterion[];
  taskCategories?: string[];
  estimatedEffort?: EstimatedEffort;
  complexity?: EpicComplexity;
  status?: EpicStatus;
  progressPercentage?: number;
}

export interface GenerateEpicRequest {
  prdId: string;
  includeTaskBreakdown?: boolean;
}

export interface GenerateEpicResponse {
  epicId: string;
  tasksCreated?: number;
}

export interface WorkStream {
  name: string;
  description: string;
  tasks: string[]; // task IDs
  filePatterns?: string[];
}

export interface WorkAnalysis {
  id: string;
  epicId: string;
  parallelStreams: WorkStream[];
  filePatterns?: Record<string, string[]>;
  dependencyGraph: {
    nodes: Array<{ id: string; label: string }>;
    edges: Array<{ from: string; to: string; type?: string }>;
  };
  conflictAnalysis?: {
    conflicts: Array<{ task1: string; task2: string; reason: string }>;
  };
  parallelizationStrategy?: string;
  analyzedAt: string;
  isCurrent: boolean;
  analysisVersion: number;
  confidenceScore?: number;
}

export type SizeEstimate = 'XS' | 'S' | 'M' | 'L' | 'XL';
export type TaskType = 'task' | 'subtask';

export interface TaskTemplate {
  title: string;
  description?: string;
  technicalDetails?: string;
  sizeEstimate?: SizeEstimate;
  effortHours?: number;
  dependsOnTitles?: string[];
  acceptanceCriteria?: string;
  testStrategy?: string;
}

export interface TaskCategory {
  name: string;
  description: string;
  tasks: TaskTemplate[];
}

export interface DecomposeEpicInput {
  epicId: string;
  taskCategories: TaskCategory[];
}

export interface ParallelGroup {
  id: string;
  name: string;
  taskIds: string[];
}

export interface DecompositionResult {
  tasks: any[]; // Task type from tasks service
  dependencyGraph: {
    nodes: Array<{ id: string; label: string }>;
    edges: Array<{ from: string; to: string; type?: string }>;
  };
  parallelGroups: ParallelGroup[];
  conflicts: Array<{ task1: string; task2: string; reason: string }>;
}

// GitHub Sync Types
export type GitHubSyncStatus = 'pending' | 'syncing' | 'synced' | 'failed' | 'conflict';
export type GitHubSyncDirection = 'local_to_github' | 'github_to_local' | 'bidirectional';
export type GitHubEntityType = 'epic' | 'task' | 'comment' | 'status';

export interface GitHubSyncResult {
  issue_number: number;
  issue_url: string;
  synced_at: string;
}

export interface GitHubSyncRecord {
  id: string;
  project_id: string;
  entity_type: GitHubEntityType;
  entity_id: string;
  github_issue_number?: number;
  github_issue_url?: string;
  sync_status: GitHubSyncStatus;
  sync_direction?: GitHubSyncDirection;
  last_synced_at?: string;
  last_sync_hash?: string;
  last_sync_error?: string;
  retry_count: number;
  created_at: string;
  updated_at: string;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class EpicsService {
  async listEpics(projectId: string): Promise<Epic[]> {
    const response = await apiClient.get<ApiResponse<Epic[]>>(`/api/projects/${projectId}/epics`);

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch epics');
    }

    return response.data.data || [];
  }

  async getEpic(projectId: string, epicId: string): Promise<Epic | null> {
    const response = await apiClient.get<ApiResponse<Epic>>(`/api/projects/${projectId}/epics/${epicId}`);

    if (response.error) {
      throw new Error(response.error);
    }

    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch epic');
    }

    return response.data.data;
  }

  async getEpicsByPRD(projectId: string, prdId: string): Promise<Epic[]> {
    const response = await apiClient.get<ApiResponse<Epic[]>>(`/api/projects/${projectId}/prds/${prdId}/epics`);

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch epics for PRD');
    }

    return response.data.data || [];
  }

  async createEpic(projectId: string, epicData: CreateEpicInput): Promise<Epic> {
    const response = await apiClient.post<ApiResponse<Epic>>(
      `/api/projects/${projectId}/epics`,
      epicData
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to create epic');
    }

    if (!response.data.data) {
      throw new Error('No epic data returned');
    }

    return response.data.data;
  }

  async updateEpic(projectId: string, epicId: string, updates: UpdateEpicInput): Promise<Epic> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<Epic>>(
      `/api/projects/${projectId}/epics/${epicId}`,
      {
        method: 'PUT',
        body: JSON.stringify(updates),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update epic');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update epic');
    }

    if (!result.data.data) {
      throw new Error('No epic data returned');
    }

    return result.data.data;
  }

  async deleteEpic(projectId: string, epicId: string): Promise<boolean> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<string>>(
      `/api/projects/${projectId}/epics/${epicId}`,
      {
        method: 'DELETE',
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to delete epic');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to delete epic');
    }

    return true;
  }

  async generateFromPRD(projectId: string, request: GenerateEpicRequest): Promise<GenerateEpicResponse> {
    const response = await apiClient.post<ApiResponse<GenerateEpicResponse>>(
      `/api/projects/${projectId}/epics/generate`,
      request
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to generate epic from PRD');
    }

    if (!response.data.data) {
      throw new Error('No generation response returned');
    }

    return response.data.data;
  }

  async getEpicTasks(projectId: string, epicId: string): Promise<any[]> {
    const response = await apiClient.get<ApiResponse<any[]>>(`/api/projects/${projectId}/epics/${epicId}/tasks`);

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch epic tasks');
    }

    return response.data.data || [];
  }

  async analyzeWorkStreams(projectId: string, epicId: string): Promise<WorkAnalysis> {
    const response = await apiClient.post<ApiResponse<WorkAnalysis>>(
      `/api/projects/${projectId}/epics/${epicId}/analyze-work`,
      {}
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to analyze work streams');
    }

    if (!response.data.data) {
      throw new Error('No work analysis returned');
    }

    return response.data.data;
  }

  async calculateProgress(projectId: string, epicId: string): Promise<number> {
    const response = await apiClient.get<ApiResponse<{ progress: number }>>(
      `/api/projects/${projectId}/epics/${epicId}/progress`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to calculate progress');
    }

    return response.data.data?.progress || 0;
  }

  async decomposeEpic(projectId: string, epicId: string, input: DecomposeEpicInput): Promise<DecompositionResult> {
    const response = await apiClient.post<ApiResponse<DecompositionResult>>(
      `/api/projects/${projectId}/epics/${epicId}/decompose`,
      input
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to decompose epic');
    }

    if (!response.data.data) {
      throw new Error('No decomposition result returned');
    }

    return response.data.data;
  }

  // GitHub Sync Operations
  async syncEpicToGitHub(epicId: string, createNew: boolean = false): Promise<GitHubSyncResult> {
    const response = await apiClient.post<ApiResponse<GitHubSyncResult>>(
      `/api/github/sync/epic/${epicId}`,
      { create_new: createNew }
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to sync epic to GitHub');
    }

    if (!response.data.data) {
      throw new Error('No sync result returned');
    }

    return response.data.data;
  }

  async syncTasksToGitHub(epicId: string): Promise<GitHubSyncResult[]> {
    const response = await apiClient.post<ApiResponse<{ results: GitHubSyncResult[] }>>(
      `/api/github/sync/tasks/${epicId}`,
      {}
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to sync tasks to GitHub');
    }

    if (!response.data.data) {
      throw new Error('No sync results returned');
    }

    return response.data.data.results;
  }

  async getGitHubSyncStatus(projectId: string): Promise<GitHubSyncRecord[]> {
    const response = await apiClient.get<ApiResponse<{ syncs: GitHubSyncRecord[] }>>(
      `/api/github/sync/status/${projectId}`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to get sync status');
    }

    return response.data.data?.syncs || [];
  }
}

export const epicsService = new EpicsService();
