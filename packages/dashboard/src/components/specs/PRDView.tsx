// ABOUTME: PRD management view displaying list of PRDs with metadata and content
// ABOUTME: Integrates with PRDUploadDialog for creating/analyzing PRDs
import { useState } from 'react';
import { FileText, Upload, Sparkles, Trash2, Calendar, User, Layers, ExternalLink, AlertTriangle, Lightbulb } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize from 'rehype-sanitize';
import { usePRDs, useDeletePRD, useTriggerPRDAnalysis } from '@/hooks/usePRDs';
import { useSpecs } from '@/hooks/useSpecs';
import { PRDUploadDialog } from '@/components/PRDUploadDialog';
import { ModelSelectionDialog } from '@/components/ModelSelectionDialog';
import { CreatePRDFlow } from '@/components/ideate/CreatePRDFlow';
import { SessionsList } from '@/components/ideate/SessionsList';
import { QuickModeFlow } from '@/components/ideate/QuickMode';
import type { PRD, PRDAnalysisResult } from '@/services/prds';
import type { IdeateSession } from '@/services/ideate';

interface PRDViewProps {
  projectId: string;
  onViewSpecs?: (prdId: string) => void;
}

export function PRDView({ projectId, onViewSpecs }: PRDViewProps) {
  const [selectedPRD, setSelectedPRD] = useState<PRD | null>(null);
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [showIdeateFlow, setShowIdeateFlow] = useState(false);
  const [showQuickModeFlow, setShowQuickModeFlow] = useState(false);
  const [activeSessionId, setActiveSessionId] = useState<string | null>(null);
  const [showModelSelection, setShowModelSelection] = useState(false);
  const [prdToAnalyze, setPrdToAnalyze] = useState<string | null>(null);
  const [analysisResult, setAnalysisResult] = useState<PRDAnalysisResult | null>(null);

  const { data: prds, isLoading, error } = usePRDs(projectId);
  const { data: allSpecs } = useSpecs(projectId);
  const deletePRDMutation = useDeletePRD(projectId);
  const analyzePRDMutation = useTriggerPRDAnalysis(projectId);

  // Count specs for each PRD
  const getSpecCountForPRD = (prdId: string) => {
    if (!allSpecs) return 0;
    return allSpecs.filter(spec => spec.prdId === prdId).length;
  };

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
      analyzePRDMutation.mutate(
        { prdId: prdToAnalyze, provider, model },
        {
          onSuccess: (result) => {
            setAnalysisResult(result);
          },
        }
      );
    }
  };

  const handleResumeSession = (session: IdeateSession) => {
    setActiveSessionId(session.id);
    if (session.mode === 'quick') {
      setShowQuickModeFlow(true);
    }
    // TODO: Handle guided and comprehensive modes when implemented
  };

  const handleSessionCreated = (sessionId: string) => {
    setActiveSessionId(sessionId);
    setShowQuickModeFlow(true);
  };

  const handleQuickModeComplete = (prdId: string) => {
    // Refresh PRD list and select the newly created PRD
    const newPRD = prds?.find(p => p.id === prdId);
    if (newPRD) {
      setSelectedPRD(newPRD);
    }
    setShowQuickModeFlow(false);
    setActiveSessionId(null);
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

  return (
    <div className="space-y-4">
      {/* Header with Upload Button */}
      <div className="flex items-center justify-between">
        <div>
          <h3 className="text-lg font-semibold">Product Requirements Documents</h3>
          <p className="text-sm text-muted-foreground">
            {prds?.length || 0} {prds?.length === 1 ? 'PRD' : 'PRDs'} in this project
          </p>
        </div>
        <div className="flex gap-2">
          <Button onClick={() => setShowIdeateFlow(true)} size="sm">
            <Lightbulb className="mr-2 h-4 w-4" />
            Create PRD
          </Button>
          <Button variant="outline" onClick={() => setShowUploadDialog(true)} size="sm">
            <Upload className="mr-2 h-4 w-4" />
            Upload PRD
          </Button>
        </div>
      </div>

      {/* Tabs for Sessions and PRDs */}
      <Tabs defaultValue="prds" className="space-y-4">
        <TabsList>
          <TabsTrigger value="sessions">Ideate Sessions</TabsTrigger>
          <TabsTrigger value="prds">PRDs</TabsTrigger>
        </TabsList>

        <TabsContent value="sessions" className="space-y-4">
          <SessionsList projectId={projectId} onResumeSession={handleResumeSession} />
        </TabsContent>

        <TabsContent value="prds" className="space-y-4">
          {(!prds || prds.length === 0) ? (
            <div className="flex flex-col items-center justify-center h-64 space-y-4">
              <FileText className="h-16 w-16 text-muted-foreground" />
              <div className="text-center space-y-2">
                <h3 className="text-lg font-semibold">No PRDs Yet</h3>
                <p className="text-sm text-muted-foreground max-w-md">
                  Create a new PRD through ideating or upload an existing document.
                </p>
              </div>
              <div className="flex gap-2">
                <Button onClick={() => setShowIdeateFlow(true)}>
                  <Lightbulb className="mr-2 h-4 w-4" />
                  Create PRD
                </Button>
                <Button variant="outline" onClick={() => setShowUploadDialog(true)}>
                  <Upload className="mr-2 h-4 w-4" />
                  Upload PRD
                </Button>
              </div>
            </div>
          ) : (

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
                  <div className="flex items-center gap-1 pt-1">
                    <Layers className="h-3 w-3" />
                    <span>{getSpecCountForPRD(prd.id)} {getSpecCountForPRD(prd.id) === 1 ? 'spec' : 'specs'}</span>
                  </div>
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
                    {getSpecCountForPRD(selectedPRD.id) > 0 && onViewSpecs && (
                      <Button
                        variant="default"
                        size="sm"
                        onClick={() => onViewSpecs(selectedPRD.id)}
                      >
                        <Layers className="mr-2 h-4 w-4" />
                        View Specs ({getSpecCountForPRD(selectedPRD.id)})
                      </Button>
                    )}
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

              {/* Display change information if available */}
              {analysisResult?.changeId && selectedPRD && (
                <CardContent className="pt-4 pb-0">
                  <Alert>
                    <FileText className="h-4 w-4" />
                    <AlertTitle>Change Proposal Created</AlertTitle>
                    <AlertDescription className="flex items-center justify-between">
                      <span>
                        Change proposal created: <code className="text-xs">{analysisResult.changeId}</code>
                      </span>
                      <Button
                        variant="link"
                        size="sm"
                        className="h-auto p-0"
                        onClick={(e) => {
                          e.stopPropagation();
                          window.location.href = `#/projects/${projectId}/changes/${analysisResult.changeId}`;
                        }}
                      >
                        View Change <ExternalLink className="ml-1 h-3 w-3" />
                      </Button>
                    </AlertDescription>
                  </Alert>
                </CardContent>
              )}

              {/* Display validation errors */}
              {analysisResult?.validationStatus === 'invalid' && analysisResult.validationErrors && (
                <CardContent className="pt-4 pb-0">
                  <Alert variant="destructive">
                    <AlertTriangle className="h-4 w-4" />
                    <AlertTitle>Validation Errors</AlertTitle>
                    <AlertDescription>
                      <ul className="list-disc list-inside mt-2">
                        {analysisResult.validationErrors.map((error, i) => (
                          <li key={i}>
                            {error.line && <span className="font-mono text-xs">Line {error.line}: </span>}
                            {error.message}
                          </li>
                        ))}
                      </ul>
                    </AlertDescription>
                  </Alert>
                </CardContent>
              )}

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
          )}
        </TabsContent>
      </Tabs>

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

      <CreatePRDFlow
        projectId={projectId}
        open={showIdeateFlow}
        onOpenChange={setShowIdeateFlow}
        onSessionCreated={handleSessionCreated}
      />

      {activeSessionId && (
        <QuickModeFlow
          projectId={projectId}
          sessionId={activeSessionId}
          open={showQuickModeFlow}
          onOpenChange={setShowQuickModeFlow}
          onComplete={handleQuickModeComplete}
        />
      )}
    </div>
  );
}
