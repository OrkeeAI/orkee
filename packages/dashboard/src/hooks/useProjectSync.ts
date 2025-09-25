import { useState, useEffect } from 'react';
import { useCloudAuth } from '@/hooks/useCloud';
import { cloudService, ProjectSyncStatus } from '@/services/cloud';

export function useProjectSync(projectId: string) {
  const { isAuthenticated } = useCloudAuth();
  const [syncStatus, setSyncStatus] = useState<ProjectSyncStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);

  const fetchStatus = async () => {
    if (!isAuthenticated) return;
    
    setIsLoading(true);
    try {
      const status = await cloudService.getProjectSyncStatus(projectId);
      setSyncStatus(status);
    } catch (error) {
      console.error('Failed to fetch project sync status:', error);
    } finally {
      setIsLoading(false);
    }
  };

  const syncProject = async () => {
    if (!isAuthenticated) return false;
    
    try {
      const result = await cloudService.syncProject(projectId);
      if (result.success) {
        await fetchStatus(); // Refresh status
        return true;
      }
      return false;
    } catch (error) {
      console.error('Sync failed:', error);
      return false;
    }
  };

  useEffect(() => {
    fetchStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isAuthenticated, projectId]);

  return {
    syncStatus,
    isLoading,
    fetchStatus,
    syncProject,
    isAuthenticated,
  };
}