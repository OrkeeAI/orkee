// ABOUTME: Loading state component displaying skeleton loaders during PRD generation
// ABOUTME: Shows progress indicator and PRD sections being generated
import { Loader2 } from 'lucide-react';
import { Skeleton } from '@/components/ui/skeleton';
import { Card, CardContent, CardHeader } from '@/components/ui/card';

const PRD_SECTIONS = [
  'Overview',
  'Core Features',
  'User Experience',
  'Technical Architecture',
  'Development Roadmap',
  'Logical Dependency Chain',
  'Risks and Mitigations',
  'Appendix',
];

interface GeneratingStateProps {
  message?: string;
}

export function GeneratingState({ message = 'Generating your PRD...' }: GeneratingStateProps) {
  return (
    <div className="space-y-6">
      {/* Header with spinner */}
      <div className="flex items-center justify-center gap-3 py-6">
        <Loader2 className="h-6 w-6 animate-spin text-primary" />
        <p className="text-lg font-medium">{message}</p>
      </div>

      {/* Section Skeletons */}
      <div className="space-y-4">
        {PRD_SECTIONS.map((section, index) => (
          <Card key={section} className="animate-pulse" style={{ animationDelay: `${index * 100}ms` }}>
            <CardHeader>
              <Skeleton className="h-6 w-48" />
            </CardHeader>
            <CardContent className="space-y-2">
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-full" />
              <Skeleton className="h-4 w-3/4" />
            </CardContent>
          </Card>
        ))}
      </div>

      {/* Progress hint */}
      <div className="text-center text-sm text-muted-foreground">
        <p>This may take 30-60 seconds depending on complexity</p>
      </div>
    </div>
  );
}
