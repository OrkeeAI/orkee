// ABOUTME: User management service for API interactions
// ABOUTME: Handles user settings, preferences, and default agent configuration

import { User } from '../types';

export class UserService {
  private apiBaseUrl: string;

  constructor(apiBaseUrl: string = 'http://localhost:4001') {
    this.apiBaseUrl = apiBaseUrl;
  }

  async getCurrentUser(): Promise<User> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/current`);
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to fetch current user');
    return this.transformUser(data.data);
  }

  async updateUserSettings(userId: string, settings: Partial<User>): Promise<User> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    });
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to update user settings');
    return this.transformUser(data.data);
  }

  async setDefaultAgent(userId: string, agentId: string): Promise<void> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/default-agent`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ agentId }),
    });
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to set default agent');
  }

  async updateApiKey(userId: string, provider: string, apiKey: string): Promise<void> {
    const response = await fetch(`${this.apiBaseUrl}/api/users/${userId}/api-keys/${provider}`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ apiKey }),
    });
    const data = await response.json();
    if (!data.success) throw new Error(data.error || 'Failed to update API key');
  }

  private transformUser(data: any): User {
    return {
      ...data,
      avatarUrl: data.avatar_url || data.avatarUrl,
      defaultAgentId: data.default_agent_id || data.defaultAgentId,
      openaiApiKey: data.openai_api_key || data.openaiApiKey,
      anthropicApiKey: data.anthropic_api_key || data.anthropicApiKey,
      googleApiKey: data.google_api_key || data.googleApiKey,
      xaiApiKey: data.xai_api_key || data.xaiApiKey,
      createdAt: new Date(data.created_at || data.createdAt),
      updatedAt: new Date(data.updated_at || data.updatedAt),
      lastLoginAt: data.last_login_at || data.lastLoginAt ? new Date(data.last_login_at || data.lastLoginAt) : undefined,
    };
  }
}

// Export a singleton instance
export const userService = new UserService();
