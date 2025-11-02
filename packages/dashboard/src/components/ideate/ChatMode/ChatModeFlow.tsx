// ABOUTME: Main container for chat PRD discovery mode
// ABOUTME: Orchestrates chat, insights, quality tracking, and PRD generation

import React, { useState, useCallback } from 'react';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Loader2, FileText, AlertCircle } from 'lucide-react';
import { ChatView } from './components/ChatView';
import { QualityIndicator } from './components/QualityIndicator';
import { InsightsSidebar } from './components/InsightsSidebar';
import { useChat } from './hooks/useChat';
import { useDiscoveryQuestions } from './hooks/useDiscoveryQuestions';
import { useStreamingResponse } from './hooks/useStreamingResponse';
import { chatService, ChatInsight } from '@/services/chat';
import { UI_TEXT } from './constants';

export interface ChatModeFlowProps {
  sessionId: string;
  projectId: string;
  onPRDGenerated: (prdId: string) => void;
}

export function ChatModeFlow({
  sessionId,
  // projectId,
  onPRDGenerated,
}: ChatModeFlowProps) {
  const [insights, setInsights] = useState<ChatInsight[]>([]);
  const [isGeneratingPRD, setIsGeneratingPRD] = useState(false);
  const [prdError, setPrdError] = useState<Error | null>(null);

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

  const { suggestedQuestions } = useDiscoveryQuestions({
    sessionId,
    autoLoad: true,
  });

  const { streamingMessage, isStreaming, startStreaming, stopStreaming } = useStreamingResponse({
    sessionId,
    chatHistory: messages,
    onMessageComplete: async (content: string) => {
      try {
        await chatService.sendMessage(sessionId, {
          content,
          message_type: 'discovery',
          role: 'assistant',
        });
      } catch (err) {
        console.error('Failed to save assistant message:', err);
      } finally {
        // Always clear streaming state to prevent race conditions with new messages
        stopStreaming();
        await refresh();
        await loadInsights();
      }
    },
    onError: (error) => {
      console.error('Streaming error:', error);
    },
  });

  const loadInsights = useCallback(async () => {
    try {
      const data = await chatService.getInsights(sessionId);
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

      const validation = await chatService.validateForPRD(sessionId);

      if (!validation.is_valid) {
        setPrdError(
          new Error(
            `Cannot generate PRD yet. Missing: ${validation.missing_required.join(', ')}`
          )
        );
        return;
      }

      const result = await chatService.generatePRD(sessionId, {
        title: `PRD from Chat - ${new Date().toLocaleDateString()}`,
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
            <h2 className="text-lg font-semibold">Chat</h2>
            <p className="text-sm text-muted-foreground">
              Discuss your project idea and I'll help discover the requirements
            </p>
          </div>

          <div className="flex-1 overflow-hidden">
            <ChatView
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

        <div className="flex items-center justify-between">
          <div className="text-sm text-muted-foreground">
            {qualityMetrics?.is_ready_for_prd ? (
              <span className="text-green-600 dark:text-green-400 font-medium">
                {UI_TEXT.READY_FOR_PRD}
              </span>
            ) : (
              <span>{UI_TEXT.KEEP_EXPLORING}</span>
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
                {UI_TEXT.GENERATING_PRD}
              </>
            ) : (
              <>
                <FileText className="h-4 w-4" />
                {UI_TEXT.GENERATE_PRD}
              </>
            )}
          </Button>
        </div>
      </div>
    </div>
  );
}
