// ABOUTME: OpenSpec changes service layer for API integration
// ABOUTME: Handles change proposals, validation, and archiving for spec-driven development
import { apiClient } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';
import type { ValidationError } from './prds';

export type ChangeStatus = 'proposal' | 'in_review' | 'approved' | 'in_progress' | 'archived';
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
  createdBy: string;
  createdAt: string;
  updatedAt: string;
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
}

export const changesService = new ChangesService();
