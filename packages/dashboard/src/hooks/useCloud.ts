import { useContext } from 'react';
import { CloudContext, CloudContextType } from '@/contexts/CloudContextTypes';

export function useCloud(): CloudContextType {
  const context = useContext(CloudContext);
  if (!context) {
    throw new Error('useCloud must be used within a CloudProvider');
  }
  return context;
}

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