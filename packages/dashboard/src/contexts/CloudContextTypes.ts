import { createContext } from 'react';
import { CloudAuthStatus, GlobalSyncStatus } from '@/services/cloud';

export interface CloudContextType {
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

export const CloudContext = createContext<CloudContextType | null>(null);