// ABOUTME: Security service for managing encryption and API key security
// ABOUTME: Provides API methods for password-based encryption and key status

import { apiClient, apiRequest } from './api';

// TypeScript interfaces matching Rust types
export interface SecurityStatus {
  encryptionMode: string; // "machine" or "password"
  isLocked: boolean;
  failedAttempts?: number;
  lockoutEndsAt?: string;
}

export interface KeyStatus {
  key: string;
  configured: boolean;
  source: 'database' | 'environment' | 'none';
  lastUpdated?: string;
}

export interface KeysStatusResponse {
  keys: KeyStatus[];
}

export interface SetPasswordRequest {
  password: string;
}

export interface ChangePasswordRequest {
  currentPassword: string;
  newPassword: string;
}

// API Response format from Rust server
interface ApiResponse<T> {
  success: boolean;
  data: T | null;
  error?: string;
}

export class SecurityService {
  /**
   * Get current security and encryption status
   */
  async getSecurityStatus(): Promise<SecurityStatus> {
    const response = await apiClient.get<ApiResponse<SecurityStatus>>('/api/security/status');

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch security status');
    }

    if (!response.data.data) {
      throw new Error('No security status returned');
    }

    return response.data.data;
  }

  /**
   * Get status of all API keys (which are configured and where they come from)
   */
  async getKeysStatus(): Promise<KeysStatusResponse> {
    const response = await apiClient.get<ApiResponse<KeysStatusResponse>>('/api/security/keys-status');

    if (response.error || !response.data?.success) {
      throw new Error(response.data?.error || response.error || 'Failed to fetch keys status');
    }

    if (!response.data.data) {
      throw new Error('No keys status returned');
    }

    return response.data.data;
  }

  /**
   * Set password to enable password-based encryption
   */
  async setPassword(password: string): Promise<{ message: string; encryptionMode: string }> {
    const result = await apiRequest<ApiResponse<{ message: string; encryption_mode: string }>>('/api/security/set-password', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to set password');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to set password');
    }

    if (!result.data.data) {
      throw new Error('No response data returned');
    }

    return {
      message: result.data.data.message,
      encryptionMode: result.data.data.encryption_mode,
    };
  }

  /**
   * Change encryption password
   */
  async changePassword(currentPassword: string, newPassword: string): Promise<{ message: string; encryptionMode: string }> {
    const result = await apiRequest<ApiResponse<{ message: string; encryption_mode: string }>>('/api/security/change-password', {
      method: 'POST',
      body: JSON.stringify({
        current_password: currentPassword,
        new_password: newPassword,
      }),
    });

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to change password');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to change password');
    }

    if (!result.data.data) {
      throw new Error('No response data returned');
    }

    return {
      message: result.data.data.message,
      encryptionMode: result.data.data.encryption_mode,
    };
  }

  /**
   * Remove password-based encryption (downgrade to machine-based)
   */
  async removePassword(): Promise<{ message: string; encryptionMode: string }> {
    const result = await apiRequest<ApiResponse<{ message: string; encryption_mode: string }>>('/api/security/remove-password', {
      method: 'POST',
    });

    if (!result.success || !result.data) {
      throw new Error(result.error || 'Failed to remove password');
    }

    if (!result.data.success) {
      throw new Error(result.data.error || 'Failed to remove password');
    }

    if (!result.data.data) {
      throw new Error('No response data returned');
    }

    return {
      message: result.data.data.message,
      encryptionMode: result.data.data.encryption_mode,
    };
  }
}

export const securityService = new SecurityService();
