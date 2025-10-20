import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  RefreshCw,
  AlertTriangle,
  CheckCircle2,
  Clock,
  Link as LinkIcon,
  FileText,
  Loader2,
  GitBranch,
  TrendingUp,
} from 'lucide-react';
import { useOrphanTasks } from '@/hooks/useTaskSpecLinks';
import { usePRDs, useSyncSpecsToPRD } from '@/hooks/usePRDs';
import { useSpecs } from '@/hooks/useSpecs';

interface SyncDashboardProps {
  projectId: string;
}

export function SyncDashboard({ projectId }: SyncDashboardProps) {
  const [selectedTab, setSelectedTab] = useState<string>('orphans');

  const { data: orphanData, isLoading: orphansLoading, refetch: refetchOrphans } = useOrphanTasks(projectId);
  const { data: prds = [], isLoading: prdsLoading } = usePRDs(projectId);
  const { data: specs = [], isLoading: specsLoading } = useSpecs(projectId);
  const syncSpecsMutation = useSyncSpecsToPRD(projectId);

  const orphanTasks = orphanData?.orphanTasks || [];
  const orphanCount = orphanData?.count || 0;

  const handleSyncToPRD = async (prdId: string) => {
    try {
      await syncSpecsMutation.mutateAsync(prdId);
    } catch (error) {
      console.error('Failed to sync specs to PRD:', error);
    }
  };

  const isLoading = orphansLoading || prdsLoading || specsLoading;

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">Spec Sync Dashboard</h2>
          <p className="text-muted-foreground">
            Manage orphan tasks, track sync status, and maintain spec-task alignment
          </p>
        </div>
        <Button
          onClick={() => {
            refetchOrphans();
          }}
          variant="outline"
          size="sm"
        >
          <RefreshCw className="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      {/* Summary Stats */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Orphan Tasks</CardTitle>
            <AlertTriangle className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{orphanCount}</div>
            <p className="text-xs text-muted-foreground">Tasks without spec links</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Active Specs</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{specs.filter(s => s.status === 'active').length}</div>
            <p className="text-xs text-muted-foreground">Total capabilities</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">PRDs</CardTitle>
            <GitBranch className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{prds.length}</div>
            <p className="text-xs text-muted-foreground">Requirements documents</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Coverage</CardTitle>
            <TrendingUp className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {specs.length > 0 ? Math.round((specs.reduce((sum, s) => sum + s.requirementCount, 0) / specs.length)) : 0}
            </div>
            <p className="text-xs text-muted-foreground">Avg requirements/spec</p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs value={selectedTab} onValueChange={setSelectedTab}>
        <TabsList className="grid w-full grid-cols-3">
          <TabsTrigger value="orphans">
            Orphan Tasks
            {orphanCount > 0 && (
              <Badge variant="destructive" className="ml-2">
                {orphanCount}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="sync">PRD Sync Status</TabsTrigger>
          <TabsTrigger value="specs">Spec Overview</TabsTrigger>
        </TabsList>

        {/* Orphan Tasks Tab */}
        <TabsContent value="orphans" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Orphan Tasks</CardTitle>
              <CardDescription>
                Tasks that are not linked to any spec requirements. Link them to maintain traceability.
              </CardDescription>
            </CardHeader>
            <CardContent>
              {orphansLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : orphanTasks.length === 0 ? (
                <Alert>
                  <CheckCircle2 className="h-4 w-4" />
                  <AlertTitle>All tasks linked!</AlertTitle>
                  <AlertDescription>
                    All tasks in this project are linked to spec requirements. Great job maintaining spec coverage!
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {orphanTasks.map((task) => (
                    <div
                      key={task.id}
                      className="flex items-start justify-between gap-4 rounded-lg border p-4 hover:bg-muted/50 transition-colors"
                    >
                      <div className="flex-1 space-y-1">
                        <div className="flex items-center gap-2">
                          <h4 className="font-medium">{task.title}</h4>
                          <Badge variant="outline">{task.status}</Badge>
                          <Badge variant={task.priority === 'high' ? 'destructive' : 'secondary'}>
                            {task.priority}
                          </Badge>
                        </div>
                        <p className="text-sm text-muted-foreground">
                          Created {new Date(task.createdAt).toLocaleDateString()}
                        </p>
                      </div>
                      <Button size="sm" variant="outline">
                        <LinkIcon className="mr-2 h-4 w-4" />
                        Link to Spec
                      </Button>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* PRD Sync Status Tab */}
        <TabsContent value="sync" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>PRD Sync Status</CardTitle>
              <CardDescription>
                Synchronize specs back to PRDs to keep requirements documents up to date with implementation.
              </CardDescription>
            </CardHeader>
            <CardContent>
              {prdsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : prds.length === 0 ? (
                <Alert>
                  <AlertTitle>No PRDs found</AlertTitle>
                  <AlertDescription>
                    Create a PRD to start tracking product requirements and syncing with specs.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {prds.map((prd) => {
                    const linkedSpecs = specs.filter(s => s.prdId === prd.id);
                    const totalRequirements = linkedSpecs.reduce((sum, s) => sum + s.requirementCount, 0);

                    return (
                      <div
                        key={prd.id}
                        className="flex items-start justify-between gap-4 rounded-lg border p-4"
                      >
                        <div className="flex-1 space-y-2">
                          <div className="flex items-center gap-2">
                            <h4 className="font-medium">{prd.title}</h4>
                            <Badge variant="outline">v{prd.version}</Badge>
                            <Badge variant={prd.status === 'approved' ? 'default' : 'secondary'}>
                              {prd.status}
                            </Badge>
                          </div>
                          <div className="flex items-center gap-4 text-sm text-muted-foreground">
                            <span className="flex items-center gap-1">
                              <FileText className="h-3 w-3" />
                              {linkedSpecs.length} spec{linkedSpecs.length !== 1 ? 's' : ''}
                            </span>
                            <span className="flex items-center gap-1">
                              <CheckCircle2 className="h-3 w-3" />
                              {totalRequirements} requirement{totalRequirements !== 1 ? 's' : ''}
                            </span>
                            <span className="flex items-center gap-1">
                              <Clock className="h-3 w-3" />
                              Updated {new Date(prd.updatedAt).toLocaleDateString()}
                            </span>
                          </div>
                        </div>
                        <Button
                          size="sm"
                          onClick={() => handleSyncToPRD(prd.id)}
                          disabled={syncSpecsMutation.isPending || linkedSpecs.length === 0}
                        >
                          {syncSpecsMutation.isPending ? (
                            <>
                              <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                              Syncing...
                            </>
                          ) : (
                            <>
                              <RefreshCw className="mr-2 h-4 w-4" />
                              Sync Specs to PRD
                            </>
                          )}
                        </Button>
                      </div>
                    );
                  })}
                </div>
              )}
            </CardContent>
          </Card>

          {syncSpecsMutation.isSuccess && (
            <Alert>
              <CheckCircle2 className="h-4 w-4" />
              <AlertTitle>Sync complete!</AlertTitle>
              <AlertDescription>
                Specs have been successfully synchronized to the PRD.
              </AlertDescription>
            </Alert>
          )}

          {syncSpecsMutation.isError && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>Sync failed</AlertTitle>
              <AlertDescription>
                {syncSpecsMutation.error instanceof Error
                  ? syncSpecsMutation.error.message
                  : 'Failed to sync specs to PRD'}
              </AlertDescription>
            </Alert>
          )}
        </TabsContent>

        {/* Spec Overview Tab */}
        <TabsContent value="specs" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Spec Capabilities Overview</CardTitle>
              <CardDescription>
                View all spec capabilities and their current status.
              </CardDescription>
            </CardHeader>
            <CardContent>
              {specsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : specs.length === 0 ? (
                <Alert>
                  <AlertTitle>No specs found</AlertTitle>
                  <AlertDescription>
                    Create your first spec capability to start defining requirements and generating tasks.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {specs.map((spec) => (
                    <div
                      key={spec.id}
                      className="rounded-lg border p-4 space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <h4 className="font-medium">{spec.name}</h4>
                          <Badge variant="outline">v{spec.version}</Badge>
                          <Badge
                            variant={
                              spec.status === 'active'
                                ? 'default'
                                : spec.status === 'deprecated'
                                ? 'secondary'
                                : 'outline'
                            }
                          >
                            {spec.status}
                          </Badge>
                        </div>
                      </div>

                      <p className="text-sm text-muted-foreground line-clamp-2">{spec.purpose}</p>

                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span>{spec.requirementCount} requirement{spec.requirementCount !== 1 ? 's' : ''}</span>
                        <Separator orientation="vertical" className="h-4" />
                        <span>Updated {new Date(spec.updatedAt).toLocaleDateString()}</span>
                        {spec.prdId && (
                          <>
                            <Separator orientation="vertical" className="h-4" />
                            <span className="flex items-center gap-1">
                              <LinkIcon className="h-3 w-3" />
                              Linked to PRD
                            </span>
                          </>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
