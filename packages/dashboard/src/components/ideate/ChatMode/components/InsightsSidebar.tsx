// ABOUTME: Sidebar component displaying extracted chat insights
// ABOUTME: Groups insights by type with visual indicators

import React from 'react';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { Lightbulb, AlertTriangle, Lock, HelpCircle, CheckSquare, RefreshCw } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { ChatInsight } from '@/services/chat';

export interface InsightsSidebarProps {
  insights: ChatInsight[];
  className?: string;
  onReanalyze?: () => void;
  isReanalyzing?: boolean;
}

const INSIGHT_ICONS = {
  requirement: Lightbulb,
  constraint: Lock,
  risk: AlertTriangle,
  assumption: HelpCircle,
  decision: CheckSquare,
};

const INSIGHT_COLORS = {
  requirement: 'text-blue-600 dark:text-blue-400',
  constraint: 'text-purple-600 dark:text-purple-400',
  risk: 'text-red-600 dark:text-red-400',
  assumption: 'text-yellow-600 dark:text-yellow-400',
  decision: 'text-green-600 dark:text-green-400',
};

const INSIGHT_LABELS = {
  requirement: 'Requirements',
  constraint: 'Constraints',
  risk: 'Risks',
  assumption: 'Assumptions',
  decision: 'Decisions',
};

export function InsightsSidebar({ insights, className, onReanalyze, isReanalyzing }: InsightsSidebarProps) {
  const groupedInsights = insights.reduce((acc, insight) => {
    if (!acc[insight.insight_type]) {
      acc[insight.insight_type] = [];
    }
    acc[insight.insight_type].push(insight);
    return acc;
  }, {} as Record<string, ChatInsight[]>);

  const sortedGroups = Object.entries(groupedInsights).sort(([a], [b]) => {
    const order = ['requirement', 'constraint', 'risk', 'assumption', 'decision'];
    return order.indexOf(a) - order.indexOf(b);
  });

  return (
    <div className={cn('bg-card rounded-lg border flex flex-col', className)}>
      <div className="p-4 border-b space-y-3">
        <div>
          <h3 className="text-sm font-medium flex items-center gap-2">
            <Lightbulb className="h-4 w-4" />
            Extracted Insights
          </h3>
          <p className="text-xs text-muted-foreground mt-1">
            {insights.length} insight{insights.length !== 1 ? 's' : ''} captured
          </p>
        </div>
        {onReanalyze && (
          <Button
            variant="outline"
            size="sm"
            onClick={onReanalyze}
            disabled={isReanalyzing}
            className="w-full gap-2"
          >
            <RefreshCw className={cn('h-3 w-3', isReanalyzing && 'animate-spin')} />
            {isReanalyzing ? 'Re-analyzing...' : 'Re-analyze All Messages'}
          </Button>
        )}
      </div>

      <ScrollArea className="flex-1">
        <div className="p-4 space-y-6">
          {sortedGroups.length === 0 ? (
            <div className="text-center py-8 text-muted-foreground text-sm">
              <p>No insights extracted yet.</p>
              <p className="mt-1">Keep the chat going!</p>
            </div>
          ) : (
            sortedGroups.map(([type, typeInsights]) => {
              const Icon = INSIGHT_ICONS[type as keyof typeof INSIGHT_ICONS] || Lightbulb;
              const color = INSIGHT_COLORS[type as keyof typeof INSIGHT_COLORS] || 'text-gray-600';
              const label = INSIGHT_LABELS[type as keyof typeof INSIGHT_LABELS] || type;

              return (
                <div key={type} className="space-y-2">
                  <div className="flex items-center gap-2">
                    <Icon className={cn('h-4 w-4', color)} />
                    <h4 className="text-xs font-medium uppercase tracking-wide text-muted-foreground">
                      {label}
                    </h4>
                    <Badge variant="secondary" className="ml-auto text-xs">
                      {typeInsights.length}
                    </Badge>
                  </div>

                  <div className="space-y-2 pl-6">
                    {typeInsights.map((insight) => (
                      <div
                        key={insight.id}
                        className="text-sm p-2 rounded-md bg-muted/50 border-l-2 border-current"
                        style={{ borderColor: `var(--${color})` }}
                      >
                        <p className="text-foreground">{insight.insight_text}</p>
                        {insight.confidence_score !== null && (
                          <p className="text-xs text-muted-foreground mt-1">
                            Confidence: {Math.round(insight.confidence_score * 100)}%
                          </p>
                        )}
                      </div>
                    ))}
                  </div>
                </div>
              );
            })
          )}
        </div>
      </ScrollArea>
    </div>
  );
}
