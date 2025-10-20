// ABOUTME: Spec capability service layer for API integration
// ABOUTME: Handles CRUD operations for spec capabilities, requirements, and scenarios
import { apiClient } from './api';

export type SpecStatus = 'active' | 'deprecated' | 'archived';

export interface SpecScenario {
  id?: string;
  name: string;
  whenClause: string;
  thenClause: string;
  andClauses: string[];
  position?: number;
}

export interface SpecRequirement {
  id?: string;
  name: string;
  content: string;
  scenarios: SpecScenario[];
  position?: number;
}

export interface SpecCapability {
  id: string;
  projectId: string;
  prdId?: string;
  name: string;
  purpose: string;
  specMarkdown: string;
  designMarkdown?: string;
  requirements: SpecRequirement[];
  requirementCount: number;
  version: number;
  status: SpecStatus;
  createdAt: string;
  updatedAt: string;
}

export interface SpecCapabilityCreateInput {
  prdId?: string;
  name: string;
  purpose: string;
  requirements: SpecRequirement[];
  designMarkdown?: string;
  status?: SpecStatus;
}

export interface SpecCapabilityUpdateInput {
  name?: string;
  purpose?: string;
  requirements?: SpecRequirement[];
  designMarkdown?: string;
  status?: SpecStatus;
}

export interface SpecValidationResult {
  valid: boolean;
  errors: string[];
  warnings: string[];
  statistics: {
    capabilityCount: number;
    requirementCount: number;
    scenarioCount: number;
  };
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class SpecsService {
  async listSpecs(projectId: string): Promise<SpecCapability[]> {
    const response = await apiClient.get<ApiResponse<SpecCapability[]>>(
      `/api/projects/${projectId}/specs`
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch specs');
    }

    return response.data.data || [];
  }

  async getSpec(projectId: string, specId: string): Promise<SpecCapability | null> {
    const response = await apiClient.get<ApiResponse<SpecCapability>>(
      `/api/projects/${projectId}/specs/${specId}`
    );

    if (response.error) {
      throw new Error(response.error);
    }

    if (!response.data?.success) {
      if (response.data?.error?.includes('not found')) {
        return null;
      }
      throw new Error(response.data?.error || 'Failed to fetch spec');
    }

    return response.data.data;
  }

  async createSpec(
    projectId: string,
    specData: SpecCapabilityCreateInput
  ): Promise<SpecCapability> {
    const response = await apiClient.post<ApiResponse<SpecCapability>>(
      `/api/projects/${projectId}/specs`,
      specData
    );

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to create spec');
    }

    if (!response.data.data) {
      throw new Error('No spec data returned');
    }

    return response.data.data;
  }

  async updateSpec(
    projectId: string,
    specId: string,
    updates: SpecCapabilityUpdateInput
  ): Promise<SpecCapability> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<SpecCapability>>(
      `/api/projects/${projectId}/specs/${specId}`,
      {
        method: 'PUT',
        body: JSON.stringify(updates),
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update spec');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update spec');
    }

    if (!result.data.data) {
      throw new Error('No spec data returned');
    }

    return result.data.data;
  }

  async deleteSpec(projectId: string, specId: string): Promise<boolean> {
    const { apiRequest } = await import('./api');
    const result = await apiRequest<ApiResponse<string>>(
      `/api/projects/${projectId}/specs/${specId}`,
      {
        method: 'DELETE',
      }
    );

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to delete spec');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to delete spec');
    }

    return true;
  }

  async validateSpec(
    projectId: string,
    specData: SpecCapabilityCreateInput
  ): Promise<SpecValidationResult> {
    const response = await apiClient.post<ApiResponse<SpecValidationResult>>(
      `/api/projects/${projectId}/specs/validate`,
      specData
    );

    if (response.error || !response.data?.success) {
      throw new Error(
        response.data?.error || response.error || 'Failed to validate spec'
      );
    }

    if (!response.data.data) {
      throw new Error('No validation result returned');
    }

    return response.data.data;
  }

  async getSpecRequirements(
    projectId: string,
    specId: string
  ): Promise<SpecRequirement[]> {
    const response = await apiClient.get<ApiResponse<SpecRequirement[]>>(
      `/api/projects/${projectId}/specs/${specId}/requirements`
    );

    if (response.error || !response.data?.success) {
      throw new Error(
        response.data?.error || response.error || 'Failed to fetch requirements'
      );
    }

    return response.data.data || [];
  }
}

export const specsService = new SpecsService();
