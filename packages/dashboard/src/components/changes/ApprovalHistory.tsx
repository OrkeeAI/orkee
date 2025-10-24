// ABOUTME: Timeline view of OpenSpec change status transitions and approval history
// ABOUTME: Displays who made each transition, timestamps, and notes
import { CheckCircle2, FileEdit, Archive, Clock, User } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { ChangeWithDeltas, ChangeStatus } from '@/services/changes';

interface ApprovalHistoryProps {
  change: ChangeWithDeltas;
}

interface TimelineEvent {
  timestamp: string;
  status: ChangeStatus;
  actor: string;
  icon: React.ReactNode;
  badgeVariant: 'default' | 'secondary' | 'outline';
  notes?: string;
}

export function ApprovalHistory({ change }: ApprovalHistoryProps) {
  const events: TimelineEvent[] = [];

  events.push({
    timestamp: change.createdAt,
    status: 'draft',
    actor: change.createdBy,
    icon: <FileEdit className="h-4 w-4" />,
    badgeVariant: 'outline',
  });

  if (change.approvedAt && change.approvedBy) {
    events.push({
      timestamp: change.approvedAt,
      status: 'approved',
      actor: change.approvedBy,
      icon: <CheckCircle2 className="h-4 w-4" />,
      badgeVariant: 'default',
    });
  }

  if (change.archivedAt) {
    events.push({
      timestamp: change.archivedAt,
      status: 'archived',
      actor: change.createdBy,
      icon: <Archive className="h-4 w-4" />,
      badgeVariant: 'secondary',
    });
  }

  events.sort((a, b) => new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime());

  const getStatusLabel = (status: ChangeStatus): string => {
    const labels: Record<ChangeStatus, string> = {
      draft: 'Draft Created',
      review: 'Submitted for Review',
      approved: 'Approved',
      implementing: 'Implementation Started',
      completed: 'Implementation Completed',
      archived: 'Archived',
    };
    return labels[status] || status;
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    return {
      date: date.toLocaleDateString('en-US', {
        month: 'short',
        day: 'numeric',
        year: 'numeric'
      }),
      time: date.toLocaleTimeString('en-US', {
        hour: '2-digit',
        minute: '2-digit'
      }),
    };
  };

  if (events.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle className="text-lg flex items-center gap-2">
            <Clock className="h-5 w-5" />
            Status History
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">No status history available</p>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <Clock className="h-5 w-5" />
          Status History
        </CardTitle>
      </CardHeader>
      <CardContent>
        <div className="relative space-y-4">
          {/* Timeline line */}
          <div className="absolute left-4 top-0 bottom-0 w-px bg-border" />

          {events.map((event, index) => {
            const { date, time } = formatTimestamp(event.timestamp);
            const isLast = index === events.length - 1;

            return (
              <div key={`${event.status}-${event.timestamp}`} className="relative flex gap-4">
                {/* Icon */}
                <div className={`relative z-10 flex items-center justify-center w-8 h-8 rounded-full border-2 ${
                  isLast ? 'bg-primary border-primary text-primary-foreground' : 'bg-background border-border'
                }`}>
                  {event.icon}
                </div>

                {/* Content */}
                <div className="flex-1 pb-4">
                  <div className="flex items-start justify-between gap-2">
                    <div className="space-y-1">
                      <div className="flex items-center gap-2">
                        <Badge variant={event.badgeVariant}>
                          {getStatusLabel(event.status)}
                        </Badge>
                        {isLast && (
                          <Badge variant="outline" className="text-xs">
                            Current
                          </Badge>
                        )}
                      </div>
                      <div className="flex items-center gap-2 text-sm text-muted-foreground">
                        <User className="h-3 w-3" />
                        <span>{event.actor}</span>
                      </div>
                      {event.notes && (
                        <p className="text-sm text-muted-foreground mt-2 p-2 bg-muted rounded">
                          {event.notes}
                        </p>
                      )}
                    </div>
                    <div className="text-right text-xs text-muted-foreground whitespace-nowrap">
                      <div>{date}</div>
                      <div>{time}</div>
                    </div>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </CardContent>
    </Card>
  );
}
