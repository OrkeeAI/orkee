// ABOUTME: Visual progress indicator for Guided Mode showing completion status
// ABOUTME: Displays circular progress ring with percentage and section count
import { CheckCircle2 } from 'lucide-react';
import type { SectionName } from './GuidedModeFlow';

interface Section {
  id: SectionName;
  label: string;
  description: string;
}

interface SectionProgressProps {
  sections: Section[];
  currentSection: SectionName;
  completedSections: number;
  totalSections: number;
}

export function SectionProgress({
  sections,
  currentSection,
  completedSections,
  totalSections,
}: SectionProgressProps) {
  const percentage = totalSections > 0 ? Math.round((completedSections / totalSections) * 100) : 0;
  const currentIndex = sections.findIndex(s => s.id === currentSection);
  const sectionNumber = currentIndex + 1;

  return (
    <div className="mb-6 p-4 bg-primary/5 rounded-lg border">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-muted-foreground">Progress</span>
        <span className="text-sm font-semibold">{percentage}%</span>
      </div>

      {/* Progress Bar */}
      <div className="w-full h-2 bg-muted rounded-full overflow-hidden mb-3">
        <div
          className="h-full bg-primary transition-all duration-300"
          style={{ width: `${percentage}%` }}
        />
      </div>

      {/* Stats */}
      <div className="flex items-center justify-between text-sm">
        <div className="flex items-center gap-2 text-muted-foreground">
          <CheckCircle2 className="w-4 h-4" />
          <span>
            {completedSections} of {totalSections} sections
          </span>
        </div>
        <div className="font-medium">
          Section {sectionNumber}/{sections.length}
        </div>
      </div>
    </div>
  );
}
