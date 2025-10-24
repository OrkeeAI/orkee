// ABOUTME: Frontend service for agent operations
// ABOUTME: Handles listing and managing AI agents with pagination support

import { apiRequest } from './api';
import type { PaginationParams, PaginatedResponse } from '@/types/pagination';
import { buildPaginationQuery } from '@/types/pagination';

export type AgentType = 'system' | 'ai' | 'human';

export interface Agent {
  id: string;
  type: AgentType;
  displayName: string;
  description: string | null;
  capabilities: string[];
}

export interface UserAgent {
  userId: string;
  agentId: string;
  isActive: boolean;
  agent: Agent;
}

interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class AgentsService {
  async listAgents(pagination?: PaginationParams): Promise<PaginatedResponse<Agent>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<Agent>>>(
      `/api/agents${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch agents');
    }

    // response.data is the PaginatedResponse, which doesn't have a success field
    return response.data;
  }

  async getAgent(agentId: string): Promise<Agent> {
    const response = await apiRequest<ApiResponse<Agent>>(
      `/api/agents/${agentId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch agent');
    }

    // For single items, response.data is the Agent directly
    return response.data;
  }

  async listUserAgents(
    userId: string,
    pagination?: PaginationParams
  ): Promise<PaginatedResponse<UserAgent>> {
    const query = pagination ? buildPaginationQuery(pagination) : '';
    const response = await apiRequest<ApiResponse<PaginatedResponse<UserAgent>>>(
      `/api/agents/users/${userId}${query}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user agents');
    }

    // response.data is the PaginatedResponse
    return response.data;
  }

  async getUserAgent(userId: string, agentId: string): Promise<UserAgent> {
    const response = await apiRequest<ApiResponse<UserAgent>>(
      `/api/agents/users/${userId}/${agentId}`
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to fetch user agent');
    }

    // For single items, response.data is the UserAgent directly
    return response.data;
  }

  async activateAgent(userId: string, agentId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<{ message: string }>>(
      `/api/agents/users/${userId}/${agentId}/activation`,
      {
        method: 'PUT',
        body: JSON.stringify({ isActive: true }),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to activate agent');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to activate agent');
    }
  }

  async deactivateAgent(userId: string, agentId: string): Promise<void> {
    const response = await apiRequest<ApiResponse<{ message: string }>>(
      `/api/agents/users/${userId}/${agentId}/activation`,
      {
        method: 'PUT',
        body: JSON.stringify({ isActive: false }),
      }
    );

    if (!response.success || !response.data) {
      throw new Error(response.error || 'Failed to deactivate agent');
    }

    if (!response.data.success) {
      throw new Error(response.data.error || 'Failed to deactivate agent');
    }
  }
}

export const agentsService = new AgentsService();
