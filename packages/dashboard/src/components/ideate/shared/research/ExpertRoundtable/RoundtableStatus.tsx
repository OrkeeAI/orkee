// ABOUTME: Roundtable statistics and status display component
// ABOUTME: Shows message counts, participation stats, duration, and current status

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import { Activity, Clock, MessageSquare, Users, TrendingUp, Loader2 } from 'lucide-react';
import { useRoundtableStatistics, useGetRoundtable } from '@/hooks/useIdeate';
import type { RoundtableStatus as RoundtableStatusType } from '@/services/ideate';

interface RoundtableStatusProps {
  roundtableId: string;
}

export function RoundtableStatus({ roundtableId }: RoundtableStatusProps) {
  const { data: roundtable, isLoading: roundtableLoading } = useGetRoundtable(roundtableId);
  const { data: statistics, isLoading: statsLoading } = useRoundtableStatistics(roundtableId);

  const getStatusBadge = (status: RoundtableStatusType) => {
    const variants: Record<RoundtableStatusType, { variant: 'default' | 'secondary' | 'destructive' | 'outline', label: string }> = {
      setup: { variant: 'secondary', label: 'Setup' },
      discussing: { variant: 'default', label: 'In Progress' },
      completed: { variant: 'outline', label: 'Completed' },
      cancelled: { variant: 'destructive', label: 'Cancelled' },
    };

    const config = variants[status];
    return <Badge variant={config.variant}>{config.label}</Badge>;
  };

  const formatDuration = (seconds: number) => {
    const minutes = Math.floor(seconds / 60);
    const hours = Math.floor(minutes / 60);
    const remainingMinutes = minutes % 60;

    if (hours > 0) {
      return `${hours}h ${remainingMinutes}m`;
    }
    return `${minutes}m`;
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return date.toLocaleString('en-US', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  if (roundtableLoading || statsLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center p-8">
          <Loader2 className="h-6 w-6 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  if (!roundtable || !statistics) {
    return null;
  }

  const participationPercentages = statistics.participation_by_expert.map((p) => ({
    ...p,
    percentage: Math.round((p.message_count / statistics.total_messages) * 100),
  }));

  return (
    <Card>
      <CardHeader className="pb-3">
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="flex items-center gap-2">
              <Activity className="h-5 w-5" />
              Roundtable Status
            </CardTitle>
            <CardDescription>{roundtable.topic}</CardDescription>
          </div>
          {getStatusBadge(roundtable.status)}
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        <div className="grid grid-cols-2 gap-4">
          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <MessageSquare className="h-4 w-4" />
              <span>Total Messages</span>
            </div>
            <p className="text-2xl font-bold">{statistics.total_messages}</p>
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Users className="h-4 w-4" />
              <span>Participants</span>
            </div>
            <p className="text-2xl font-bold">{roundtable.num_experts}</p>
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Clock className="h-4 w-4" />
              <span>Duration</span>
            </div>
            <p className="text-2xl font-bold">
              {roundtable.duration_seconds
                ? formatDuration(roundtable.duration_seconds)
                : '-'}
            </p>
          </div>

          <div className="space-y-2">
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <TrendingUp className="h-4 w-4" />
              <span>Avg. per Expert</span>
            </div>
            <p className="text-2xl font-bold">
              {roundtable.num_experts > 0
                ? Math.round(statistics.total_messages / roundtable.num_experts)
                : 0}
            </p>
          </div>
        </div>

        <Separator />

        <div className="space-y-3">
          <h4 className="text-sm font-medium">Expert Participation</h4>
          <div className="space-y-3">
            {participationPercentages.map((participant) => (
              <div key={participant.expert_id} className="space-y-1.5">
                <div className="flex items-center justify-between text-sm">
                  <span className="font-medium">{participant.expert_name}</span>
                  <div className="flex items-center gap-2">
                    <span className="text-muted-foreground">
                      {participant.message_count} messages
                    </span>
                    <Badge variant="secondary" className="text-xs">
                      {participant.percentage}%
                    </Badge>
                  </div>
                </div>
                <Progress value={participant.percentage} className="h-2" />
              </div>
            ))}
          </div>
        </div>

        <Separator />

        <div className="space-y-2 text-sm">
          <div className="flex justify-between">
            <span className="text-muted-foreground">Created</span>
            <span className="font-medium">{formatDate(roundtable.created_at)}</span>
          </div>
          {roundtable.started_at && (
            <div className="flex justify-between">
              <span className="text-muted-foreground">Started</span>
              <span className="font-medium">{formatDate(roundtable.started_at)}</span>
            </div>
          )}
          {roundtable.completed_at && (
            <div className="flex justify-between">
              <span className="text-muted-foreground">Completed</span>
              <span className="font-medium">{formatDate(roundtable.completed_at)}</span>
            </div>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
