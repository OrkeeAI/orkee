// ABOUTME: Security service for managing encryption and API key security
// ABOUTME: Provides API methods for password-based encryption and key status

import { apiClient, apiRequest } from './api';

// Encryption mode enum matching Rust backend
export enum EncryptionMode {
  Machine = 'machine',
  Password = 'password',
}

// TypeScript interfaces matching Rust types
export interface SecurityStatus {
  encryptionMode: EncryptionMode;
  isLocked: boolean;
  failedAttempts?: number;
  lockoutEndsAt?: string;
}

export interface KeyStatus {
  key: string;
  configured: boolean;
  source: 'database' | 'environment' | 'none';
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

/**
 * Helper to extract data from nested API response with error checking
 * Handles the common pattern of checking success flags and extracting data
 */
function extractApiData<T>(
  response: { error?: string; data?: ApiResponse<T> | null },
  errorMessage: string
): T {
  // Check outer response wrapper
  if (response.error || !response.data?.success) {
    throw new Error(response.data?.error || response.error || errorMessage);
  }

  // Check inner data
  if (!response.data.data) {
    throw new Error('No data returned from server');
  }

  return response.data.data;
}

/**
 * Helper to extract data from mutation API response with error checking
 * Handles the common pattern for POST/PUT/DELETE operations
 */
function extractMutationData<T>(
  result: { success: boolean; data?: ApiResponse<T> | null; error?: string },
  errorMessage: string
): T {
  // Check outer result
  if (!result.success || !result.data) {
    throw new Error(result.error || errorMessage);
  }

  // Check inner API response
  if (!result.data.success) {
    throw new Error(result.data.error || errorMessage);
  }

  // Check inner data
  if (!result.data.data) {
    throw new Error('No response data returned');
  }

  return result.data.data;
}

export class SecurityService {
  /**
   * Get current security and encryption status
   */
  async getSecurityStatus(): Promise<SecurityStatus> {
    const response = await apiClient.get<ApiResponse<SecurityStatus>>('/api/security/status');
    return extractApiData(response, 'Failed to fetch security status');
  }

  /**
   * Get status of all API keys (which are configured and where they come from)
   */
  async getKeysStatus(): Promise<KeysStatusResponse> {
    const response = await apiClient.get<ApiResponse<KeysStatusResponse>>('/api/security/keys-status');
    return extractApiData(response, 'Failed to fetch keys status');
  }

  /**
   * Set password to enable password-based encryption
   */
  async setPassword(password: string): Promise<{ message: string; encryptionMode: EncryptionMode }> {
    const result = await apiRequest<ApiResponse<{ message: string; encryptionMode: EncryptionMode }>>('/api/security/set-password', {
      method: 'POST',
      body: JSON.stringify({ password }),
    });

    return extractMutationData(result, 'Failed to set password');
  }

  /**
   * Change encryption password
   */
  async changePassword(currentPassword: string, newPassword: string): Promise<{ message: string; encryptionMode: EncryptionMode }> {
    const result = await apiRequest<ApiResponse<{ message: string; encryptionMode: EncryptionMode }>>('/api/security/change-password', {
      method: 'POST',
      body: JSON.stringify({
        currentPassword,
        newPassword,
      }),
    });

    return extractMutationData(result, 'Failed to change password');
  }

  /**
   * Remove password-based encryption (downgrade to machine-based)
   */
  async removePassword(currentPassword: string): Promise<{ message: string; encryptionMode: EncryptionMode }> {
    const result = await apiRequest<ApiResponse<{ message: string; encryptionMode: EncryptionMode }>>('/api/security/remove-password', {
      method: 'POST',
      body: JSON.stringify({ currentPassword }),
    });

    return extractMutationData(result, 'Failed to remove password');
  }
}

export const securityService = new SecurityService();
