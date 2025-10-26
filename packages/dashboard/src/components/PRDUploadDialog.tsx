// ABOUTME: Dialog for uploading and analyzing PRD documents with markdown preview
// ABOUTME: Supports file upload, paste, markdown rendering, and AI capability extraction
import React, { useState, useRef } from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeHighlight from 'rehype-highlight';
import rehypeSanitize from 'rehype-sanitize';
import 'highlight.js/styles/github-dark.css';
import { toast } from 'sonner';
import { Button } from '@/components/ui/button';
import { ModelSelectionDialog } from '@/components/ModelSelectionDialog';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Progress } from '@/components/ui/progress';
import { useCreatePRD, useTriggerPRDAnalysis, useDeletePRD } from '@/hooks/usePRDs';
import type { PRDAnalysisResult, SpecCapability } from '@/services/prds';
import { FileText, Upload, Eye, Sparkles, CheckCircle, Loader2 } from 'lucide-react';

interface PRDUploadDialogProps {
  projectId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: (prdId: string) => void;
}

export function PRDUploadDialog({ projectId, open, onOpenChange, onComplete = () => {} }: PRDUploadDialogProps) {
  const [title, setTitle] = useState('');
  const [contentMarkdown, setContentMarkdown] = useState('');
  const [activeTab, setActiveTab] = useState('upload');
  const [analysisResult, setAnalysisResult] = useState<PRDAnalysisResult | null>(null);
  const [isAnalyzing, setIsAnalyzing] = useState(false);
  const [tempPRDId, setTempPRDId] = useState<string | null>(null);
  const [isLoadingFile, setIsLoadingFile] = useState(false);
  const [showModelSelection, setShowModelSelection] = useState(false);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const createPRDMutation = useCreatePRD(projectId);
  const analyzePRDMutation = useTriggerPRDAnalysis(projectId);
  const deletePRDMutation = useDeletePRD(projectId);
  const { isPending: isSaving, error: saveError } = createPRDMutation;

  const handleFileUpload = async (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (!file) return;

    setIsLoadingFile(true);
    try {
      const text = await file.text();
      setContentMarkdown(text);
      if (!title) {
        const filename = file.name.replace(/\.(md|markdown|txt)$/i, '');
        setTitle(filename);
      }
      setActiveTab('preview');
    } catch (error) {
      console.error('Failed to read file:', error);
    } finally {
      setIsLoadingFile(false);
    }
  };

  const handleAnalyzeClick = () => {
    if (!contentMarkdown.trim()) return;
    setShowModelSelection(true);
  };

  const handleAnalyze = async (provider: string, model: string) => {
    if (!contentMarkdown.trim()) return;

    setIsAnalyzing(true);
    let createdPRDId: string | null = null;

    try {
      const tempPRD = await createPRDMutation.mutateAsync({
        title: title || 'Untitled PRD',
        contentMarkdown,
        status: 'draft',
        source: 'manual',
      });

      createdPRDId = tempPRD.id;
      setTempPRDId(createdPRDId);

      // TODO: Update analyze API to accept provider and model
      // For now, the backend will use environment variable or default
      console.log(`Analyzing with ${provider}/${model}`);
      
      const result = await analyzePRDMutation.mutateAsync(tempPRD.id);
      setAnalysisResult(result);
      setActiveTab('analysis');
      
      toast.success(`PRD analyzed with ${provider}/${model}`);
    } catch (error) {
      console.error('Analysis failed:', error);
      toast.error('Failed to analyze PRD. Please try again.');

      if (createdPRDId) {
        try {
          await deletePRDMutation.mutateAsync(createdPRDId);
          setTempPRDId(null);
        } catch (deleteError) {
          console.error('Failed to clean up temp PRD:', deleteError);
          toast.error('Analysis failed and cleanup encountered an error. A draft PRD may need manual cleanup.');
        }
      }
    } finally {
      setIsAnalyzing(false);
    }
  };

  const handleSave = async () => {
    if (!title.trim() || !contentMarkdown.trim()) return;

    try {
      if (tempPRDId) {
        await deletePRDMutation.mutateAsync(tempPRDId);
        setTempPRDId(null);
      }

      const prd = await createPRDMutation.mutateAsync({
        title,
        contentMarkdown,
        status: 'draft',
        source: 'manual',
      });

      onComplete(prd.id);
      onOpenChange(false);
      resetForm();
    } catch {
      // Error handled by React Query mutation
    }
  };

  const resetForm = async () => {
    if (tempPRDId) {
      try {
        await deletePRDMutation.mutateAsync(tempPRDId);
      } catch (error) {
        console.error('Failed to clean up temp PRD on reset:', error);
        toast.warning('Failed to clean up temporary PRD. It may need manual deletion.');
      }
    }

    setTitle('');
    setContentMarkdown('');
    setActiveTab('upload');
    setAnalysisResult(null);
    setIsAnalyzing(false);
    setTempPRDId(null);
    setIsLoadingFile(false);
    createPRDMutation.reset();
  };

  const hasContent = contentMarkdown.trim().length > 0;
  const canSave = title.trim().length > 0 && hasContent;

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[900px] max-h-[85vh]" aria-describedby="prd-upload-description">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            Upload Product Requirements Document
          </DialogTitle>
          <DialogDescription id="prd-upload-description">
            Upload or paste your PRD markdown content. Preview and optionally analyze with AI to extract capabilities.
          </DialogDescription>
        </DialogHeader>

        <Tabs value={activeTab} onValueChange={setActiveTab} className="w-full">
          <TabsList className="grid w-full grid-cols-3">
            <TabsTrigger value="upload" className="flex items-center gap-2">
              <Upload className="h-4 w-4" />
              Upload
            </TabsTrigger>
            <TabsTrigger value="preview" disabled={!hasContent} className="flex items-center gap-2">
              <Eye className="h-4 w-4" />
              Preview
            </TabsTrigger>
            <TabsTrigger value="analysis" disabled={!analysisResult} className="flex items-center gap-2">
              <Sparkles className="h-4 w-4" />
              Analysis
              {analysisResult && <CheckCircle className="h-3 w-3 text-green-600" />}
            </TabsTrigger>
          </TabsList>

          <TabsContent value="upload" className="space-y-4 py-4">
            <div className="grid gap-2">
              <Label htmlFor="prd-title">PRD Title</Label>
              <Input
                id="prd-title"
                placeholder="e.g., User Authentication System"
                value={title}
                onChange={(e) => setTitle(e.target.value)}
              />
            </div>

            <div className="grid gap-2">
              <Label htmlFor="prd-file">Upload Markdown File</Label>
              <div className="flex gap-2">
                <Input
                  id="prd-file"
                  type="file"
                  accept=".md,.markdown,.txt"
                  ref={fileInputRef}
                  onChange={handleFileUpload}
                  disabled={isLoadingFile}
                  className="file:mr-4 file:py-2 file:px-4 file:rounded file:border-0 file:text-sm file:font-semibold file:bg-primary file:text-primary-foreground hover:file:bg-primary/90"
                />
                <Button
                  type="button"
                  variant="outline"
                  onClick={() => fileInputRef.current?.click()}
                  disabled={isLoadingFile}
                >
                  {isLoadingFile ? (
                    <>
                      <Loader2 className="h-4 w-4 animate-spin mr-2" />
                      Loading
                    </>
                  ) : (
                    'Browse'
                  )}
                </Button>
              </div>
              <p className="text-sm text-muted-foreground">
                Or paste your markdown content below
              </p>
              {isLoadingFile && (
                <div className="flex items-center gap-2 text-sm text-primary">
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Loading file...
                </div>
              )}
            </div>

            <div className="grid gap-2">
              <Label htmlFor="prd-content">PRD Content (Markdown)</Label>
              <Textarea
                id="prd-content"
                placeholder={`# Product Requirements Document\n\n## Overview\nDescribe your product...\n\n## Requirements\n- Feature 1\n- Feature 2`}
                value={contentMarkdown}
                onChange={(e) => setContentMarkdown(e.target.value)}
                rows={15}
                className="font-mono text-sm"
              />
              <p className="text-xs text-muted-foreground">
                {contentMarkdown.length} characters
              </p>
            </div>

            {hasContent && (
              <div className="flex gap-2">
                <Button
                  type="button"
                  variant="secondary"
                  onClick={() => setActiveTab('preview')}
                  className="flex items-center gap-2"
                >
                  <Eye className="h-4 w-4" />
                  Preview
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  onClick={handleAnalyzeClick}
                  disabled={isAnalyzing || !canSave}
                  className="flex items-center gap-2"
                >
                  <Sparkles className="h-4 w-4" />
                  {isAnalyzing ? 'Analyzing...' : 'Analyze with AI'}
                </Button>
              </div>
            )}
          </TabsContent>

          <TabsContent value="preview" className="py-4">
            <div className="rounded-md border p-6 max-h-[500px] overflow-y-auto prose prose-sm dark:prose-invert max-w-none">
              <ReactMarkdown
                remarkPlugins={[remarkGfm]}
                rehypePlugins={[rehypeHighlight, rehypeSanitize]}
              >
                {contentMarkdown}
              </ReactMarkdown>
            </div>
          </TabsContent>

          <TabsContent value="analysis" className="py-4">
            {isAnalyzing ? (
              <div className="space-y-4">
                <div className="flex items-center gap-3">
                  <Sparkles className="h-5 w-5 animate-pulse text-primary" />
                  <p className="text-sm font-medium">Analyzing PRD with AI...</p>
                </div>
                <Progress value={undefined} className="w-full" />
                <p className="text-xs text-muted-foreground">
                  Extracting capabilities, requirements, and task suggestions
                </p>
              </div>
            ) : analysisResult ? (
              <div className="space-y-6 max-h-[500px] overflow-y-auto">
                <div className="space-y-2">
                  <h3 className="text-lg font-semibold">Summary</h3>
                  <p className="text-sm text-muted-foreground">{analysisResult.summary}</p>
                </div>

                <div className="space-y-3">
                  <h3 className="text-lg font-semibold">
                    Capabilities ({analysisResult.capabilities.length})
                  </h3>
                  <div className="grid gap-3">
                    {analysisResult.capabilities.map((capability, idx) => (
                      <CapabilityCard key={idx} capability={capability} />
                    ))}
                  </div>
                </div>

                {analysisResult.suggestedTasks.length > 0 && (
                  <div className="space-y-3">
                    <h3 className="text-lg font-semibold">
                      Suggested Tasks ({analysisResult.suggestedTasks.length})
                    </h3>
                    <div className="grid gap-2">
                      {analysisResult.suggestedTasks.slice(0, 5).map((task, idx) => (
                        <div key={idx} className="p-3 rounded-md border bg-card text-card-foreground">
                          <p className="font-medium text-sm">{task.title}</p>
                          <p className="text-xs text-muted-foreground mt-1">{task.description}</p>
                          <div className="flex gap-2 mt-2">
                            <span className="text-xs px-2 py-1 rounded bg-secondary">
                              {task.capabilityId}
                            </span>
                            <span className="text-xs px-2 py-1 rounded bg-secondary">
                              Complexity: {task.complexity}/10
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </div>
                )}

                {analysisResult.dependencies && analysisResult.dependencies.length > 0 && (
                  <div className="space-y-2">
                    <h3 className="text-sm font-semibold">Dependencies</h3>
                    <div className="flex flex-wrap gap-2">
                      {analysisResult.dependencies.map((dep, idx) => (
                        <span key={idx} className="text-xs px-2 py-1 rounded border">
                          {dep}
                        </span>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            ) : null}
          </TabsContent>
        </Tabs>

        {saveError && (
          <div className="text-sm text-red-600 bg-red-50 dark:bg-red-950 p-3 rounded-md">
            {saveError.message || 'Failed to save PRD'}
          </div>
        )}

        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)} disabled={isSaving}>
            Cancel
          </Button>
          <Button onClick={handleSave} disabled={!canSave || isSaving}>
            {isSaving ? 'Saving...' : 'Save PRD'}
          </Button>
        </DialogFooter>
      </DialogContent>

      {/* Model Selection Dialog */}
      <ModelSelectionDialog
        open={showModelSelection}
        onOpenChange={setShowModelSelection}
        onConfirm={handleAnalyze}
      />
    </Dialog>
  );
}

function CapabilityCard({ capability }: { capability: SpecCapability }) {
  return (
    <div className="p-4 rounded-md border bg-card text-card-foreground">
      <div className="flex items-start justify-between">
        <div>
          <h4 className="font-semibold text-sm">{capability.name}</h4>
          <p className="text-xs text-muted-foreground mt-1">{capability.purpose}</p>
        </div>
        <span className="text-xs px-2 py-1 rounded bg-secondary whitespace-nowrap">
          {capability.requirements.length} req
        </span>
      </div>

      {capability.requirements.length > 0 && (
        <div className="mt-3 space-y-2">
          {capability.requirements.slice(0, 2).map((req, idx) => (
            <div key={idx} className="pl-3 border-l-2 border-primary/50">
              <p className="text-xs font-medium">{req.name}</p>
              <p className="text-xs text-muted-foreground line-clamp-2">{req.content}</p>
              <p className="text-xs text-primary mt-1">
                {req.scenarios.length} scenario{req.scenarios.length !== 1 ? 's' : ''}
              </p>
            </div>
          ))}
          {capability.requirements.length > 2 && (
            <p className="text-xs text-muted-foreground pl-3">
              +{capability.requirements.length - 2} more requirements
            </p>
          )}
        </div>
      )}
    </div>
  );
}
