// ABOUTME: Main orchestrator for Quick Mode PRD generation flow
// ABOUTME: Manages 4-step process: Input → Generating → Review/Edit → Save
import { useState, useEffect } from 'react';
import { ArrowLeft, Lightbulb } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { OneLineInput } from './OneLineInput';
import { GeneratingState } from './GeneratingState';
import { PRDEditor } from './PRDEditor';
import { SavePreview } from './SavePreview';
import { ModelSelectionDialog } from '@/components/ModelSelectionDialog';
import { useQuickExpand, useSaveAsPRD, useIdeateSession } from '@/hooks/useIdeate';
import { ideateService } from '@/services/ideate';
import type { GeneratedPRD } from '@/services/ideate';
import { toast } from 'sonner';

type FlowStep = 'input' | 'generating' | 'edit' | 'save';

interface QuickModeFlowProps {
  projectId: string;
  sessionId: string;
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: (prdId: string) => void;
}

export function QuickModeFlow({
  projectId,
  sessionId,
  open,
  onOpenChange,
  onComplete,
}: QuickModeFlowProps) {
  const [step, setStep] = useState<FlowStep>('input');
  const [description, setDescription] = useState('');
  const [generatedPRD, setGeneratedPRD] = useState<GeneratedPRD | null>(null);
  const [partialPRD, setPartialPRD] = useState<any>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isRegenerating, setIsRegenerating] = useState<Record<string, boolean>>({});
  const [showModelSelection, setShowModelSelection] = useState(false);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [selectedProvider, setSelectedProvider] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [selectedModel, setSelectedModel] = useState<string>('');

  const { data: session } = useIdeateSession(sessionId);
  const expandMutation = useQuickExpand(projectId, sessionId);
  const saveMutation = useSaveAsPRD(projectId, sessionId);

  // Initialize description from session
  useEffect(() => {
    if (session?.initial_description) {
      setDescription(session.initial_description);
    }
  }, [session]);

  const handleGenerate = () => {
    // Show model selection dialog instead of directly generating
    setShowModelSelection(true);
  };

  const handleModelConfirm = async (provider: string, model: string, templateId: string) => {
    setSelectedProvider(provider);
    setSelectedModel(model);

    try {
      setStep('generating');
      setIsGenerating(true);
      setPartialPRD(null);
      toast.info('Generating your PRD with streaming...', { duration: 2000 });

      // Call streaming version with real-time callback
      const result = await ideateService.quickGenerateStreaming(
        sessionId,
        (partial) => {
          console.log('[Stream Update]', partial);
          setPartialPRD(partial);
        },
        { provider, model, templateId }
      );

      // Final result received and saved to database
      setGeneratedPRD(result);
      setStep('edit');
      setIsGenerating(false);

      toast.success('PRD generated successfully!');
    } catch (error) {
      setStep('input');
      setIsGenerating(false);
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';

      // Provide specific error messages
      if (errorMessage.includes('API key')) {
        toast.error('Invalid API Key', {
          description: 'Please update your API key in Settings → AI Configuration.',
        });
      } else if (errorMessage.includes('token') || errorMessage.includes('limit')) {
        toast.error('Token Limit Exceeded', {
          description: 'PRD is too large. Try using fewer sections or a shorter description.',
        });
      } else if (errorMessage.includes('network') || errorMessage.includes('fetch')) {
        toast.error('Network Error', {
          description: 'Please check your connection and try again.',
        });
      } else if (errorMessage.includes('unavailable') || errorMessage.includes('service')) {
        toast.error('AI Service Unavailable', {
          description: 'The AI service is temporarily unavailable. Please try again later.',
        });
      } else {
        toast.error('Failed to generate PRD', {
          description: errorMessage,
        });
      }
    }
  };

  const handleRegenerateSection = async (sectionId: string) => {
    try {
      setIsRegenerating((prev) => ({ ...prev, [sectionId]: true }));
      toast.info(`Regenerating ${sectionId} section...`);

      const result = await expandMutation.mutateAsync({ sections: [sectionId] });

      setGeneratedPRD((prev) => {
        if (!prev) return result;
        return {
          ...prev,
          sections: {
            ...prev.sections,
            ...result.sections,
          },
          content: result.content, // Update full content too
        };
      });

      toast.success('Section regenerated successfully!');
    } catch (error) {
      toast.error('Failed to regenerate section', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setIsRegenerating((prev) => ({ ...prev, [sectionId]: false }));
    }
  };

  const handleSectionUpdate = (sectionId: string, content: string) => {
    setGeneratedPRD((prev) => {
      if (!prev) return null;
      return {
        ...prev,
        sections: {
          ...prev.sections,
          [sectionId]: content,
        },
      };
    });
    toast.success('Section updated');
  };

  const handleSaveClick = () => {
    setStep('save');
  };

  const handleConfirmSave = async (title: string) => {
    if (!generatedPRD) {
      toast.error('No PRD to save');
      return;
    }

    try {
      const result = await saveMutation.mutateAsync({
        title,
        contentMarkdown: generatedPRD.content,
      });

      toast.success('PRD saved successfully!', {
        description: 'Your PRD has been saved to the OpenSpec system.',
      });

      // Close dialog and notify completion
      onOpenChange(false);
      onComplete?.(result.prd_id);

      // Reset state
      resetFlow();
    } catch (error) {
      toast.error('Failed to save PRD', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    }
  };

  const handleBack = () => {
    if (step === 'edit') {
      setStep('input');
    } else if (step === 'save') {
      setStep('edit');
    }
  };

  const resetFlow = () => {
    setStep('input');
    setDescription('');
    setGeneratedPRD(null);
    setPartialPRD(null);
    setIsGenerating(false);
    setIsRegenerating({});
    setSelectedProvider('');
    setSelectedModel('');
  };

  const handleClose = () => {
    if (step !== 'generating') {
      onOpenChange(false);
      // Optionally reset on close
      // resetFlow();
    }
  };

  return (
    <>
      <Dialog open={open} onOpenChange={handleClose}>
        <DialogContent className="max-w-5xl max-h-[90vh] overflow-hidden flex flex-col">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Lightbulb className="h-5 w-5" />
              Quick Mode - Generate PRD
            </DialogTitle>
            <DialogDescription>
              {step === 'input' && 'Provide a description of your project to generate a complete PRD'}
              {step === 'generating' && 'Generating your PRD with AI...'}
              {step === 'edit' && 'Review and edit your generated PRD'}
              {step === 'save' && 'Preview and save your PRD'}
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 overflow-auto">
            {/* Step: Input */}
            {step === 'input' && (
              <OneLineInput
                value={description}
                onChange={setDescription}
                onGenerate={handleGenerate}
                isGenerating={isGenerating}
                error={undefined}
              />
            )}

            {/* Step: Generating */}
            {step === 'generating' && (
              <div className="space-y-4">
                <GeneratingState message="Generating your comprehensive PRD..." />
                {partialPRD && (
                  <div className="mt-4 p-4 border rounded-lg bg-muted/30">
                    <h3 className="text-sm font-medium mb-2">Streaming Progress:</h3>
                    <pre className="text-xs overflow-auto max-h-96 whitespace-pre-wrap">
                      {JSON.stringify(partialPRD, null, 2)}
                    </pre>
                  </div>
                )}
              </div>
            )}

            {/* Step: Edit */}
            {step === 'edit' && generatedPRD && (
              <div className="space-y-4">
                <PRDEditor
                  prdContent={generatedPRD.content}
                  sections={generatedPRD.sections}
                  onSectionUpdate={handleSectionUpdate}
                  onRegenerateSection={handleRegenerateSection}
                  onSave={handleSaveClick}
                  isRegenerating={isRegenerating}
                />
              </div>
            )}
          </div>

          {/* Footer with Back button for edit step */}
          {step === 'edit' && (
            <div className="flex justify-between pt-4 border-t">
              <Button variant="outline" onClick={handleBack} className="gap-2">
                <ArrowLeft className="h-4 w-4" />
                Back to Edit
              </Button>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Save Preview Dialog (separate from main dialog) */}
      {generatedPRD && (
        <SavePreview
          open={step === 'save'}
          onOpenChange={(open) => {
            if (!open) {
              setStep('edit');
            }
          }}
          prdContent={generatedPRD.content}
          projectName={session?.initial_description.slice(0, 50) || 'My PRD'}
          onConfirmSave={handleConfirmSave}
          isSaving={saveMutation.isPending}
        />
      )}

      {/* Model Selection Dialog */}
      <ModelSelectionDialog
        open={showModelSelection}
        onOpenChange={setShowModelSelection}
        onConfirm={handleModelConfirm}
        defaultProvider="anthropic"
        defaultModel="claude-3-5-sonnet-20241022"
      />
    </>
  );
}
