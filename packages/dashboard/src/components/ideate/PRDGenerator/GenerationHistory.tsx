// ABOUTME: History panel showing all PRD generation attempts with version tracking
// ABOUTME: Displays generation method, validation status, and timestamps for each version

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { History, CheckCircle2, AlertCircle, AlertTriangle } from 'lucide-react';
import type { GenerationHistoryItem } from '@/services/ideate';

interface GenerationHistoryProps {
  history: GenerationHistoryItem[];
}

export function GenerationHistory({ history }: GenerationHistoryProps) {
  const getValidationBadge = (status: string) => {
    switch (status) {
      case 'valid':
        return (
          <Badge variant="default" className="gap-1">
            <CheckCircle2 className="h-3 w-3" />
            Valid
          </Badge>
        );
      case 'warnings':
        return (
          <Badge variant="secondary" className="gap-1">
            <AlertTriangle className="h-3 w-3" />
            Warnings
          </Badge>
        );
      case 'invalid':
        return (
          <Badge variant="destructive" className="gap-1">
            <AlertCircle className="h-3 w-3" />
            Invalid
          </Badge>
        );
      default:
        return (
          <Badge variant="outline" className="gap-1">
            {status}
          </Badge>
        );
    }
  };

  const getMethodBadge = (method: string) => {
    switch (method) {
      case 'full':
        return <Badge variant="default">Full Generation</Badge>;
      case 'partial':
        return <Badge variant="outline">Partial</Badge>;
      case 'section_regeneration':
        return <Badge variant="secondary">Section Regen</Badge>;
      case 'ai_fill':
        return <Badge variant="secondary">AI-Filled</Badge>;
      default:
        return <Badge variant="outline">{method}</Badge>;
    }
  };

  const formatDate = (dateString: string) => {
    const date = new Date(dateString);
    return new Intl.DateTimeFormat('en-US', {
      month: 'short',
      day: 'numeric',
      hour: 'numeric',
      minute: '2-digit',
      hour12: true,
    }).format(date);
  };

  const sortedHistory = [...history].sort((a, b) => b.version - a.version);

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <History className="h-5 w-5" />
          Generation History
        </CardTitle>
      </CardHeader>
      <CardContent>
        {history.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-muted-foreground">
            <History className="h-8 w-8 mb-2" />
            <p>No generation history yet</p>
          </div>
        ) : (
          <div className="rounded-lg border">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-20">Version</TableHead>
                  <TableHead>Method</TableHead>
                  <TableHead>Status</TableHead>
                  <TableHead>Generated</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {sortedHistory.map((item) => (
                  <TableRow key={item.id}>
                    <TableCell className="font-mono font-semibold">
                      v{item.version}
                      {item.version === sortedHistory[0].version && (
                        <Badge variant="outline" className="ml-2 text-xs">
                          Latest
                        </Badge>
                      )}
                    </TableCell>
                    <TableCell>{getMethodBadge(item.generationMethod)}</TableCell>
                    <TableCell>{getValidationBadge(item.validationStatus)}</TableCell>
                    <TableCell className="text-sm text-muted-foreground">
                      {formatDate(item.createdAt)}
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>
        )}

        {history.length > 0 && (
          <div className="mt-4 text-xs text-muted-foreground">
            Showing {history.length} generation{history.length !== 1 ? 's' : ''}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
