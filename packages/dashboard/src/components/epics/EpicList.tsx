// ABOUTME: Epic list view displaying all epics with status, progress, and metadata
// ABOUTME: Supports filtering by status and PRD, with actions for viewing details and deletion

import { FileText, Trash2, Calendar, BarChart3, Target, AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import type { Epic, EpicStatus, EpicComplexity } from '@/services/epics';

interface EpicListProps {
  epics: Epic[];
  isLoading?: boolean;
  onSelect: (epic: Epic) => void;
  onDelete: (epicId: string) => void;
  selectedEpicId?: string;
}

export function EpicList({ epics, isLoading, onSelect, onDelete, selectedEpicId }: EpicListProps) {
  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
    });
  };

  const getStatusBadge = (status: EpicStatus) => {
    const variants: Record<EpicStatus, { variant: 'default' | 'secondary' | 'outline' | 'destructive'; label: string }> = {
      draft: { variant: 'outline', label: 'Draft' },
      ready: { variant: 'secondary', label: 'Ready' },
      in_progress: { variant: 'default', label: 'In Progress' },
      blocked: { variant: 'destructive', label: 'Blocked' },
      completed: { variant: 'default', label: 'Completed' },
      cancelled: { variant: 'outline', label: 'Cancelled' },
    };
    const { variant, label } = variants[status];
    return <Badge variant={variant}>{label}</Badge>;
  };

  const getComplexityBadge = (complexity?: EpicComplexity) => {
    if (!complexity) return null;
    const variants: Record<EpicComplexity, { variant: 'default' | 'secondary' | 'outline' | 'destructive'; label: string }> = {
      low: { variant: 'outline', label: 'Low' },
      medium: { variant: 'secondary', label: 'Medium' },
      high: { variant: 'default', label: 'High' },
      very_high: { variant: 'destructive', label: 'Very High' },
    };
    const { variant, label } = variants[complexity];
    return <Badge variant={variant}>{label}</Badge>;
  };

  const getEffortLabel = (effort?: string) => {
    if (!effort) return null;
    const labels: Record<string, string> = {
      days: 'Days',
      weeks: 'Weeks',
      months: 'Months',
    };
    return labels[effort] || effort;
  };

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[1, 2, 3].map((i) => (
          <Card key={i} className="animate-pulse">
            <CardHeader>
              <div className="h-5 bg-muted rounded w-3/4 mb-2" />
              <div className="h-4 bg-muted rounded w-1/2" />
            </CardHeader>
          </Card>
        ))}
      </div>
    );
  }

  if (epics.length === 0) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12">
          <FileText className="h-12 w-12 text-muted-foreground mb-4" />
          <p className="text-lg font-medium text-muted-foreground mb-2">No epics found</p>
          <p className="text-sm text-muted-foreground">
            Create an epic from a PRD to get started
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-3">
      {epics.map((epic) => (
        <Card
          key={epic.id}
          className={`cursor-pointer transition-all hover:border-primary ${
            selectedEpicId === epic.id ? 'border-primary bg-accent' : ''
          }`}
          onClick={() => onSelect(epic)}
        >
          <CardHeader className="pb-3">
            <div className="flex items-start justify-between">
              <div className="flex-1">
                <div className="flex items-center gap-2 mb-2">
                  <CardTitle className="text-lg">{epic.name}</CardTitle>
                  {getStatusBadge(epic.status)}
                </div>
                <CardDescription className="line-clamp-2">
                  {epic.overviewMarkdown.substring(0, 150)}
                  {epic.overviewMarkdown.length > 150 ? '...' : ''}
                </CardDescription>
              </div>
              <Button
                variant="ghost"
                size="icon"
                onClick={(e) => {
                  e.stopPropagation();
                  if (confirm('Are you sure you want to delete this epic? This action cannot be undone.')) {
                    onDelete(epic.id);
                  }
                }}
                className="ml-2"
              >
                <Trash2 className="h-4 w-4" />
              </Button>
            </div>
          </CardHeader>
          <CardContent className="space-y-3">
            {/* Progress Bar */}
            <div className="space-y-1">
              <div className="flex items-center justify-between text-sm">
                <span className="text-muted-foreground">Progress</span>
                <span className="font-medium">{epic.progressPercentage}%</span>
              </div>
              <Progress value={epic.progressPercentage} className="h-2" />
            </div>

            {/* Metadata */}
            <div className="flex flex-wrap gap-3 text-sm text-muted-foreground">
              {epic.complexity && (
                <div className="flex items-center gap-1">
                  <AlertCircle className="h-4 w-4" />
                  <span>Complexity: {getComplexityBadge(epic.complexity)}</span>
                </div>
              )}
              {epic.estimatedEffort && (
                <div className="flex items-center gap-1">
                  <BarChart3 className="h-4 w-4" />
                  <span>Effort: {getEffortLabel(epic.estimatedEffort)}</span>
                </div>
              )}
              <div className="flex items-center gap-1">
                <Calendar className="h-4 w-4" />
                <span>Created {formatDate(epic.createdAt)}</span>
              </div>
            </div>

            {/* Task Categories */}
            {epic.taskCategories && epic.taskCategories.length > 0 && (
              <div className="flex flex-wrap gap-1">
                {epic.taskCategories.map((category, idx) => (
                  <Badge key={idx} variant="outline" className="text-xs">
                    {category}
                  </Badge>
                ))}
              </div>
            )}

            {/* Success Criteria Count */}
            {epic.successCriteria && epic.successCriteria.length > 0 && (
              <div className="flex items-center gap-1 text-sm text-muted-foreground">
                <Target className="h-4 w-4" />
                <span>{epic.successCriteria.length} success criteria defined</span>
              </div>
            )}
          </CardContent>
        </Card>
      ))}
    </div>
  );
}
