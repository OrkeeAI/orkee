// ABOUTME: Dialog for skipping a section with optional AI generation
// ABOUTME: Allows users to skip manually or have AI generate placeholder content
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog';
import { Checkbox } from '@/components/ui/checkbox';
import { Label } from '@/components/ui/label';
import { Sparkles, SkipForward } from 'lucide-react';
import { useState } from 'react';

interface SkipDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  sectionName: string;
  onConfirm: (aiGenerate: boolean) => void;
}

export function SkipDialog({
  open,
  onOpenChange,
  sectionName,
  onConfirm,
}: SkipDialogProps) {
  const [aiGenerate, setAiGenerate] = useState(false);

  const handleConfirm = () => {
    onConfirm(aiGenerate);
    setAiGenerate(false); // Reset for next time
  };

  return (
    <AlertDialog open={open} onOpenChange={onOpenChange}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle className="flex items-center gap-2">
            <SkipForward className="w-5 h-5" />
            Skip {sectionName}?
          </AlertDialogTitle>
          <AlertDialogDescription className="space-y-4">
            <p>
              You can skip this section and come back to it later. The section will be
              marked as optional in your PRD.
            </p>

            <div className="flex items-start space-x-3 p-4 bg-primary/5 rounded-lg border">
              <Checkbox
                id="ai-generate"
                checked={aiGenerate}
                onCheckedChange={(checked) => setAiGenerate(checked === true)}
              />
              <div className="space-y-1 flex-1">
                <Label
                  htmlFor="ai-generate"
                  className="flex items-center gap-2 font-medium cursor-pointer"
                >
                  <Sparkles className="w-4 h-4 text-primary" />
                  Let AI generate this section
                </Label>
                <p className="text-sm text-muted-foreground">
                  AI will create placeholder content based on your project description.
                  You can edit it later.
                </p>
              </div>
            </div>

            <p className="text-sm text-muted-foreground">
              You can return to any skipped section using the navigation sidebar.
            </p>
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={handleConfirm}>
            {aiGenerate ? 'Skip & Generate' : 'Skip Section'}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
}
