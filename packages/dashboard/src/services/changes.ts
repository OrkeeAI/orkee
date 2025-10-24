// ABOUTME: OpenSpec changes service layer for API integration
// ABOUTME: Handles change proposals, validation, and archiving for spec-driven development
import { apiClient } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';
import type { ValidationError } from './prds';

export type ChangeStatus = 'draft' | 'review' | 'approved' | 'implementing' | 'completed' | 'archived';
export type DeltaType = 'added' | 'modified' | 'removed' | 'renamed';

export interface SpecChange {
  id: string;
  projectId: string;
  prdId?: string;
  proposalMarkdown: string;
  tasksMarkdown: string;
  designMarkdown?: string;
  status: ChangeStatus;
  verbPrefix?: string;
  changeNumber?: number;
  validationStatus?: 'pending' | 'valid' | 'invalid';
  validationErrors?: ValidationError[];
  tasksCompletionPercentage?: number;
  tasksParsedAt?: string;
  tasksTotalCount?: number;
  tasksCompletedCount?: number;
  createdBy: string;
  createdAt: string;
  updatedAt: string;
  approvedBy?: string;
  approvedAt?: string;
  archivedAt?: string;
}

export interface SpecDelta {
  id: string;
  changeId: string;
  capabilityId?: string;
  capabilityName: string;
  deltaType: DeltaType;
  deltaMarkdown: string;
  createdAt: string;
}

export interface ChangeWithDeltas extends SpecChange {
  deltas: SpecDelta[];
}

export interface ChangeListItem {
  id: string;
  projectId: string;
  prdId?: string;
  status: ChangeStatus;
  verbPrefix?: string;
  changeNumber?: number;
  validationStatus?: 'pending' | 'valid' | 'invalid';
  createdBy: string;
  createdAt: string;
  deltaCount: number;
}

export interface ValidationResult {
  changeId: string;
  isValid: boolean;
  errors: ValidationError[];
}

export interface ArchiveResult {
  changeId: string;
  success: boolean;
  appliedDeltas: number;
  createdCapabilities: string[];
}

export interface ChangeTask {
  id: string;
  changeId: string;
  taskNumber: string;
  taskText: string;
  isCompleted: boolean;
  completedBy?: string;
  completedAt?: string;
  displayOrder: number;
  parentNumber?: string;
  createdAt: string;
  updatedAt: string;
}

export interface TaskStats {
  total: number;
  completed: number;
  pending: number;
  percentage: number;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class ChangesService {
  async listChanges(
    projectId: string,
    status?: ChangeStatus,
    pagination?: PaginationParams
  ): Promise<PaginatedResponse<ChangeListItem>> {
    const params = new URLSearchParams();
    if (status) params.set('status', status);
    if (pagination) {
      const paginationQuery = buildPaginationQuery(pagination);
      params.set('page', String(pagination.page || 1));
      params.set('limit', String(pagination.limit || 20));
    }

    const query = params.toString() ? `?${params.toString()}` : '';
    const response = await apiClient.get<ApiResponse<PaginatedResponse<ChangeListItem>>>(
      `/api/projects/${projectId}/changes${query}`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch changes');
    }

    return response.data.data!;
  }

  async getChange(projectId: string, changeId: string): Promise<ChangeWithDeltas | null> {
    const response = await apiClient.get<ApiResponse<ChangeWithDeltas>>(
      `/api/projects/${projectId}/changes/${changeId}`
    );

    if (response.error) {
      throw new Error(response.error);
    }

    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch change');
    }

    return response.data.data;
  }

  async validateChange(projectId: string, changeId: string, strict: boolean = false): Promise<ValidationResult> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<ValidationResult>>(
      `/api/projects/${projectId}/changes/${changeId}/validate`,
      {
        method: 'POST',
        body: JSON.stringify({ strict }),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to validate change');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to validate change');
    }

    if (!result.data.data) {
      throw new Error('No validation result returned');
    }

    return result.data.data;
  }

  async archiveChange(
    projectId: string,
    changeId: string,
    applySpecs: boolean = true
  ): Promise<ArchiveResult> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<ArchiveResult>>(
      `/api/projects/${projectId}/changes/${changeId}/archive`,
      {
        method: 'POST',
        body: JSON.stringify({ applySpecs }),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to archive change');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to archive change');
    }

    if (!result.data.data) {
      throw new Error('No archive result returned');
    }

    return result.data.data;
  }

  async getDeltas(projectId: string, changeId: string): Promise<SpecDelta[]> {
    const response = await apiClient.get<ApiResponse<SpecDelta[]>>(
      `/api/projects/${projectId}/changes/${changeId}/deltas`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch deltas');
    }

    return response.data.data || [];
  }

  async updateChangeStatus(
    projectId: string,
    changeId: string,
    status: ChangeStatus,
    metadata?: {
      approvedBy?: string;
      notes?: string;
    }
  ): Promise<ChangeWithDeltas> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<ChangeWithDeltas>>(
      `/api/projects/${projectId}/changes/${changeId}/status`,
      {
        method: 'PUT',
        body: JSON.stringify({ status, ...metadata }),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update change status');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update change status');
    }

    if (!result.data.data) {
      throw new Error('No change data returned');
    }

    return result.data.data;
  }

  async getChangeTasks(projectId: string, changeId: string): Promise<ChangeTask[]> {
    const response = await apiClient.get<ApiResponse<ChangeTask[]>>(
      `/api/projects/${projectId}/changes/${changeId}/tasks`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch tasks');
    }

    return response.data.data || [];
  }

  async updateTask(
    projectId: string,
    changeId: string,
    taskId: string,
    isCompleted: boolean,
    completedBy?: string
  ): Promise<ChangeTask> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<ChangeTask>>(
      `/api/projects/${projectId}/changes/${changeId}/tasks/${taskId}`,
      {
        method: 'PUT',
        body: JSON.stringify({ isCompleted, completedBy }),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update task');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update task');
    }

    if (!result.data.data) {
      throw new Error('No task data returned');
    }

    return result.data.data;
  }

  async bulkUpdateTasks(
    projectId: string,
    changeId: string,
    tasks: Array<{ taskId: string; isCompleted: boolean; completedBy?: string }>
  ): Promise<ChangeTask[]> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<ChangeTask[]>>(
      `/api/projects/${projectId}/changes/${changeId}/tasks/bulk`,
      {
        method: 'PUT',
        body: JSON.stringify({ tasks }),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to bulk update tasks');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to bulk update tasks');
    }

    if (!result.data.data) {
      throw new Error('No task data returned');
    }

    return result.data.data;
  }

  async parseChangeTasks(projectId: string, changeId: string): Promise<ChangeTask[]> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<ChangeTask[]>>(
      `/api/projects/${projectId}/changes/${changeId}/tasks/parse`,
      {
        method: 'POST',
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to parse tasks');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to parse tasks');
    }

    if (!result.data.data) {
      throw new Error('No task data returned');
    }

    return result.data.data;
  }

  calculateTaskStats(tasks: ChangeTask[]): TaskStats {
    const total = tasks.length;
    const completed = tasks.filter((t) => t.isCompleted).length;
    const pending = total - completed;
    const percentage = total > 0 ? Math.round((completed / total) * 100) : 0;

    return { total, completed, pending, percentage };
  }
}

export const changesService = new ChangesService();
