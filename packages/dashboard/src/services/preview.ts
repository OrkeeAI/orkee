import { apiClient } from './api';

// Types matching the Rust backend
export interface DevServerInstance {
  id: string;
  project_id: string;
  config: {
    project_type: string;
    dev_command: string;
    port: number | null;
    package_manager: string;
    framework?: {
      name: string;
      version?: string;
    };
  };
  status: 'stopped' | 'starting' | 'running' | 'stopping' | 'error';
  preview_url?: string;
  started_at?: string;
  last_activity?: string;
  error?: string;
  pid?: number;
}

export interface DevServerLog {
  timestamp: string;
  log_type: 'stdout' | 'stderr' | 'system';
  message: string;
}

export interface StartServerRequest {
  custom_port?: number;
}

export interface StartServerResponse {
  instance: DevServerInstance;
}

export interface ServerStatusResponse {
  instance?: DevServerInstance;
}

export interface ServerLogsResponse {
  logs: DevServerLog[];
}

export interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

class PreviewService {
  private baseUrl = '/api/preview';

  /**
   * Start a development server for a project
   */
  async startServer(projectId: string, customPort?: number): Promise<DevServerInstance> {
    const response = await apiClient.post<ApiResponse<StartServerResponse>>(
      `${this.baseUrl}/servers/${projectId}/start`,
      { custom_port: customPort } satisfies StartServerRequest
    );

    if (response.error) {
      throw new Error(response.error);
    }

    const apiResponse = response.data;
    if (!apiResponse.success || !apiResponse.data) {
      throw new Error(apiResponse.error || 'Failed to start server');
    }

    return apiResponse.data.instance;
  }

  /**
   * Stop a development server
   */
  async stopServer(projectId: string): Promise<void> {
    const response = await apiClient.post<ApiResponse<void>>(
      `${this.baseUrl}/servers/${projectId}/stop`,
      {}
    );

    if (response.error) {
      throw new Error(response.error);
    }

    const apiResponse = response.data;
    if (!apiResponse.success) {
      throw new Error(apiResponse.error || 'Failed to stop server');
    }
  }

  /**
   * Get server status
   */
  async getServerStatus(projectId: string): Promise<DevServerInstance | null> {
    const response = await apiClient.get<ApiResponse<ServerStatusResponse>>(
      `${this.baseUrl}/servers/${projectId}/status`
    );

    if (response.error) {
      console.warn('Failed to get server status:', response.error);
      return null;
    }

    const apiResponse = response.data;
    if (!apiResponse.success || !apiResponse.data) {
      return null;
    }

    return apiResponse.data.instance || null;
  }

  /**
   * Get server logs
   */
  async getServerLogs(
    projectId: string,
    options: { since?: string; limit?: number } = {}
  ): Promise<DevServerLog[]> {
    const params = new URLSearchParams();
    if (options.since) params.set('since', options.since);
    if (options.limit) params.set('limit', options.limit.toString());

    const queryString = params.toString();
    const url = `${this.baseUrl}/servers/${projectId}/logs${queryString ? `?${queryString}` : ''}`;

    const response = await apiClient.get<ApiResponse<ServerLogsResponse>>(url);

    if (response.error) {
      console.warn('Failed to get server logs:', response.error);
      return [];
    }

    const apiResponse = response.data;
    if (!apiResponse.success || !apiResponse.data) {
      return [];
    }

    return apiResponse.data.logs;
  }

  /**
   * Clear server logs
   */
  async clearServerLogs(projectId: string): Promise<void> {
    const response = await apiClient.post<ApiResponse<void>>(
      `${this.baseUrl}/servers/${projectId}/logs/clear`,
      {}
    );

    if (response.error) {
      throw new Error(response.error);
    }

    const apiResponse = response.data;
    if (!apiResponse.success) {
      throw new Error(apiResponse.error || 'Failed to clear logs');
    }
  }

  /**
   * Update server activity timestamp
   */
  async updateServerActivity(projectId: string): Promise<void> {
    const response = await apiClient.post<ApiResponse<void>>(
      `${this.baseUrl}/servers/${projectId}/activity`,
      {}
    );

    if (response.error) {
      console.warn('Failed to update server activity:', response.error);
      return;
    }

    const apiResponse = response.data;
    if (!apiResponse.success) {
      console.warn('Failed to update server activity:', apiResponse.error);
    }
  }

  /**
   * Get all active servers (for debugging)
   */
  async getActiveServers(): Promise<string[]> {
    const response = await apiClient.get<ApiResponse<string[]>>(
      `${this.baseUrl}/servers`
    );

    if (response.error) {
      console.warn('Failed to get active servers:', response.error);
      return [];
    }

    const apiResponse = response.data;
    if (!apiResponse.success || !apiResponse.data) {
      return [];
    }

    return apiResponse.data;
  }

  /**
   * Health check for preview service
   */
  async healthCheck(): Promise<boolean> {
    try {
      const response = await apiClient.get<ApiResponse<string>>(
        `${this.baseUrl}/health`
      );
      
      if (response.error) {
        return false;
      }

      return response.data.success;
    } catch {
      return false;
    }
  }
}

// Export singleton instance
export const previewService = new PreviewService();