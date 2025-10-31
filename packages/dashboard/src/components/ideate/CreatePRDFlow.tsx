// ABOUTME: Main entry component for creating a new ideate session
// ABOUTME: Multi-step dialog: mode selection, template selection, description input, session creation
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
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ModeSelector } from './ModeSelector';
import { TemplateSelector } from './TemplateSelector';
import { useCreateIdeateSession, useTemplates } from '@/hooks/useIdeate';
import type { IdeateMode } from '@/services/ideate';
import { Lightbulb, AlertCircle } from 'lucide-react';

interface CreatePRDFlowProps {
  projectId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onSessionCreated: (sessionId: string, mode: IdeateMode) => void;
}

type FlowStep = 'mode' | 'template' | 'description';

export function CreatePRDFlow({
  projectId,
  open,
  onOpenChange,
  onSessionCreated,
}: CreatePRDFlowProps) {
  const [step, setStep] = useState<FlowStep>('mode');
  const [selectedMode, setSelectedMode] = useState<IdeateMode | null>(null);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string | null>(null);
  const [description, setDescription] = useState('');

  const createSessionMutation = useCreateIdeateSession(projectId);
  const { isPending: loading, error } = createSessionMutation;
  const { data: templates = [], isLoading: templatesLoading } = useTemplates();

  const handleModeConfirm = () => {
    if (selectedMode) {
      setStep('template');
    }
  };

  const handleTemplateConfirm = () => {
    setStep('description');
  };

  const handleBack = () => {
    if (step === 'template') {
      setStep('mode');
    } else if (step === 'description') {
      setStep('template');
    }
  };

  const handleCreateSession = async () => {
    if (!selectedMode || !description.trim()) return;

    try {
      const session = await createSessionMutation.mutateAsync({
        projectId,
        initialDescription: description.trim(),
        mode: selectedMode,
        templateId: selectedTemplateId || undefined,
      });

      // Reset and close
      resetFlow();
      onOpenChange(false);
      onSessionCreated(session.id, selectedMode);
    } catch {
      // Error handled by React Query mutation
    }
  };

  const resetFlow = () => {
    setStep('mode');
    setSelectedMode(null);
    setSelectedTemplateId(null);
    setDescription('');
    createSessionMutation.reset();
  };

  const handleClose = () => {
    resetFlow();
    onOpenChange(false);
  };

  return (
    <Dialog open={open} onOpenChange={handleClose}>
      <DialogContent
        className="sm:max-w-[800px]"
        aria-describedby="create-prd-flow-description"
      >
        {step === 'mode' ? (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Lightbulb className="h-5 w-5" />
                Start Ideateing
              </DialogTitle>
              <DialogDescription id="create-prd-flow-description">
                Choose how you want to create your PRD. Each mode offers different levels of
                detail and AI assistance.
              </DialogDescription>
            </DialogHeader>

            <div className="py-4">
              <ModeSelector
                selectedMode={selectedMode}
                onSelectMode={setSelectedMode}
              />
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={handleClose}>
                Cancel
              </Button>
              <Button onClick={handleModeConfirm} disabled={!selectedMode}>
                Continue
              </Button>
            </DialogFooter>
          </>
        ) : step === 'template' ? (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Lightbulb className="h-5 w-5" />
                Choose a Template
              </DialogTitle>
              <DialogDescription id="create-prd-flow-description">
                Select a quickstart template to pre-populate your PRD, or start from scratch.
              </DialogDescription>
            </DialogHeader>

            <div className="py-4">
              <TemplateSelector
                templates={templates}
                selectedTemplateId={selectedTemplateId}
                onSelectTemplate={setSelectedTemplateId}
                isLoading={templatesLoading}
              />
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={handleBack}>
                Back
              </Button>
              <Button onClick={handleTemplateConfirm}>
                Continue
              </Button>
            </DialogFooter>
          </>
        ) : (
          <>
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Lightbulb className="h-5 w-5" />
                Describe Your Idea
              </DialogTitle>
              <DialogDescription id="create-prd-flow-description">
                Provide an initial description of your project idea. This will be used to
                generate or guide your PRD creation.
              </DialogDescription>
            </DialogHeader>

            <div className="py-4 space-y-4">
              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error.message}</AlertDescription>
                </Alert>
              )}

              <div className="space-y-2">
                <Label htmlFor="description">Project Description *</Label>
                <Textarea
                  id="description"
                  value={description}
                  onChange={(e) => setDescription(e.target.value)}
                  placeholder={
                    selectedMode === 'quick'
                      ? 'Example: A mobile app for tracking daily water intake with reminders and progress visualization'
                      : selectedMode === 'guided'
                      ? 'Provide a brief description of your project idea. You will expand on this in the following steps.'
                      : selectedMode === 'comprehensive'
                      ? 'Describe your project vision. We will explore it deeply through research and expert discussions.'
                      : 'Describe your initial project idea. We will discover requirements through conversation.'
                  }
                  rows={6}
                  className="resize-none"
                  required
                />
                <p className="text-xs text-muted-foreground">
                  {selectedMode === 'quick'
                    ? 'Be specific! The more detail you provide, the better the AI-generated PRD will be.'
                    : selectedMode === 'guided'
                    ? 'This will be your starting point. You can refine it as you work through each section.'
                    : selectedMode === 'comprehensive'
                    ? 'This description will be the foundation for comprehensive research and ideation.'
                    : 'This will start the conversation. You can elaborate and refine through back-and-forth dialogue.'}
                </p>
              </div>

              <Alert>
                <Lightbulb className="h-4 w-4" />
                <AlertDescription className="text-xs">
                  <strong>Selected Mode: {selectedMode?.charAt(0).toUpperCase()}{selectedMode?.slice(1)}</strong>
                  <br />
                  {selectedMode === 'quick' &&
                    'Your PRD will be generated automatically from this description.'}
                  {selectedMode === 'guided' &&
                    'You will be guided through each PRD section step-by-step.'}
                  {selectedMode === 'comprehensive' &&
                    'This will kick off in-depth research and expert roundtable discussions.'}
                  {selectedMode === 'conversational' &&
                    'Start a conversation to discover and refine your requirements naturally.'}
                </AlertDescription>
              </Alert>
            </div>

            <DialogFooter>
              <Button variant="outline" onClick={handleBack} disabled={loading}>
                Back
              </Button>
              <Button
                onClick={handleCreateSession}
                disabled={loading || !description.trim()}
              >
                {loading ? 'Creating Session...' : 'Start Ideateing'}
              </Button>
            </DialogFooter>
          </>
        )}
      </DialogContent>
    </Dialog>
  );
}
