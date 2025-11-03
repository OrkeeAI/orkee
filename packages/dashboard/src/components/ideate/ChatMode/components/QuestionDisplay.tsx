// ABOUTME: One-question-at-a-time display with progress tracking
// ABOUTME: Shows current question, progress counter, and navigation controls

import React from 'react';
import { Card, CardContent, CardHeader } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { ChevronLeft } from 'lucide-react';
import { Progress } from '@/components/ui/progress';
import { AnswerSelector, FormattedOption, AnswerFormat } from './AnswerSelector';
import { cn } from '@/lib/utils';

export interface Question {
  question_text: string;
  question_type: 'open' | 'multiple_choice' | 'yes_no';
  options?: string[];
  category: string;
  can_skip: boolean;
  answer_format: AnswerFormat;
  formatted_options?: FormattedOption[];
  question_number?: number;
  total_questions?: number;
}

export interface QuestionDisplayProps {
  question: Question;
  onSubmitAnswer: (answer: string) => void;
  onGoBack?: () => void;
  canGoBack?: boolean;
  disabled?: boolean;
  className?: string;
}

export function QuestionDisplay({
  question,
  onSubmitAnswer,
  onGoBack,
  canGoBack = false,
  disabled = false,
  className,
}: QuestionDisplayProps) {
  const questionNumber = question.question_number || 1;
  const totalQuestions = question.total_questions || 10;
  const progress = (questionNumber / totalQuestions) * 100;

  return (
    <Card className={cn('w-full', className)}>
      <CardHeader className="space-y-4">
        {/* Progress Bar */}
        <div className="space-y-2">
          <div className="flex items-center justify-between text-sm">
            <span className="font-medium text-muted-foreground">
              Question {questionNumber} of ~{totalQuestions}
            </span>
            <span className="text-muted-foreground">{Math.round(progress)}%</span>
          </div>
          <Progress value={progress} className="h-2" />
        </div>

        {/* Category Badge */}
        <div className="flex items-center gap-2">
          <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-primary/10 text-primary">
            {question.category}
          </span>
          {question.can_skip && (
            <span className="text-xs text-muted-foreground italic">(Optional)</span>
          )}
        </div>

        {/* Question Text */}
        <h3 className="text-xl font-semibold leading-relaxed">{question.question_text}</h3>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Answer Selector */}
        <AnswerSelector
          questionText={question.question_text}
          answerFormat={question.answer_format}
          formattedOptions={question.formatted_options}
          canSkip={question.can_skip}
          onSubmit={onSubmitAnswer}
          disabled={disabled}
        />

        {/* Navigation */}
        {canGoBack && onGoBack && (
          <div className="pt-4 border-t">
            <Button
              onClick={onGoBack}
              disabled={disabled}
              variant="ghost"
              size="sm"
              className="gap-2"
            >
              <ChevronLeft className="h-4 w-4" />
              Go Back
            </Button>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
