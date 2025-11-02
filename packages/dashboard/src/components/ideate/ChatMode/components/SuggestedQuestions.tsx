// ABOUTME: Suggested questions component for guiding chat discovery
// ABOUTME: Displays clickable question chips based on chat context

import React from 'react';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { HelpCircle } from 'lucide-react';
import type { DiscoveryQuestion } from '@/services/chat';

export interface SuggestedQuestionsProps {
  questions: DiscoveryQuestion[];
  onSelectQuestion: (question: string) => void;
  isDisabled?: boolean;
}

const CATEGORY_LABELS: Record<string, string> = {
  problem: 'Problem',
  users: 'Users',
  features: 'Features',
  technical: 'Technical',
  risks: 'Risks',
  constraints: 'Constraints',
  success: 'Success',
};

const CATEGORY_COLORS: Record<string, string> = {
  problem: 'bg-red-100 text-red-800 dark:bg-red-900 dark:text-red-200',
  users: 'bg-blue-100 text-blue-800 dark:bg-blue-900 dark:text-blue-200',
  features: 'bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200',
  technical: 'bg-purple-100 text-purple-800 dark:bg-purple-900 dark:text-purple-200',
  risks: 'bg-orange-100 text-orange-800 dark:bg-orange-900 dark:text-orange-200',
  constraints: 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900 dark:text-yellow-200',
  success: 'bg-teal-100 text-teal-800 dark:bg-teal-900 dark:text-teal-200',
};

export function SuggestedQuestions({
  questions,
  onSelectQuestion,
  isDisabled = false,
}: SuggestedQuestionsProps) {
  if (questions.length === 0) {
    return null;
  }

  const topQuestions = questions
    .sort((a, b) => b.priority - a.priority)
    .slice(0, 5);

  return (
    <div className="bg-muted/50 rounded-lg p-4 space-y-3">
      <div className="flex items-center gap-2 text-sm font-medium text-muted-foreground">
        <HelpCircle className="h-4 w-4" />
        <span>Suggested Questions</span>
      </div>

      <div className="flex flex-wrap gap-2">
        {topQuestions.map((question) => (
          <Button
            key={question.id}
            variant="outline"
            size="sm"
            onClick={() => onSelectQuestion(question.question_text)}
            disabled={isDisabled}
            className="h-auto py-2 px-3 text-left whitespace-normal justify-start"
          >
            <div className="flex flex-col gap-1 items-start">
              <Badge
                variant="secondary"
                className={CATEGORY_COLORS[question.category] || ''}
              >
                {CATEGORY_LABELS[question.category] || question.category}
              </Badge>
              <span className="text-sm">{question.question_text}</span>
            </div>
          </Button>
        ))}
      </div>
    </div>
  );
}
