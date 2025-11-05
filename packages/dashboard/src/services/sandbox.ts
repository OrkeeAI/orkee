// ABOUTME: Frontend service for sandbox execution operations
// ABOUTME: Handles execution lifecycle, logs, and artifacts for containerized agent runs

import { apiRequest } from './api';

export interface SandboxConfig {
  id: string;
  name: string;
  provider: string;
  description: string;
  supported_images: string[];
  default_image: string;
  max_concurrent: number;
  resource_limits: {
    memory_mb: number;
    cpu_cores: number;
    timeout_seconds: number;
  };
  is_available: boolean;
  requires_config: boolean;
}

export interface LogEntry {
  id: string;
  execution_id: string;
  timestamp: string;
  log_level: 'debug' | 'info' | 'warn' | 'error' | 'fatal';
  message: string;
  source?: string;
  metadata?: Record<string, unknown>;
  stack_trace?: string;
  sequence_number: number;
}

export interface Artifact {
  id: string;
  execution_id: string;
  artifact_type: 'file' | 'screenshot' | 'test_report' | 'coverage' | 'output';
  file_path: string;
  file_name: string;
  file_size_bytes?: number;
  mime_type?: string;
  stored_path?: string;
  storage_backend: 'local' | 's3' | 'gcs';
  description?: string;
  metadata?: Record<string, unknown>;
  checksum?: string;
  created_at: string;
}

export interface ResourceUsage {
  memory_used_mb?: number;
  cpu_usage_percent?: number;
}

export interface SandboxExecution {
  id: string;
  task_id: string;
  agent_id?: string;
  model?: string;
  sandbox_provider?: string;
  container_id?: string;
  container_image?: string;
  container_status?: 'creating' | 'running' | 'stopped' | 'error';
  memory_limit_mb?: number;
  memory_used_mb?: number;
  cpu_limit_cores?: number;
  cpu_usage_percent?: number;
  workspace_path?: string;
  output_files?: string[];
  vibekit_session_id?: string;
  vibekit_version?: string;
  environment_variables?: Record<string, string>;
  started_at?: string;
  completed_at?: string;
  error_message?: string;
}

export interface StopExecutionRequest {
  containerId: string;
}

export interface RetryExecutionRequest {
  taskId: string;
  agentId?: string;
  model?: string;
}

export interface LogsResponse {
  logs: LogEntry[];
  total: number;
}

export interface LogQueryParams {
  limit?: number;
  offset?: number;
}

export interface SearchLogsParams {
  logLevel?: string;
  source?: string;
  search?: string;
  limit?: number;
  offset?: number;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class SandboxService {
  /**
   * Stop a running execution
   */
  async stopExecution(
    executionId: string,
    containerId: string
  ): Promise<string> {
    const response = await apiRequest<ApiResponse<string>>(
      `/api/sandbox/executions/${executionId}/stop`,
      {
        method: 'POST',
        body: JSON.stringify({ containerId }),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to stop execution');
    }

    return response.data;
  }

  /**
   * Retry a failed execution
   */
  async retryExecution(
    executionId: string,
    request: RetryExecutionRequest
  ): Promise<void> {
    const response = await apiRequest<ApiResponse<void>>(
      `/api/sandbox/executions/${executionId}/retry`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );

    if (!response.success) {
      throw new Error(response.error || 'Failed to retry execution');
    }
  }

  /**
   * Get paginated logs for an execution
   */
  async getLogs(
    executionId: string,
    params?: LogQueryParams
  ): Promise<LogsResponse> {
    const query = params
      ? `?${new URLSearchParams(
          Object.entries(params)
            .filter(([, v]) => v !== undefined)
            .map(([k, v]) => [k, String(v)])
        ).toString()}`
      : '';

    const response = await apiRequest<ApiResponse<LogsResponse>>(
      `/api/sandbox/executions/${executionId}/logs${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch logs');
    }

    return response.data;
  }

  /**
   * Search logs with filters
   */
  async searchLogs(
    executionId: string,
    params: SearchLogsParams
  ): Promise<LogsResponse> {
    const query = new URLSearchParams(
      Object.entries(params)
        .filter(([, v]) => v !== undefined)
        .map(([k, v]) => [k, String(v)])
    ).toString();

    const response = await apiRequest<ApiResponse<LogsResponse>>(
      `/api/sandbox/executions/${executionId}/logs/search?${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to search logs');
    }

    return response.data;
  }

  /**
   * List artifacts for an execution
   */
  async listArtifacts(executionId: string): Promise<Artifact[]> {
    const response = await apiRequest<ApiResponse<Artifact[]>>(
      `/api/sandbox/executions/${executionId}/artifacts`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to list artifacts');
    }

    return response.data;
  }

  /**
   * Get artifact metadata
   */
  async getArtifact(artifactId: string): Promise<Artifact> {
    const response = await apiRequest<ApiResponse<Artifact>>(
      `/api/sandbox/artifacts/${artifactId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to get artifact');
    }

    return response.data;
  }

  /**
   * Get download URL for an artifact
   */
  getArtifactDownloadUrl(artifactId: string): string {
    const baseUrl = import.meta.env.VITE_API_URL || 'http://localhost:4001';
    return `${baseUrl}/api/sandbox/artifacts/${artifactId}/download`;
  }

  /**
   * Delete an artifact
   */
  async deleteArtifact(artifactId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<string>>(
      `/api/sandbox/artifacts/${artifactId}`,
      {
        method: 'DELETE',
      }
    );

    if (!response.success) {
      throw new Error(response.error || 'Failed to delete artifact');
    }
  }

  /**
   * Load sandbox configurations
   */
  async loadSandboxConfigs(): Promise<SandboxConfig[]> {
    // In the future this might come from an API endpoint
    // For now, we'll hardcode the local-docker config from the JSON file
    return [
      {
        id: 'local-docker',
        name: 'Local Docker',
        provider: 'local',
        description: 'Run sandboxes in local Docker containers',
        supported_images: ['ubuntu:22.04', 'node:20', 'python:3.11'],
        default_image: 'ubuntu:22.04',
        max_concurrent: 5,
        resource_limits: {
          memory_mb: 2048,
          cpu_cores: 2.0,
          timeout_seconds: 3600,
        },
        is_available: true,
        requires_config: false,
      },
    ];
  }
}

export const sandboxService = new SandboxService();
