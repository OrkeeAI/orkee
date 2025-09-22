import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { cloudService, CloudAuthStatus, GlobalSyncStatus } from '@/services/cloud';

// Context types
interface CloudContextType {
  // Authentication state
  authStatus: CloudAuthStatus;
  isAuthenticating: boolean;
  authError: string | null;
  
  // Sync state
  syncStatus: GlobalSyncStatus;
  isLoadingSyncStatus: boolean;
  
  // Actions
  login: () => Promise<void>;
  logout: () => Promise<void>;
  refreshAuthStatus: () => Promise<void>;
  refreshSyncStatus: () => Promise<void>;
  
  // Real-time sync updates
  subscribeSyncUpdates: () => void;
  unsubscribeSyncUpdates: () => void;
}

const CloudContext = createContext<CloudContextType | null>(null);

// Local storage key for auth persistence
const AUTH_STATUS_STORAGE_KEY = 'orkee-cloud-auth-status';
const LAST_SYNC_CHECK_KEY = 'orkee-cloud-last-sync-check';

interface CloudProviderProps {
  children: React.ReactNode;
}

export function CloudProvider({ children }: CloudProviderProps) {
  // Authentication state
  const [authStatus, setAuthStatus] = useState<CloudAuthStatus>(() => {
    // Try to restore auth status from localStorage
    try {
      const stored = localStorage.getItem(AUTH_STATUS_STORAGE_KEY);
      if (stored) {
        const parsed = JSON.parse(stored);
        // Only restore if it looks valid and recent
        if (parsed.authenticated && parsed.user_email) {
          return parsed;
        }
      }
    } catch (error) {
      console.warn('Failed to restore auth status from localStorage:', error);
    }
    
    return {
      authenticated: false,
      user_id: undefined,
      user_email: undefined,
      user_name: undefined,
      subscription_tier: undefined,
    };
  });
  
  const [isAuthenticating, setIsAuthenticating] = useState(false);
  const [authError, setAuthError] = useState<string | null>(null);
  
  // Sync state
  const [syncStatus, setSyncStatus] = useState<GlobalSyncStatus>({
    total_projects: 0,
    synced_projects: 0,
    pending_projects: 0,
    syncing_projects: 0,
    conflict_projects: 0,
    last_sync: undefined,
    is_syncing: false,
    current_sync_progress: 0.0,
  });
  
  const [isLoadingSyncStatus, setIsLoadingSyncStatus] = useState(false);
  const [syncSubscriptionActive, setSyncSubscriptionActive] = useState(false);
  
  // Persist auth status to localStorage
  useEffect(() => {
    try {
      if (authStatus.authenticated) {
        localStorage.setItem(AUTH_STATUS_STORAGE_KEY, JSON.stringify(authStatus));
      } else {
        localStorage.removeItem(AUTH_STATUS_STORAGE_KEY);
      }
    } catch (error) {
      console.warn('Failed to persist auth status to localStorage:', error);
    }
  }, [authStatus]);
  
  // Actions
  const login = useCallback(async (): Promise<void> => {
    if (isAuthenticating) {
      return;
    }
    
    setIsAuthenticating(true);
    setAuthError(null);
    
    try {
      const authResult = await cloudService.openOAuthWindow();
      setAuthStatus(authResult);
      
      // Refresh sync status after successful auth
      if (authResult.authenticated) {
        // Manually call with new auth status since state update is async
        setIsLoadingSyncStatus(true);
        try {
          const status = await cloudService.getGlobalSyncStatus();
          setSyncStatus(status);
          localStorage.setItem(LAST_SYNC_CHECK_KEY, new Date().toISOString());
        } catch (error) {
          console.error('[CloudContext] Failed to refresh sync status:', error);
        } finally {
          setIsLoadingSyncStatus(false);
        }
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Login failed';
      setAuthError(errorMessage);
      console.error('[CloudContext] Cloud login failed:', error);
    } finally {
      setIsAuthenticating(false);
    }
  }, [isAuthenticating]);
  
  const logout = useCallback(async (): Promise<void> => {
    if (isAuthenticating) return;
    
    setIsAuthenticating(true);
    setAuthError(null);
    
    try {
      await cloudService.logout();
      
      // Clear stored tokens
      localStorage.removeItem('orkee_access_token');
      localStorage.removeItem('orkee_refresh_token');
      
      setAuthStatus({
        authenticated: false,
        user_id: undefined,
        user_email: undefined,
        user_name: undefined,
        subscription_tier: undefined,
      });
      
      // Reset sync status
      setSyncStatus({
        total_projects: 0,
        synced_projects: 0,
        pending_projects: 0,
        syncing_projects: 0,
        conflict_projects: 0,
        last_sync: undefined,
        is_syncing: false,
        current_sync_progress: 0.0,
      });
      
      // Stop sync subscription
      if (syncSubscriptionActive) {
        cloudService.cleanup();
        setSyncSubscriptionActive(false);
      }
      
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Logout failed';
      setAuthError(errorMessage);
      console.error('Cloud logout failed:', error);
    } finally {
      setIsAuthenticating(false);
    }
  }, [isAuthenticating, syncSubscriptionActive]);
  
  const refreshAuthStatus = useCallback(async (): Promise<void> => {
    try {
      const status = await cloudService.getAuthStatus();
      setAuthStatus(status);
      setAuthError(null);
      
      // Clear stored auth if server says we're not authenticated
      if (!status.authenticated) {
        localStorage.removeItem(AUTH_STATUS_STORAGE_KEY);
      }
      
    } catch (error) {
      console.error('[CloudContext] Failed to refresh auth status:', error);
      // Don't show error to user for background refresh
    }
  }, []);
  
  const refreshSyncStatus = useCallback(async (): Promise<void> => {
    if (!authStatus.authenticated) {
      setSyncStatus({
        total_projects: 0,
        synced_projects: 0,
        pending_projects: 0,
        syncing_projects: 0,
        conflict_projects: 0,
        last_sync: undefined,
        is_syncing: false,
        current_sync_progress: 0.0,
      });
      return;
    }
    
    setIsLoadingSyncStatus(true);
    
    try {
      const status = await cloudService.getGlobalSyncStatus();
      setSyncStatus(status);
      
      // Update last check time
      localStorage.setItem(LAST_SYNC_CHECK_KEY, new Date().toISOString());
      
    } catch (error) {
      console.error('Failed to refresh sync status:', error);
      // Don't throw error for non-critical sync status
    } finally {
      setIsLoadingSyncStatus(false);
    }
  }, [authStatus.authenticated]);
  
  const subscribeSyncUpdates = useCallback(() => {
    if (!authStatus.authenticated || syncSubscriptionActive) return;
    
    const unsubscribe = cloudService.subscribeSyncUpdates((status) => {
      setSyncStatus(status);
    });
    
    setSyncSubscriptionActive(true);
    
    // Store unsubscribe function for cleanup
    return unsubscribe;
  }, [authStatus.authenticated, syncSubscriptionActive]);
  
  const unsubscribeSyncUpdates = useCallback(() => {
    if (syncSubscriptionActive) {
      cloudService.cleanup();
      setSyncSubscriptionActive(false);
    }
  }, [syncSubscriptionActive]);
  
  // Initialize auth status on mount
  useEffect(() => {
    refreshAuthStatus();
  }, [refreshAuthStatus]);
  
  // Listen for OAuth success events from popup
  useEffect(() => {
    const handleOAuthMessage = async (event: MessageEvent) => {
      if (event.data?.type === 'oauth-success') {
        // OAuth completed successfully, refresh auth status
        // Wait a moment for the backend to process the tokens
        setTimeout(async () => {
          await refreshAuthStatus();
          // Then refresh sync status - don't check authStatus.authenticated here
          // because it won't be updated yet (state updates are async)
          await refreshSyncStatus();
        }, 1000);
      }
    };
    
    const handleStorageChange = async (event: StorageEvent) => {
      // Check if OAuth tokens were added to localStorage
      if (event.key === 'orkee_access_token' && event.newValue) {
        // Token was added, refresh auth status
        await refreshAuthStatus();
        // Also refresh sync status
        await refreshSyncStatus();
      }
    };
    
    window.addEventListener('message', handleOAuthMessage);
    window.addEventListener('storage', handleStorageChange);
    
    return () => {
      window.removeEventListener('message', handleOAuthMessage);
      window.removeEventListener('storage', handleStorageChange);
    };
  }, [refreshAuthStatus, refreshSyncStatus]);
  
  // Auto-refresh auth status periodically
  useEffect(() => {
    const interval = setInterval(() => {
      refreshAuthStatus();
    }, 5 * 60 * 1000); // Every 5 minutes
    
    return () => clearInterval(interval);
  }, [refreshAuthStatus]);
  
  // Load sync status when authenticated
  useEffect(() => {
    if (authStatus.authenticated) {
      refreshSyncStatus();
    }
  }, [authStatus.authenticated, refreshSyncStatus]);
  
  // Auto-subscribe to sync updates when authenticated
  useEffect(() => {
    if (authStatus.authenticated && !syncSubscriptionActive) {
      subscribeSyncUpdates();
    } else if (!authStatus.authenticated && syncSubscriptionActive) {
      unsubscribeSyncUpdates();
    }
    
    return () => {
      if (syncSubscriptionActive) {
        unsubscribeSyncUpdates();
      }
    };
  }, [authStatus.authenticated, syncSubscriptionActive, subscribeSyncUpdates, unsubscribeSyncUpdates]);
  
  // Cleanup on unmount
  useEffect(() => {
    return () => {
      cloudService.cleanup();
    };
  }, []);
  
  const contextValue: CloudContextType = {
    authStatus,
    isAuthenticating,
    authError,
    syncStatus,
    isLoadingSyncStatus,
    login,
    logout,
    refreshAuthStatus,
    refreshSyncStatus,
    subscribeSyncUpdates,
    unsubscribeSyncUpdates,
  };
  
  return (
    <CloudContext.Provider value={contextValue}>
      {children}
    </CloudContext.Provider>
  );
}

// Hook to use the cloud context
export function useCloud(): CloudContextType {
  const context = useContext(CloudContext);
  if (!context) {
    throw new Error('useCloud must be used within a CloudProvider');
  }
  return context;
}

// Hook specifically for auth status
export function useCloudAuth() {
  const { authStatus, isAuthenticating, authError, login, logout, refreshAuthStatus } = useCloud();
  
  return {
    authStatus,
    isAuthenticating,
    authError,
    login,
    logout,
    refreshAuthStatus,
    isAuthenticated: authStatus.authenticated,
    user: authStatus.authenticated ? {
      id: authStatus.user_id!,
      email: authStatus.user_email!,
      name: authStatus.user_name!,
      tier: authStatus.subscription_tier!,
    } : null,
  };
}

// Hook specifically for sync status
export function useCloudSync() {
  const { syncStatus, isLoadingSyncStatus, refreshSyncStatus } = useCloud();
  
  return {
    syncStatus,
    isLoadingSyncStatus,
    refreshSyncStatus,
    hasPendingSync: syncStatus.pending_projects > 0 || syncStatus.syncing_projects > 0,
    hasConflicts: syncStatus.conflict_projects > 0,
    syncProgress: syncStatus.current_sync_progress,
    lastSync: syncStatus.last_sync,
  };
}