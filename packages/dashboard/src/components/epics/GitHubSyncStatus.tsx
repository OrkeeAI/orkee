// ABOUTME: GitHub synchronization status component for Epics
// ABOUTME: Displays sync status, manual sync buttons, and error messages

import { useState } from 'react';
import { Github, RefreshCw, AlertCircle, CheckCircle2, Clock, XCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { epicsService, type Epic, type GitHubSyncStatus as SyncStatus } from '@/services/epics';
import { useToast } from '@/hooks/use-toast';

interface GitHubSyncStatusProps {
  epic: Epic;
  onSyncSuccess?: () => void;
}

export function GitHubSyncStatus({ epic, onSyncSuccess }: GitHubSyncStatusProps) {
  const [isSyncingEpic, setIsSyncingEpic] = useState(false);
  const [isSyncingTasks, setIsSyncingTasks] = useState(false);
  const { toast } = useToast();

  const getSyncStatusBadge = (status?: SyncStatus) => {
    if (!epic.githubIssueNumber) {
      return (
        <Badge variant="outline" className="gap-1">
          <Clock className="h-3 w-3" />
          Not Synced
        </Badge>
      );
    }

    if (epic.githubSyncedAt) {
      return (
        <Badge variant="default" className="gap-1 bg-green-600">
          <CheckCircle2 className="h-3 w-3" />
          Synced
        </Badge>
      );
    }

    return (
      <Badge variant="secondary" className="gap-1">
        <Clock className="h-3 w-3" />
        Unknown
      </Badge>
    );
  };

  const handleSyncEpic = async () => {
    setIsSyncingEpic(true);
    try {
      const result = await epicsService.syncEpicToGitHub(epic.id, !epic.githubIssueNumber);

      toast({
        title: 'Epic synced successfully',
        description: `Created/updated GitHub issue #${result.issue_number}`,
      });

      if (onSyncSuccess) {
        onSyncSuccess();
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to sync epic';
      toast({
        title: 'Sync failed',
        description: errorMessage,
        variant: 'destructive',
      });
    } finally {
      setIsSyncingEpic(false);
    }
  };

  const handleSyncTasks = async () => {
    setIsSyncingTasks(true);
    try {
      const results = await epicsService.syncTasksToGitHub(epic.id);

      toast({
        title: 'Tasks synced successfully',
        description: `Created ${results.length} GitHub issues for tasks`,
      });

      if (onSyncSuccess) {
        onSyncSuccess();
      }
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to sync tasks';
      toast({
        title: 'Sync failed',
        description: errorMessage,
        variant: 'destructive',
      });
    } finally {
      setIsSyncingTasks(false);
    }
  };

  const formatSyncTime = (dateString?: string) => {
    if (!dateString) return 'Never';
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minutes ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours} hours ago`;
    const diffDays = Math.floor(diffHours / 24);
    return `${diffDays} days ago`;
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Github className="h-5 w-5" />
            <CardTitle>GitHub Sync</CardTitle>
          </div>
          {getSyncStatusBadge()}
        </div>
        <CardDescription>
          Synchronize this epic and its tasks with GitHub Issues
        </CardDescription>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Epic Sync Section */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium">Epic Issue</p>
              <p className="text-xs text-muted-foreground">
                {epic.githubIssueNumber
                  ? `GitHub Issue #${epic.githubIssueNumber}`
                  : 'Not synced to GitHub yet'}
              </p>
            </div>
            {epic.githubIssueUrl && (
              <a
                href={epic.githubIssueUrl}
                target="_blank"
                rel="noopener noreferrer"
                className="text-sm text-primary hover:underline flex items-center gap-1"
              >
                View on GitHub
                <Github className="h-3 w-3" />
              </a>
            )}
          </div>

          <Button
            onClick={handleSyncEpic}
            disabled={isSyncingEpic}
            className="w-full"
            variant={epic.githubIssueNumber ? 'outline' : 'default'}
          >
            {isSyncingEpic ? (
              <>
                <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                Syncing...
              </>
            ) : (
              <>
                <Github className="mr-2 h-4 w-4" />
                {epic.githubIssueNumber ? 'Update Epic on GitHub' : 'Create Epic on GitHub'}
              </>
            )}
          </Button>

          {epic.githubSyncedAt && (
            <p className="text-xs text-muted-foreground text-center">
              Last synced: {formatSyncTime(epic.githubSyncedAt)}
            </p>
          )}
        </div>

        {/* Task Sync Section */}
        {epic.githubIssueNumber && (
          <>
            <div className="border-t pt-4 space-y-3">
              <div>
                <p className="text-sm font-medium">Task Issues</p>
                <p className="text-xs text-muted-foreground">
                  Create GitHub issues for all tasks in this epic
                </p>
              </div>

              <Button
                onClick={handleSyncTasks}
                disabled={isSyncingTasks}
                className="w-full"
                variant="outline"
              >
                {isSyncingTasks ? (
                  <>
                    <RefreshCw className="mr-2 h-4 w-4 animate-spin" />
                    Syncing Tasks...
                  </>
                ) : (
                  <>
                    <Github className="mr-2 h-4 w-4" />
                    Sync Tasks to GitHub
                  </>
                )}
              </Button>
            </div>

            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription className="text-xs">
                Task issues will be linked to the Epic issue and tagged appropriately.
                Existing task issues will not be re-created.
              </AlertDescription>
            </Alert>
          </>
        )}

        {/* Configuration Warning */}
        {!epic.githubIssueNumber && (
          <Alert>
            <AlertCircle className="h-4 w-4" />
            <AlertDescription className="text-xs">
              Make sure GitHub integration is configured in project settings before syncing.
              You'll need a GitHub personal access token with repo permissions.
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
}
