// ABOUTME: Approval dialog for OpenSpec changes with validation checks and delta summary
// ABOUTME: Captures approval metadata including approver and optional notes
import { useState } from 'react';
import { CheckCircle2, XCircle, AlertTriangle, FileEdit } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Textarea } from '@/components/ui/textarea';
import { Label } from '@/components/ui/label';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import type { ChangeWithDeltas, DeltaType } from '@/services/changes';

interface ApprovalDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  change: ChangeWithDeltas;
  onApprove: (notes?: string) => void;
  isApproving?: boolean;
}

export function ApprovalDialog({
  open,
  onOpenChange,
  change,
  onApprove,
  isApproving = false,
}: ApprovalDialogProps) {
  const [notes, setNotes] = useState('');

  const handleApprove = () => {
    onApprove(notes || undefined);
    setNotes('');
  };

  const handleCancel = () => {
    setNotes('');
    onOpenChange(false);
  };

  const canApprove = change.validationStatus === 'valid';

  const getDeltaTypeBadge = (deltaType: DeltaType) => {
    const variants: Record<DeltaType, { variant: 'default' | 'secondary' | 'outline' | 'destructive'; label: string }> = {
      added: { variant: 'default', label: 'ADDED' },
      modified: { variant: 'secondary', label: 'MODIFIED' },
      removed: { variant: 'destructive', label: 'REMOVED' },
      renamed: { variant: 'outline', label: 'RENAMED' },
    };

    const { variant, label } = variants[deltaType] || variants.added;
    return <Badge variant={variant} className="text-xs">{label}</Badge>;
  };

  const deltaTypeCounts = change.deltas?.reduce((acc, delta) => {
    acc[delta.deltaType] = (acc[delta.deltaType] || 0) + 1;
    return acc;
  }, {} as Record<DeltaType, number>) || {};

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[80vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileEdit className="h-5 w-5" />
            Approve Change
            {change.verbPrefix && change.changeNumber && (
              <span className="font-mono text-sm text-muted-foreground">
                {change.verbPrefix}-{change.changeNumber}
              </span>
            )}
          </DialogTitle>
          <DialogDescription>
            Review and approve this change proposal for implementation
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Validation Status */}
          <div>
            <Label className="text-sm font-medium">Validation Status</Label>
            <div className="mt-2">
              {change.validationStatus === 'valid' && (
                <Alert>
                  <CheckCircle2 className="h-4 w-4 text-green-600" />
                  <AlertTitle className="text-green-600">Valid</AlertTitle>
                  <AlertDescription>
                    This change passes all OpenSpec validation checks
                  </AlertDescription>
                </Alert>
              )}
              {change.validationStatus === 'invalid' && (
                <Alert variant="destructive">
                  <XCircle className="h-4 w-4" />
                  <AlertTitle>Invalid</AlertTitle>
                  <AlertDescription>
                    This change has validation errors. Please fix them before approving.
                    <ul className="list-disc list-inside mt-2">
                      {change.validationErrors?.map((error, i) => (
                        <li key={i}>
                          {error.line && <span className="font-mono">Line {error.line}: </span>}
                          {error.message}
                        </li>
                      ))}
                    </ul>
                  </AlertDescription>
                </Alert>
              )}
              {change.validationStatus === 'pending' && (
                <Alert>
                  <AlertTriangle className="h-4 w-4" />
                  <AlertTitle>Pending Validation</AlertTitle>
                  <AlertDescription>
                    This change has not been validated yet. Run validation before approving.
                  </AlertDescription>
                </Alert>
              )}
            </div>
          </div>

          {/* Delta Summary */}
          <div>
            <Label className="text-sm font-medium">Change Summary</Label>
            <div className="mt-2 p-4 border rounded-lg space-y-3">
              <div className="flex items-center gap-2 flex-wrap">
                <span className="text-sm text-muted-foreground">Total Deltas:</span>
                <Badge variant="outline">{change.deltas?.length || 0}</Badge>
                {Object.entries(deltaTypeCounts).map(([type, count]) => (
                  <div key={type} className="flex items-center gap-1">
                    {getDeltaTypeBadge(type as DeltaType)}
                    <span className="text-sm text-muted-foreground">×{count}</span>
                  </div>
                ))}
              </div>

              {change.deltas && change.deltas.length > 0 && (
                <div className="space-y-2">
                  <p className="text-sm font-medium">Affected Capabilities:</p>
                  <ul className="list-disc list-inside text-sm space-y-1">
                    {change.deltas.map((delta) => (
                      <li key={delta.id} className="flex items-center gap-2">
                        {getDeltaTypeBadge(delta.deltaType)}
                        <span className="font-mono text-xs">{delta.capabilityName}</span>
                      </li>
                    ))}
                  </ul>
                </div>
              )}
            </div>
          </div>

          {/* Approval Notes */}
          <div>
            <Label htmlFor="notes" className="text-sm font-medium">
              Approval Notes <span className="text-muted-foreground">(optional)</span>
            </Label>
            <Textarea
              id="notes"
              placeholder="Add any notes about this approval..."
              value={notes}
              onChange={(e) => setNotes(e.target.value)}
              rows={3}
              className="mt-2"
            />
          </div>

          {!canApprove && (
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>Cannot Approve</AlertTitle>
              <AlertDescription>
                This change must pass validation before it can be approved.
              </AlertDescription>
            </Alert>
          )}
        </div>

        <DialogFooter>
          <Button
            variant="outline"
            onClick={handleCancel}
            disabled={isApproving}
          >
            Cancel
          </Button>
          <Button
            onClick={handleApprove}
            disabled={!canApprove || isApproving}
          >
            {isApproving ? 'Approving...' : 'Approve Change'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
