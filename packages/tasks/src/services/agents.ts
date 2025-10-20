// ABOUTME: Agent management service for API interactions
// ABOUTME: Handles CRUD operations for agents and user-agent configurations

import { Agent, UserAgent } from '../types';

export class AgentService {
  private apiBaseUrl: string;

  constructor(apiBaseUrl: string = 'http://localhost:4001') {
    this.apiBaseUrl = apiBaseUrl;
  }

  async listAllAgents(): Promise<Agent[]> {
    const response = await fetch(`${this.apiBaseUrl}/api/agents`);
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to fetch agents');
    return this.transformAgents(data.data);
  }

  async listUserAgents(userId: string): Promise<UserAgent[]> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/agents`);
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to fetch user agents');
    return this.transformUserAgents(data.data);
  }

  async getActiveAgents(userId: string): Promise<Agent[]> {
    const userAgents = await this.listUserAgents(userId);
    return userAgents.filter(ua => ua.isActive).map(ua => ua.agent);
  }

  async getAgent(agentId: string): Promise<Agent> {
    const response = await fetch(`${this.apiBaseUrl}/api/agents/${agentId}`);
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to fetch agent');
    return this.transformAgent(data.data);
  }

  async updateUserAgent(userId: string, agentId: string, updates: Partial<UserAgent>): Promise<UserAgent> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/agents/${agentId}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(updates),
    });
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to update user agent');
    return this.transformUserAgent(data.data);
  }

  async activateAgent(userId: string, agentId: string): Promise<void> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/agents/${agentId}/activate`, {
      method: 'POST',
    });
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to activate agent');
  }

  async deactivateAgent(userId: string, agentId: string): Promise<void> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/agents/${agentId}/deactivate`, {
      method: 'POST',
    });
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to deactivate agent');
  }

  private transformAgent(data: any): Agent {
    return {
      ...data,
      createdAt: new Date(data.created_at || data.createdAt),
      updatedAt: new Date(data.updated_at || data.updatedAt),
      displayName: data.display_name || data.displayName,
      avatarUrl: data.avatar_url || data.avatarUrl,
      maxContextTokens: data.max_context_tokens || data.maxContextTokens,
      supportsTools: data.supports_tools || data.supportsTools,
      supportsVision: data.supports_vision || data.supportsVision,
      supportsWebSearch: data.supports_web_search || data.supportsWebSearch,
      apiEndpoint: data.api_endpoint || data.apiEndpoint,
      systemPrompt: data.system_prompt || data.systemPrompt,
      costPer1kInputTokens: data.cost_per_1k_input_tokens || data.costPer1kInputTokens,
      costPer1kOutputTokens: data.cost_per_1k_output_tokens || data.costPer1kOutputTokens,
      isAvailable: data.is_available ?? data.isAvailable ?? true,
      requiresApiKey: data.requires_api_key ?? data.requiresApiKey ?? true,
    };
  }

  private transformAgents(data: any[]): Agent[] {
    return data.map(item => this.transformAgent(item));
  }

  private transformUserAgent(data: any): UserAgent {
    return {
      ...data,
      userId: data.user_id || data.userId,
      agentId: data.agent_id || data.agentId,
      agent: this.transformAgent(data.agent),
      isActive: data.is_active ?? data.isActive ?? true,
      isFavorite: data.is_favorite ?? data.isFavorite ?? false,
      customName: data.custom_name || data.customName,
      customSystemPrompt: data.custom_system_prompt || data.customSystemPrompt,
      customTemperature: data.custom_temperature || data.customTemperature,
      customMaxTokens: data.custom_max_tokens || data.customMaxTokens,
      tasksAssigned: data.tasks_assigned || data.tasksAssigned || 0,
      tasksCompleted: data.tasks_completed || data.tasksCompleted || 0,
      totalTokensUsed: data.total_tokens_used || data.totalTokensUsed || 0,
      totalCostCents: data.total_cost_cents || data.totalCostCents || 0,
      lastUsedAt: data.last_used_at || data.lastUsedAt ? new Date(data.last_used_at || data.lastUsedAt) : undefined,
      createdAt: new Date(data.created_at || data.createdAt),
      updatedAt: new Date(data.updated_at || data.updatedAt),
    };
  }

  private transformUserAgents(data: any[]): UserAgent[] {
    return data.map(item => this.transformUserAgent(item));
  }
}

// Export a singleton instance
export const agentService = new AgentService();
