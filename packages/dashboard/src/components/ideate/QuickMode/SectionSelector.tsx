// ABOUTME: Section selection component for expanding specific PRD sections
// ABOUTME: Checkbox list with select all/deselect all functionality
import { useState, useEffect } from 'react';
import { Checkbox } from '@/components/ui/checkbox';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';

export const PRD_SECTIONS = [
  { id: 'overview', name: 'Overview', description: 'Problem, target audience, value proposition' },
  { id: 'features', name: 'Core Features', description: 'What, why, and how of each feature' },
  { id: 'ux', name: 'User Experience', description: 'Personas, flows, UI/UX considerations' },
  { id: 'technical', name: 'Technical Architecture', description: 'Components, data model, APIs' },
  { id: 'roadmap', name: 'Development Roadmap', description: 'MVP and future phases (scope only)' },
  { id: 'dependencies', name: 'Logical Dependency Chain', description: 'Foundation → Visible → Enhancement' },
  { id: 'risks', name: 'Risks and Mitigations', description: 'Technical and resource risks' },
  { id: 'appendix', name: 'Appendix', description: 'Research notes and references' },
];

interface SectionSelectorProps {
  selectedSections: string[];
  onSelectionChange: (sections: string[]) => void;
  onConfirm?: () => void;
  showConfirmButton?: boolean;
}

export function SectionSelector({
  selectedSections,
  onSelectionChange,
  onConfirm,
  showConfirmButton = false,
}: SectionSelectorProps) {
  const [localSelection, setLocalSelection] = useState<string[]>(selectedSections);

  useEffect(() => {
    setLocalSelection(selectedSections);
  }, [selectedSections]);

  const handleToggle = (sectionId: string) => {
    const newSelection = localSelection.includes(sectionId)
      ? localSelection.filter((id) => id !== sectionId)
      : [...localSelection, sectionId];
    setLocalSelection(newSelection);
    onSelectionChange(newSelection);
  };

  const handleSelectAll = () => {
    const allIds = PRD_SECTIONS.map((s) => s.id);
    setLocalSelection(allIds);
    onSelectionChange(allIds);
  };

  const handleDeselectAll = () => {
    setLocalSelection([]);
    onSelectionChange([]);
  };

  const allSelected = localSelection.length === PRD_SECTIONS.length;
  const noneSelected = localSelection.length === 0;

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Select Sections to Expand</CardTitle>
            <CardDescription>
              Choose which PRD sections you want to generate or regenerate
            </CardDescription>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={handleSelectAll}
              disabled={allSelected}
            >
              Select All
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={handleDeselectAll}
              disabled={noneSelected}
            >
              Deselect All
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {PRD_SECTIONS.map((section) => (
            <div
              key={section.id}
              className="flex items-start space-x-3 p-3 rounded-lg border hover:bg-accent transition-colors cursor-pointer"
              onClick={() => handleToggle(section.id)}
            >
              <Checkbox
                id={section.id}
                checked={localSelection.includes(section.id)}
                onCheckedChange={() => handleToggle(section.id)}
                onClick={(e) => e.stopPropagation()}
              />
              <div className="flex-1 space-y-1">
                <Label
                  htmlFor={section.id}
                  className="text-sm font-medium cursor-pointer leading-none"
                >
                  {section.name}
                </Label>
                <p className="text-xs text-muted-foreground">{section.description}</p>
              </div>
            </div>
          ))}
        </div>

        {showConfirmButton && onConfirm && (
          <div className="flex justify-end pt-4 border-t">
            <Button onClick={onConfirm} disabled={noneSelected}>
              Expand {localSelection.length} Section{localSelection.length !== 1 ? 's' : ''}
            </Button>
          </div>
        )}

        <p className="text-xs text-muted-foreground text-center">
          Selected {localSelection.length} of {PRD_SECTIONS.length} sections
        </p>
      </CardContent>
    </Card>
  );
}
