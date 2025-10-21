// ABOUTME: Task-spec linking service layer for API integration
// ABOUTME: Handles task-requirement linking, validation, orphan detection, and task generation
import { apiClient } from './api';
import type { SpecRequirement } from './specs';

export interface TaskSpecLink {
  taskId: string;
  requirementId: string;
  validationStatus?: 'pending' | 'passed' | 'failed';
  validationResult?: string;
  createdAt: string;
}

export interface TaskSpecLinkCreateInput {
  requirementId: string;
}

export interface TaskValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  passedScenarios: number;
  totalScenarios: number;
}

export interface SuggestSpecResponse {
  suggestedRequirement?: string;
  suggestedContent?: string;
  confidence: number;
  note: string;
}

export interface GenerateTasksInput {
  capabilityId: string;
  tagId: string;
}

export interface GenerateTasksResponse {
  taskIds: string[];
  count: number;
}

export interface OrphanTask {
  id: string;
  title: string;
  status: string;
  priority: string;
  createdAt: string;
}

export interface OrphanTasksResponse {
  orphanTasks: OrphanTask[];
  count: number;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class TaskSpecLinksService {
  async linkTaskToRequirement(
    taskId: string,
    linkData: TaskSpecLinkCreateInput
  ): Promise<boolean> {
    const response = await apiClient.post<ApiResponse<boolean>>(
      `/api/tasks/${taskId}/link-spec`,
      linkData
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to link task to requirement');
    }

    return response.data.data ?? true;
  }

  async getTaskSpecLinks(taskId: string): Promise<SpecRequirement[]> {
    const response = await apiClient.get<ApiResponse<SpecRequirement[]>>(
      `/api/tasks/${taskId}/spec-links`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch task spec links');
    }

    return response.data.data || [];
  }

  async validateTaskAgainstSpec(taskId: string): Promise<TaskValidationResult> {
    const response = await apiClient.post<ApiResponse<TaskValidationResult>>(
      `/api/tasks/${taskId}/validate-spec`,
      {}
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to validate task');
    }

    if (!response.data.data) {
      throw new Error('No validation result returned');
    }

    return response.data.data;
  }

  async suggestSpecFromTask(taskId: string): Promise<SuggestSpecResponse> {
    const response = await apiClient.post<ApiResponse<SuggestSpecResponse>>(
      `/api/tasks/${taskId}/suggest-spec`,
      {}
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to get spec suggestion');
    }

    if (!response.data.data) {
      throw new Error('No suggestion returned');
    }

    return response.data.data;
  }

  async generateTasksFromSpec(
    projectId: string,
    taskData: GenerateTasksInput
  ): Promise<GenerateTasksResponse> {
    const response = await apiClient.post<ApiResponse<GenerateTasksResponse>>(
      `/api/${projectId}/tasks/generate-from-spec`,
      taskData
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to generate tasks');
    }

    if (!response.data.data) {
      throw new Error('No task generation result returned');
    }

    return response.data.data;
  }

  async getOrphanTasks(projectId: string): Promise<OrphanTasksResponse> {
    const response = await apiClient.get<ApiResponse<OrphanTasksResponse>>(
      `/api/${projectId}/tasks/orphans`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch orphan tasks');
    }

    if (!response.data.data) {
      throw new Error('No orphan tasks result returned');
    }

    return response.data.data;
  }
}

export const taskSpecLinksService = new TaskSpecLinksService();
