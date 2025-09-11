import { useState } from 'react';
import { Button } from '@/components/ui/button';
import { Card, CardContent } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from '@/components/ui/tooltip';
import { useCloudAuth, useCloudSync } from '@/contexts/CloudContext';
import { cloudService } from '@/services/cloud';
import { formatLastSync } from '@/services/cloud';
import {
  Cloud,
  RefreshCw,
  Play,
  Pause,
  AlertTriangle,
  CheckCircle,
  Clock,
  Settings,
} from 'lucide-react';

interface GlobalSyncStatusProps {
  className?: string;
  onSyncAll?: () => void;
}

export function GlobalSyncStatus({ className, onSyncAll }: GlobalSyncStatusProps) {
  const { isAuthenticated } = useCloudAuth();
  const { syncStatus, isLoadingSyncStatus, refreshSyncStatus, hasPendingSync, hasConflicts } = useCloudSync();
  const [isSyncing, setIsSyncing] = useState(false);
  const [syncError, setSyncError] = useState<string | null>(null);

  // Don't render if not authenticated
  if (!isAuthenticated) {
    return null;
  }

  const handleSyncAll = async () => {
    setIsSyncing(true);
    setSyncError(null);
    
    try {
      const results = await cloudService.syncAllProjects();
      
      // Check for any failed syncs
      const failedSyncs = results.filter(result => !result.success);
      if (failedSyncs.length > 0) {
        setSyncError(`${failedSyncs.length} projects failed to sync`);
      }
      
      // Refresh sync status to show latest state
      await refreshSyncStatus();
      
      // Call parent callback if provided
      onSyncAll?.();
      
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Sync failed';
      setSyncError(message);
    } finally {
      setIsSyncing(false);
    }
  };

  const handleRefresh = async () => {
    await refreshSyncStatus();
  };

  // Calculate sync progress percentage
  const totalProjects = syncStatus.total_projects;
  const syncedProjects = syncStatus.synced_projects;
  const syncProgress = totalProjects > 0 ? (syncedProjects / totalProjects) * 100 : 0;
  
  // Determine overall status
  const getOverallStatus = () => {
    if (syncStatus.is_syncing || syncStatus.syncing_projects > 0) {
      return { label: 'Syncing...', color: 'text-blue-600', icon: RefreshCw, variant: 'default' as const };
    }
    if (hasConflicts) {
      return { label: 'Conflicts', color: 'text-red-600', icon: AlertTriangle, variant: 'destructive' as const };
    }
    if (hasPendingSync) {
      return { label: 'Pending', color: 'text-yellow-600', icon: Clock, variant: 'secondary' as const };
    }
    if (syncedProjects === totalProjects && totalProjects > 0) {
      return { label: 'Up to date', color: 'text-green-600', icon: CheckCircle, variant: 'secondary' as const };
    }
    return { label: 'Ready', color: 'text-gray-600', icon: Cloud, variant: 'outline' as const };
  };

  const status = getOverallStatus();
  const StatusIcon = status.icon;

  return (
    <Card className={`border-l-4 border-l-blue-500 ${className}`}>
      <CardContent className="p-4">
        <div className="flex items-center justify-between">
          {/* Left side - Status info */}
          <div className="flex items-center space-x-4">
            <div className="flex items-center space-x-2">
              <StatusIcon 
                className={`h-5 w-5 ${status.color} ${
                  (syncStatus.is_syncing || syncStatus.syncing_projects > 0) ? 'animate-spin' : ''
                }`} 
              />
              <div className="flex flex-col">
                <div className="flex items-center space-x-2">
                  <span className="font-medium">Cloud Sync</span>
                  <Badge variant={status.variant} className="text-xs">
                    {status.label}
                  </Badge>
                </div>
                <div className="flex items-center space-x-4 text-sm text-muted-foreground">
                  <span>
                    {syncedProjects} of {totalProjects} projects synced
                  </span>
                  {syncStatus.last_sync && (
                    <span>Last sync: {formatLastSync(syncStatus.last_sync)}</span>
                  )}
                </div>
              </div>
            </div>

            {/* Progress bar */}
            {totalProjects > 0 && (
              <div className="flex items-center space-x-2">
                <Progress 
                  value={syncProgress} 
                  className="w-24 h-2" 
                />
                <span className="text-xs text-muted-foreground min-w-12">
                  {Math.round(syncProgress)}%
                </span>
              </div>
            )}

            {/* Status badges */}
            <div className="flex items-center space-x-1">
              {syncStatus.syncing_projects > 0 && (
                <Badge variant="default" className="text-xs">
                  {syncStatus.syncing_projects} syncing
                </Badge>
              )}
              {syncStatus.pending_projects > 0 && (
                <Badge variant="secondary" className="text-xs">
                  {syncStatus.pending_projects} pending
                </Badge>
              )}
              {syncStatus.conflict_projects > 0 && (
                <Badge variant="destructive" className="text-xs">
                  {syncStatus.conflict_projects} conflicts
                </Badge>
              )}
            </div>
          </div>

          {/* Right side - Actions */}
          <div className="flex items-center space-x-2">
            {syncError && (
              <TooltipProvider>
                <Tooltip>
                  <TooltipTrigger>
                    <AlertTriangle className="h-4 w-4 text-red-500" />
                  </TooltipTrigger>
                  <TooltipContent>
                    <p>{syncError}</p>
                  </TooltipContent>
                </Tooltip>
              </TooltipProvider>
            )}

            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={handleRefresh}
                    disabled={isLoadingSyncStatus}
                  >
                    <RefreshCw className={`h-4 w-4 ${isLoadingSyncStatus ? 'animate-spin' : ''}`} />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Refresh sync status</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>

            {hasPendingSync && !syncStatus.is_syncing && (
              <Button
                size="sm"
                onClick={handleSyncAll}
                disabled={isSyncing || totalProjects === 0}
                className="min-w-20"
              >
                {isSyncing ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Syncing...
                  </>
                ) : (
                  <>
                    <Play className="mr-2 h-4 w-4" />
                    Sync All
                  </>
                )}
              </Button>
            )}

            {syncStatus.is_syncing && (
              <Button
                size="sm"
                variant="outline"
                onClick={() => {
                  // TODO: Implement pause sync functionality
                  console.log('Pause sync - not yet implemented');
                }}
              >
                <Pause className="mr-2 h-4 w-4" />
                Pause
              </Button>
            )}

            <TooltipProvider>
              <Tooltip>
                <TooltipTrigger asChild>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => {
                      // TODO: Navigate to cloud settings
                      console.log('Navigate to cloud settings');
                    }}
                  >
                    <Settings className="h-4 w-4" />
                  </Button>
                </TooltipTrigger>
                <TooltipContent>
                  <p>Cloud settings</p>
                </TooltipContent>
              </Tooltip>
            </TooltipProvider>
          </div>
        </div>

        {/* Detailed progress during active sync */}
        {(syncStatus.is_syncing || syncStatus.syncing_projects > 0) && (
          <div className="mt-3 pt-3 border-t">
            <div className="flex items-center justify-between">
              <span className="text-sm text-muted-foreground">
                Current sync progress
              </span>
              <span className="text-sm font-medium">
                {Math.round(syncStatus.current_sync_progress * 100)}%
              </span>
            </div>
            <Progress 
              value={syncStatus.current_sync_progress * 100} 
              className="mt-1 h-1" 
            />
          </div>
        )}
      </CardContent>
    </Card>
  );
}

// Compact version for smaller spaces
export function GlobalSyncStatusCompact() {
  const { isAuthenticated } = useCloudAuth();
  const { syncStatus, hasPendingSync, hasConflicts } = useCloudSync();

  if (!isAuthenticated) {
    return null;
  }

  const getStatusColor = () => {
    if (syncStatus.is_syncing || syncStatus.syncing_projects > 0) return 'text-blue-600';
    if (hasConflicts) return 'text-red-600';
    if (hasPendingSync) return 'text-yellow-600';
    return 'text-green-600';
  };

  const getStatusText = () => {
    if (syncStatus.is_syncing || syncStatus.syncing_projects > 0) return 'Syncing';
    if (hasConflicts) return 'Conflicts';
    if (hasPendingSync) return 'Pending';
    return 'Synced';
  };

  return (
    <div className="flex items-center space-x-2 text-sm">
      <Cloud className={`h-4 w-4 ${getStatusColor()}`} />
      <span className={getStatusColor()}>{getStatusText()}</span>
      <span className="text-muted-foreground">
        {syncStatus.synced_projects}/{syncStatus.total_projects}
      </span>
    </div>
  );
}