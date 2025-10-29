// ABOUTME: Real-time PRD generation status screen for Quick Mode
// ABOUTME: Shows streaming progress with section-by-section updates and completion indicators

import { CheckCircle2, Loader2, FileText } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Progress } from '@/components/ui/progress';
import { cn } from '@/lib/utils';

const PRD_SECTIONS = [
  { id: 'overview', label: 'Overview' },
  { id: 'features', label: 'Core Features' },
  { id: 'ux', label: 'User Experience' },
  { id: 'technical', label: 'Technical Architecture' },
  { id: 'roadmap', label: 'Development Roadmap' },
  { id: 'dependencies', label: 'Logical Dependency Chain' },
  { id: 'risks', label: 'Risks and Mitigations' },
  { id: 'research', label: 'Appendix' },
];

interface GenerationStatusProps {
  partialPRD: any;
  message?: string;
}

export function GenerationStatus({ partialPRD, message = 'Generating your PRD...' }: GenerationStatusProps) {
  // Calculate progress based on completed sections
  // The streaming data has top-level keys for each section (overview, features, ux, etc.)
  const completedSections = partialPRD ? Object.keys(partialPRD).filter(key => 
    PRD_SECTIONS.some(section => section.id === key)
  ).length : 0;
  const totalSections = PRD_SECTIONS.length;
  const progressPercentage = (completedSections / totalSections) * 100;

  // Determine which section is currently being generated
  const currentSectionIndex = completedSections;
  const currentSection = PRD_SECTIONS[currentSectionIndex];

  return (
    <div className="space-y-6">
      {/* Header with Progress */}
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <CardTitle className="flex items-center gap-2">
              <FileText className="h-5 w-5" />
              {message}
            </CardTitle>
            <Badge variant="secondary" className="gap-1">
              <Loader2 className="h-3 w-3 animate-spin" />
              Generating
            </Badge>
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">Overall Progress</span>
              <span className="font-medium">{completedSections} / {totalSections} sections</span>
            </div>
            <Progress value={progressPercentage} className="h-2" />
          </div>
          {currentSection && (
            <div className="flex items-center gap-2 text-sm text-muted-foreground">
              <Loader2 className="h-4 w-4 animate-spin" />
              <span>Currently generating: <span className="font-medium text-foreground">{currentSection.label}</span></span>
            </div>
          )}
        </CardContent>
      </Card>

      {/* Section Status List */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base">Section Status</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="space-y-2">
            {PRD_SECTIONS.map((section, index) => {
              const isCompleted = partialPRD?.[section.id] !== undefined;
              const isCurrent = index === currentSectionIndex && !isCompleted;
              const isPending = index > currentSectionIndex;

              return (
                <div
                  key={section.id}
                  className={cn(
                    'flex items-center justify-between p-3 rounded-lg border transition-colors',
                    isCompleted && 'bg-green-50 border-green-200 dark:bg-green-950/20 dark:border-green-900',
                    isCurrent && 'bg-blue-50 border-blue-200 dark:bg-blue-950/20 dark:border-blue-900',
                    isPending && 'bg-muted/30 border-muted'
                  )}
                >
                  <div className="flex items-center gap-3">
                    {isCompleted ? (
                      <CheckCircle2 className="h-5 w-5 text-green-600 dark:text-green-400" />
                    ) : isCurrent ? (
                      <Loader2 className="h-5 w-5 text-blue-600 dark:text-blue-400 animate-spin" />
                    ) : (
                      <div className="h-5 w-5 rounded-full border-2 border-muted-foreground/30" />
                    )}
                    <span className={cn(
                      'font-medium',
                      isCompleted && 'text-green-700 dark:text-green-300',
                      isCurrent && 'text-blue-700 dark:text-blue-300',
                      isPending && 'text-muted-foreground'
                    )}>
                      {section.label}
                    </span>
                  </div>
                  <Badge
                    variant={isCompleted ? 'default' : isCurrent ? 'secondary' : 'outline'}
                    className="text-xs"
                  >
                    {isCompleted ? 'Complete' : isCurrent ? 'Generating...' : 'Pending'}
                  </Badge>
                </div>
              );
            })}
          </div>
        </CardContent>
      </Card>

      {/* Content Preview (if available) */}
      {partialPRD && completedSections > 0 && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Latest Generated Content</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4 max-h-96 overflow-y-auto">
              {PRD_SECTIONS.filter(section => partialPRD[section.id]).map((section) => {
                const content = partialPRD[section.id];
                // Extract a preview text from the content
                let previewText = '';
                if (typeof content === 'string') {
                  previewText = content.slice(0, 200);
                } else if (content && typeof content === 'object') {
                  // For objects, try to extract meaningful text
                  if (content.problemStatement) previewText = content.problemStatement.slice(0, 200);
                  else if (content.mvpScope) previewText = `MVP includes ${content.mvpScope.length} features`;
                  else if (Array.isArray(content)) previewText = `${content.length} items defined`;
                  else previewText = JSON.stringify(content).slice(0, 200);
                }
                
                return (
                  <div key={section.id} className="space-y-2">
                    <div className="flex items-center gap-2">
                      <CheckCircle2 className="h-4 w-4 text-green-600" />
                      <h4 className="font-medium text-sm">{section.label}</h4>
                    </div>
                    <div className="text-sm text-muted-foreground pl-6 line-clamp-3">
                      {previewText}...
                    </div>
                  </div>
                );
              })}
            </div>
          </CardContent>
        </Card>
      )}

      {/* Progress hint */}
      <div className="text-center text-sm text-muted-foreground">
        <p>This may take 30-60 seconds depending on complexity</p>
      </div>
    </div>
  );
}
