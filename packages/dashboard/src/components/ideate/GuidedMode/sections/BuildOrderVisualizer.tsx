// ABOUTME: Build order timeline visualization with phases and parallel work groups
// ABOUTME: Shows optimal feature implementation order based on dependency analysis

import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Card, CardContent } from '@/components/ui/card';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Zap, Shield, TrendingUp, Loader2 } from 'lucide-react';
import { useBuildOrder, useOptimizeBuildOrder } from '@/hooks/useIdeate';
import type { OptimizationStrategy } from '@/services/ideate';
import { useState } from 'react';
import { toast } from 'sonner';

interface BuildOrderVisualizerProps {
  sessionId: string;
}

const strategyIcons = {
  fastest: Zap,
  balanced: TrendingUp,
  safest: Shield,
};

const strategyDescriptions = {
  fastest: 'Maximum parallelization, higher risk',
  balanced: 'Balance speed and risk mitigation',
  safest: 'Sequential phases, lowest risk',
};

export function BuildOrderVisualizer({ sessionId }: BuildOrderVisualizerProps) {
  const [strategy, setStrategy] = useState<OptimizationStrategy>('balanced');
  const { data: buildOrder, isLoading, error } = useBuildOrder(sessionId);
  const optimizeMutation = useOptimizeBuildOrder(sessionId);

  const handleOptimize = async () => {
    try {
      await optimizeMutation.mutateAsync(strategy);
      toast.success('Build order optimized');
    } catch (error: unknown) {
      toast.error((error as Error).message || 'Failed to optimize');
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <Loader2 className="w-8 h-8 animate-spin" />
      </div>
    );
  }

  if (error || !buildOrder) {
    return (
      <Card>
        <CardContent className="py-8">
          <div className="text-center space-y-4">
            <p className="text-sm text-muted-foreground">
              No build order yet. Optimize to generate implementation timeline.
            </p>
            <div className="flex items-center justify-center gap-4">
              <Select value={strategy} onValueChange={(v) => setStrategy(v as OptimizationStrategy)}>
                <SelectTrigger className="w-48">
                  <SelectValue />
                </SelectTrigger>
                <SelectContent>
                  <SelectItem value="fastest">⚡ Fastest</SelectItem>
                  <SelectItem value="balanced">📊 Balanced</SelectItem>
                  <SelectItem value="safest">🛡️ Safest</SelectItem>
                </SelectContent>
              </Select>
              <Button onClick={handleOptimize} disabled={optimizeMutation.isPending}>
                {optimizeMutation.isPending ? 'Optimizing...' : 'Optimize Build Order'}
              </Button>
            </div>
          </div>
        </CardContent>
      </Card>
    );
  }

  const StrategyIcon = strategyIcons[buildOrder.strategy];
  const isCritical = (feature: string) => buildOrder.critical_path.includes(feature);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <h3 className="font-medium">Build Order Timeline</h3>
            <Badge variant="outline" className="gap-1">
              <StrategyIcon className="w-3 h-3" />
              {buildOrder.strategy}
            </Badge>
          </div>
          <p className="text-sm text-muted-foreground">
            {strategyDescriptions[buildOrder.strategy]}
          </p>
        </div>
        <div className="flex items-center gap-2">
          <Select value={strategy} onValueChange={(v) => setStrategy(v as OptimizationStrategy)}>
            <SelectTrigger className="w-32">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="fastest">⚡ Fastest</SelectItem>
              <SelectItem value="balanced">📊 Balanced</SelectItem>
              <SelectItem value="safest">🛡️ Safest</SelectItem>
            </SelectContent>
          </Select>
          <Button
            size="sm"
            variant="outline"
            onClick={handleOptimize}
            disabled={optimizeMutation.isPending}
          >
            {optimizeMutation.isPending ? 'Optimizing...' : 'Re-optimize'}
          </Button>
        </div>
      </div>

      {/* Parallel Groups Timeline */}
      <div className="space-y-4">
        {buildOrder.parallel_groups.map((group, index) => (
          <Card key={index}>
            <CardContent className="p-4">
              <div className="flex items-start gap-4">
                <div className="flex-shrink-0">
                  <div className="flex items-center justify-center w-8 h-8 rounded-full bg-primary/10 text-primary font-medium text-sm">
                    {index + 1}
                  </div>
                </div>
                <div className="flex-1 space-y-2">
                  <div className="flex items-center justify-between">
                    <span className="font-medium">Phase {index + 1}</span>
                    <span className="text-sm text-muted-foreground">
                      {group.features.length} feature{group.features.length !== 1 ? 's' : ''} ·{' '}
                      ~{group.estimated_time}h
                    </span>
                  </div>
                  <div className="flex flex-wrap gap-2">
                    {group.features.map((feature) => (
                      <Badge
                        key={feature}
                        variant={isCritical(feature) ? 'default' : 'secondary'}
                        className="font-normal"
                      >
                        {feature}
                        {isCritical(feature) && ' ⭐'}
                      </Badge>
                    ))}
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Critical Path */}
      {buildOrder.critical_path.length > 0 && (
        <Card className="border-yellow-500/50 bg-yellow-500/5">
          <CardContent className="p-4">
            <div className="space-y-2">
              <div className="flex items-center gap-2">
                <span className="font-medium">Critical Path</span>
                <Badge variant="outline" className="bg-yellow-500/10">
                  {buildOrder.critical_path.length} features
                </Badge>
              </div>
              <p className="text-sm text-muted-foreground">
                Features on the critical path must be completed sequentially. Delays here impact the
                entire timeline.
              </p>
              <div className="flex items-center gap-2 text-sm">
                {buildOrder.critical_path.map((feature, index) => (
                  <span key={feature} className="flex items-center">
                    <span className="font-mono">{feature}</span>
                    {index < buildOrder.critical_path.length - 1 && (
                      <span className="mx-2 text-muted-foreground">→</span>
                    )}
                  </span>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Optimization Notes */}
      {buildOrder.optimization_notes && (
        <Card>
          <CardContent className="p-4">
            <p className="text-sm text-muted-foreground whitespace-pre-wrap">
              {buildOrder.optimization_notes}
            </p>
          </CardContent>
        </Card>
      )}

      <div className="text-xs text-muted-foreground">
        <p>⭐ = Critical path feature · Features in same phase can be built in parallel</p>
      </div>
    </div>
  );
}
