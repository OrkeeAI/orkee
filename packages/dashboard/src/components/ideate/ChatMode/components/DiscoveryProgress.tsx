// ABOUTME: Discovery progress indicator for one-question-at-a-time flow
// ABOUTME: Shows question number, progress bar, and estimated remaining time

import React from 'react';
import { Progress } from '@/components/ui/progress';
import { Clock } from 'lucide-react';
import { DiscoveryProgress as DiscoveryProgressType } from '@/services/ideate';

export interface DiscoveryProgressProps {
  progress: DiscoveryProgressType | null;
  className?: string;
}

export function DiscoveryProgress({ progress, className = '' }: DiscoveryProgressProps) {
  if (!progress) {
    return null;
  }

  return (
    <div className={`space-y-2 ${className}`}>
      <div className="flex items-center justify-between text-sm">
        <span className="font-medium text-muted-foreground">
          Question {progress.current_question_number} of ~{progress.total_questions}
        </span>
        <div className="flex items-center gap-1 text-muted-foreground">
          <Clock className="h-3 w-3" />
          <span>{progress.estimated_remaining} min remaining</span>
        </div>
      </div>
      <Progress value={progress.completion_percentage} className="h-2" />
      <p className="text-xs text-muted-foreground">
        {progress.answered_questions} questions answered
      </p>
    </div>
  );
}
