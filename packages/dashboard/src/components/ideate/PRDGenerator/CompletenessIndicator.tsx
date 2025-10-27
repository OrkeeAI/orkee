// ABOUTME: Visual indicator showing PRD completeness metrics
// ABOUTME: Displays progress bar, section breakdown, and completion percentage

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { CheckCircle2, Circle, AlertCircle } from 'lucide-react';
import type { CompletenessMetrics } from '@/services/ideate';

interface CompletenessIndicatorProps {
  completeness: CompletenessMetrics;
}

export function CompletenessIndicator({ completeness }: CompletenessIndicatorProps) {
  const {
    total_sections,
    completed_sections,
    skipped_sections,
    ai_filled_sections,
    completeness_percentage,
  } = completeness;

  const getStatusColor = (percentage: number) => {
    if (percentage >= 90) return 'text-green-600';
    if (percentage >= 70) return 'text-yellow-600';
    return 'text-red-600';
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm font-medium">PRD Completeness</CardTitle>
          <span className={`text-2xl font-bold ${getStatusColor(completeness_percentage)}`}>
            {completeness_percentage}%
          </span>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Progress Bar */}
        <div className="space-y-2">
          <Progress
            value={completeness_percentage}
            className="h-2"
          />
          <p className="text-xs text-muted-foreground">
            {completed_sections} of {total_sections} sections completed
          </p>
        </div>

        {/* Section Breakdown */}
        <div className="grid grid-cols-3 gap-2">
          <div className="flex items-center gap-2 rounded-lg border p-2">
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <div className="flex flex-col">
              <span className="text-xs text-muted-foreground">Completed</span>
              <span className="text-sm font-semibold">{completed_sections}</span>
            </div>
          </div>

          <div className="flex items-center gap-2 rounded-lg border p-2">
            <Circle className="h-4 w-4 text-gray-400" />
            <div className="flex flex-col">
              <span className="text-xs text-muted-foreground">Skipped</span>
              <span className="text-sm font-semibold">{skipped_sections}</span>
            </div>
          </div>

          <div className="flex items-center gap-2 rounded-lg border p-2">
            <AlertCircle className="h-4 w-4 text-blue-600" />
            <div className="flex flex-col">
              <span className="text-xs text-muted-foreground">AI-Filled</span>
              <span className="text-sm font-semibold">{ai_filled_sections}</span>
            </div>
          </div>
        </div>

        {/* Status Badge */}
        {completeness_percentage >= 100 && (
          <Badge variant="default" className="w-full justify-center gap-1">
            <CheckCircle2 className="h-3 w-3" />
            All sections complete
          </Badge>
        )}
        {completeness_percentage < 100 && skipped_sections > 0 && (
          <Badge variant="secondary" className="w-full justify-center">
            {skipped_sections} section{skipped_sections !== 1 ? 's' : ''} can be AI-filled
          </Badge>
        )}
      </CardContent>
    </Card>
  );
}
