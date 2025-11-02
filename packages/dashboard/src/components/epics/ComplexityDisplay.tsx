// ABOUTME: Complexity score and reasoning display for epics
// ABOUTME: Shows complexity analysis with reasoning and recommendations

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { AlertCircle, CheckCircle2, TrendingUp } from 'lucide-react';
import type { ComplexityReport } from '@/services/epics';

interface ComplexityDisplayProps {
  report: ComplexityReport | null;
}

export function ComplexityDisplay({ report }: ComplexityDisplayProps) {
  if (!report) {
    return (
      <Card>
        <CardContent className="flex flex-col items-center justify-center py-12">
          <AlertCircle className="h-12 w-12 text-muted-foreground mb-4" />
          <p className="text-lg font-medium text-muted-foreground">
            No complexity analysis available
          </p>
        </CardContent>
      </Card>
    );
  }

  const getComplexityLabel = (score: number) => {
    if (score <= 3) return { label: 'Low', color: 'bg-green-500' };
    if (score <= 6) return { label: 'Medium', color: 'bg-yellow-500' };
    if (score <= 8) return { label: 'High', color: 'bg-orange-500' };
    return { label: 'Very High', color: 'bg-red-500' };
  };

  const { label, color } = getComplexityLabel(report.complexityScore);

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Complexity Analysis</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex items-center justify-between">
            <div className="space-y-1">
              <p className="text-sm font-medium text-muted-foreground">Complexity Score</p>
              <div className="flex items-center gap-3">
                <p className="text-3xl font-bold">{report.complexityScore}/10</p>
                <Badge className={`${color} text-white border-0`}>{label}</Badge>
              </div>
            </div>
            <div className="space-y-1 text-right">
              <p className="text-sm font-medium text-muted-foreground">Recommended Tasks</p>
              <p className="text-3xl font-bold">{report.recommendedTaskCount}</p>
            </div>
          </div>
          <Progress value={report.complexityScore * 10} className="h-3" />
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <CheckCircle2 className="h-5 w-5" />
            Reasoning
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground whitespace-pre-wrap">{report.reasoning}</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base flex items-center gap-2">
            <TrendingUp className="h-5 w-5" />
            Expansion Strategy
          </CardTitle>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground whitespace-pre-wrap">{report.expansionStrategy}</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-base">Task Count vs. Limit</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-muted-foreground">Recommended: {report.recommendedTaskCount}</span>
            <span className="text-sm text-muted-foreground">Limit: 20</span>
          </div>
          <Progress value={(report.recommendedTaskCount / 20) * 100} className="h-2" />
        </CardContent>
      </Card>
    </div>
  );
}
