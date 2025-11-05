// ABOUTME: Component for viewing past executions of a task
// ABOUTME: Shows execution history with filtering, status indicators, and quick actions

import { useState } from 'react';
import { useQuery } from '@tanstack/react-query';
import {
  History,
  Clock,
  CheckCircle,
  XCircle,
  Circle,
  Filter,
  RefreshCw,
  Eye,
  Download,
  Calendar,
  Loader2,
} from 'lucide-react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { executionsService, type AgentExecution } from '@/services/executions';
import { formatDistanceToNow } from 'date-fns';

interface ExecutionHistoryProps {
  taskId: string;
  onViewExecution?: (executionId: string) => void;
  onRetryExecution?: (execution: AgentExecution) => void;
}

export function ExecutionHistory({
  taskId,
  onViewExecution,
  onRetryExecution,
}: ExecutionHistoryProps) {
  const [statusFilter, setStatusFilter] = useState<string>('all');

  // Fetch execution history
  const {
    data: executionsData,
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['executions', taskId],
    queryFn: () => executionsService.listExecutions(taskId),
  });

  const executions = executionsData?.items || [];

  // Filter executions
  const filteredExecutions = executions.filter((execution) => {
    if (statusFilter !== 'all' && execution.status !== statusFilter) {
      return false;
    }
    return true;
  });

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'running':
        return <Loader2 className="h-4 w-4 animate-spin" />;
      case 'succeeded':
        return <CheckCircle className="h-4 w-4 text-green-600" />;
      case 'failed':
        return <XCircle className="h-4 w-4 text-red-600" />;
      case 'cancelled':
        return <Circle className="h-4 w-4 text-gray-500" />;
      case 'pending':
        return <Clock className="h-4 w-4 text-yellow-600" />;
      default:
        return <Circle className="h-4 w-4" />;
    }
  };

  const getStatusBadgeVariant = (status: string): "default" | "secondary" | "destructive" | "outline" => {
    switch (status) {
      case 'running':
        return 'default';
      case 'succeeded':
        return 'default';
      case 'failed':
        return 'destructive';
      case 'cancelled':
        return 'secondary';
      case 'pending':
        return 'outline';
      default:
        return 'outline';
    }
  };

  const formatDuration = (seconds: number | null) => {
    if (!seconds) return 'N/A';
    const minutes = Math.floor(seconds / 60);
    const remainingSeconds = seconds % 60;
    if (minutes > 0) {
      return `${minutes}m ${remainingSeconds}s`;
    }
    return `${remainingSeconds}s`;
  };

  const formatCost = (cost: number | null) => {
    if (!cost) return 'N/A';
    return `$${cost.toFixed(4)}`;
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  if (error) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12 text-center">
          <XCircle className="h-12 w-12 text-destructive mb-4" />
          <p className="text-muted-foreground">
            {error instanceof Error ? error.message : 'Failed to load execution history'}
          </p>
          <Button variant="outline" onClick={() => refetch()} className="mt-4">
            <RefreshCw className="mr-2 h-4 w-4" />
            Retry
          </Button>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-start justify-between">
          <div className="space-y-1">
            <CardTitle className="flex items-center gap-2">
              <History className="h-5 w-5" />
              Execution History
            </CardTitle>
            <CardDescription>
              Past execution attempts for this task
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-[150px]">
                <Filter className="mr-2 h-4 w-4" />
                <SelectValue placeholder="Filter status" />
              </SelectTrigger>
              <SelectContent>
                <SelectItem value="all">All Status</SelectItem>
                <SelectItem value="running">Running</SelectItem>
                <SelectItem value="succeeded">Succeeded</SelectItem>
                <SelectItem value="failed">Failed</SelectItem>
                <SelectItem value="cancelled">Cancelled</SelectItem>
                <SelectItem value="pending">Pending</SelectItem>
              </SelectContent>
            </Select>
            <Button variant="outline" size="sm" onClick={() => refetch()}>
              <RefreshCw className="h-4 w-4" />
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        {filteredExecutions.length === 0 ? (
          <div className="text-center py-12 text-muted-foreground">
            <History className="h-12 w-12 mx-auto mb-4 opacity-50" />
            <p>
              {statusFilter !== 'all'
                ? 'No executions match the current filter'
                : 'No executions yet'}
            </p>
          </div>
        ) : (
          <div className="space-y-3">
            {filteredExecutions.map((execution) => (
              <div
                key={execution.id}
                className="border rounded-lg p-4 hover:bg-muted/50 transition-colors"
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex items-start gap-3 flex-1">
                    <div className="mt-0.5">{getStatusIcon(execution.status)}</div>
                    <div className="flex-1 min-w-0">
                      <div className="flex items-center gap-2 mb-1">
                        <Badge variant={getStatusBadgeVariant(execution.status)}>
                          {execution.status}
                        </Badge>
                        {execution.retryAttempt !== null && execution.retryAttempt > 0 && (
                          <Badge variant="outline">Retry #{execution.retryAttempt}</Badge>
                        )}
                      </div>
                      <div className="flex items-center gap-4 text-xs text-muted-foreground">
                        <div className="flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          <span>
                            {formatDistanceToNow(new Date(execution.startedAt), {
                              addSuffix: true,
                            })}
                          </span>
                        </div>
                        {execution.executionTimeSeconds && (
                          <div className="flex items-center gap-1">
                            <Clock className="h-3 w-3" />
                            <span>{formatDuration(execution.executionTimeSeconds)}</span>
                          </div>
                        )}
                        {execution.agentId && (
                          <span className="truncate">Agent: {execution.agentId}</span>
                        )}
                        {execution.model && (
                          <span className="truncate">Model: {execution.model}</span>
                        )}
                      </div>
                    </div>
                  </div>
                  <div className="flex gap-2 shrink-0">
                    {onViewExecution && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onViewExecution(execution.id)}
                      >
                        <Eye className="h-3 w-3" />
                      </Button>
                    )}
                    {execution.status === 'failed' && onRetryExecution && (
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => onRetryExecution(execution)}
                      >
                        <RefreshCw className="h-3 w-3" />
                      </Button>
                    )}
                  </div>
                </div>

                {/* Execution Metrics */}
                {(execution.tokensInput || execution.tokensOutput || execution.totalCost) && (
                  <div className="grid grid-cols-3 gap-4 pt-3 border-t text-xs">
                    {execution.tokensInput && (
                      <div>
                        <span className="text-muted-foreground">Input Tokens</span>
                        <p className="font-medium">{execution.tokensInput.toLocaleString()}</p>
                      </div>
                    )}
                    {execution.tokensOutput && (
                      <div>
                        <span className="text-muted-foreground">Output Tokens</span>
                        <p className="font-medium">{execution.tokensOutput.toLocaleString()}</p>
                      </div>
                    )}
                    {execution.totalCost && (
                      <div>
                        <span className="text-muted-foreground">Cost</span>
                        <p className="font-medium">{formatCost(execution.totalCost)}</p>
                      </div>
                    )}
                  </div>
                )}

                {/* Error Message */}
                {execution.errorMessage && (
                  <div className="mt-3 p-2 bg-destructive/10 border border-destructive/20 rounded text-xs">
                    <p className="text-destructive">{execution.errorMessage}</p>
                  </div>
                )}

                {/* File Changes */}
                {(execution.filesChanged || execution.linesAdded || execution.linesRemoved) && (
                  <div className="flex items-center gap-4 mt-3 pt-3 border-t text-xs text-muted-foreground">
                    {execution.filesChanged && (
                      <span>{execution.filesChanged} files changed</span>
                    )}
                    {execution.linesAdded && (
                      <span className="text-green-600">+{execution.linesAdded}</span>
                    )}
                    {execution.linesRemoved && (
                      <span className="text-red-600">-{execution.linesRemoved}</span>
                    )}
                  </div>
                )}
              </div>
            ))}
          </div>
        )}

        {/* Pagination Info */}
        {executionsData && executionsData.total > executionsData.items.length && (
          <p className="text-xs text-muted-foreground mt-4 text-center">
            Showing {filteredExecutions.length} of {executionsData.total} executions
          </p>
        )}
      </CardContent>
    </Card>
  );
}
