// ABOUTME: Quality indicator component showing PRD readiness
// ABOUTME: Displays progress bar and coverage metrics for conversation quality

import React from 'react';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { CheckCircle2, Circle, AlertCircle } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { QualityMetrics } from '@/services/conversational';

export interface QualityIndicatorProps {
  metrics: QualityMetrics | null;
  className?: string;
}

const COVERAGE_LABELS: Record<keyof QualityMetrics['coverage'], string> = {
  problem: 'Problem Definition',
  users: 'Target Users',
  features: 'Features',
  technical: 'Technical Approach',
  risks: 'Risks & Mitigations',
  constraints: 'Constraints',
  success: 'Success Criteria',
};

export function QualityIndicator({ metrics, className }: QualityIndicatorProps) {
  if (!metrics) {
    return null;
  }

  const coverageItems = Object.entries(metrics.coverage) as [
    keyof QualityMetrics['coverage'],
    boolean
  ][];

  const getScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600 dark:text-green-400';
    if (score >= 60) return 'text-yellow-600 dark:text-yellow-400';
    return 'text-red-600 dark:text-red-400';
  };

  const getProgressColor = (score: number) => {
    if (score >= 80) return 'bg-green-600';
    if (score >= 60) return 'bg-yellow-600';
    return 'bg-red-600';
  };

  return (
    <div className={cn('bg-card rounded-lg border p-4 space-y-4', className)}>
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Conversation Quality</h3>
        <Badge
          variant={metrics.is_ready_for_prd ? 'default' : 'secondary'}
          className={cn(
            metrics.is_ready_for_prd && 'bg-green-600 hover:bg-green-700'
          )}
        >
          {metrics.is_ready_for_prd ? 'Ready for PRD' : 'Keep Exploring'}
        </Badge>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-muted-foreground">Quality Score</span>
          <span className={cn('font-semibold', getScoreColor(metrics.quality_score))}>
            {metrics.quality_score}%
          </span>
        </div>
        <Progress value={metrics.quality_score} className={getProgressColor(metrics.quality_score)} />
      </div>

      <div className="space-y-2">
        <h4 className="text-xs font-medium text-muted-foreground uppercase tracking-wide">
          Coverage
        </h4>
        <div className="grid grid-cols-1 gap-2">
          {coverageItems.map(([key, covered]) => (
            <div key={key} className="flex items-center gap-2 text-sm">
              {covered ? (
                <CheckCircle2 className="h-4 w-4 text-green-600 dark:text-green-400 flex-shrink-0" />
              ) : (
                <Circle className="h-4 w-4 text-muted-foreground flex-shrink-0" />
              )}
              <span className={cn(covered ? 'text-foreground' : 'text-muted-foreground')}>
                {COVERAGE_LABELS[key]}
              </span>
            </div>
          ))}
        </div>
      </div>

      {metrics.missing_areas.length > 0 && (
        <div className="pt-2 border-t space-y-2">
          <div className="flex items-center gap-2 text-sm text-muted-foreground">
            <AlertCircle className="h-4 w-4" />
            <span>Consider discussing:</span>
          </div>
          <ul className="text-sm space-y-1 pl-6">
            {metrics.missing_areas.map((area, index) => (
              <li key={index} className="text-muted-foreground list-disc">
                {area}
              </li>
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}
