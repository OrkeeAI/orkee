// ABOUTME: React hook for managing conversational mode state and operations
// ABOUTME: Handles message sending, history fetching, and quality metrics tracking

import { useState, useEffect, useCallback } from 'react';
import { conversationalService, ConversationMessage, QualityMetrics } from '@/services/conversational';

export interface UseConversationOptions {
  sessionId: string;
  autoLoadHistory?: boolean;
}

export function useConversation({ sessionId, autoLoadHistory = true }: UseConversationOptions) {
  const [messages, setMessages] = useState<ConversationMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetrics | null>(null);

  const loadHistory = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const history = await conversationalService.getHistory(sessionId);
      setMessages(history);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to load conversation history'));
    } finally {
      setIsLoading(false);
    }
  }, [sessionId]);

  const loadQualityMetrics = useCallback(async () => {
    try {
      const metrics = await conversationalService.getQualityMetrics(sessionId);
      setQualityMetrics(metrics);
    } catch (err) {
      console.error('Failed to load quality metrics:', err);
    }
  }, [sessionId]);

  const sendMessage = useCallback(
    async (content: string, messageType?: 'discovery' | 'refinement' | 'validation' | 'general') => {
      try {
        setIsSending(true);
        setError(null);

        const newMessage = await conversationalService.sendMessage(sessionId, {
          content,
          message_type: messageType,
        });

        setMessages((prev) => [...prev, newMessage]);

        await loadQualityMetrics();

        return newMessage;
      } catch (err) {
        const error = err instanceof Error ? err : new Error('Failed to send message');
        setError(error);
        throw error;
      } finally {
        setIsSending(false);
      }
    },
    [sessionId, loadQualityMetrics]
  );

  const refresh = useCallback(async () => {
    await Promise.all([loadHistory(), loadQualityMetrics()]);
  }, [loadHistory, loadQualityMetrics]);

  useEffect(() => {
    if (autoLoadHistory) {
      loadHistory();
      loadQualityMetrics();
    }
  }, [autoLoadHistory, loadHistory, loadQualityMetrics]);

  return {
    messages,
    qualityMetrics,
    isLoading,
    isSending,
    error,
    sendMessage,
    refresh,
    loadHistory,
    loadQualityMetrics,
  };
}
