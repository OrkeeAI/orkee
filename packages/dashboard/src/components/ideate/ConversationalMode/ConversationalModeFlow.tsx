// ABOUTME: Main container for conversational PRD discovery mode
// ABOUTME: Orchestrates conversation, insights, quality tracking, and PRD generation

import React, { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, FileText, AlertCircle } from 'lucide-react';
import { ConversationView } from './components/ConversationView';
import { QualityIndicator } from './components/QualityIndicator';
import { InsightsSidebar } from './components/InsightsSidebar';
import { useConversation } from './hooks/useConversation';
import { useDiscoveryQuestions } from './hooks/useDiscoveryQuestions';
import { useStreamingResponse } from './hooks/useStreamingResponse';
import { conversationalService, ConversationInsight } from '@/services/conversational';

export interface ConversationalModeFlowProps {
  sessionId: string;
  projectId: string;
  onPRDGenerated: (prdId: string) => void;
}

export function ConversationalModeFlow({
  sessionId,
  // projectId,
  onPRDGenerated,
}: ConversationalModeFlowProps) {
  const [insights, setInsights] = useState<ConversationInsight[]>([]);
  const [isGeneratingPRD, setIsGeneratingPRD] = useState(false);
  const [prdError, setPrdError] = useState<Error | null>(null);

  const {
    messages,
    qualityMetrics,
    isLoading,
    isSending,
    error: conversationError,
    sendMessage,
    refresh,
  } = useConversation({
    sessionId,
    autoLoadHistory: true,
  });

  const { suggestedQuestions } = useDiscoveryQuestions({
    sessionId,
    autoLoad: true,
  });

  const { streamingMessage, isStreaming, startStreaming, stopStreaming } = useStreamingResponse({
    sessionId,
    conversationHistory: messages,
    onMessageComplete: async (content: string) => {
      // Save assistant message to backend
      try {
        await conversationalService.sendMessage(sessionId, {
          content,
          message_type: 'discovery',
          role: 'assistant',
        });
        await refresh();
        await loadInsights();
        // Clear streaming message after saving to prevent duplication
        stopStreaming();
      } catch (err) {
        console.error('Failed to save assistant message:', err);
      }
    },
    onError: (error) => {
      console.error('Streaming error:', error);
    },
  });

  const loadInsights = useCallback(async () => {
    try {
      const data = await conversationalService.getInsights(sessionId);
      setInsights(data);
    } catch (err) {
      console.error('Failed to load insights:', err);
    }
  }, [sessionId]);

  React.useEffect(() => {
    loadInsights();
  }, [loadInsights]);

  const handleSendMessage = useCallback(
    async (content: string) => {
      try {
        // Save user message to backend
        await sendMessage(content, 'discovery');
        // Start AI streaming response
        await startStreaming(content);
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

      const validation = await conversationalService.validateForPRD(sessionId);

      if (!validation.is_valid) {
        setPrdError(
          new Error(
            `Cannot generate PRD yet. Missing: ${validation.missing_required.join(', ')}`
          )
        );
        return;
      }

      const result = await conversationalService.generatePRD(sessionId, {
        title: `PRD from Conversation - ${new Date().toLocaleDateString()}`,
      });

      onPRDGenerated(result.prd_id);
    } catch (err) {
      setPrdError(err instanceof Error ? err : new Error('Failed to generate PRD'));
    } finally {
      setIsGeneratingPRD(false);
    }
  }, [sessionId, onPRDGenerated]);

  return (
    <div className="h-full flex flex-col">
      <div className="flex-1 grid grid-cols-12 gap-4 overflow-hidden">
        <div className="col-span-8 flex flex-col border rounded-lg bg-card">
          <div className="border-b p-4">
            <h2 className="text-lg font-semibold">Conversation</h2>
            <p className="text-sm text-muted-foreground">
              Discuss your project idea and I'll help discover the requirements
            </p>
          </div>

          <div className="flex-1 overflow-hidden">
            <ConversationView
              messages={messages}
              streamingMessage={streamingMessage}
              suggestedQuestions={suggestedQuestions}
              onSendMessage={handleSendMessage}
              isLoading={isLoading}
              isSending={isSending || isStreaming}
            />
          </div>
        </div>

        <div className="col-span-4 flex flex-col gap-4 overflow-hidden">
          <QualityIndicator metrics={qualityMetrics} />

          <InsightsSidebar insights={insights} className="flex-1 overflow-hidden" />
        </div>
      </div>

      <div className="mt-4 p-4 border-t bg-background space-y-3">
        {conversationError && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{conversationError.message}</AlertDescription>
          </Alert>
        )}

        {prdError && (
          <Alert variant="destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>{prdError.message}</AlertDescription>
          </Alert>
        )}

        <div className="flex items-center justify-between">
          <div className="text-sm text-muted-foreground">
            {qualityMetrics?.is_ready_for_prd ? (
              <span className="text-green-600 dark:text-green-400 font-medium">
                ✓ Ready to generate your PRD
              </span>
            ) : (
              <span>Keep exploring to improve PRD quality</span>
            )}
          </div>

          <Button
            onClick={handleGeneratePRD}
            disabled={
              !qualityMetrics?.is_ready_for_prd || isGeneratingPRD || isSending || isStreaming
            }
            size="lg"
            className="gap-2"
          >
            {isGeneratingPRD ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Generating PRD...
              </>
            ) : (
              <>
                <FileText className="h-4 w-4" />
                Generate PRD
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}
