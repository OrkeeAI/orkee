// ABOUTME: User service for managing user credentials and settings
// ABOUTME: Provides API methods for fetching and updating user data securely

import { apiClient, apiRequest } from './api';

// TypeScript interfaces matching Rust types
export interface MaskedUser {
  id: string;
  email: string;
  name: string;
  avatar_url: string | null;
  default_agent_id: string | null;
  theme: string | null;
  has_openai_api_key: boolean;
  has_anthropic_api_key: boolean;
  has_google_api_key: boolean;
  has_xai_api_key: boolean;
  ai_gateway_enabled: boolean;
  has_ai_gateway_key: boolean;
  ai_gateway_url: string | null;
  preferences: unknown | null;
  created_at: string;
  updated_at: string;
  last_login_at: string | null;
}

export interface UserCredentialsUpdate {
  openai_api_key?: string;
  anthropic_api_key?: string;
  google_api_key?: string;
  xai_api_key?: string;
  ai_gateway_enabled?: boolean;
  ai_gateway_url?: string;
  ai_gateway_key?: string;
}

// API Response format from Rust server
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class UsersService {
  /**
   * Get current user with masked credentials
   * Returns user data with boolean flags instead of actual API keys
   */
  async getCurrentUser(): Promise<MaskedUser> {
    const response = await apiClient.get<ApiResponse<MaskedUser>>('/api/users/current');

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch user');
    }

    if (!response.data.data) {
      throw new Error('No user data returned');
    }

    return response.data.data;
  }

  /**
   * Update user credentials
   * Accepts partial updates - only provided fields will be updated
   */
  async updateCredentials(updates: UserCredentialsUpdate): Promise<MaskedUser> {
    const result = await apiRequest<ApiResponse<MaskedUser>>('/api/users/credentials', {
      method: 'PUT',
      body: JSON.stringify(updates),
    });

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to update credentials');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to update credentials');
    }

    if (!result.data.data) {
      throw new Error('No user data returned');
    }

    return result.data.data;
  }
}

export const usersService = new UsersService();
