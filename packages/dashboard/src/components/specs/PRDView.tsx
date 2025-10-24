// ABOUTME: PRD management view displaying list of PRDs with metadata and content
// ABOUTME: Integrates with PRDUploadDialog for creating/analyzing PRDs
import { useState } from 'react';
import { FileText, Upload, Sparkles, Trash2, Calendar, User } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize from 'rehype-sanitize';
import { usePRDs, useDeletePRD, useTriggerPRDAnalysis } from '@/hooks/usePRDs';
import { PRDUploadDialog } from '@/components/PRDUploadDialog';
import { ModelSelectionDialog } from '@/components/ModelSelectionDialog';
import type { PRD } from '@/services/prds';

interface PRDViewProps {
  projectId: string;
}

export function PRDView({ projectId }: PRDViewProps) {
  const [selectedPRD, setSelectedPRD] = useState<PRD | null>(null);
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [showModelSelection, setShowModelSelection] = useState(false);
  const [prdToAnalyze, setPrdToAnalyze] = useState<string | null>(null);

  const { data: prds, isLoading, error } = usePRDs(projectId);
  const deletePRDMutation = useDeletePRD(projectId);
  const analyzePRDMutation = useTriggerPRDAnalysis(projectId);

  const handleDelete = (prdId: string) => {
    if (confirm('Are you sure you want to delete this PRD? This action cannot be undone.')) {
      deletePRDMutation.mutate(prdId);
      if (selectedPRD?.id === prdId) {
        setSelectedPRD(null);
      }
    }
  };

  const handleAnalyzeClick = (prdId: string) => {
    setPrdToAnalyze(prdId);
    setShowModelSelection(true);
  };

  const handleAnalyze = (provider: string, model: string) => {
    if (prdToAnalyze) {
      console.log(`Analyzing PRD ${prdToAnalyze} with ${provider}/${model}`);
      analyzePRDMutation.mutate({ prdId: prdToAnalyze, provider, model });
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

  const getStatusBadge = (status: string) => {
    switch (status) {
      case 'approved':
        return <Badge variant="default">Approved</Badge>;
      case 'superseded':
        return <Badge variant="secondary">Superseded</Badge>;
      default:
        return <Badge variant="outline">Draft</Badge>;
    }
  };

  const getSourceBadge = (source: string) => {
    switch (source) {
      case 'generated':
        return <Badge variant="secondary">AI Generated</Badge>;
      case 'synced':
        return <Badge variant="secondary">Synced</Badge>;
      default:
        return <Badge variant="outline">Manual</Badge>;
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-center">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-primary border-t-transparent mx-auto mb-4" />
          <p className="text-muted-foreground">Loading PRDs...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <Alert variant="destructive">
        <AlertDescription>
          {error instanceof Error ? error.message : 'Failed to load PRDs'}
        </AlertDescription>
      </Alert>
    );
  }

  if (!prds || prds.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-64 space-y-4">
        <FileText className="h-16 w-16 text-muted-foreground" />
        <div className="text-center space-y-2">
          <h3 className="text-lg font-semibold">No PRDs Yet</h3>
          <p className="text-sm text-muted-foreground max-w-md">
            Upload a Product Requirements Document to get started with spec-driven development.
          </p>
        </div>
        <Button onClick={() => setShowUploadDialog(true)}>
          <Upload className="mr-2 h-4 w-4" />
          Upload PRD
        </Button>

        <PRDUploadDialog
          projectId={projectId}
          open={showUploadDialog}
          onOpenChange={setShowUploadDialog}
          onComplete={(prdId) => {
            // Select the newly created PRD
            const newPRD = prds?.find(p => p.id === prdId);
            if (newPRD) {
              setSelectedPRD(newPRD);
            }
          }}
        />
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header with Upload Button */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Product Requirements Documents</h3>
          <p className="text-sm text-muted-foreground">
            {prds.length} {prds.length === 1 ? 'PRD' : 'PRDs'} in this project
          </p>
        </div>
        <Button onClick={() => setShowUploadDialog(true)} size="sm">
          <Upload className="mr-2 h-4 w-4" />
          Upload PRD
        </Button>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
        {/* PRD List */}
        <div className="space-y-2">
          {prds.map((prd) => (
            <Card
              key={prd.id}
              className={`cursor-pointer transition-colors ${
                selectedPRD?.id === prd.id ? 'border-primary' : ''
              }`}
              onClick={() => setSelectedPRD(prd)}
            >
              <CardHeader className="pb-3">
                <div className="flex items-start justify-between">
                  <CardTitle className="text-sm font-medium line-clamp-1">{prd.title}</CardTitle>
                  {getStatusBadge(prd.status)}
                </div>
                <CardDescription className="text-xs">
                  <div className="flex items-center gap-2 mt-1">
                    {getSourceBadge(prd.source)}
                    <span>v{prd.version}</span>
                  </div>
                </CardDescription>
              </CardHeader>
              <CardContent className="pb-3">
                <div className="space-y-1 text-xs text-muted-foreground">
                  <div className="flex items-center gap-1">
                    <Calendar className="h-3 w-3" />
                    <span>{formatDate(prd.createdAt)}</span>
                  </div>
                  {prd.createdBy && (
                    <div className="flex items-center gap-1">
                      <User className="h-3 w-3" />
                      <span>{prd.createdBy}</span>
                    </div>
                  )}
                </div>
              </CardContent>
            </Card>
          ))}
        </div>

        {/* PRD Content Viewer */}
        <div className="lg:col-span-2">
          {selectedPRD ? (
            <Card>
              <CardHeader>
                <div className="flex items-start justify-between">
                  <div className="space-y-1">
                    <CardTitle>{selectedPRD.title}</CardTitle>
                    <CardDescription>
                      Version {selectedPRD.version} • {formatDate(selectedPRD.updatedAt)}
                    </CardDescription>
                  </div>
                  <div className="flex gap-2">
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleAnalyzeClick(selectedPRD.id)}
                      disabled={analyzePRDMutation.isPending}
                    >
                      {analyzePRDMutation.isPending ? (
                        <>
                          <div className="mr-2 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                          Analyzing...
                        </>
                      ) : (
                        <>
                          <Sparkles className="mr-2 h-4 w-4" />
                          Analyze
                        </>
                      )}
                    </Button>
                    <Button
                      variant="outline"
                      size="sm"
                      onClick={() => handleDelete(selectedPRD.id)}
                      disabled={deletePRDMutation.isPending}
                    >
                      <Trash2 className="mr-2 h-4 w-4" />
                      Delete
                    </Button>
                  </div>
                </div>
              </CardHeader>
              <Separator />
              <CardContent className="pt-6">
                <div className="prose prose-sm dark:prose-invert max-w-none">
                  <ReactMarkdown
                    remarkPlugins={[remarkGfm]}
                    rehypePlugins={[rehypeHighlight, rehypeSanitize]}
                  >
                    {selectedPRD.contentMarkdown}
                  </ReactMarkdown>
                </div>
              </CardContent>
            </Card>
          ) : (
            <Card className="h-full flex items-center justify-center">
              <CardContent>
                <div className="text-center text-muted-foreground">
                  <FileText className="h-12 w-12 mx-auto mb-2 opacity-50" />
                  <p>Select a PRD to view its content</p>
                </div>
              </CardContent>
            </Card>
          )}
        </div>
      </div>

      <PRDUploadDialog
        projectId={projectId}
        open={showUploadDialog}
        onOpenChange={setShowUploadDialog}
        onComplete={(prdId) => {
          // Select the newly created PRD after save
          const newPRD = prds?.find(p => p.id === prdId);
          if (newPRD) {
            setSelectedPRD(newPRD);
          }
        }}
      />

      <ModelSelectionDialog
        open={showModelSelection}
        onOpenChange={setShowModelSelection}
        onConfirm={handleAnalyze}
      />
    </div>
  );
}
