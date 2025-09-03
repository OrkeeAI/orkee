import { apiClient } from './api';

export interface HealthStatus {
  status: string;
  timestamp: number;
  version: string;
  service: string;
}

export interface DetailedStatus extends HealthStatus {
  uptime: number;
  memory: {
    used: string;
    available: string;
  };
  connections: {
    active: number;
    total: number;
  };
}

export class HealthService {
  async checkHealth(): Promise<HealthStatus | null> {
    const result = await apiClient.get<HealthStatus>('/api/health');
    if (result.error) {
      console.error('Health check failed:', result.error);
      return null;
    }
    return result.data;
  }

  async getDetailedStatus(): Promise<DetailedStatus | null> {
    const result = await apiClient.get<DetailedStatus>('/api/status');
    if (result.error) {
      console.error('Status check failed:', result.error);
      return null;
    }
    return result.data;
  }
}

export const healthService = new HealthService();