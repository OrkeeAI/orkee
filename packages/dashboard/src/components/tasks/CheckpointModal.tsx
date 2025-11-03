// ABOUTME: Checkpoint modal for task execution validation and progress tracking
// ABOUTME: Displays validation checklists and allows marking checkpoints as complete

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
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertCircle, CheckCircle2 } from 'lucide-react';
import type { ExecutionCheckpoint } from '@/services/tasks';

interface CheckpointModalProps {
  checkpoint: ExecutionCheckpoint | null;
  open: boolean;
  onClose: () => void;
  onComplete: (validationResults: Record<string, boolean>) => void;
  onSkip?: () => void;
}

export function CheckpointModal({
  checkpoint,
  open,
  onClose,
  onComplete,
  onSkip,
}: CheckpointModalProps) {
  const [validationState, setValidationState] = useState<Record<string, boolean>>({});

  if (!checkpoint) return null;

  const handleValidationChange = (item: string, checked: boolean) => {
    setValidationState((prev) => ({ ...prev, [item]: checked }));
  };

  const allValidated = checkpoint.requiredValidation.every(
    (item) => validationState[item] === true
  );

  const getCheckpointTypeColor = (type: string) => {
    switch (type) {
      case 'review':
        return 'bg-blue-500';
      case 'test':
        return 'bg-green-500';
      case 'integration':
        return 'bg-purple-500';
      case 'approval':
        return 'bg-orange-500';
      default:
        return 'bg-gray-500';
    }
  };

  const handleComplete = () => {
    onComplete(validationState);
    setValidationState({});
    onClose();
  };

  const handleSkip = () => {
    if (onSkip) onSkip();
    setValidationState({});
    onClose();
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="sm:max-w-[600px]">
        <DialogHeader>
          <div className="flex items-center gap-2 mb-2">
            <Badge className={`${getCheckpointTypeColor(checkpoint.checkpointType)} text-white border-0`}>
              {checkpoint.checkpointType}
            </Badge>
            <DialogTitle>Checkpoint Reached</DialogTitle>
          </div>
          <DialogDescription>{checkpoint.message}</DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {checkpoint.requiredValidation.length > 0 ? (
            <>
              <Alert>
                <AlertCircle className="h-4 w-4" />
                <AlertDescription>
                  Please verify the following items before continuing:
                </AlertDescription>
              </Alert>

              <div className="space-y-3">
                {checkpoint.requiredValidation.map((item, index) => (
                  <div
                    key={index}
                    className="flex items-start space-x-3 p-3 rounded border"
                  >
                    <Checkbox
                      id={`validation-${index}`}
                      checked={validationState[item] || false}
                      onCheckedChange={(checked) =>
                        handleValidationChange(item, checked as boolean)
                      }
                    />
                    <label
                      htmlFor={`validation-${index}`}
                      className="text-sm font-medium leading-none peer-disabled:cursor-not-allowed peer-disabled:opacity-70 cursor-pointer"
                    >
                      {item}
                    </label>
                  </div>
                ))}
              </div>

              {allValidated && (
                <Alert className="bg-green-50 border-green-200">
                  <CheckCircle2 className="h-4 w-4 text-green-600" />
                  <AlertDescription className="text-green-800">
                    All validation items checked! Ready to continue.
                  </AlertDescription>
                </Alert>
              )}
            </>
          ) : (
            <Alert>
              <AlertDescription>
                No specific validation required. Click Continue when ready.
              </AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter className="flex gap-2">
          {onSkip && (
            <Button variant="outline" onClick={handleSkip}>
              Skip Checkpoint
            </Button>
          )}
          <Button
            onClick={handleComplete}
            disabled={
              checkpoint.requiredValidation.length > 0 && !allValidated
            }
          >
            {allValidated ? 'Continue' : 'Validate & Continue'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
