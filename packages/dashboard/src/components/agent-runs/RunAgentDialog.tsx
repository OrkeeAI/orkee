// ABOUTME: Multi-stage dialog for creating agent runs from a PRD
// ABOUTME: Generates stories from PRD content, allows review/editing, then starts an autonomous run
import { useState, useEffect, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';
import { toast } from 'sonner';
import { Bot, Loader2, Play, ChevronLeft, ChevronRight, Pencil, AlertTriangle, Check, Terminal } from 'lucide-react';
import { Button } from '@/components/ui/button';
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
import { Badge } from '@/components/ui/badge';
import { Separator } from '@/components/ui/separator';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { convertPrdToStories } from '@/services/story-converter';
import { startRun } from '@/services/agent-runs';
import { api } from '@/services/api';
import { useAuth } from '@/contexts/AuthContext';
import { useCurrentUser } from '@/hooks/useUsers';
import { useModelPreferences, getModelForTask } from '@/services/model-preferences';
import type { PrdJson, UserStory } from '@/services/agent-runs';
import type { PRD } from '@/services/prds';

type Stage = 'generating' | 'review' | 'configure';

interface RunAgentDialogProps {
  projectId: string;
  projectName: string;
  prd: PRD;
  open: boolean;
  onOpenChange: (open: boolean) => void;
}

export function RunAgentDialog({ projectId, projectName, prd, open, onOpenChange }: RunAgentDialogProps) {
  const navigate = useNavigate();
  const { data: currentUser } = useCurrentUser();
  const { data: preferences } = useModelPreferences(currentUser?.id);
  const { authStatus, refreshAuth } = useAuth();

  const [stage, setStage] = useState<Stage>('generating');
  const [prdJson, setPrdJson] = useState<PrdJson | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [maxIterations, setMaxIterations] = useState(10);
  const [isStarting, setIsStarting] = useState(false);
  const [editingStoryId, setEditingStoryId] = useState<string | null>(null);
  const [editTitle, setEditTitle] = useState('');
  const [tokenInput, setTokenInput] = useState('');
  const [isImporting, setIsImporting] = useState(false);
  const [importError, setImportError] = useState<string | null>(null);
  const [importSuccess, setImportSuccess] = useState(false);

  const isClaudeAuthenticated = authStatus.claude?.authenticated ?? false;

  const generateStories = useCallback(async () => {
    setStage('generating');
    setError(null);
    setPrdJson(null);

    try {
      const modelConfig = getModelForTask(preferences, 'prd_analysis');
      const result = await convertPrdToStories(
        prd.contentMarkdown,
        projectName,
        projectId,
        modelConfig,
      );
      setPrdJson(result);
      setStage('review');
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to generate stories';
      setError(message);
      toast.error('Story generation failed', { description: message });
    }
  }, [prd.contentMarkdown, projectName, projectId, preferences]);

  // Generate stories when dialog opens
  useEffect(() => {
    if (open) {
      generateStories();
    } else {
      // Reset state on close
      setStage('generating');
      setPrdJson(null);
      setError(null);
      setMaxIterations(10);
      setIsStarting(false);
      setEditingStoryId(null);
      setTokenInput('');
      setIsImporting(false);
      setImportError(null);
      setImportSuccess(false);
    }
  }, [open, generateStories]);

  const handleUpdateStoryTitle = (storyId: string, title: string) => {
    if (!prdJson) return;
    setPrdJson({
      ...prdJson,
      userStories: prdJson.userStories.map(s =>
        s.id === storyId ? { ...s, title } : s
      ),
    });
    setEditingStoryId(null);
  };

  const handleImportToken = async () => {
    const trimmed = tokenInput.trim();
    if (!trimmed) return;

    setIsImporting(true);
    setImportError(null);

    try {
      const response = await api('/api/auth/claude/import', {
        method: 'POST',
        body: JSON.stringify({ token: trimmed }),
      });

      if (response.success) {
        setImportSuccess(true);
        setTokenInput('');
        await refreshAuth();
        toast.success('Claude token imported successfully');
      } else {
        setImportError(response.error || 'Failed to import token');
      }
    } catch (err) {
      setImportError(err instanceof Error ? err.message : 'Failed to import token');
    } finally {
      setIsImporting(false);
    }
  };

  const handleStartRun = async () => {
    if (!prdJson) return;
    setIsStarting(true);

    try {
      const run = await startRun({
        project_id: projectId,
        prd_id: prd.id,
        prd_json: prdJson,
        max_iterations: maxIterations,
      });
      toast.success('Agent run started');
      onOpenChange(false);
      navigate(`/agent-runs/${run.id}`);
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Failed to start run';
      toast.error('Failed to start run', { description: message });
      setIsStarting(false);
    }
  };

  const stories = prdJson?.userStories || [];
  const epicGroups = stories.reduce<Record<string, UserStory[]>>((acc, story) => {
    const epic = story.epic || 'Ungrouped';
    if (!acc[epic]) acc[epic] = [];
    acc[epic].push(story);
    return acc;
  }, {});

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[750px] max-h-[85vh]" aria-describedby="run-agent-description">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Bot className="h-5 w-5" />
            Run Agent on &ldquo;{prd.title}&rdquo;
          </DialogTitle>
          <DialogDescription id="run-agent-description">
            {stage === 'generating' && 'Generating user stories from PRD content...'}
            {stage === 'review' && `Review the ${stories.length} generated stories before starting the run.`}
            {stage === 'configure' && 'Configure run settings and start the autonomous agent.'}
          </DialogDescription>
        </DialogHeader>

        {/* Stage: Generating */}
        {stage === 'generating' && !error && (
          <div className="flex flex-col items-center justify-center py-12 space-y-4">
            <Loader2 className="h-8 w-8 animate-spin text-primary" />
            <p className="text-sm text-muted-foreground">
              Breaking down PRD into right-sized user stories...
            </p>
          </div>
        )}

        {/* Stage: Generation Error */}
        {stage === 'generating' && error && (
          <div className="space-y-4 py-6">
            <div className="bg-destructive/10 text-destructive rounded-md p-4 text-sm">
              {error}
            </div>
            <div className="flex justify-center">
              <Button variant="outline" onClick={generateStories}>
                Retry
              </Button>
            </div>
          </div>
        )}

        {/* Stage: Review Stories */}
        {stage === 'review' && prdJson && (
          <div className="space-y-4 max-h-[55vh] overflow-y-auto pr-1">
            <div className="flex items-center justify-between">
              <div className="text-sm text-muted-foreground">
                {stories.length} stories across {Object.keys(epicGroups).length} epics
              </div>
              <Badge variant="outline">{prdJson.branchName}</Badge>
            </div>

            {Object.entries(epicGroups).map(([epic, epicStories]) => (
              <div key={epic} className="space-y-2">
                <h4 className="text-sm font-medium text-muted-foreground">{epic}</h4>
                <div className="space-y-1">
                  {epicStories.sort((a, b) => a.priority - b.priority).map(story => (
                    <div
                      key={story.id}
                      className="flex items-center gap-3 rounded-md border p-3 text-sm"
                    >
                      <Badge variant="secondary" className="shrink-0 font-mono text-xs">
                        {story.id}
                      </Badge>
                      <div className="flex-1 min-w-0">
                        {editingStoryId === story.id ? (
                          <Input
                            value={editTitle}
                            onChange={(e) => setEditTitle(e.target.value)}
                            onBlur={() => handleUpdateStoryTitle(story.id, editTitle)}
                            onKeyDown={(e) => {
                              if (e.key === 'Enter') handleUpdateStoryTitle(story.id, editTitle);
                              if (e.key === 'Escape') setEditingStoryId(null);
                            }}
                            className="h-7 text-sm"
                            autoFocus
                          />
                        ) : (
                          <span className="truncate block">{story.title}</span>
                        )}
                      </div>
                      <span className="text-xs text-muted-foreground shrink-0">
                        {story.acceptanceCriteria.length} AC
                      </span>
                      <Button
                        variant="ghost"
                        size="icon"
                        className="h-6 w-6 shrink-0"
                        onClick={() => {
                          setEditingStoryId(story.id);
                          setEditTitle(story.title);
                        }}
                      >
                        <Pencil className="h-3 w-3" />
                      </Button>
                    </div>
                  ))}
                </div>
              </div>
            ))}
          </div>
        )}

        {/* Stage: Configure & Start */}
        {stage === 'configure' && prdJson && (
          <div className="space-y-6 py-4">
            {!isClaudeAuthenticated && (
              <Alert variant="destructive" className="border-amber-500/50 bg-amber-500/10 text-amber-700 dark:text-amber-400 [&>svg]:text-amber-600">
                <AlertTriangle className="h-4 w-4" />
                <AlertDescription className="space-y-3">
                  <p className="font-medium">Claude Authentication Required</p>
                  <p className="text-sm">
                    The agent runner needs a Claude OAuth token to execute. Get one by running:
                  </p>
                  <div className="flex items-center gap-2 rounded bg-muted/50 px-3 py-2 font-mono text-xs">
                    <Terminal className="h-3.5 w-3.5 shrink-0" />
                    claude setup-token
                  </div>
                  <p className="text-sm">Then paste the token below:</p>
                  <div className="flex gap-2">
                    <Input
                      type="password"
                      placeholder="sk-ant-oat01-..."
                      value={tokenInput}
                      onChange={(e) => {
                        setTokenInput(e.target.value);
                        setImportError(null);
                      }}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') handleImportToken();
                      }}
                      className="font-mono text-xs"
                    />
                    <Button
                      size="sm"
                      variant="outline"
                      onClick={handleImportToken}
                      disabled={isImporting || !tokenInput.trim()}
                    >
                      {isImporting ? (
                        <Loader2 className="h-4 w-4 animate-spin" />
                      ) : (
                        'Import'
                      )}
                    </Button>
                  </div>
                  {importError && (
                    <p className="text-xs text-destructive">{importError}</p>
                  )}
                </AlertDescription>
              </Alert>
            )}

            {isClaudeAuthenticated && importSuccess && (
              <Alert className="border-green-500/50 bg-green-500/10 text-green-700 dark:text-green-400 [&>svg]:text-green-600">
                <Check className="h-4 w-4" />
                <AlertDescription>
                  Claude token imported successfully.
                </AlertDescription>
              </Alert>
            )}

            <div className="space-y-2">
              <h4 className="text-sm font-medium">Run Summary</h4>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Stories:</span>{' '}
                  <span className="font-medium">{stories.length}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Epics:</span>{' '}
                  <span className="font-medium">{Object.keys(epicGroups).length}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Branch:</span>{' '}
                  <code className="text-xs">{prdJson.branchName}</code>
                </div>
                <div>
                  <span className="text-muted-foreground">Source PRD:</span>{' '}
                  <span className="font-medium">{prd.title}</span>
                </div>
              </div>
            </div>

            <Separator />

            <div className="space-y-2">
              <Label htmlFor="max-iterations">Max Iterations</Label>
              <Input
                id="max-iterations"
                type="number"
                min={1}
                max={100}
                value={maxIterations}
                onChange={(e) => setMaxIterations(Math.max(1, parseInt(e.target.value) || 1))}
                className="w-32"
              />
              <p className="text-xs text-muted-foreground">
                The agent will attempt up to this many iterations to complete all stories.
              </p>
            </div>
          </div>
        )}

        <DialogFooter className="gap-2 sm:gap-0">
          {stage === 'review' && (
            <>
              <Button variant="outline" onClick={() => onOpenChange(false)}>
                Cancel
              </Button>
              <Button onClick={() => setStage('configure')}>
                Next
                <ChevronRight className="ml-1 h-4 w-4" />
              </Button>
            </>
          )}
          {stage === 'configure' && (
            <>
              <Button variant="outline" onClick={() => setStage('review')}>
                <ChevronLeft className="mr-1 h-4 w-4" />
                Back
              </Button>
              <Button onClick={handleStartRun} disabled={isStarting || !isClaudeAuthenticated}>
                {isStarting ? (
                  <>
                    <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                    Starting...
                  </>
                ) : (
                  <>
                    <Play className="mr-2 h-4 w-4" />
                    Start Run
                  </>
                )}
              </Button>
            </>
          )}
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
