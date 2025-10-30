// ABOUTME: Section navigation sidebar with completion indicators
// ABOUTME: Clickable list of sections showing current, completed, and skipped states
import { CheckCircle2, Circle, SkipForward } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { SectionName } from './GuidedModeFlow';
import type { SessionCompletionStatus } from '@/services/ideate';

interface Section {
  id: SectionName;
  label: string;
  description: string;
}

interface SectionNavigatorProps {
  sections: Section[];
  currentSection: SectionName;
  onSectionSelect: (sectionId: SectionName) => void;
  completionStatus?: SessionCompletionStatus;
}

export function SectionNavigator({
  sections,
  currentSection,
  onSectionSelect,
  completionStatus,
}: SectionNavigatorProps) {
  const isSkipped = (sectionId: string) => {
    return completionStatus?.skipped_sections?.includes(sectionId) || false;
  };

  const isCompleted = (sectionId: string, index: number) => {
    const currentIndex = sections.findIndex(s => s.id === currentSection);
    // Section is completed if it's before the current section and not skipped
    return index < currentIndex && !isSkipped(sectionId);
  };

  return (
    <nav className="flex-1 overflow-y-auto">
      <div className="space-y-1">
        {sections.map((section, index) => {
          const isCurrent = section.id === currentSection;
          const completed = isCompleted(section.id, index);
          const skipped = isSkipped(section.id);

          return (
            <button
              key={section.id}
              onClick={() => onSectionSelect(section.id)}
              className={cn(
                'w-full text-left p-3 rounded-lg transition-colors',
                'hover:bg-muted',
                isCurrent && 'bg-primary/10 border border-primary',
                !isCurrent && !completed && !skipped && 'opacity-60'
              )}
            >
              <div className="flex items-start gap-3">
                {/* Status Icon */}
                <div className="mt-0.5">
                  {completed && (
                    <CheckCircle2 className="w-5 h-5 text-green-600" />
                  )}
                  {skipped && (
                    <SkipForward className="w-5 h-5 text-orange-500" />
                  )}
                  {!completed && !skipped && (
                    <Circle className={cn(
                      'w-5 h-5',
                      isCurrent ? 'text-primary fill-primary' : 'text-muted-foreground'
                    )} />
                  )}
                </div>

                {/* Section Info */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center gap-2">
                    <span className={cn(
                      'font-medium text-sm',
                      isCurrent && 'text-primary'
                    )}>
                      {section.label}
                    </span>
                    {skipped && (
                      <span className="text-xs px-1.5 py-0.5 rounded bg-orange-100 text-orange-700">
                        Skipped
                      </span>
                    )}
                  </div>
                  <p className="text-xs text-muted-foreground mt-0.5 line-clamp-2">
                    {section.description}
                  </p>
                </div>

                {/* Step Number */}
                <div className="text-xs text-muted-foreground font-medium">
                  {index + 1}
                </div>
              </div>
            </button>
          );
        })}
      </div>
    </nav>
  );
}
