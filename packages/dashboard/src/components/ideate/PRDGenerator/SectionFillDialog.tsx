// ABOUTME: Dialog for selecting and AI-filling skipped PRD sections
// ABOUTME: Allows batch selection of sections to be filled with AI-generated content

import { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Checkbox } from '@/components/ui/checkbox';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { RefreshCw, Loader2, Info, AlertCircle } from 'lucide-react';
import { useFillSkippedSections } from '@/hooks/useIdeate';

interface SectionFillDialogProps {
  sessionId: string;
  skippedSections: string[];
  onClose: () => void;
}

export function SectionFillDialog({
  sessionId,
  skippedSections,
  onClose,
}: SectionFillDialogProps) {
  const [selectedSections, setSelectedSections] = useState<string[]>([...skippedSections]);
  const fillMutation = useFillSkippedSections(sessionId);

  const handleToggleSection = (section: string) => {
    setSelectedSections((prev) =>
      prev.includes(section)
        ? prev.filter((s) => s !== section)
        : [...prev, section]
    );
  };

  const handleToggleAll = () => {
    if (selectedSections.length === skippedSections.length) {
      setSelectedSections([]);
    } else {
      setSelectedSections([...skippedSections]);
    }
  };

  const handleFill = async () => {
    if (selectedSections.length === 0) return;

    try {
      await fillMutation.mutateAsync(selectedSections);
      onClose();
    } catch (error) {
      console.error('Failed to fill sections:', error);
    }
  };

  const sectionDisplayNames: Record<string, string> = {
    overview: 'Product Overview',
    features: 'Features & Requirements',
    ux: 'UX & User Experience',
    technical: 'Technical Requirements',
    roadmap: 'Roadmap & Milestones',
    dependencies: 'Dependencies & Prerequisites',
    risks: 'Risks & Mitigation',
    research: 'Research & Competitive Analysis',
  };

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="max-w-lg">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <RefreshCw className="h-5 w-5" />
            Fill Skipped Sections
          </DialogTitle>
          <DialogDescription>
            Select sections to be filled with AI-generated content based on your session data.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Info Alert */}
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              AI will generate content for selected sections using context from your ideation
              session. This helps complete your PRD quickly while maintaining consistency.
            </AlertDescription>
          </Alert>

          {/* No Skipped Sections */}
          {skippedSections.length === 0 && (
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                No skipped sections found. All sections have been completed!
              </AlertDescription>
            </Alert>
          )}

          {/* Section Selection */}
          {skippedSections.length > 0 && (
            <div className="space-y-3">
              {/* Select All Toggle */}
              <div className="flex items-center space-x-2 pb-2 border-b">
                <Checkbox
                  id="selectAll"
                  checked={selectedSections.length === skippedSections.length}
                  onCheckedChange={handleToggleAll}
                />
                <label
                  htmlFor="selectAll"
                  className="text-sm font-semibold leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70"
                >
                  Select All ({skippedSections.length} sections)
                </label>
              </div>

              {/* Individual Sections */}
              <div className="space-y-2 max-h-64 overflow-y-auto">
                {skippedSections.map((section) => (
                  <div
                    key={section}
                    className="flex items-center space-x-2 rounded-lg border p-3 hover:bg-muted/50"
                  >
                    <Checkbox
                      id={section}
                      checked={selectedSections.includes(section)}
                      onCheckedChange={() => handleToggleSection(section)}
                    />
                    <label
                      htmlFor={section}
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 flex-1 cursor-pointer"
                    >
                      {sectionDisplayNames[section] || section}
                    </label>
                  </div>
                ))}
              </div>

              {/* Selection Summary */}
              <div className="pt-2 border-t">
                <p className="text-sm text-muted-foreground">
                  {selectedSections.length} of {skippedSections.length} section
                  {skippedSections.length !== 1 ? 's' : ''} selected
                </p>
              </div>
            </div>
          )}

          {/* Error Display */}
          {fillMutation.isError && (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                Failed to fill sections. Please try again.
              </AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter>
          <Button variant="outline" onClick={onClose} disabled={fillMutation.isPending}>
            Cancel
          </Button>
          <Button
            onClick={handleFill}
            disabled={selectedSections.length === 0 || fillMutation.isPending}
            className="gap-2"
          >
            {fillMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Filling Sections...
              </>
            ) : (
              <>
                <RefreshCw className="h-4 w-4" />
                Fill {selectedSections.length} Section{selectedSections.length !== 1 ? 's' : ''}
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
