// ABOUTME: Dialog for generating an Epic from a PRD using AI
// ABOUTME: Supports manual creation and AI-powered generation with model selection

import { useState } from 'react';
import { Sparkles, FileText, Loader2 } from 'lucide-react';
import { Button } from '@/components/ui/button';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Textarea } from '@/components/ui/textarea';
import { Switch } from '@/components/ui/switch';
import { Alert, AlertDescription } from '@/components/ui/alert';
import type { CreateEpicInput } from '@/services/epics';

interface EpicGeneratorProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  prdId: string;
  prdTitle: string;
  onGenerate: (input: CreateEpicInput, generateTasks: boolean) => Promise<void>;
  isGenerating?: boolean;
}

export function EpicGenerator({
  open,
  onOpenChange,
  prdId,
  prdTitle,
  onGenerate,
  isGenerating = false,
}: EpicGeneratorProps) {
  const [mode, setMode] = useState<'manual' | 'ai'>('ai');
  const [name, setName] = useState('');
  const [overview, setOverview] = useState('');
  const [technicalApproach, setTechnicalApproach] = useState('');
  const [includeTaskBreakdown, setIncludeTaskBreakdown] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    if (!name.trim()) {
      setError('Epic name is required');
      return;
    }

    if (mode === 'manual' && !overview.trim()) {
      setError('Overview is required for manual creation');
      return;
    }

    if (mode === 'manual' && !technicalApproach.trim()) {
      setError('Technical approach is required for manual creation');
      return;
    }

    try {
      const input: CreateEpicInput = {
        prdId,
        name: name.trim(),
        overviewMarkdown: overview.trim() || `# ${name}\n\nGenerated from PRD: ${prdTitle}`,
        technicalApproach: technicalApproach.trim() || 'To be determined',
      };

      await onGenerate(input, includeTaskBreakdown);

      // Reset form
      setName('');
      setOverview('');
      setTechnicalApproach('');
      setIncludeTaskBreakdown(false);
      onOpenChange(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to generate epic');
    }
  };

  const handleCancel = () => {
    setName('');
    setOverview('');
    setTechnicalApproach('');
    setError(null);
    setIncludeTaskBreakdown(false);
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle>Generate Epic from PRD</DialogTitle>
          <DialogDescription>
            Create an Epic from <strong>{prdTitle}</strong>
          </DialogDescription>
        </DialogHeader>

        <form onSubmit={handleSubmit} className="space-y-6">
          {/* Mode Selection */}
          <div className="flex items-center justify-center gap-2 p-4 border rounded-lg bg-muted/50">
            <Button
              type="button"
              variant={mode === 'ai' ? 'default' : 'outline'}
              onClick={() => setMode('ai')}
              className="flex-1"
            >
              <Sparkles className="h-4 w-4 mr-2" />
              AI Generate
            </Button>
            <Button
              type="button"
              variant={mode === 'manual' ? 'default' : 'outline'}
              onClick={() => setMode('manual')}
              className="flex-1"
            >
              <FileText className="h-4 w-4 mr-2" />
              Manual Create
            </Button>
          </div>

          {/* AI Mode Info */}
          {mode === 'ai' && (
            <Alert>
              <Sparkles className="h-4 w-4" />
              <AlertDescription>
                AI generation is not yet implemented. The Epic will be created with a basic
                template that you can edit after creation.
              </AlertDescription>
            </Alert>
          )}

          {/* Epic Name */}
          <div className="space-y-2">
            <Label htmlFor="name">Epic Name *</Label>
            <Input
              id="name"
              placeholder="e.g., User Authentication System"
              value={name}
              onChange={(e) => setName(e.target.value)}
              required
            />
          </div>

          {/* Manual Mode Fields */}
          {mode === 'manual' && (
            <>
              <div className="space-y-2">
                <Label htmlFor="overview">Overview *</Label>
                <Textarea
                  id="overview"
                  placeholder="Describe the epic's purpose, scope, and high-level objectives..."
                  value={overview}
                  onChange={(e) => setOverview(e.target.value)}
                  rows={6}
                  required
                />
                <p className="text-xs text-muted-foreground">Supports Markdown formatting</p>
              </div>

              <div className="space-y-2">
                <Label htmlFor="technical-approach">Technical Approach *</Label>
                <Textarea
                  id="technical-approach"
                  placeholder="Outline the technical strategy, architecture patterns, and implementation details..."
                  value={technicalApproach}
                  onChange={(e) => setTechnicalApproach(e.target.value)}
                  rows={6}
                  required
                />
                <p className="text-xs text-muted-foreground">Supports Markdown formatting</p>
              </div>
            </>
          )}

          {/* Task Breakdown Option */}
          <div className="flex items-center justify-between p-4 border rounded-lg">
            <div className="space-y-0.5">
              <Label htmlFor="task-breakdown">Generate task breakdown</Label>
              <p className="text-sm text-muted-foreground">
                Automatically decompose this epic into tasks
              </p>
            </div>
            <Switch
              id="task-breakdown"
              checked={includeTaskBreakdown}
              onCheckedChange={setIncludeTaskBreakdown}
              disabled={true} // Disabled until Phase 4
            />
          </div>

          {includeTaskBreakdown && (
            <Alert>
              <AlertDescription>
                Task decomposition is not yet implemented (Phase 4). The Epic will be created
                without tasks.
              </AlertDescription>
            </Alert>
          )}

          {/* Error Message */}
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}

          <DialogFooter>
            <Button type="button" variant="outline" onClick={handleCancel} disabled={isGenerating}>
              Cancel
            </Button>
            <Button type="submit" disabled={isGenerating}>
              {isGenerating ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  {mode === 'ai' ? 'Generating...' : 'Creating...'}
                </>
              ) : (
                <>
                  {mode === 'ai' ? (
                    <>
                      <Sparkles className="h-4 w-4 mr-2" />
                      Generate Epic
                    </>
                  ) : (
                    <>
                      <FileText className="h-4 w-4 mr-2" />
                      Create Epic
                    </>
                  )}
                </>
              )}
            </Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  );
}
