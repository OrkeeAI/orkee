// ABOUTME: Quality indicator component showing PRD readiness
// ABOUTME: Displays progress bar and coverage metrics for conversation quality

import React from 'react';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import { CheckCircle2, Circle } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { QualityMetrics } from '@/services/chat';

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
  // Default empty coverage when no metrics available
  const defaultCoverage = {
    problem: false,
    users: false,
    features: false,
    technical: false,
    risks: false,
    constraints: false,
    success: false,
  };

  const coverage = metrics?.coverage ?? defaultCoverage;
  const coverageItems = Object.entries(coverage) as [
    keyof typeof coverage,
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

  const qualityScore = metrics?.quality_score ?? 0;
  const isReadyForPRD = metrics?.is_ready_for_prd ?? false;

  return (
    <div className={cn('bg-card rounded-lg border p-4 space-y-4', className)}>
      <div className="flex items-center justify-between">
        <h3 className="text-sm font-medium">Chat Quality</h3>
        <Badge
          variant={isReadyForPRD ? 'default' : 'secondary'}
          className={cn(
            isReadyForPRD && 'bg-green-600 hover:bg-green-700'
          )}
        >
          {isReadyForPRD ? 'Ready for PRD' : 'Keep Exploring'}
        </Badge>
      </div>

      <div className="space-y-2">
        <div className="flex items-center justify-between text-sm">
          <span className="text-muted-foreground">Quality Score</span>
          <span className={cn('font-semibold', getScoreColor(qualityScore))}>
            {qualityScore}%
          </span>
        </div>
        <Progress value={qualityScore} className={getProgressColor(qualityScore)} />
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
    </div>
  );
}
