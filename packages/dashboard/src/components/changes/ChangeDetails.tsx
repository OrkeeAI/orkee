// ABOUTME: OpenSpec change details view displaying proposal, tasks, design, and deltas
// ABOUTME: Shows validation errors and provides actions for validation and archiving
import { useState } from 'react';
import { FileEdit, CheckCircle2, XCircle, AlertTriangle, Package, Play, RotateCcw } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSanitize from 'rehype-sanitize';
import { useChange, useValidateChange, useArchiveChange, useUpdateChangeStatus } from '@/hooks/useChanges';
import type { DeltaType, ChangeStatus } from '@/services/changes';
import { ApprovalDialog } from './ApprovalDialog';
import { ApprovalHistory } from './ApprovalHistory';

interface ChangeDetailsProps {
  projectId: string;
  changeId: string;
}

export function ChangeDetails({ projectId, changeId }: ChangeDetailsProps) {
  const { data: change, isLoading, error } = useChange(projectId, changeId);
  const validateMutation = useValidateChange(projectId);
  const archiveMutation = useArchiveChange(projectId);
  const updateStatusMutation = useUpdateChangeStatus(projectId);
  const [showApprovalDialog, setShowApprovalDialog] = useState(false);

  const handleValidate = () => {
    validateMutation.mutate({ changeId, strict: true });
  };

  const handleArchive = () => {
    if (confirm('Archive this change and apply deltas to specifications? This action cannot be undone.')) {
      archiveMutation.mutate({ changeId, applySpecs: true });
    }
  };

  const handleStatusTransition = (status: ChangeStatus, confirmMessage?: string) => {
    if (confirmMessage) {
      if (!confirm(confirmMessage)) return;
    }
    updateStatusMutation.mutate({ changeId, status });
  };

  const handleApprove = (notes?: string) => {
    updateStatusMutation.mutate(
      {
        changeId,
        status: 'approved',
        metadata: {
          approvedBy: change?.createdBy,
          notes,
        },
      },
      {
        onSuccess: () => {
          setShowApprovalDialog(false);
        },
      }
    );
  };

  const getStatusBadgeVariant = (status: ChangeStatus): 'default' | 'secondary' | 'outline' | 'destructive' => {
    const variants: Record<ChangeStatus, 'default' | 'secondary' | 'outline' | 'destructive'> = {
      draft: 'outline',
      review: 'secondary',
      approved: 'default',
      implementing: 'secondary',
      completed: 'default',
      archived: 'outline',
    };
    return variants[status] || 'outline';
  };

  const getStatusLabel = (status: ChangeStatus): string => {
    const labels: Record<ChangeStatus, string> = {
      draft: 'Draft',
      review: 'In Review',
      approved: 'Approved',
      implementing: 'Implementing',
      completed: 'Completed',
      archived: 'Archived',
    };
    return labels[status] || status;
  };

  const getDeltaTypeBadge = (deltaType: DeltaType) => {
    const variants: Record<DeltaType, { variant: 'default' | 'secondary' | 'outline' | 'destructive'; label: string }> = {
      added: { variant: 'default', label: 'ADDED' },
      modified: { variant: 'secondary', label: 'MODIFIED' },
      removed: { variant: 'destructive', label: 'REMOVED' },
      renamed: { variant: 'outline', label: 'RENAMED' },
    };

    const { variant, label } = variants[deltaType] || variants.added;
    return <Badge variant={variant}>{label}</Badge>;
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading change details...</p>
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
          Failed to load change: {error instanceof Error ? error.message : 'Unknown error'}
        </AlertDescription>
      </Alert>
    );
  }

  if (!change) {
    return (
      <Alert>
        <AlertTriangle className="h-4 w-4" />
        <AlertTitle>Not Found</AlertTitle>
        <AlertDescription>Change not found</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div>
              <CardTitle className="text-2xl flex items-center gap-2">
                <FileEdit className="h-6 w-6" />
                {change.verbPrefix && change.changeNumber && (
                  <span className="font-mono">
                    {change.verbPrefix}-{change.changeNumber}
                  </span>
                )}
              </CardTitle>
              <CardDescription className="mt-2">
                Created {new Date(change.createdAt).toLocaleDateString()} by {change.createdBy}
                {change.archivedAt && (
                  <span className="ml-2">
                    • Archived {new Date(change.archivedAt).toLocaleDateString()}
                  </span>
                )}
              </CardDescription>
            </div>
            <div className="flex flex-col gap-2">
              <Badge variant={getStatusBadgeVariant(change.status)}>
                {getStatusLabel(change.status)}
              </Badge>
              {change.validationStatus === 'valid' && (
                <Badge variant="default" className="flex items-center gap-1">
                  <CheckCircle2 className="h-3 w-3" />
                  Valid
                </Badge>
              )}
              {change.validationStatus === 'invalid' && (
                <Badge variant="destructive" className="flex items-center gap-1">
                  <XCircle className="h-3 w-3" />
                  Invalid
                </Badge>
              )}
              {change.approvedBy && change.approvedAt && (
                <Badge variant="outline" className="text-xs">
                  Approved by {change.approvedBy}
                </Badge>
              )}
            </div>
          </div>
        </CardHeader>

        {change.validationErrors && change.validationErrors.length > 0 && (
          <CardContent>
            <Alert variant="destructive">
              <AlertTriangle className="h-4 w-4" />
              <AlertTitle>Validation Errors</AlertTitle>
              <AlertDescription>
                <ul className="list-disc list-inside mt-2">
                  {change.validationErrors.map((error, i) => (
                    <li key={i}>
                      {error.line && <span className="font-mono">Line {error.line}: </span>}
                      {error.message}
                    </li>
                  ))}
                </ul>
              </AlertDescription>
            </Alert>
          </CardContent>
        )}

        <CardContent>
          <div className="flex items-center gap-2 flex-wrap">
            {/* Draft Status Actions */}
            {change.status === 'draft' && (
              <>
                <Button
                  variant="outline"
                  onClick={handleValidate}
                  disabled={validateMutation.isPending}
                >
                  {validateMutation.isPending ? 'Validating...' : 'Validate'}
                </Button>
                {change.validationStatus === 'valid' && (
                  <Button
                    variant="default"
                    onClick={() =>
                      handleStatusTransition('review', 'Submit this change for review?')
                    }
                    disabled={updateStatusMutation.isPending}
                  >
                    Submit for Review
                  </Button>
                )}
              </>
            )}

            {/* Review Status Actions */}
            {change.status === 'review' && (
              <>
                <Button
                  variant="outline"
                  onClick={handleValidate}
                  disabled={validateMutation.isPending}
                >
                  {validateMutation.isPending ? 'Validating...' : 'Validate'}
                </Button>
                {change.validationStatus === 'valid' && (
                  <Button
                    variant="default"
                    onClick={() => setShowApprovalDialog(true)}
                    disabled={updateStatusMutation.isPending}
                  >
                    Approve Change
                  </Button>
                )}
                <Button
                  variant="outline"
                  onClick={() =>
                    handleStatusTransition('draft', 'Request changes and return to draft?')
                  }
                  disabled={updateStatusMutation.isPending}
                >
                  <RotateCcw className="h-4 w-4 mr-2" />
                  Request Changes
                </Button>
              </>
            )}

            {/* Approved Status Actions */}
            {change.status === 'approved' && (
              <>
                <Button
                  variant="default"
                  onClick={() =>
                    handleStatusTransition('implementing', 'Begin implementing this change?')
                  }
                  disabled={updateStatusMutation.isPending}
                >
                  <Play className="h-4 w-4 mr-2" />
                  Start Implementation
                </Button>
                <Button
                  variant="outline"
                  onClick={() =>
                    handleStatusTransition('review', 'Revoke approval and return to review?')
                  }
                  disabled={updateStatusMutation.isPending}
                >
                  Revoke Approval
                </Button>
              </>
            )}

            {/* Implementing Status Actions */}
            {change.status === 'implementing' && (
              <>
                <Button
                  variant="default"
                  onClick={() =>
                    handleStatusTransition('completed', 'Mark implementation as complete?')
                  }
                  disabled={updateStatusMutation.isPending}
                >
                  <CheckCircle2 className="h-4 w-4 mr-2" />
                  Mark Complete
                </Button>
              </>
            )}

            {/* Completed Status Actions */}
            {change.status === 'completed' && (
              <>
                <Button
                  variant="default"
                  onClick={handleArchive}
                  disabled={archiveMutation.isPending}
                >
                  {archiveMutation.isPending ? 'Archiving...' : 'Archive & Apply'}
                </Button>
              </>
            )}

            {/* Archived Status - No Actions */}
            {change.status === 'archived' && (
              <p className="text-sm text-muted-foreground">
                This change has been archived and applied to specifications.
              </p>
            )}
          </div>
        </CardContent>
      </Card>

      <Tabs defaultValue="proposal" className="w-full">
        <TabsList className="grid w-full grid-cols-5">
          <TabsTrigger value="proposal">Proposal</TabsTrigger>
          <TabsTrigger value="tasks">Tasks</TabsTrigger>
          <TabsTrigger value="design">Design</TabsTrigger>
          <TabsTrigger value="deltas">
            Deltas ({change.deltas?.length || 0})
          </TabsTrigger>
          <TabsTrigger value="history">History</TabsTrigger>
        </TabsList>

        <TabsContent value="proposal">
          <Card>
            <CardHeader>
              <CardTitle>Change Proposal</CardTitle>
              <CardDescription>Why this change is needed and what it impacts</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="prose prose-sm max-w-none dark:prose-invert">
                <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                  {change.proposalMarkdown}
                </ReactMarkdown>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="tasks">
          <Card>
            <CardHeader>
              <CardTitle>Implementation Tasks</CardTitle>
              <CardDescription>Steps required to implement this change</CardDescription>
            </CardHeader>
            <CardContent>
              <div className="prose prose-sm max-w-none dark:prose-invert">
                <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                  {change.tasksMarkdown}
                </ReactMarkdown>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="design">
          <Card>
            <CardHeader>
              <CardTitle>Design Details</CardTitle>
              <CardDescription>Technical design and architecture decisions</CardDescription>
            </CardHeader>
            <CardContent>
              {change.designMarkdown ? (
                <div className="prose prose-sm max-w-none dark:prose-invert">
                  <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                    {change.designMarkdown}
                  </ReactMarkdown>
                </div>
              ) : (
                <p className="text-sm text-muted-foreground">No design document provided</p>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="deltas">
          <div className="space-y-4">
            {change.deltas && change.deltas.length > 0 ? (
              change.deltas.map((delta) => (
                <Card key={delta.id}>
                  <CardHeader>
                    <div className="flex items-center justify-between">
                      <CardTitle className="text-lg flex items-center gap-2">
                        <Package className="h-5 w-5" />
                        {delta.capabilityName}
                      </CardTitle>
                      {getDeltaTypeBadge(delta.deltaType)}
                    </div>
                    <CardDescription>
                      {delta.capabilityId && (
                        <span className="text-xs font-mono">ID: {delta.capabilityId}</span>
                      )}
                    </CardDescription>
                  </CardHeader>
                  <CardContent>
                    <div className="prose prose-sm max-w-none dark:prose-invert">
                      <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                        {delta.deltaMarkdown}
                      </ReactMarkdown>
                    </div>
                  </CardContent>
                  <Separator />
                </Card>
              ))
            ) : (
              <Card>
                <CardContent className="pt-6">
                  <p className="text-sm text-muted-foreground text-center">
                    No deltas found for this change
                  </p>
                </CardContent>
              </Card>
            )}
          </div>
        </TabsContent>

        <TabsContent value="history">
          <ApprovalHistory change={change} />
        </TabsContent>
      </Tabs>

      {/* Approval Dialog */}
      {change && (
        <ApprovalDialog
          open={showApprovalDialog}
          onOpenChange={setShowApprovalDialog}
          change={change}
          onApprove={handleApprove}
          isApproving={updateStatusMutation.isPending}
        />
      )}
    </div>
  );
}
