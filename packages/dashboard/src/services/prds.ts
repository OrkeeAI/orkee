// ABOUTME: PRD (Product Requirements Document) service layer for API integration
// ABOUTME: Handles CRUD operations, AI analysis, and spec synchronization for PRDs
import { apiClient } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';

export type PRDStatus = 'draft' | 'approved' | 'superseded';
export type PRDSource = 'manual' | 'generated' | 'synced';

export interface PRD {
  id: string;
  projectId: string;
  title: string;
  contentMarkdown: string;
  version: number;
  status: PRDStatus;
  source: PRDSource;
  createdAt: string;
  updatedAt: string;
  createdBy?: string;
}

export interface PRDCreateInput {
  title: string;
  contentMarkdown: string;
  status?: PRDStatus;
  source?: PRDSource;
  createdBy?: string;
}

export interface PRDUpdateInput {
  title?: string;
  contentMarkdown?: string;
  status?: PRDStatus;
  source?: PRDSource;
}

export interface SpecCapability {
  id: string;
  name: string;
  purpose: string;
  requirements: SpecRequirement[];
}

export interface SpecRequirement {
  name: string;
  content: string;
  scenarios: SpecScenario[];
}

export interface SpecScenario {
  name: string;
  whenClause: string;
  thenClause: string;
  andClauses: string[];
}

export interface TaskSuggestion {
  title: string;
  description: string;
  capabilityId: string;
  requirementName: string;
  complexity: number;
  estimatedHours?: number;
}

export interface PRDAnalysisResult {
  summary: string;
  capabilities: SpecCapability[];
  suggestedTasks: TaskSuggestion[];
  dependencies?: string[];
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class PRDsService {
  async listPRDs(projectId: string, pagination?: PaginationParams): Promise<PaginatedResponse<PRD>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiClient.get<ApiResponse<PaginatedResponse<PRD>>>(`/api/projects/${projectId}/prds${query}`);

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch PRDs');
    }

    return response.data.data!;
  }

  async getPRDCapabilities(projectId: string, prdId: string, pagination?: PaginationParams): Promise<PaginatedResponse<SpecCapability>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiClient.get<ApiResponse<PaginatedResponse<SpecCapability>>>(`/api/projects/${projectId}/prds/${prdId}/capabilities${query}`);

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch PRD capabilities');
    }

    return response.data.data!;
  }

  async getPRD(projectId: string, prdId: string): Promise<PRD | null> {
    const response = await apiClient.get<ApiResponse<PRD>>(`/api/projects/${projectId}/prds/${prdId}`);

    if (response.error) {
      throw new Error(response.error);
    }

    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch PRD');
    }

    return response.data.data;
  }

  async createPRD(projectId: string, prdData: PRDCreateInput): Promise<PRD> {
    const response = await apiClient.post<ApiResponse<PRD>>(
      `/api/projects/${projectId}/prds`,
      prdData
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to create PRD');
    }

    if (!response.data.data) {
      throw new Error('No PRD data returned');
    }

    return response.data.data;
  }

  async updatePRD(projectId: string, prdId: string, updates: PRDUpdateInput): Promise<PRD> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<PRD>>(
      `/api/projects/${projectId}/prds/${prdId}`,
      {
        method: 'PUT',
        body: JSON.stringify(updates),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update PRD');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update PRD');
    }

    if (!result.data.data) {
      throw new Error('No PRD data returned');
    }

    return result.data.data;
  }

  async deletePRD(projectId: string, prdId: string): Promise<boolean> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<string>>(
      `/api/projects/${projectId}/prds/${prdId}`,
      {
        method: 'DELETE',
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to delete PRD');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to delete PRD');
    }

    return true;
  }

  async analyzePRD(projectId: string, prdId: string): Promise<PRDAnalysisResult> {
    // Fetch the PRD content first
    const prd = await this.getPRD(projectId, prdId);
    if (!prd) {
      throw new Error(`PRD with ID ${prdId} not found`);
    }

    // Use real AI service to analyze the PRD
    const { aiSpecService } = await import('@/lib/ai/services');
    const aiResult = await aiSpecService.analyzePRD(prd.contentMarkdown);

    // Transform AI schema format to PRD service format
    // AI schema uses: {when, then, and}
    // PRD service expects: {whenClause, thenClause, andClauses}
    const capabilities: SpecCapability[] = aiResult.data.capabilities.map((cap) => ({
      id: cap.id,
      name: cap.name,
      purpose: cap.purpose,
      requirements: cap.requirements.map((req) => ({
        name: req.name,
        content: req.content,
        scenarios: req.scenarios.map((scenario) => ({
          name: scenario.name,
          whenClause: scenario.when,
          thenClause: scenario.then,
          andClauses: scenario.and || [],
        })),
      })),
    }));

    return {
      summary: aiResult.data.summary,
      capabilities,
      suggestedTasks: aiResult.data.suggestedTasks,
      dependencies: aiResult.data.dependencies,
    };
  }

  async syncSpecsToPRD(projectId: string, prdId: string): Promise<PRD> {
    const response = await apiClient.post<ApiResponse<PRD>>(
      `/api/projects/${projectId}/prds/${prdId}/sync`,
      {}
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to sync specs to PRD');
    }

    if (!response.data.data) {
      throw new Error('No PRD data returned');
    }

    return response.data.data;
  }
}

export const prdsService = new PRDsService();
