// ABOUTME: API service for context generation and OpenSpec integration
// ABOUTME: Provides methods for generating context from files, PRDs, tasks, and validating specs

import { ApiClient } from './api';

const api = new ApiClient();

export interface ContextGenerationRequest {
  include_patterns: string[];
  exclude_patterns?: string[];
  max_tokens?: number;
}

export interface GeneratedContext {
  content: string;
  file_count: number;
  total_tokens: number;
  files_included: string[];
  truncated: boolean;
}

export interface FileInfo {
  path: string;
  size: number;
  extension?: string;
  is_directory: boolean;
}

export interface ListFilesResponse {
  total_count: number;
  files: FileInfo[];
}

export interface ContextConfiguration {
  id: string;
  project_id: string;
  name: string;
  description?: string;
  include_patterns: string[];
  exclude_patterns: string[];
  max_tokens: number;
  created_at: string;
  updated_at: string;
}

export interface SpecContext {
  capability: string;
  requirements: Array<{
    id: string;
    content: string;
    hasImplementation: boolean;
  }>;
  suggestedFiles: string[];
  contextSize: number;
}

export interface RequirementValidation {
  requirement: string;
  status: 'implemented' | 'partiallyimplemented' | 'notimplemented';
  code_references: string[];
}

export interface SpecValidationReport {
  capability_name: string;
  total_requirements: number;
  implemented: number;
  partially_implemented: number;
  not_implemented: number;
  details: RequirementValidation[];
}

export interface ContextSnapshot {
  id: string;
  project_id: string;
  file_count: number;
  total_tokens: number;
  created_at: string;
  metadata?: {
    files_included: string[];
    generation_time_ms: number;
    git_commit?: string;
  };
}

export interface ContextStats {
  total_snapshots: number;
  most_used_files: Array<{
    path: string;
    inclusion_count: number;
    last_used: string;
  }>;
}

export const contextService = {
  // Basic context generation
  async generateContext(projectId: string, request: ContextGenerationRequest) {
    return api.post<GeneratedContext>(
      `/api/projects/${projectId}/context/generate`,
      request
    );
  },

  async listProjectFiles(projectId: string, maxDepth?: number) {
    const queryParams = maxDepth ? `?max_depth=${maxDepth}` : '';
    return api.get<ListFilesResponse>(
      `/api/projects/${projectId}/files${queryParams}`
    );
  },

  async listConfigurations(projectId: string) {
    return api.get<ContextConfiguration[]>(
      `/api/projects/${projectId}/context/configurations`
    );
  },

  async saveConfiguration(projectId: string, config: Partial<ContextConfiguration>) {
    return api.post<ContextConfiguration>(
      `/api/projects/${projectId}/context/configurations`,
      config
    );
  },

  // OpenSpec integration
  async generateContextFromPRD(projectId: string, prdId: string) {
    return api.post<GeneratedContext>(
      `/api/projects/${projectId}/context/from-prd`,
      { prd_id: prdId }
    );
  },

  async generateContextFromTask(projectId: string, taskId: string) {
    return api.post<GeneratedContext>(
      `/api/projects/${projectId}/context/from-task`,
      { task_id: taskId }
    );
  },

  async validateSpecImplementation(projectId: string, capabilityId: string) {
    return api.post<SpecValidationReport>(
      `/api/projects/${projectId}/context/validate-spec`,
      { capability_id: capabilityId }
    );
  },

  // History and analytics
  async getContextHistory(projectId: string) {
    return api.get<{ snapshots: ContextSnapshot[] }>(
      `/api/projects/${projectId}/context/history`
    );
  },

  async getContextStats(projectId: string) {
    return api.get<ContextStats>(
      `/api/projects/${projectId}/context/stats`
    );
  },

  async restoreSnapshot(projectId: string, snapshotId: string) {
    return api.post<GeneratedContext>(
      `/api/projects/${projectId}/context/restore`,
      { snapshot_id: snapshotId }
    );
  },
};
