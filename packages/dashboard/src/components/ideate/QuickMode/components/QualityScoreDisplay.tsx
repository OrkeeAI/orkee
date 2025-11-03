// ABOUTME: Displays overall quality score for Quick Mode generated PRD
// ABOUTME: Shows section-by-section scores and overall readiness status

import { CheckCircle, XCircle, AlertCircle } from 'lucide-react';
import { Progress } from '@/components/ui/progress';
import { Card, CardContent, CardHeader, CardTitle, CardDescription } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import type { QualityScore } from '@/services/ideate';

interface QualityScoreDisplayProps {
  qualityScore: QualityScore;
  className?: string;
}

export function QualityScoreDisplay({ qualityScore, className = '' }: QualityScoreDisplayProps) {
  const getScoreColor = (score: number) => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getScoreIcon = (score: number) => {
    if (score >= 80) return <CheckCircle className="h-5 w-5 text-green-600" />;
    if (score >= 60) return <AlertCircle className="h-5 w-5 text-yellow-600" />;
    return <XCircle className="h-5 w-5 text-red-600" />;
  };

  return (
    <Card className={className}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Overall Quality Score</CardTitle>
            <CardDescription>
              {qualityScore.is_ready_for_prd ? 'Ready to save' : 'Needs improvement'}
            </CardDescription>
          </div>
          <Badge variant={qualityScore.is_ready_for_prd ? 'default' : 'secondary'}>
            {qualityScore.is_ready_for_prd ? 'Ready' : 'Not Ready'}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Overall Score */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Overall Score</span>
            <div className="flex items-center gap-2">
              {getScoreIcon(qualityScore.overall_score)}
              <span className={`text-lg font-bold ${getScoreColor(qualityScore.overall_score)}`}>
                {qualityScore.overall_score}/100
              </span>
            </div>
          </div>
          <Progress value={qualityScore.overall_score} className="h-2" />
        </div>

        {/* Section Scores */}
        {Object.keys(qualityScore.section_scores).length > 0 && (
          <div className="space-y-3">
            <span className="text-sm font-medium">Section Scores</span>
            {Object.entries(qualityScore.section_scores).map(([section, score]) => (
              <div key={section} className="space-y-1">
                <div className="flex items-center justify-between">
                  <span className="text-sm capitalize">{section}</span>
                  <div className="flex items-center gap-2">
                    {getScoreIcon(score)}
                    <span className={`text-sm font-semibold ${getScoreColor(score)}`}>
                      {score}/100
                    </span>
                  </div>
                </div>
                <Progress value={score} className="h-1" />
              </div>
            ))}
          </div>
        )}

        {/* Missing Required Sections */}
        {qualityScore.missing_required.length > 0 && (
          <div className="p-3 border border-red-200 bg-red-50 rounded-md">
            <div className="flex items-start gap-2">
              <XCircle className="h-4 w-4 text-red-600 mt-0.5" />
              <div className="flex-1">
                <div className="text-sm font-semibold text-red-900">Missing Required Sections</div>
                <ul className="mt-1 text-xs text-red-700 list-disc pl-4 space-y-0.5">
                  {qualityScore.missing_required.map((section) => (
                    <li key={section} className="capitalize">{section}</li>
                  ))}
                </ul>
              </div>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
