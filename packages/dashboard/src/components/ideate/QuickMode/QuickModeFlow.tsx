// ABOUTME: Main orchestrator for Quick Mode PRD generation flow
// ABOUTME: Manages 4-step process: Input → Generating → Review/Edit → Save
import { useState, useEffect, useRef } from 'react';
import { ArrowLeft, Lightbulb } from 'lucide-react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { OneLineInput } from './OneLineInput';
import { GenerationStatus } from './GenerationStatus';
import { PRDEditor } from './PRDEditor';
import { SavePreview } from './SavePreview';
import { ModelSelectionDialog } from '@/components/ModelSelectionDialog';
import { RegenerateTemplateDialog } from '../PRDGenerator/RegenerateTemplateDialog';
import { useQuickExpand, useSaveAsPRD, useIdeateSession, useUpdateIdeateSession } from '@/hooks/useIdeate';
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
  skipToGenerate?: boolean;
}

export function QuickModeFlow({
  projectId,
  sessionId,
  open,
  onOpenChange,
  onComplete,
  skipToGenerate = false,
}: QuickModeFlowProps) {
  const [step, setStep] = useState<FlowStep>('input');
  const [description, setDescription] = useState('');
  const [generatedPRD, setGeneratedPRD] = useState<GeneratedPRD | null>(null);
  const [partialPRD, setPartialPRD] = useState<Record<string, unknown> | null>(null);
  const [isGenerating, setIsGenerating] = useState(false);
  const [isRegenerating, setIsRegenerating] = useState<Record<string, boolean>>({});
  const [showModelSelection, setShowModelSelection] = useState(skipToGenerate);
  const [showRegenerateDialog, setShowRegenerateDialog] = useState(false);
  const [isSavingDraft, setIsSavingDraft] = useState(false);
  const justSavedDraftRef = useRef(false);
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [selectedProvider, setSelectedProvider] = useState<string>('');
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const [selectedModel, setSelectedModel] = useState<string>('');

  const { data: session } = useIdeateSession(sessionId);
  const expandMutation = useQuickExpand(projectId, sessionId);
  const saveMutation = useSaveAsPRD(projectId, sessionId);
  const updateSessionMutation = useUpdateIdeateSession(projectId, sessionId);

  // Initialize from session: load description and check for existing PRD
  useEffect(() => {
    if (session?.initial_description) {
      setDescription(session.initial_description);
    }

    // Skip PRD loading if we just saved a draft
    if (justSavedDraftRef.current) {
      console.log('[QuickModeFlow] Just saved draft, skipping PRD load');
      return;
    }

    // Try to load existing PRD (for resumed sessions)
    const loadPRD = async () => {
      try {
        console.log('[QuickModeFlow] Attempting to load PRD for session:', sessionId);
        const prd = await ideateService.previewPRD(sessionId);
        console.log('[QuickModeFlow] PRD loaded:', { 
          hasPRD: !!prd, 
          hasContent: !!prd?.content, 
          hasSections: !!prd?.sections,
          sectionKeys: prd?.sections ? Object.keys(prd.sections) : [],
          sectionCount: prd?.sections ? Object.keys(prd.sections).length : 0,
          fullPRD: prd
        });
        
        // Check if we have either content or sections (Quick Mode stores sections)
        if (prd && (prd.content || (prd.sections && Object.keys(prd.sections).length > 0))) {
          console.log('[QuickModeFlow] Found existing PRD data, jumping to edit step');
          
          // If we have sections but no content, generate content from sections
          if (!prd.content && prd.sections) {
            console.log('[QuickModeFlow] Generating content from sections');
            prd.content = Object.entries(prd.sections)
              .map(([section, data]) => `## ${section}\n\n${typeof data === 'string' ? data : JSON.stringify(data, null, 2)}\n\n`)
              .join('');
          }
          
          setGeneratedPRD(prd);
          setStep('edit');
        } else {
          console.log('[QuickModeFlow] PRD incomplete - missing content and sections. Staying on input step.');
        }
      } catch (error) {
        // No existing PRD, stay on input step
        const errorMsg = error instanceof Error ? error.message : String(error);
        console.error('[QuickModeFlow] Error loading PRD (will stay on input):', errorMsg);
        console.log('[QuickModeFlow] Session status:', session?.status);
      }
    };

    // Only try to load if we have a session and the description is already set
    if (session?.initial_description) {
      console.log('[QuickModeFlow] Session loaded with description, attempting to load PRD');
      loadPRD();
    }
  }, [session, sessionId, skipToGenerate]);

  const handleGenerate = () => {
    // Show model selection dialog instead of directly generating
    setShowModelSelection(true);
  };

  const handleSaveDraft = async () => {
    if (!description.trim()) return;
    
    try {
      setIsSavingDraft(true);
      justSavedDraftRef.current = true;
      await updateSessionMutation.mutateAsync({
        initialDescription: description.trim(),
      });
      toast.success('Draft saved successfully!');
    } catch (error) {
      toast.error('Failed to save draft', {
        description: error instanceof Error ? error.message : 'Unknown error',
      });
    } finally {
      setIsSavingDraft(false);
      justSavedDraftRef.current = false;
    }
  };

  const handleModelConfirm = async (provider: string, model: string) => {
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
        { provider, model }
      );

      // Final result received and saved to database
      setGeneratedPRD(result);
      setStep('edit');
      setIsGenerating(false);

      // Update session status to 'completed' after successful PRD generation
      await updateSessionMutation.mutateAsync({
        status: 'completed'
      });

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

  const handleGeneratePRD = () => {
    setShowRegenerateDialog(true);
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

  const handleCancel = () => {
    onOpenChange(false);
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
              Quick Mode - Generate PRD Details
            </DialogTitle>
            <DialogDescription>
              {step === 'input' && 'Provide a description of your project to generate detailed PRD sections'}
              {step === 'generating' && 'Generating your PRD details with AI...'}
              {step === 'edit' && 'Review and edit your generated PRD details'}
              {step === 'save' && 'Preview and save your PRD'}
            </DialogDescription>
          </DialogHeader>

          <div className="flex-1 overflow-auto">
            {/* Step: Input */}
            {step === 'input' && !showModelSelection && (
              <OneLineInput
                value={description}
                onChange={setDescription}
                onGenerate={handleGenerate}
                onSaveDraft={handleSaveDraft}
                onCancel={handleCancel}
                isGenerating={isGenerating}
                isSavingDraft={isSavingDraft}
                error={undefined}
              />
            )}

            {/* Step: Generating */}
            {step === 'generating' && (
              <GenerationStatus 
                partialPRD={partialPRD}
                message="Generating your comprehensive PRD details..."
              />
            )}

            {/* Step: Edit */}
            {step === 'edit' && generatedPRD && (
              <div className="space-y-4">
                <PRDEditor
                  prdContent={generatedPRD.content}
                  sections={generatedPRD.sections}
                  onSectionUpdate={handleSectionUpdate}
                  onRegenerateSection={handleRegenerateSection}
                  onGeneratePRD={handleGeneratePRD}
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

      {/* Regenerate Template Dialog */}
      {showRegenerateDialog && (
        <RegenerateTemplateDialog
          sessionId={sessionId}
          onClose={() => setShowRegenerateDialog(false)}
          onSuccess={() => {
            toast.success('PRD regenerated with new template!');
          }}
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
