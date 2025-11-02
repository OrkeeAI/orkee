// ABOUTME: Frontend service for task operations
// ABOUTME: Handles CRUD operations for tasks with pagination support

import { apiRequest } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';

export type TaskStatus = 'pending' | 'in-progress' | 'review' | 'done' | 'deferred' | 'cancelled';
export type TaskPriority = 'low' | 'medium' | 'high';

export interface Task {
  id: string;
  projectId: string;
  title: string;
  description: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  createdAt: string;
  updatedAt: string;
  completedAt: string | null;
  tagId: string | null;
  parentTaskId: string | null;
  complexityScore: number;
  specRequirementId: string | null;
}

export interface TaskCreateInput {
  title: string;
  description?: string;
  status?: TaskStatus;
  priority?: TaskPriority;
  tagId?: string;
  parentTaskId?: string;
  complexityScore?: number;
  specRequirementId?: string;
}

export interface TaskUpdateInput {
  title?: string;
  description?: string;
  status?: TaskStatus;
  priority?: TaskPriority;
  tagId?: string;
  parentTaskId?: string;
  complexityScore?: number;
  specRequirementId?: string;
  completedAt?: string;
}

// TDD Task Execution Types
export interface TaskStep {
  stepNumber: number;
  action: string;
  testCommand?: string;
  expectedOutput: string;
  estimatedMinutes: number;
}

export interface FileReference {
  path: string;
  operation: 'create' | 'modify' | 'delete';
  reason: string;
}

export interface ValidationEntry {
  timestamp: string;
  entryType: 'progress' | 'issue' | 'decision' | 'checkpoint';
  content: string;
  author: string;
}

export interface ExecutionCheckpoint {
  afterTaskId: string;
  checkpointType: 'review' | 'test' | 'integration' | 'approval';
  message: string;
  requiredValidation: string[];
}

export interface TaskExecutionSteps {
  taskId: string;
  steps: TaskStep[];
  testStrategy: string;
  acceptanceCriteria: string[];
  relevantFiles: FileReference[];
  similarImplementations: string[];
}

export interface ProgressUpdate {
  content: string;
  entryType: ValidationEntry['entryType'];
  author?: string;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class TasksService {
  async listTasks(
    projectId: string,
    pagination?: PaginationParams
  ): Promise<PaginatedResponse<Task>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<Task>>>(
      `/api/projects/${projectId}/tasks${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch tasks');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch tasks');
    }

    return response.data.data!;
  }

  async getTask(projectId: string, taskId: string): Promise<Task> {
    const response = await apiRequest<ApiResponse<Task>>(
      `/api/projects/${projectId}/tasks/${taskId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch task');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to fetch task');
    }

    return response.data.data!;
  }

  async createTask(projectId: string, input: TaskCreateInput): Promise<Task> {
    const response = await apiRequest<ApiResponse<Task>>(
      `/api/projects/${projectId}/tasks`,
      {
        method: 'POST',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to create task');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to create task');
    }

    return response.data.data!;
  }

  async updateTask(
    projectId: string,
    taskId: string,
    input: TaskUpdateInput
  ): Promise<Task> {
    const response = await apiRequest<ApiResponse<Task>>(
      `/api/projects/${projectId}/tasks/${taskId}`,
      {
        method: 'PUT',
        body: JSON.stringify(input),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to update task');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to update task');
    }

    return response.data.data!;
  }

  async deleteTask(projectId: string, taskId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<string>>(
      `/api/projects/${projectId}/tasks/${taskId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to delete task');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to delete task');
    }
  }

  // Task Execution Tracking Operations
  async generateExecutionSteps(taskId: string): Promise<TaskExecutionSteps> {
    const response = await apiRequest<ApiResponse<TaskExecutionSteps>>(
      `/api/tasks/${taskId}/generate-steps`,
      {
        method: 'POST',
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to generate execution steps');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to generate execution steps');
    }

    return response.data.data!;
  }

  async appendProgress(taskId: string, update: ProgressUpdate): Promise<ValidationEntry> {
    const response = await apiRequest<ApiResponse<ValidationEntry>>(
      `/api/tasks/${taskId}/append-progress`,
      {
        method: 'POST',
        body: JSON.stringify(update),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to append progress');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to append progress');
    }

    return response.data.data!;
  }

  async getValidationHistory(taskId: string): Promise<ValidationEntry[]> {
    const response = await apiRequest<ApiResponse<{ history: ValidationEntry[] }>>(
      `/api/tasks/${taskId}/validation-history`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get validation history');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get validation history');
    }

    return response.data.data?.history || [];
  }

  async getTaskCheckpoints(taskId: string): Promise<ExecutionCheckpoint[]> {
    const response = await apiRequest<ApiResponse<{ checkpoints: ExecutionCheckpoint[] }>>(
      `/api/tasks/${taskId}/checkpoints`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get task checkpoints');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get task checkpoints');
    }

    return response.data.data?.checkpoints || [];
  }

  async getEpicCheckpoints(epicId: string): Promise<ExecutionCheckpoint[]> {
    const response = await apiRequest<ApiResponse<{ checkpoints: ExecutionCheckpoint[] }>>(
      `/api/epics/${epicId}/checkpoints`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get epic checkpoints');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to get epic checkpoints');
    }

    return response.data.data?.checkpoints || [];
  }
}

export const tasksService = new TasksService();
