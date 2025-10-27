// ABOUTME: Input component for Quick Mode one-liner description
// ABOUTME: Includes validation, character counter, and clear button
import { useState } from 'react';
import { X } from 'lucide-react';
import { Textarea } from '@/components/ui/textarea';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { cn } from '@/lib/utils';

interface OneLineInputProps {
  value: string;
  onChange: (value: string) => void;
  onGenerate: () => void;
  isGenerating?: boolean;
  error?: string;
}

const MIN_LENGTH = 10;
const SHOW_COUNTER_THRESHOLD = 500;

export function OneLineInput({
  value,
  onChange,
  onGenerate,
  isGenerating = false,
  error,
}: OneLineInputProps) {
  const [isFocused, setIsFocused] = useState(false);

  const isValid = value.trim().length >= MIN_LENGTH;
  const showCounter = value.length >= SHOW_COUNTER_THRESHOLD;

  const handleClear = () => {
    onChange('');
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && (e.metaKey || e.ctrlKey) && isValid && !isGenerating) {
      e.preventDefault();
      onGenerate();
    }
  };

  return (
    <div className="space-y-4">
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label htmlFor="description">Project Description *</Label>
          {showCounter && (
            <span className="text-xs text-muted-foreground">{value.length} characters</span>
          )}
        </div>

        <div className="relative">
          <Textarea
            id="description"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            onFocus={() => setIsFocused(true)}
            onBlur={() => setIsFocused(false)}
            onKeyDown={handleKeyDown}
            placeholder="Example: A mobile app for tracking daily water intake with reminders, progress visualization, and social sharing features"
            rows={6}
            className={cn(
              'resize-none pr-10',
              error && 'border-destructive focus-visible:ring-destructive',
              !isValid && value.length > 0 && !isFocused && 'border-warning'
            )}
            disabled={isGenerating}
            required
          />
          {value && !isGenerating && (
            <Button
              type="button"
              variant="ghost"
              size="sm"
              className="absolute right-2 top-2 h-6 w-6 p-0"
              onClick={handleClear}
              tabIndex={-1}
            >
              <X className="h-4 w-4" />
              <span className="sr-only">Clear</span>
            </Button>
          )}
        </div>

        {error && <p className="text-sm text-destructive">{error}</p>}

        {!isValid && value.length > 0 && !isFocused && (
          <p className="text-sm text-warning">
            Please provide at least {MIN_LENGTH} characters (currently {value.trim().length})
          </p>
        )}

        <p className="text-xs text-muted-foreground">
          Be specific! The more detail you provide, the better the AI-generated PRD will be.
          Press{' '}
          <kbd className="px-1.5 py-0.5 text-xs font-semibold bg-muted rounded">
            {navigator.platform.includes('Mac') ? '⌘' : 'Ctrl'}+Enter
          </kbd>{' '}
          to generate.
        </p>
      </div>

      <div className="flex gap-2">
        <Button onClick={onGenerate} disabled={!isValid || isGenerating} className="flex-1">
          {isGenerating ? 'Generating PRD...' : 'Generate PRD'}
        </Button>
        {value && !isGenerating && (
          <Button type="button" variant="outline" onClick={handleClear}>
            Clear
          </Button>
        )}
      </div>
    </div>
  );
}
