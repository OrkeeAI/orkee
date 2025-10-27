// ABOUTME: OpenSpec changes list view displaying all change proposals with status and validation
// ABOUTME: Integrates with change validation and archiving functionality
import { useState, useMemo } from 'react';
import { FileEdit, CheckCircle2, XCircle, Clock, Archive, AlertTriangle } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { useChanges, useValidateChange, useArchiveChange } from '@/hooks/useChanges';
import type { ChangeStatus } from '@/services/changes';

interface ChangesListProps {
  projectId: string;
  onSelectChange?: (changeId: string) => void;
  statusFilter?: ChangeStatus;
}

export function ChangesList({ projectId, onSelectChange, statusFilter: propStatusFilter }: ChangesListProps) {
  const [internalStatusFilter, setInternalStatusFilter] = useState<ChangeStatus | undefined>(undefined);
  const [selectedChangeId, setSelectedChangeId] = useState<string | null>(null);
  const [changeToArchive, setChangeToArchive] = useState<string | null>(null);

  // Use prop status filter if provided, otherwise use internal state
  // Using useMemo prevents race conditions when propStatusFilter changes rapidly
  const activeStatusFilter = useMemo(
    () => propStatusFilter ?? internalStatusFilter,
    [propStatusFilter, internalStatusFilter]
  );

  const { data: changes, isLoading, error } = useChanges(projectId, activeStatusFilter);
  const validateMutation = useValidateChange(projectId);
  const archiveMutation = useArchiveChange(projectId);

  const handleValidate = (changeId: string) => {
    validateMutation.mutate({ changeId, strict: true });
  };

  const handleArchive = (changeId: string) => {
    setChangeToArchive(changeId);
  };

  const confirmArchive = () => {
    if (changeToArchive) {
      archiveMutation.mutate({ changeId: changeToArchive, applySpecs: true });
      setChangeToArchive(null);
    }
  };

  const cancelArchive = () => {
    setChangeToArchive(null);
  };

  const handleSelectChange = (changeId: string) => {
    setSelectedChangeId(changeId);
    if (onSelectChange) {
      onSelectChange(changeId);
    }
  };

  const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString('en-US', {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  };

  const getStatusBadge = (status: ChangeStatus) => {
    const variants: Record<ChangeStatus, { variant: 'default' | 'secondary' | 'outline' | 'destructive'; icon: React.ReactNode; label: string }> = {
      draft: { variant: 'outline', icon: <Clock className="h-3 w-3 mr-1" />, label: 'Draft' },
      review: { variant: 'secondary', icon: <FileEdit className="h-3 w-3 mr-1" />, label: 'In Review' },
      approved: { variant: 'default', icon: <CheckCircle2 className="h-3 w-3 mr-1" />, label: 'Approved' },
      implementing: { variant: 'secondary', icon: <Clock className="h-3 w-3 mr-1" />, label: 'Implementing' },
      completed: { variant: 'default', icon: <CheckCircle2 className="h-3 w-3 mr-1" />, label: 'Completed' },
      archived: { variant: 'outline', icon: <Archive className="h-3 w-3 mr-1" />, label: 'Archived' },
    };

    const { variant, icon, label } = variants[status] || variants.draft;
    return (
      <Badge variant={variant} className="flex items-center gap-1">
        {icon}
        {label}
      </Badge>
    );
  };

  const getValidationBadge = (validationStatus?: 'pending' | 'valid' | 'invalid') => {
    if (!validationStatus || validationStatus === 'pending') {
      return <Badge variant="outline">Not Validated</Badge>;
    }
    if (validationStatus === 'valid') {
      return (
        <Badge variant="default" className="flex items-center gap-1">
          <CheckCircle2 className="h-3 w-3" />
          Valid
        </Badge>
      );
    }
    return (
      <Badge variant="destructive" className="flex items-center gap-1">
        <XCircle className="h-3 w-3" />
        Invalid
      </Badge>
    );
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading changes...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertTriangle className="h-4 w-4" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>
          Failed to load changes: {error instanceof Error ? error.message : 'Unknown error'}
        </AlertDescription>
      </Alert>
    );
  }

  if (!changes || changes.length === 0) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>No Changes</CardTitle>
          <CardDescription>
            No OpenSpec change proposals found for this project.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            Analyze a PRD to create change proposals.
          </p>
        </CardContent>
      </Card>
    );
  }

  return (
    <div className="space-y-4">
      <div className="flex items-center justify-between">
        <h2 className="text-2xl font-bold">OpenSpec Changes</h2>
        <div className="flex items-center gap-2">
          <select
            className="border rounded px-3 py-1 text-sm"
            value={activeStatusFilter || ''}
            onChange={(e) => setInternalStatusFilter(e.target.value as ChangeStatus || undefined)}
            disabled={!!propStatusFilter}
          >
            <option value="">All Statuses</option>
            <option value="draft">Draft</option>
            <option value="review">Review</option>
            <option value="approved">Approved</option>
            <option value="implementing">Implementing</option>
            <option value="completed">Completed</option>
            <option value="archived">Archived</option>
          </select>
        </div>
      </div>

      <div className="space-y-3">
        {changes.map((change) => (
          <Card
            key={change.id}
            className={`cursor-pointer transition-colors ${
              selectedChangeId === change.id ? 'border-primary' : ''
            }`}
            onClick={() => handleSelectChange(change.id)}
          >
            <CardHeader>
              <div className="flex items-start justify-between">
                <div className="flex-1">
                  <CardTitle className="text-lg flex items-center gap-2">
                    {change.verbPrefix && change.changeNumber && (
                      <span className="font-mono text-sm">
                        {change.verbPrefix}-{change.changeNumber}
                      </span>
                    )}
                    <span className="text-muted-foreground text-sm">
                      Change #{change.id.slice(0, 8)}
                    </span>
                  </CardTitle>
                  <CardDescription className="mt-1">
                    Created {formatDate(change.createdAt)} by {change.createdBy}
                  </CardDescription>
                </div>
                <div className="flex flex-col gap-2">
                  {getStatusBadge(change.status)}
                  {getValidationBadge(change.validationStatus)}
                </div>
              </div>
            </CardHeader>
            <CardContent>
              <div className="flex items-center justify-between">
                <div className="text-sm text-muted-foreground">
                  {change.deltaCount} delta{change.deltaCount !== 1 ? 's' : ''}
                  {change.prdId && (
                    <span className="ml-2">
                      • PRD: {change.prdId.slice(0, 8)}
                    </span>
                  )}
                </div>
                <div className="flex items-center gap-2" onClick={(e) => e.stopPropagation()}>
                  {change.status !== 'archived' && (
                    <>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleValidate(change.id)}
                        disabled={validateMutation.isPending}
                      >
                        {validateMutation.isPending ? 'Validating...' : 'Validate'}
                      </Button>
                      {change.validationStatus === 'valid' && (
                        <Button
                          size="sm"
                          variant="default"
                          onClick={() => handleArchive(change.id)}
                          disabled={archiveMutation.isPending}
                        >
                          {archiveMutation.isPending ? 'Archiving...' : 'Archive'}
                        </Button>
                      )}
                    </>
                  )}
                </div>
              </div>
            </CardContent>
          </Card>
        ))}
      </div>

      <Dialog open={changeToArchive !== null} onOpenChange={(open) => !open && cancelArchive()}>
        <DialogContent>
          <DialogHeader>
            <DialogTitle>Archive Change</DialogTitle>
            <DialogDescription>
              Archive this change and apply deltas to specifications? This action cannot be undone.
            </DialogDescription>
          </DialogHeader>
          <DialogFooter>
            <Button variant="outline" onClick={cancelArchive}>
              Cancel
            </Button>
            <Button variant="default" onClick={confirmArchive}>
              Archive
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}
