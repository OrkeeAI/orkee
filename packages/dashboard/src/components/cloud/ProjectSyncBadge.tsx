import { useState, useEffect } from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useCloudAuth } from '@/hooks/useCloud';
import { cloudService, ProjectSyncStatus, formatSyncStatus, formatLastSync } from '@/services/cloud';
import {
  RefreshCw,
  AlertTriangle,
  Cloud,
} from 'lucide-react';

interface ProjectSyncBadgeProps {
  projectId: string;
  variant?: 'default' | 'compact' | 'detailed';
  className?: string;
  onSyncComplete?: () => void;
}

export function ProjectSyncBadge({ 
  projectId, 
  variant = 'default', 
  className,
  onSyncComplete 
}: ProjectSyncBadgeProps) {
  const { isAuthenticated } = useCloudAuth();
  const [syncStatus, setSyncStatus] = useState<ProjectSyncStatus | null>(null);
  const [isLoading, setIsLoading] = useState(false);
  const [isSyncing, setIsSyncing] = useState(false);

  // Fetch sync status
  const fetchSyncStatus = async () => {
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

  // Initial load
  useEffect(() => {
    fetchSyncStatus();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [isAuthenticated, projectId]);

  // Handle sync action
  const handleSync = async (e: React.MouseEvent) => {
    e.stopPropagation(); // Prevent triggering parent click events
    
    if (!isAuthenticated || isSyncing) return;
    
    setIsSyncing(true);
    try {
      const result = await cloudService.syncProject(projectId);
      
      if (result.success) {
        // Refresh status after successful sync
        await fetchSyncStatus();
        onSyncComplete?.();
      } else {
        console.error('Sync failed:', result.message);
      }
    } catch (error) {
      console.error('Sync error:', error);
    } finally {
      setIsSyncing(false);
    }
  };

  // Don't render if not authenticated
  if (!isAuthenticated) {
    return null;
  }

  // Loading state
  if (isLoading && !syncStatus) {
    return (
      <div className={`flex items-center space-x-1 ${className}`}>
        <RefreshCw className="h-3 w-3 animate-spin text-gray-400" />
        {variant !== 'compact' && (
          <span className="text-xs text-gray-400">Loading...</span>
        )}
      </div>
    );
  }

  // No sync status available
  if (!syncStatus) {
    return null;
  }

  const statusInfo = formatSyncStatus(syncStatus.status);
  
  // Compact variant - just the badge
  if (variant === 'compact') {
    return (
      <TooltipProvider>
        <Tooltip>
          <TooltipTrigger asChild>
            <Badge
              variant={syncStatus.status === 'synced' ? 'secondary' : 
                      syncStatus.status === 'conflict' || syncStatus.status === 'error' ? 'destructive' : 'outline'}
              className={`text-xs ${statusInfo.color} ${className}`}
            >
              <span className="mr-1">{statusInfo.icon}</span>
              {statusInfo.label}
            </Badge>
          </TooltipTrigger>
          <TooltipContent>
            <div className="text-sm">
              <p>Status: {statusInfo.label}</p>
              {syncStatus.last_sync && (
                <p>Last sync: {formatLastSync(syncStatus.last_sync)}</p>
              )}
              {syncStatus.error_message && (
                <p className="text-red-400">Error: {syncStatus.error_message}</p>
              )}
            </div>
          </TooltipContent>
        </Tooltip>
      </TooltipProvider>
    );
  }

  // Detailed variant - full sync info and actions
  if (variant === 'detailed') {
    return (
      <div className={`flex items-center justify-between p-2 border rounded-md bg-gray-50 ${className}`}>
        <div className="flex items-center space-x-2">
          <Cloud className="h-4 w-4 text-blue-500" />
          <div className="flex flex-col">
            <div className="flex items-center space-x-2">
              <Badge
                variant={syncStatus.status === 'synced' ? 'secondary' : 
                        syncStatus.status === 'conflict' || syncStatus.status === 'error' ? 'destructive' : 'outline'}
                className="text-xs"
              >
                <span className="mr-1">{statusInfo.icon}</span>
                {statusInfo.label}
              </Badge>
              {syncStatus.has_conflicts && (
                <Badge variant="destructive" className="text-xs">
                  <AlertTriangle className="mr-1 h-3 w-3" />
                  Conflicts
                </Badge>
              )}
            </div>
            <div className="text-xs text-muted-foreground">
              {syncStatus.last_sync ? (
                `Last sync: ${formatLastSync(syncStatus.last_sync)}`
              ) : (
                'Never synced'
              )}
            </div>
            {syncStatus.error_message && (
              <div className="text-xs text-red-600">
                {syncStatus.error_message}
              </div>
            )}
          </div>
        </div>

        <div className="flex items-center space-x-1">
          {syncStatus.sync_progress !== undefined && (
            <div className="flex items-center space-x-1">
              <div className="w-12 h-1 bg-gray-200 rounded-full overflow-hidden">
                <div 
                  className="h-full bg-blue-500 transition-all duration-300"
                  style={{ width: `${(syncStatus.sync_progress || 0) * 100}%` }}
                />
              </div>
              <span className="text-xs text-muted-foreground">
                {Math.round((syncStatus.sync_progress || 0) * 100)}%
              </span>
            </div>
          )}
          
          <Button
            size="sm"
            variant="ghost"
            onClick={handleSync}
            disabled={isSyncing || syncStatus.status === 'syncing'}
            className="h-6 px-2"
          >
            <RefreshCw className={`h-3 w-3 ${isSyncing || syncStatus.status === 'syncing' ? 'animate-spin' : ''}`} />
          </Button>
        </div>
      </div>
    );
  }

  // Default variant - badge with optional sync button
  return (
    <div className={`flex items-center space-x-1 ${className}`}>
      <Badge
        variant={syncStatus.status === 'synced' ? 'secondary' : 
                syncStatus.status === 'conflict' || syncStatus.status === 'error' ? 'destructive' : 'outline'}
        className={`text-xs ${statusInfo.color}`}
      >
        <span className="mr-1">{statusInfo.icon}</span>
        {statusInfo.label}
      </Badge>
      
      {/* Show sync button for pending/error states */}
      {(syncStatus.status === 'pending' || syncStatus.status === 'error') && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <Button
                size="sm"
                variant="ghost"
                onClick={handleSync}
                disabled={isSyncing}
                className="h-5 w-5 p-0"
              >
                <RefreshCw className={`h-3 w-3 ${isSyncing ? 'animate-spin' : ''}`} />
              </Button>
            </TooltipTrigger>
            <TooltipContent>
              <p>Sync project</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}

      {/* Show conflict indicator */}
      {syncStatus.has_conflicts && (
        <TooltipProvider>
          <Tooltip>
            <TooltipTrigger asChild>
              <AlertTriangle className="h-3 w-3 text-red-500" />
            </TooltipTrigger>
            <TooltipContent>
              <p>This project has sync conflicts that need resolution</p>
            </TooltipContent>
          </Tooltip>
        </TooltipProvider>
      )}
    </div>
  );
}