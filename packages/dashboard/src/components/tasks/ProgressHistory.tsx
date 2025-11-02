// ABOUTME: Progress history timeline displaying append-only validation entries
// ABOUTME: Shows chronological task progress with different entry types

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import {
  CheckCircle2,
  AlertCircle,
  Lightbulb,
  Flag,
  Clock,
} from 'lucide-react';
import type { ValidationEntry } from '@/services/tasks';

interface ProgressHistoryProps {
  entries: ValidationEntry[];
  maxHeight?: string;
}

export function ProgressHistory({
  entries,
  maxHeight = '500px',
}: ProgressHistoryProps) {
  if (entries.length === 0) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12">
          <Clock className="h-12 w-12 text-muted-foreground mb-4" />
          <p className="text-lg font-medium text-muted-foreground">
            No progress history yet
          </p>
          <p className="text-sm text-muted-foreground mt-2">
            Updates will appear here as work progresses
          </p>
        </CardContent>
      </Card>
    );
  }

  const getEntryIcon = (type: ValidationEntry['entryType']) => {
    switch (type) {
      case 'progress':
        return <CheckCircle2 className="h-5 w-5 text-green-600" />;
      case 'issue':
        return <AlertCircle className="h-5 w-5 text-red-600" />;
      case 'decision':
        return <Lightbulb className="h-5 w-5 text-yellow-600" />;
      case 'checkpoint':
        return <Flag className="h-5 w-5 text-blue-600" />;
      default:
        return <Clock className="h-5 w-5 text-gray-600" />;
    }
  };

  const getEntryTypeColor = (type: ValidationEntry['entryType']) => {
    switch (type) {
      case 'progress':
        return 'bg-green-100 text-green-800 border-green-300';
      case 'issue':
        return 'bg-red-100 text-red-800 border-red-300';
      case 'decision':
        return 'bg-yellow-100 text-yellow-800 border-yellow-300';
      case 'checkpoint':
        return 'bg-blue-100 text-blue-800 border-blue-300';
      default:
        return 'bg-gray-100 text-gray-800 border-gray-300';
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString(undefined, {
      month: 'short',
      day: 'numeric',
      year: date.getFullYear() !== now.getFullYear() ? 'numeric' : undefined,
    });
  };

  // Sort entries by timestamp (newest first)
  const sortedEntries = [...entries].sort(
    (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
  );

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">Progress History</CardTitle>
        <p className="text-sm text-muted-foreground">
          {entries.length} {entries.length === 1 ? 'entry' : 'entries'}
        </p>
      </CardHeader>
      <CardContent>
        <ScrollArea style={{ maxHeight }} className="pr-4">
          <div className="space-y-4">
            {sortedEntries.map((entry, index) => (
              <div
                key={index}
                className="relative pl-8 pb-4 border-l-2 border-gray-200 last:border-0"
              >
                <div className="absolute -left-[13px] top-0 bg-white">
                  {getEntryIcon(entry.entryType)}
                </div>

                <div className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Badge
                      variant="outline"
                      className={getEntryTypeColor(entry.entryType)}
                    >
                      {entry.entryType}
                    </Badge>
                    <span className="text-xs text-muted-foreground">
                      {formatTimestamp(entry.timestamp)}
                    </span>
                  </div>

                  <p className="text-sm whitespace-pre-wrap">{entry.content}</p>

                  {entry.author && (
                    <p className="text-xs text-muted-foreground">
                      by {entry.author}
                    </p>
                  )}
                </div>
              </div>
            ))}
          </div>
        </ScrollArea>
      </CardContent>
    </Card>
  );
}
