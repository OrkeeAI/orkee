// ABOUTME: Smart answer selector for formatted question options
// ABOUTME: Supports letter (A,B,C), number (1,2), and open-ended formats with keyboard shortcuts

import React, { useState, useEffect } from 'react';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { cn } from '@/lib/utils';

export interface FormattedOption {
  prefix: string; // "A", "B", "C" or "1", "2"
  text: string;
  value: string;
}

export type AnswerFormat = 'open' | 'letter' | 'number' | 'scale';

export interface AnswerSelectorProps {
  questionText: string;
  answerFormat: AnswerFormat;
  formattedOptions?: FormattedOption[];
  canSkip?: boolean;
  onSubmit: (answer: string) => void;
  disabled?: boolean;
}

export function AnswerSelector({
  questionText,
  answerFormat,
  formattedOptions = [],
  canSkip = false,
  onSubmit,
  disabled = false,
}: AnswerSelectorProps) {
  const [selectedOption, setSelectedOption] = useState<string | null>(null);
  const [otherText, setOtherText] = useState('');
  const [showOther, setShowOther] = useState(false);

  // Reset state when question changes
  useEffect(() => {
    setSelectedOption(null);
    setOtherText('');
    setShowOther(false);
  }, [questionText]);

  // Keyboard shortcuts for letter/number selection
  useEffect(() => {
    if (disabled || answerFormat === 'open') return;

    const handleKeyDown = (e: KeyboardEvent) => {
      // Check if user is typing in an input
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
        return;
      }

      const key = e.key.toUpperCase();

      // Handle letter shortcuts (A, B, C, D)
      if (answerFormat === 'letter') {
        const option = formattedOptions.find((opt) => opt.prefix === key);
        if (option) {
          e.preventDefault();
          setSelectedOption(option.value);
        }
      }

      // Handle number shortcuts (1, 2, 3, 4)
      if (answerFormat === 'number') {
        const option = formattedOptions.find((opt) => opt.prefix === key);
        if (option) {
          e.preventDefault();
          setSelectedOption(option.value);
        }
      }

      // Handle Enter to submit
      if (e.key === 'Enter' && selectedOption) {
        e.preventDefault();
        handleSubmit();
      }

      // Handle 'O' for Other
      if (key === 'O') {
        e.preventDefault();
        setShowOther(true);
        setSelectedOption(null);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [answerFormat, formattedOptions, selectedOption, disabled]);

  const handleSubmit = () => {
    if (showOther && otherText.trim()) {
      onSubmit(otherText.trim());
    } else if (selectedOption) {
      onSubmit(selectedOption);
    }
  };

  const handleSkip = () => {
    onSubmit('');
  };

  // Open-ended answer format
  if (answerFormat === 'open') {
    return (
      <div className="space-y-3">
        <Input
          value={otherText}
          onChange={(e) => setOtherText(e.target.value)}
          placeholder="Type your answer..."
          disabled={disabled}
          onKeyDown={(e) => {
            if (e.key === 'Enter' && !e.shiftKey && otherText.trim()) {
              e.preventDefault();
              handleSubmit();
            }
          }}
          className="text-base"
        />
        <div className="flex gap-2">
          <Button
            onClick={handleSubmit}
            disabled={disabled || !otherText.trim()}
            className="flex-1"
          >
            Submit
          </Button>
          {canSkip && (
            <Button onClick={handleSkip} disabled={disabled} variant="outline">
              Skip
            </Button>
          )}
        </div>
      </div>
    );
  }

  // Formatted options (letter/number)
  return (
    <div className="space-y-4">
      {/* Options Grid */}
      <div className="grid gap-2">
        {formattedOptions.map((option) => (
          <button
            key={option.prefix}
            onClick={() => setSelectedOption(option.value)}
            disabled={disabled}
            className={cn(
              'flex items-start gap-3 p-3 rounded-lg border-2 transition-all text-left',
              'hover:border-primary/50 hover:bg-accent/50',
              'disabled:opacity-50 disabled:cursor-not-allowed',
              selectedOption === option.value
                ? 'border-primary bg-primary/5'
                : 'border-border'
            )}
          >
            <div
              className={cn(
                'flex items-center justify-center w-8 h-8 rounded-full font-semibold text-sm shrink-0',
                selectedOption === option.value
                  ? 'bg-primary text-primary-foreground'
                  : 'bg-muted text-muted-foreground'
              )}
            >
              {option.prefix}
            </div>
            <div className="flex-1 pt-1">
              <span className="text-base">{option.text}</span>
            </div>
          </button>
        ))}

        {/* Other Option */}
        <button
          onClick={() => {
            setShowOther(true);
            setSelectedOption(null);
          }}
          disabled={disabled}
          className={cn(
            'flex items-start gap-3 p-3 rounded-lg border-2 transition-all text-left',
            'hover:border-primary/50 hover:bg-accent/50',
            'disabled:opacity-50 disabled:cursor-not-allowed',
            showOther ? 'border-primary bg-primary/5' : 'border-border'
          )}
        >
          <div
            className={cn(
              'flex items-center justify-center w-8 h-8 rounded-full font-semibold text-sm shrink-0',
              showOther
                ? 'bg-primary text-primary-foreground'
                : 'bg-muted text-muted-foreground'
            )}
          >
            O
          </div>
          <div className="flex-1 pt-1">
            <span className="text-base">Other (specify)</span>
          </div>
        </button>
      </div>

      {/* Other Text Input */}
      {showOther && (
        <div className="space-y-2 pl-11">
          <Label htmlFor="other-text" className="text-sm text-muted-foreground">
            Please specify:
          </Label>
          <Input
            id="other-text"
            value={otherText}
            onChange={(e) => setOtherText(e.target.value)}
            placeholder="Type your answer..."
            disabled={disabled}
            autoFocus
            onKeyDown={(e) => {
              if (e.key === 'Enter' && otherText.trim()) {
                e.preventDefault();
                handleSubmit();
              }
            }}
          />
        </div>
      )}

      {/* Submit Button */}
      <div className="flex gap-2">
        <Button
          onClick={handleSubmit}
          disabled={disabled || (!selectedOption && (!showOther || !otherText.trim()))}
          className="flex-1"
        >
          Submit Answer
        </Button>
        {canSkip && (
          <Button onClick={handleSkip} disabled={disabled} variant="outline">
            Skip
          </Button>
        )}
      </div>

      {/* Keyboard Hint */}
      {!disabled && (
        <p className="text-xs text-muted-foreground text-center">
          {answerFormat === 'letter'
            ? 'Press A, B, C, D or O for quick selection'
            : 'Press 1, 2, 3, 4 or O for quick selection'}
        </p>
      )}
    </div>
  );
}
