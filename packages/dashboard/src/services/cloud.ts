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
    console.log('[CLOUD_SERVICE] syncAllProjects called with options:', options);
    
    const response = await apiRequest<ApiResponse<SyncResult[]>>('/api/cloud/projects/sync-all', {
      method: 'POST',
      body: JSON.stringify(options),
    });

    console.log('[CLOUD_SERVICE] syncAllProjects response:', response);

    if (!response.success || !response.data?.success) {
      console.error('[CLOUD_SERVICE] syncAllProjects failed:', response.error || response.data?.error);
      throw new Error(response.error || response.data?.error || 'Failed to sync all projects');
    }

    const results = response.data.data || [];
    console.log('[CLOUD_SERVICE] syncAllProjects returning results:', results);
    return results;
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
      // Calculate center position for popup
      const width = 600;
      const height = 700;
      const left = window.screen.width / 2 - width / 2;
      const top = window.screen.height / 2 - height / 2;
      
      const popup = window.open(
        authInit.auth_url,
        'oauth-popup',
        `width=${width},height=${height},left=${left},top=${top},scrollbars=yes,resizable=yes`
      );

      if (!popup) {
        reject(new Error('Failed to open OAuth window. Please disable popup blockers.'));
        return;
      }

      let handled = false;

      // Listen for postMessage from the popup
      const messageHandler = async (event: MessageEvent) => {
        console.log('[CLOUD_SERVICE] Received postMessage from:', event.origin);
        console.log('[CLOUD_SERVICE] Message data:', event.data);
        
        // Accept messages from the popup (could be from cloud server domain)
        if (!event.origin.startsWith('http://localhost:') && 
            !event.origin.startsWith('http://127.0.0.1:')) {
          console.log('[CLOUD_SERVICE] Ignoring message from untrusted origin');
          return;
        }
        
        // Handle OAuth success with tokens
        if (event.data?.type === 'oauth-success') {
          console.log('[CLOUD_SERVICE] OAuth success message received!');
          console.log('[CLOUD_SERVICE] Has tokens:', {
            hasAccessToken: !!event.data.accessToken,
            hasRefreshToken: !!event.data.refreshToken,
            state: event.data.state
          });
          
          if (handled) {
            console.log('[CLOUD_SERVICE] Already handled, ignoring');
            return;
          }
          handled = true;
          
          window.removeEventListener('message', messageHandler);
          clearInterval(pollTimer);
          
          // Close popup if still open
          if (popup && !popup.closed) {
            popup.close();
          }
          
          // If we have tokens, send them to the backend
          if (event.data.accessToken && event.data.refreshToken) {
            console.log('[CLOUD_SERVICE] Storing tokens locally');
            // Store tokens locally for immediate use
            localStorage.setItem('orkee_access_token', event.data.accessToken);
            localStorage.setItem('orkee_refresh_token', event.data.refreshToken);
            
            // Send tokens to backend via callback endpoint
            try {
              console.log('[CLOUD_SERVICE] Sending tokens to backend...');
              // Create a mock OAuth callback to register tokens with backend
              const callbackResponse = await apiRequest<ApiResponse<CloudAuthStatus>>('/api/cloud/auth/callback', {
                method: 'POST',
                body: JSON.stringify({
                  code: 'oauth_success_' + Date.now(),
                  state: 'oauth_success',
                  access_token: event.data.accessToken,
                  refresh_token: event.data.refreshToken
                }),
              });
              
              console.log('[CLOUD_SERVICE] Backend response:', callbackResponse);
              
              if (callbackResponse.success && callbackResponse.data) {
                console.log('[CLOUD_SERVICE] Auth successful! User:', callbackResponse.data);
                resolve(callbackResponse.data);
                return;
              } else {
                console.error('[CLOUD_SERVICE] Backend rejected tokens:', callbackResponse);
              }
            } catch (e) {
              console.error('[CLOUD_SERVICE] Failed to register tokens with backend:', e);
            }
          } else {
            console.error('[CLOUD_SERVICE] Missing tokens in OAuth success message');
          }
          
          // Fallback: refresh auth status
          console.log('[CLOUD_SERVICE] Fallback: refreshing auth status...');
          try {
            const authStatus = await this.getAuthStatus();
            console.log('[CLOUD_SERVICE] Fallback auth status:', authStatus);
            resolve(authStatus);
          } catch (error) {
            console.error('[CLOUD_SERVICE] Fallback failed:', error);
            reject(error);
          }
        } 
        // Legacy callback handling
        else if (event.data?.type === 'oauth-callback' && event.data.code && event.data.state) {
          if (handled) return;
          handled = true;
          
          window.removeEventListener('message', messageHandler);
          clearInterval(pollTimer);
          
          try {
            const authStatus = await this.handleOAuthCallback(event.data.code, event.data.state);
            resolve(authStatus);
          } catch (error) {
            reject(error);
          }
        }
        // Handle OAuth errors
        else if (event.data?.type === 'oauth-error') {
          handled = true;
          window.removeEventListener('message', messageHandler);
          clearInterval(pollTimer);
          
          if (popup && !popup.closed) {
            popup.close();
          }
          
          reject(new Error(event.data.error || 'OAuth authentication failed'));
        }
      };

      window.addEventListener('message', messageHandler);

      // Also poll localStorage as backup
      const pollTimer = setInterval(async () => {
        try {
          if (popup.closed) {
            clearInterval(pollTimer);
            window.removeEventListener('message', messageHandler);
            
            // Check localStorage for callback or success
            const storedCallback = localStorage.getItem('oauth_callback');
            const storedSuccess = localStorage.getItem('oauth_success');
            
            if (storedSuccess) {
              const { authenticated, timestamp } = JSON.parse(storedSuccess);
              // Only use if recent (within 10 seconds)
              if (authenticated && Date.now() - timestamp < 10000) {
                localStorage.removeItem('oauth_success');
                if (!handled) {
                  handled = true;
                  try {
                    const authStatus = await this.getAuthStatus();
                    resolve(authStatus);
                    return;
                  } catch (error) {
                    reject(error);
                    return;
                  }
                }
              }
            } else if (storedCallback) {
              const { code, state, timestamp } = JSON.parse(storedCallback);
              // Only use if recent (within 10 seconds)
              if (Date.now() - timestamp < 10000) {
                localStorage.removeItem('oauth_callback');
                if (!handled) {
                  handled = true;
                  try {
                    const authStatus = await this.handleOAuthCallback(code, state);
                    resolve(authStatus);
                    return;
                  } catch (error) {
                    reject(error);
                    return;
                  }
                }
              }
            }
            
            if (!handled) {
              reject(new Error('OAuth flow cancelled by user'));
            }
            return;
          }
        } catch (e) {
          // Continue polling
        }
      }, 1000);

      // Timeout after 5 minutes
      setTimeout(() => {
        clearInterval(pollTimer);
        window.removeEventListener('message', messageHandler);
        if (!popup.closed) {
          popup.close();
        }
        if (!handled) {
          reject(new Error('OAuth flow timed out'));
        }
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