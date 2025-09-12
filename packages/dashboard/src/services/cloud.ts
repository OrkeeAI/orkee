import { apiRequest } from './api';

// Type definitions matching the Rust API
export interface CloudAuthStatus {
  authenticated: boolean;
  user_id?: string;
  user_email?: string;
  user_name?: string;
  subscription_tier?: string;
}

export interface GlobalSyncStatus {
  total_projects: number;
  synced_projects: number;
  pending_projects: number;
  syncing_projects: number;
  conflict_projects: number;
  last_sync?: string;
  is_syncing: boolean;
  current_sync_progress: number; // 0.0 to 1.0
}

export interface ProjectSyncStatus {
  project_id: string;
  cloud_project_id?: string;
  status: 'synced' | 'pending' | 'syncing' | 'conflict' | 'error' | 'not_available' | 'not_authenticated';
  last_sync?: string;
  has_conflicts: boolean;
  sync_progress?: number;
  error_message?: string;
}

export interface SyncResult {
  project_id: string;
  success: boolean;
  message: string;
  conflicts_detected: boolean;
}

export interface CloudProject {
  id: string;
  name: string;
  description?: string;
  created_at: string;
  updated_at: string;
  owner_id: string;
  // Add other fields as needed
}

export interface OAuthInitResponse {
  auth_url: string;
  state: string;
  expires_at: string;
}

// Request types
export interface OAuthCallbackRequest {
  code: string;
  state: string;
}

export interface SyncAllRequest {
  force?: boolean;
  exclude_projects?: string[];
}

export interface SyncProjectRequest {
  force?: boolean;
}

// Generic API response wrapper
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

/**
 * CloudService handles all cloud sync operations
 * Communicates with the CLI server's cloud API endpoints
 */
export class CloudService {
  constructor() {
  }

  // Authentication methods
  async initOAuthFlow(): Promise<OAuthInitResponse> {
    const response = await apiRequest<ApiResponse<OAuthInitResponse>>('/api/cloud/auth/init', {
      method: 'POST',
    });

    if (!response.success || !response.data?.success || !response.data.data) {
      throw new Error(response.error || response.data?.error || 'Failed to initialize OAuth flow');
    }

    return response.data.data;
  }

  async handleOAuthCallback(code: string, state: string): Promise<CloudAuthStatus> {
    const response = await apiRequest<ApiResponse<CloudAuthStatus>>('/api/cloud/auth/callback', {
      method: 'POST',
      body: JSON.stringify({ code, state }),
    });

    if (!response.success || !response.data?.success || !response.data.data) {
      throw new Error(response.error || response.data?.error || 'OAuth callback failed');
    }

    return response.data.data;
  }

  async getAuthStatus(): Promise<CloudAuthStatus> {
    const response = await apiRequest<ApiResponse<CloudAuthStatus>>('/api/cloud/auth/status');

    if (!response.success || !response.data?.success || !response.data.data) {
      // Return unauthenticated status instead of throwing
      return {
        authenticated: false,
        user_id: undefined,
        user_email: undefined,
        user_name: undefined,
        subscription_tier: undefined,
      };
    }

    return response.data.data;
  }

  async logout(): Promise<void> {
    const response = await apiRequest<ApiResponse<string>>('/api/cloud/auth/logout', {
      method: 'POST',
    });

    if (!response.success || !response.data?.success) {
      throw new Error(response.error || response.data?.error || 'Logout failed');
    }
  }

  // Sync status methods
  async getGlobalSyncStatus(): Promise<GlobalSyncStatus> {
    const response = await apiRequest<ApiResponse<GlobalSyncStatus>>('/api/cloud/sync/status');

    if (!response.success || !response.data?.success || !response.data.data) {
      // Return default status instead of throwing for non-critical data
      return {
        total_projects: 0,
        synced_projects: 0,
        pending_projects: 0,
        syncing_projects: 0,
        conflict_projects: 0,
        last_sync: undefined,
        is_syncing: false,
        current_sync_progress: 0.0,
      };
    }

    return response.data.data;
  }

  async getProjectSyncStatus(projectId: string): Promise<ProjectSyncStatus> {
    const response = await apiRequest<ApiResponse<ProjectSyncStatus>>(`/api/cloud/projects/${projectId}/status`);

    if (!response.success || !response.data?.success || !response.data.data) {
      // Return default status for individual projects
      return {
        project_id: projectId,
        cloud_project_id: undefined,
        status: 'not_available',
        last_sync: undefined,
        has_conflicts: false,
        sync_progress: undefined,
        error_message: response.error || response.data?.error || 'Unable to get sync status',
      };
    }

    return response.data.data;
  }

  // Project sync methods
  async listCloudProjects(): Promise<CloudProject[]> {
    const response = await apiRequest<ApiResponse<CloudProject[]>>('/api/cloud/projects');

    if (!response.success || !response.data?.success) {
      throw new Error(response.error || response.data?.error || 'Failed to list cloud projects');
    }

    return response.data.data || [];
  }

  async syncAllProjects(options: SyncAllRequest = {}): Promise<SyncResult[]> {
    const response = await apiRequest<ApiResponse<SyncResult[]>>('/api/cloud/projects/sync-all', {
      method: 'POST',
      body: JSON.stringify(options),
    });

    if (!response.success || !response.data?.success) {
      throw new Error(response.error || response.data?.error || 'Failed to sync all projects');
    }

    return response.data.data || [];
  }

  async syncProject(projectId: string, options: SyncProjectRequest = {}): Promise<SyncResult> {
    const response = await apiRequest<ApiResponse<SyncResult>>(`/api/cloud/projects/${projectId}/sync`, {
      method: 'POST',
      body: JSON.stringify(options),
    });

    if (!response.success || !response.data?.success || !response.data.data) {
      throw new Error(response.error || response.data?.error || 'Failed to sync project');
    }

    return response.data.data;
  }

  // Usage and subscription methods
  async getUsageStats(): Promise<string> {
    const response = await apiRequest<ApiResponse<string>>('/api/cloud/usage');

    if (!response.success || !response.data?.success) {
      throw new Error(response.error || response.data?.error || 'Failed to get usage stats');
    }

    return response.data.data || 'No usage data available';
  }

  // OAuth helpers for browser-based auth
  async openOAuthWindow(): Promise<CloudAuthStatus> {
    const authInit = await this.initOAuthFlow();
    
    return new Promise((resolve, reject) => {
      const popup = window.open(
        authInit.auth_url,
        'orkee-cloud-auth',
        'width=500,height=600,scrollbars=yes,resizable=yes'
      );

      if (!popup) {
        reject(new Error('Failed to open OAuth window. Please disable popup blockers.'));
        return;
      }

      // Poll for the callback
      const pollTimer = setInterval(async () => {
        try {
          if (popup.closed) {
            clearInterval(pollTimer);
            reject(new Error('OAuth flow cancelled by user'));
            return;
          }

          // Check if we can access the popup URL (same-origin after callback)
          if (popup.location.origin === window.location.origin) {
            const urlParams = new URLSearchParams(popup.location.search);
            const code = urlParams.get('code');
            const state = urlParams.get('state');

            if (code && state) {
              clearInterval(pollTimer);
              popup.close();

              try {
                const authStatus = await this.handleOAuthCallback(code, state);
                resolve(authStatus);
              } catch (error) {
                reject(error);
              }
            }
          }
        } catch (e) {
          // Can't access popup.location yet (cross-origin)
          // Continue polling
        }
      }, 1000);

      // Timeout after 5 minutes
      setTimeout(() => {
        clearInterval(pollTimer);
        if (!popup.closed) {
          popup.close();
        }
        reject(new Error('OAuth flow timed out'));
      }, 5 * 60 * 1000);
    });
  }

  // Real-time sync status polling
  private syncStatusCallbacks: ((status: GlobalSyncStatus) => void)[] = [];
  private syncStatusPollingInterval: number | null = null;

  subscribeSyncUpdates(callback: (status: GlobalSyncStatus) => void): () => void {
    this.syncStatusCallbacks.push(callback);

    // Start polling if this is the first subscriber
    if (this.syncStatusCallbacks.length === 1) {
      this.startSyncStatusPolling();
    }

    // Return unsubscribe function
    return () => {
      const index = this.syncStatusCallbacks.indexOf(callback);
      if (index > -1) {
        this.syncStatusCallbacks.splice(index, 1);
      }

      // Stop polling if no more subscribers
      if (this.syncStatusCallbacks.length === 0) {
        this.stopSyncStatusPolling();
      }
    };
  }

  private startSyncStatusPolling(): void {
    if (this.syncStatusPollingInterval) return;

    this.syncStatusPollingInterval = setInterval(async () => {
      try {
        const status = await this.getGlobalSyncStatus();
        this.syncStatusCallbacks.forEach(callback => callback(status));

        // If nothing is syncing, reduce polling frequency
        if (!status.is_syncing && status.syncing_projects === 0) {
          this.stopSyncStatusPolling();
          // Restart with longer interval after a delay
          setTimeout(() => {
            if (this.syncStatusCallbacks.length > 0) {
              this.startSyncStatusPolling();
            }
          }, 10000); // 10 second delay
        }
      } catch (error) {
        console.error('Failed to poll sync status:', error);
      }
    }, 2000); // Poll every 2 seconds during active sync
  }

  private stopSyncStatusPolling(): void {
    if (this.syncStatusPollingInterval) {
      clearInterval(this.syncStatusPollingInterval);
      this.syncStatusPollingInterval = null;
    }
  }

  // Cleanup method
  cleanup(): void {
    this.stopSyncStatusPolling();
    this.syncStatusCallbacks = [];
  }
}

// Export singleton instance
export const cloudService = new CloudService();

// Helper functions for UI
export const formatSyncStatus = (status: ProjectSyncStatus['status']): { label: string; color: string; icon: string } => {
  switch (status) {
    case 'synced':
      return { label: 'Synced', color: 'text-green-600', icon: '✓' };
    case 'pending':
      return { label: 'Pending', color: 'text-yellow-600', icon: '⏳' };
    case 'syncing':
      return { label: 'Syncing...', color: 'text-blue-600', icon: '🔄' };
    case 'conflict':
      return { label: 'Conflict', color: 'text-red-600', icon: '⚠️' };
    case 'error':
      return { label: 'Error', color: 'text-red-600', icon: '❌' };
    case 'not_authenticated':
      return { label: 'Not authenticated', color: 'text-gray-500', icon: '🔒' };
    case 'not_available':
    default:
      return { label: 'Not available', color: 'text-gray-400', icon: '—' };
  }
};

export const formatLastSync = (lastSync?: string): string => {
  if (!lastSync) return 'Never';
  
  const date = new Date(lastSync);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / (1000 * 60));
  
  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  
  const diffDays = Math.floor(diffHours / 24);
  if (diffDays < 7) return `${diffDays}d ago`;
  
  return date.toLocaleDateString();
};