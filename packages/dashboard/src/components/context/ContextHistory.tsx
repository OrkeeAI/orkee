import { useState, useEffect } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import {
  BarChart, Bar, LineChart, Line, XAxis, YAxis,
  CartesianGrid, Tooltip, ResponsiveContainer
} from 'recharts';
import { Clock, TrendingUp, FileText, Hash } from 'lucide-react';
import { formatDistanceToNow } from 'date-fns';

interface ContextSnapshot {
  id: string;
  createdAt: Date;
  tokenCount: number;
  fileCount: number;
  taskId?: string;
  taskSuccess?: boolean;
  configuration: {
    name: string;
    includePatterns: string[];
  };
  filesIncluded: string[];
}

interface UsageStats {
  totalContextsGenerated: number;
  averageTokens: number;
  successRate: number;
  mostUsedFiles: Array<{ file: string; count: number }>;
  tokenUsageOverTime: Array<{ date: string; tokens: number }>;
}

interface ContextHistoryProps {
  projectId: string;
}

export function ContextHistory({ projectId }: ContextHistoryProps) {
  const [snapshots, setSnapshots] = useState<ContextSnapshot[]>([]);
  const [stats, setStats] = useState<UsageStats | null>(null);
  const [selectedSnapshot, setSelectedSnapshot] = useState<ContextSnapshot | null>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadHistory();
    loadStats();
  }, [projectId]);

  const loadHistory = async () => {
    try {
      const response = await fetch(`/api/projects/${projectId}/context/history`);
      const data = await response.json();
      if (data.success) {
        setSnapshots(data.data.snapshots.map((s: any) => ({
          ...s,
          createdAt: new Date(s.createdAt)
        })));
      }
    } catch (error) {
      console.error('Failed to load history:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadStats = async () => {
    try {
      const response = await fetch(`/api/projects/${projectId}/context/stats`);
      const data = await response.json();
      if (data.success) {
        setStats(data.data);
      }
    } catch (error) {
      console.error('Failed to load stats:', error);
    }
  };

  const restoreContext = async (snapshotId: string) => {
    try {
      await fetch(`/api/projects/${projectId}/context/restore`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ snapshot_id: snapshotId }),
      });
    } catch (error) {
      console.error('Failed to restore context:', error);
    }
  };

  return (
    <div className="space-y-4">
      {/* Statistics Overview */}
      {stats && (
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Total Contexts</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.totalContextsGenerated}</div>
              <p className="text-xs text-muted-foreground">Generated this month</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Average Tokens</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {stats.averageTokens.toLocaleString()}
              </div>
              <p className="text-xs text-muted-foreground">Per context</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader className="pb-2">
              <CardTitle className="text-sm">Success Rate</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">{stats.successRate}%</div>
              <p className="text-xs text-muted-foreground">Tasks completed</p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Token Usage Chart */}
      {stats && stats.tokenUsageOverTime.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Token Usage Over Time</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={stats.tokenUsageOverTime}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="date" />
                <YAxis />
                <Tooltip />
                <Line
                  type="monotone"
                  dataKey="tokens"
                  stroke="#8884d8"
                  strokeWidth={2}
                />
              </LineChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      )}

      {/* Most Used Files */}
      {stats && stats.mostUsedFiles.length > 0 && (
        <Card>
          <CardHeader>
            <CardTitle>Most Frequently Included Files</CardTitle>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={200}>
              <BarChart data={stats.mostUsedFiles.slice(0, 10)}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="file" angle={-45} textAnchor="end" height={100} />
                <YAxis />
                <Tooltip />
                <Bar dataKey="count" fill="#82ca9d" />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      )}

      {/* Context History List */}
      <Card>
        <CardHeader>
          <CardTitle>Recent Contexts</CardTitle>
        </CardHeader>
        <CardContent>
          <ScrollArea className="h-[400px]">
            <div className="space-y-2">
              {snapshots.map(snapshot => (
                <div
                  key={snapshot.id}
                  className="flex items-center justify-between p-3 border rounded-lg hover:bg-accent cursor-pointer"
                  onClick={() => setSelectedSnapshot(snapshot)}
                >
                  <div className="flex-1">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">
                        {snapshot.configuration.name}
                      </span>
                      {snapshot.taskSuccess !== undefined && (
                        <Badge variant={snapshot.taskSuccess ? 'default' : 'destructive'}>
                          {snapshot.taskSuccess ? 'Success' : 'Failed'}
                        </Badge>
                      )}
                    </div>
                    <div className="flex items-center gap-4 mt-1 text-xs text-muted-foreground">
                      <span className="flex items-center gap-1">
                        <Clock className="h-3 w-3" />
                        {formatDistanceToNow(snapshot.createdAt, { addSuffix: true })}
                      </span>
                      <span className="flex items-center gap-1">
                        <Hash className="h-3 w-3" />
                        {snapshot.tokenCount.toLocaleString()} tokens
                      </span>
                      <span className="flex items-center gap-1">
                        <FileText className="h-3 w-3" />
                        {snapshot.fileCount} files
                      </span>
                    </div>
                  </div>
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      restoreContext(snapshot.id);
                    }}
                  >
                    Restore
                  </Button>
                </div>
              ))}
            </div>
          </ScrollArea>
        </CardContent>
      </Card>

      {/* Snapshot Details Modal */}
      {selectedSnapshot && (
        <Card>
          <CardHeader>
            <CardTitle>Context Details</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-2">
              <div>
                <span className="font-medium">Files Included:</span>
                <div className="mt-1 max-h-[200px] overflow-y-auto">
                  {selectedSnapshot.filesIncluded.map(file => (
                    <div key={file} className="text-sm text-muted-foreground">
                      {file}
                    </div>
                  ))}
                </div>
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
