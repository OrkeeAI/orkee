// ABOUTME: Mode selection component for brainstorm session creation
// ABOUTME: Displays three mode options (Quick, Guided, Comprehensive) with descriptions
import React from 'react';
import { Card, CardHeader, CardTitle, CardDescription, CardContent } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Zap, MapPin, Sparkles } from 'lucide-react';
import type { BrainstormMode } from '@/services/brainstorm';
import { cn } from '@/lib/utils';

interface ModeSelectorProps {
  selectedMode: BrainstormMode | null;
  onSelectMode: (mode: BrainstormMode) => void;
  onConfirm?: () => void;
}

interface ModeOption {
  mode: BrainstormMode;
  title: string;
  description: string;
  icon: React.ReactNode;
  features: string[];
}

const MODE_OPTIONS: ModeOption[] = [
  {
    mode: 'quick',
    title: 'Quick Mode',
    description: 'One-liner to complete PRD in seconds',
    icon: <Zap className="h-6 w-6" />,
    features: [
      'Enter a simple description',
      'AI generates full PRD instantly',
      'All 8 sections auto-filled',
      'Edit before saving',
    ],
  },
  {
    mode: 'guided',
    title: 'Guided Mode',
    description: 'Step-by-step PRD creation with AI assistance',
    icon: <MapPin className="h-6 w-6" />,
    features: [
      'Navigate through sections',
      'Skip optional sections',
      'AI suggestions for each part',
      'Full control over content',
    ],
  },
  {
    mode: 'comprehensive',
    title: 'Comprehensive Mode',
    description: 'Deep brainstorming with research and expert roundtables',
    icon: <Sparkles className="h-6 w-6" />,
    features: [
      'Competitor analysis',
      'Expert AI roundtable discussions',
      'Similar project research',
      'In-depth ideation',
    ],
  },
];

export function ModeSelector({ selectedMode, onSelectMode, onConfirm }: ModeSelectorProps) {
  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold tracking-tight">Choose Your Brainstorming Mode</h2>
        <p className="text-muted-foreground mt-2">
          Select how you want to create your PRD. You can always edit the result later.
        </p>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {MODE_OPTIONS.map((option) => (
          <Card
            key={option.mode}
            className={cn(
              'cursor-pointer transition-all hover:shadow-md',
              selectedMode === option.mode
                ? 'ring-2 ring-primary border-primary bg-primary/5'
                : 'hover:border-primary/50'
            )}
            onClick={() => onSelectMode(option.mode)}
          >
            <CardHeader>
              <div className="flex items-center gap-3 mb-2">
                <div
                  className={cn(
                    'p-2 rounded-lg',
                    selectedMode === option.mode
                      ? 'bg-primary text-primary-foreground'
                      : 'bg-muted'
                  )}
                >
                  {option.icon}
                </div>
                <CardTitle className="text-xl">{option.title}</CardTitle>
              </div>
              <CardDescription className="text-sm">{option.description}</CardDescription>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2 text-sm">
                {option.features.map((feature, index) => (
                  <li key={index} className="flex items-start gap-2">
                    <span className="text-primary mt-0.5">•</span>
                    <span className="text-muted-foreground">{feature}</span>
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        ))}
      </div>

      {onConfirm && (
        <div className="flex justify-end">
          <Button
            onClick={onConfirm}
            disabled={!selectedMode}
            size="lg"
            className="gap-2"
          >
            Continue with {selectedMode ? MODE_OPTIONS.find(m => m.mode === selectedMode)?.title : 'Selected Mode'}
          </Button>
        </div>
      )}
    </div>
  );
}
