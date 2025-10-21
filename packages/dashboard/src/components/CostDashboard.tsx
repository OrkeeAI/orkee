// ABOUTME: AI usage cost tracking dashboard displaying costs, tokens, and usage statistics
// ABOUTME: Shows breakdowns by operation, model, and provider with detailed log viewing

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  RefreshCw,
  DollarSign,
  Activity,
  Zap,
  Clock,
  Loader2,
  AlertCircle,
} from 'lucide-react';
import { useAiUsageStats, useAiUsageLogs } from '@/hooks/useAiUsage';
import { formatCost, formatTokens, formatDuration } from '@/services/aiUsage';

interface CostDashboardProps {
  projectId?: string;
}

export function CostDashboard({ projectId }: CostDashboardProps) {
  const [selectedTab, setSelectedTab] = useState<string>('operations');

  const { data: stats, isLoading: statsLoading, refetch: refetchStats } = useAiUsageStats({
    projectId,
  });

  const { data: logs = [], isLoading: logsLoading } = useAiUsageLogs({
    projectId,
    limit: 50,
  });

  const handleRefresh = () => {
    refetchStats();
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold tracking-tight">AI Usage & Cost Dashboard</h2>
          <p className="text-muted-foreground">
            Monitor AI API usage, costs, and performance across all operations
          </p>
        </div>
        <Button onClick={handleRefresh} variant="outline" size="sm">
          <RefreshCw className="mr-2 h-4 w-4" />
          Refresh
        </Button>
      </div>

      {/* Summary Stats */}
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Cost</CardTitle>
            <DollarSign className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats ? formatCost(stats.totalCost) : '$0.00'}
            </div>
            <p className="text-xs text-muted-foreground">
              {stats?.totalRequests || 0} requests
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Total Tokens</CardTitle>
            <Zap className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats ? formatTokens(stats.totalTokens) : '0'}
            </div>
            <p className="text-xs text-muted-foreground">
              {stats ? formatTokens(stats.totalInputTokens) : '0'} in /{' '}
              {stats ? formatTokens(stats.totalOutputTokens) : '0'} out
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Success Rate</CardTitle>
            <Activity className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats && stats.totalRequests > 0
                ? Math.round((stats.successfulRequests / stats.totalRequests) * 100)
                : 0}
              %
            </div>
            <p className="text-xs text-muted-foreground">
              {stats?.failedRequests || 0} failed
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Avg Duration</CardTitle>
            <Clock className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats ? formatDuration(stats.averageDurationMs) : '0ms'}
            </div>
            <p className="text-xs text-muted-foreground">Per request</p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs value={selectedTab} onValueChange={setSelectedTab}>
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="operations">By Operation</TabsTrigger>
          <TabsTrigger value="models">By Model</TabsTrigger>
          <TabsTrigger value="providers">By Provider</TabsTrigger>
          <TabsTrigger value="logs">Recent Logs</TabsTrigger>
        </TabsList>

        {/* Operations Tab */}
        <TabsContent value="operations" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Usage by Operation</CardTitle>
              <CardDescription>
                Breakdown of AI usage and costs by operation type
              </CardDescription>
            </CardHeader>
            <CardContent>
              {statsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : !stats || !stats.byOperation || stats.byOperation.length === 0 ? (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertTitle>No data yet</AlertTitle>
                  <AlertDescription>
                    AI usage data will appear here once you start using AI features.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {stats.byOperation.map((op) => {
                    const maxCost = Math.max(...stats.byOperation.map(o => o.totalCost));
                    const costPercentage = (op.totalCost / maxCost) * 100;

                    return (
                      <div
                        key={op.operation}
                        className="rounded-lg border p-4 space-y-3"
                      >
                        <div className="flex items-center justify-between gap-4">
                          <div className="flex-1 space-y-1">
                            <div className="flex items-center gap-2">
                              <h4 className="font-medium">{op.operation}</h4>
                              <Badge variant="outline">{op.count} calls</Badge>
                            </div>
                            <div className="flex items-center gap-4 text-sm text-muted-foreground">
                              <span>{formatTokens(op.totalTokens)} tokens</span>
                              <Separator orientation="vertical" className="h-4" />
                              <span className="font-medium">{formatCost(op.totalCost)}</span>
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="text-2xl font-bold">{formatCost(op.totalCost)}</div>
                            <p className="text-xs text-muted-foreground">
                              {formatCost(op.totalCost / op.count)} avg
                            </p>
                          </div>
                        </div>

                        {/* Visual cost bar */}
                        <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
                          <div
                            className="bg-primary h-full transition-all duration-500 ease-out"
                            style={{ width: `${costPercentage}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Models Tab */}
        <TabsContent value="models" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Usage by Model</CardTitle>
              <CardDescription>
                Breakdown of AI usage and costs by model
              </CardDescription>
            </CardHeader>
            <CardContent>
              {statsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : !stats || !stats.byModel || stats.byModel.length === 0 ? (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertTitle>No data yet</AlertTitle>
                  <AlertDescription>
                    Model usage data will appear here once you start using AI features.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {stats.byModel.map((model) => {
                    const maxCost = Math.max(...stats.byModel.map(m => m.totalCost));
                    const costPercentage = (model.totalCost / maxCost) * 100;

                    return (
                      <div
                        key={model.model}
                        className="rounded-lg border p-4 space-y-3"
                      >
                        <div className="flex items-center justify-between gap-4">
                          <div className="flex-1 space-y-1">
                            <div className="flex items-center gap-2">
                              <h4 className="font-medium">{model.model}</h4>
                              <Badge variant="outline">{model.count} calls</Badge>
                            </div>
                            <div className="flex items-center gap-4 text-sm text-muted-foreground">
                              <span>{formatTokens(model.totalTokens)} tokens</span>
                              <Separator orientation="vertical" className="h-4" />
                              <span className="font-medium">{formatCost(model.totalCost)}</span>
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="text-2xl font-bold">{formatCost(model.totalCost)}</div>
                            <p className="text-xs text-muted-foreground">
                              {formatCost(model.totalCost / model.count)} avg
                            </p>
                          </div>
                        </div>

                        {/* Visual cost bar */}
                        <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
                          <div
                            className="bg-primary h-full transition-all duration-500 ease-out"
                            style={{ width: `${costPercentage}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Providers Tab */}
        <TabsContent value="providers" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Usage by Provider</CardTitle>
              <CardDescription>
                Breakdown of AI usage and costs by provider
              </CardDescription>
            </CardHeader>
            <CardContent>
              {statsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : !stats || !stats.byProvider || stats.byProvider.length === 0 ? (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertTitle>No data yet</AlertTitle>
                  <AlertDescription>
                    Provider usage data will appear here once you start using AI features.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {stats.byProvider.map((provider) => {
                    const maxCost = Math.max(...stats.byProvider.map(p => p.totalCost));
                    const costPercentage = (provider.totalCost / maxCost) * 100;

                    return (
                      <div
                        key={provider.provider}
                        className="rounded-lg border p-4 space-y-3"
                      >
                        <div className="flex items-center justify-between gap-4">
                          <div className="flex-1 space-y-1">
                            <div className="flex items-center gap-2">
                              <h4 className="font-medium">{provider.provider}</h4>
                              <Badge variant="outline">{provider.count} calls</Badge>
                            </div>
                            <div className="flex items-center gap-4 text-sm text-muted-foreground">
                              <span>{formatTokens(provider.totalTokens)} tokens</span>
                              <Separator orientation="vertical" className="h-4" />
                              <span className="font-medium">{formatCost(provider.totalCost)}</span>
                            </div>
                          </div>
                          <div className="text-right">
                            <div className="text-2xl font-bold">{formatCost(provider.totalCost)}</div>
                            <p className="text-xs text-muted-foreground">
                              {formatCost(provider.totalCost / provider.count)} avg
                            </p>
                          </div>
                        </div>

                        {/* Visual cost bar */}
                        <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
                          <div
                            className="bg-primary h-full transition-all duration-500 ease-out"
                            style={{ width: `${costPercentage}%` }}
                          />
                        </div>
                      </div>
                    );
                  })}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Recent Logs Tab */}
        <TabsContent value="logs" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Recent API Calls</CardTitle>
              <CardDescription>
                Detailed log of recent AI API requests
              </CardDescription>
            </CardHeader>
            <CardContent>
              {logsLoading ? (
                <div className="flex items-center justify-center py-8">
                  <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
                </div>
              ) : !Array.isArray(logs) || logs.length === 0 ? (
                <Alert>
                  <AlertCircle className="h-4 w-4" />
                  <AlertTitle>No logs yet</AlertTitle>
                  <AlertDescription>
                    API call logs will appear here once you start using AI features.
                  </AlertDescription>
                </Alert>
              ) : (
                <div className="space-y-3">
                  {logs.map((log) => (
                    <div
                      key={log.id}
                      className="rounded-lg border p-4 space-y-2"
                    >
                      <div className="flex items-center justify-between">
                        <div className="flex items-center gap-2">
                          <h4 className="font-medium">{log.operation}</h4>
                          {log.error ? (
                            <Badge variant="destructive">Failed</Badge>
                          ) : (
                            <Badge variant="default">Success</Badge>
                          )}
                        </div>
                        <div className="text-sm font-medium">{formatCost(log.estimatedCost || 0)}</div>
                      </div>

                      <div className="flex items-center gap-4 text-sm text-muted-foreground">
                        <span className="flex items-center gap-1">
                          <span className="font-medium">{log.model}</span>
                          <span>({log.provider})</span>
                        </span>
                        <Separator orientation="vertical" className="h-4" />
                        <span>{formatTokens(log.totalTokens || 0)} tokens</span>
                        <Separator orientation="vertical" className="h-4" />
                        <span>{formatDuration(log.durationMs || 0)}</span>
                        <Separator orientation="vertical" className="h-4" />
                        <span>{new Date(log.createdAt).toLocaleString()}</span>
                      </div>

                      {log.error && (
                        <Alert variant="destructive">
                          <AlertCircle className="h-4 w-4" />
                          <AlertDescription className="text-xs">{log.error}</AlertDescription>
                        </Alert>
                      )}
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
