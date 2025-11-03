// ABOUTME: React hook for managing chat mode state and operations
// ABOUTME: Handles message sending, history fetching, and quality metrics tracking

import { useState, useEffect, useCallback } from 'react';
import { chatService, ChatMessage, QualityMetrics } from '@/services/chat';

export interface UseChatOptions {
  sessionId: string;
  autoLoadHistory?: boolean;
}

export function useChat({ sessionId, autoLoadHistory = true }: UseChatOptions) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSending, setIsSending] = useState(false);
  const [error, setError] = useState<Error | null>(null);
  const [qualityMetrics, setQualityMetrics] = useState<QualityMetrics | null>(null);

  const loadHistory = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const history = await chatService.getHistory(sessionId);
      setMessages(history);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to load chat history'));
    } finally {
      setIsLoading(false);
    }
  }, [sessionId]);

  const loadQualityMetrics = useCallback(async () => {
    try {
      const metrics = await chatService.getQualityMetrics(sessionId);
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

        const newMessage = await chatService.sendMessage(sessionId, {
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
