// ABOUTME: PRD management view displaying list of PRDs with metadata and content
// ABOUTME: Integrates with PRDUploadDialog and RunAgentDialog for creating, analyzing, and executing PRDs
import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { FileText, Upload, Sparkles, Trash2, Calendar, User, ExternalLink, AlertTriangle, Bot } from 'lucide-react';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { Progress } from '@/components/ui/progress';
import { Separator } from '@/components/ui/separator';
import { MarkdownRenderer } from '@/components/MarkdownRenderer';
import { RunStatusBadge } from '@/components/agent-runs/RunStatusBadge';
import { RunAgentDialog } from '@/components/agent-runs/RunAgentDialog';
import { usePRDs, useDeletePRD, useTriggerPRDAnalysis } from '@/hooks/usePRDs';
import { useCurrentUser } from '@/hooks/useUsers';
import { useModelPreferences } from '@/services/model-preferences';
import { PRDUploadDialog } from '@/components/PRDUploadDialog';
import { listRuns, type AgentRun } from '@/services/agent-runs';
import type { PRD, PRDAnalysisResult } from '@/services/prds';

interface PRDViewProps {
  projectId: string;
  projectName: string;
}

export function PRDView({ projectId, projectName }: PRDViewProps) {
  const navigate = useNavigate();
  const [selectedPRD, setSelectedPRD] = useState<PRD | null>(null);
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [showRunDialog, setShowRunDialog] = useState(false);
  const [analysisResult, setAnalysisResult] = useState<PRDAnalysisResult | null>(null);
  const [prdRuns, setPrdRuns] = useState<AgentRun[]>([]);

  const { data: prds, isLoading, error } = usePRDs(projectId);
  const { data: currentUser } = useCurrentUser();
  const { data: preferences } = useModelPreferences(currentUser?.id);
  const deletePRDMutation = useDeletePRD(projectId);
  const analyzePRDMutation = useTriggerPRDAnalysis(projectId);

  // Load runs linked to the selected PRD
  const loadPrdRuns = useCallback(async (prdId: string) => {
    try {
      const runs = await listRuns(projectId, undefined, undefined, prdId);
      setPrdRuns(runs);
    } catch {
      setPrdRuns([]);
    }
  }, [projectId]);

  useEffect(() => {
    if (selectedPRD) {
      loadPrdRuns(selectedPRD.id);
    } else {
      setPrdRuns([]);
    }
  }, [selectedPRD, loadPrdRuns]);

  const handleDelete = (prdId: string) => {
    if (confirm('Are you sure you want to delete this PRD? This action cannot be undone.')) {
      deletePRDMutation.mutate(prdId);
      if (selectedPRD?.id === prdId) {
        setSelectedPRD(null);
      }
    }
  };

  const handleAnalyzeClick = (prdId: string) => {
    if (!preferences) {
      alert('Model preferences not loaded. Please visit Settings → AI Models to configure your preferred models.');
      return;
    }

    console.log(`Analyzing PRD ${prdId} with user's preferred model for PRD analysis`);
    analyzePRDMutation.mutate(
      { prdId },
      {
        onSuccess: (result) => {
          setAnalysisResult(result);
        },
      }
    );
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
          <Button variant="outline" onClick={() => setShowUploadDialog(true)} size="sm">
            <Upload className="mr-2 h-4 w-4" />
            Upload PRD
          </Button>
        </div>
      </div>

      {(!prds || prds.length === 0) ? (
        <div className="flex flex-col items-center justify-center h-64 space-y-4">
          <FileText className="h-16 w-16 text-muted-foreground" />
          <div className="text-center space-y-2">
            <h3 className="text-lg font-semibold">No PRDs Yet</h3>
            <p className="text-sm text-muted-foreground max-w-md">
              Upload a PRD or create one through the Ideate tab.
            </p>
          </div>
          <Button variant="outline" onClick={() => setShowUploadDialog(true)}>
            <Upload className="mr-2 h-4 w-4" />
            Upload PRD
          </Button>
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
                  <div className="text-xs text-muted-foreground">
                    <div className="flex items-center gap-2 mt-1">
                      {getSourceBadge(prd.source)}
                      <span>v{prd.version}</span>
                    </div>
                  </div>
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
                        onClick={() => setShowRunDialog(true)}
                        disabled={selectedPRD.status === 'superseded'}
                      >
                        <Bot className="mr-2 h-4 w-4" />
                        Run Agent
                      </Button>
                      <Button
                        variant="outline"
                        size="sm"
                        onClick={() => handleAnalyzeClick(selectedPRD.id)}
                        disabled={analyzePRDMutation.isPending || !preferences}
                      >
                        {analyzePRDMutation.isPending ? (
                          <>
                            <div className="mr-2 h-3 w-3 animate-spin rounded-full border-2 border-current border-t-transparent" />
                            Analyzing...
                          </>
                        ) : (
                          <>
                            <Sparkles className="mr-2 h-4 w-4" />
                            Analyze with {preferences?.prdAnalysis?.model || 'Default Model'}
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
                  <MarkdownRenderer content={selectedPRD.contentMarkdown} />
                </CardContent>

                {/* Run history for this PRD */}
                {prdRuns.length > 0 && (
                  <>
                    <Separator />
                    <CardContent className="pt-4">
                      <h4 className="text-sm font-medium mb-3">Agent Runs ({prdRuns.length})</h4>
                      <div className="space-y-2">
                        {prdRuns.map(run => {
                          const progress = run.storiesTotal > 0
                            ? (run.storiesCompleted / run.storiesTotal) * 100
                            : 0;
                          return (
                            <div
                              key={run.id}
                              className="flex items-center gap-3 rounded-md border p-3 cursor-pointer hover:bg-accent/50 transition-colors"
                              onClick={() => navigate(`/agent-runs/${run.id}`)}
                            >
                              <RunStatusBadge status={run.status} />
                              <div className="flex-1 min-w-0">
                                <div className="text-sm">
                                  {run.storiesCompleted}/{run.storiesTotal} stories
                                </div>
                                <Progress value={progress} className="h-1.5 mt-1" />
                              </div>
                              <span className="text-xs text-muted-foreground shrink-0">
                                ${run.totalCost.toFixed(2)}
                              </span>
                            </div>
                          );
                        })}
                      </div>
                    </CardContent>
                  </>
                )}
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

      {selectedPRD && (
        <RunAgentDialog
          projectId={projectId}
          projectName={projectName}
          prd={selectedPRD}
          open={showRunDialog}
          onOpenChange={setShowRunDialog}
        />
      )}
    </div>
  );
}
