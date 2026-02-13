// ABOUTME: Chat-first PRD creation flow with readiness sidebar
// ABOUTME: Replaces separate Quick/Guided/Chat modes with one unified chat experience

import { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Loader2, FileText, AlertCircle, ArrowLeft, Circle, CheckCircle2 } from 'lucide-react';
import { ChatView } from '@/components/ideate/ChatMode/components/ChatView';
import { useChat } from '@/components/ideate/ChatMode/hooks/useChat';
import { useStreamingResponse } from '@/components/ideate/ChatMode/hooks/useStreamingResponse';
import { chatService } from '@/services/chat';
import { generatePRDFromChat, extractInsights } from '@/services/chat-ai';
import { useCurrentUser } from '@/hooks/useUsers';
import { useModelPreferences, getModelForTask } from '@/services/model-preferences';
import { useSaveAsPRD } from '@/hooks/useIdeate';
import { SavePreview } from './SavePreview';

export interface PRDChatFlowProps {
  projectId: string;
  sessionId: string;
  onClose: () => void;
  onPRDSaved: (prdId: string) => void;
}

export function PRDChatFlow({
  projectId,
  sessionId,
  onClose,
  onPRDSaved,
}: PRDChatFlowProps) {
  const [isGeneratingPRD, setIsGeneratingPRD] = useState(false);
  const [generatedContent, setGeneratedContent] = useState<string | null>(null);
  const [prdError, setPrdError] = useState<Error | null>(null);
  const [showSavePreview, setShowSavePreview] = useState(false);

  const { data: currentUser } = useCurrentUser();
  const { data: modelPreferences } = useModelPreferences(currentUser?.id);
  const saveAsPRD = useSaveAsPRD(projectId, sessionId);

  const {
    messages,
    qualityMetrics,
    isLoading,
    isSending,
    error: chatError,
    sendMessage,
    refresh,
  } = useChat({
    sessionId,
    autoLoadHistory: true,
  });

  const { streamingMessage, isStreaming, startStreaming, stopStreaming } = useStreamingResponse({
    sessionId,
    chatHistory: messages,
    projectId,
    onMessageComplete: async (content: string) => {
      try {
        await chatService.sendMessage(sessionId, {
          content,
          message_type: 'discovery',
          role: 'assistant',
        });

        try {
          const insightPreferences = getModelForTask(modelPreferences, 'insight_extraction');
          await extractInsights(sessionId, messages, insightPreferences, projectId);
        } catch (insightError) {
          console.warn('[PRDChatFlow] Failed to extract insights:', insightError);
        }
      } catch (err) {
        console.error('Failed to save assistant message:', err);
      } finally {
        stopStreaming();
        await refresh();
      }
    },
    onError: (error) => {
      console.error('Streaming error:', error);
    },
  });

  const handleSendMessage = useCallback(
    async (content: string, model?: string, provider?: string) => {
      try {
        await sendMessage(content, 'discovery', model);
        await startStreaming(content, provider, model);
      } catch (err) {
        console.error('Failed to send message:', err);
      }
    },
    [sendMessage, startStreaming]
  );

  const handleGeneratePRD = useCallback(async () => {
    try {
      setIsGeneratingPRD(true);
      setPrdError(null);

      const prdPreferences = getModelForTask(modelPreferences, 'prd_generation');
      const result = await generatePRDFromChat(
        sessionId,
        `PRD - ${new Date().toLocaleDateString()}`,
        messages,
        [],
        prdPreferences,
        projectId
      );

      setGeneratedContent(result.prd_markdown);
      setShowSavePreview(true);
    } catch (err) {
      setPrdError(err instanceof Error ? err : new Error('Failed to generate PRD'));
    } finally {
      setIsGeneratingPRD(false);
    }
  }, [sessionId, messages, modelPreferences, projectId]);

  const handleSavePRD = useCallback(async (title: string) => {
    if (!generatedContent) return;

    try {
      const result = await saveAsPRD.mutateAsync({
        title,
        contentMarkdown: generatedContent,
      });
      setShowSavePreview(false);
      onPRDSaved(result.id);
    } catch (err) {
      console.error('Failed to save PRD:', err);
    }
  }, [generatedContent, saveAsPRD, onPRDSaved]);

  const readinessItems = [
    { label: 'Problem defined', covered: qualityMetrics?.coverage?.problem ?? false },
    { label: 'Target users identified', covered: qualityMetrics?.coverage?.users ?? false },
    { label: 'Core features discussed', covered: qualityMetrics?.coverage?.features ?? false },
    { label: 'Technical approach', covered: qualityMetrics?.coverage?.technical ?? false },
  ];

  return (
    <div className="fixed inset-0 z-50 bg-background">
      <div className="h-full flex flex-col">
        {/* Header */}
        <div className="border-b px-6 py-3 flex items-center justify-between flex-shrink-0">
          <Button variant="ghost" size="sm" onClick={onClose}>
            <ArrowLeft className="mr-2 h-4 w-4" />
            Back to PRDs
          </Button>

          <Button
            onClick={handleGeneratePRD}
            disabled={isGeneratingPRD || isSending || isStreaming}
            variant={qualityMetrics?.is_ready_for_prd ? 'default' : 'outline'}
            size="sm"
            className="gap-2"
          >
            {isGeneratingPRD ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Generating...
              </>
            ) : (
              <>
                <FileText className="h-4 w-4" />
                Generate PRD
              </>
            )}
          </Button>
        </div>

        {/* Main content */}
        <div className="flex-1 grid grid-cols-12 gap-4 p-4 min-h-0 overflow-hidden">
          {/* Chat area */}
          <div className="col-span-8 flex flex-col border rounded-lg bg-card min-h-0">
            {/* Error alerts */}
            {(chatError || prdError) && (
              <div className="p-4 space-y-2 flex-shrink-0">
                {chatError && (
                  <Alert variant="destructive">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>{chatError.message}</AlertDescription>
                  </Alert>
                )}
                {prdError && (
                  <Alert variant="destructive">
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>{prdError.message}</AlertDescription>
                  </Alert>
                )}
              </div>
            )}

            <div className="flex-1 overflow-hidden min-h-0">
              <ChatView
                messages={messages}
                streamingMessage={streamingMessage}
                onSendMessage={handleSendMessage}
                isLoading={isLoading}
                isSending={isSending || isStreaming}
              />
            </div>
          </div>

          {/* Readiness sidebar */}
          <div className="col-span-4 flex flex-col gap-4 min-h-0 overflow-y-auto">
            <Card>
              <CardHeader className="pb-3">
                <CardTitle className="text-sm font-medium">PRD Readiness</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {readinessItems.map((item) => (
                    <div key={item.label} className="flex items-center gap-2 text-sm">
                      {item.covered ? (
                        <CheckCircle2 className="h-4 w-4 text-green-500 flex-shrink-0" />
                      ) : (
                        <Circle className="h-4 w-4 text-muted-foreground flex-shrink-0" />
                      )}
                      <span className={item.covered ? 'text-foreground' : 'text-muted-foreground'}>
                        {item.label}
                      </span>
                    </div>
                  ))}
                </div>

                {qualityMetrics && (
                  <div className="mt-4 pt-3 border-t">
                    <div className="text-xs text-muted-foreground">
                      Quality score: {qualityMetrics.quality_score}%
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </div>
        </div>
      </div>

      {/* Save preview dialog */}
      {generatedContent && (
        <SavePreview
          open={showSavePreview}
          onOpenChange={setShowSavePreview}
          prdContent={generatedContent}
          onConfirmSave={handleSavePRD}
          isSaving={saveAsPRD.isPending}
        />
      )}
    </div>
  );
}
