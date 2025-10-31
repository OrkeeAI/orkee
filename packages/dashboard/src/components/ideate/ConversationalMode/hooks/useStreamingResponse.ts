// ABOUTME: React hook for handling AI streaming responses
// ABOUTME: Manages real-time AI assistant responses in conversational mode using AI SDK

import { useState, useCallback } from 'react';
import { streamConversationalResponse } from '@/services/conversational-ai';
import type { ConversationMessage } from '@/services/conversational';

export interface StreamingMessage {
  id: string;
  role: 'assistant';
  content: string;
  isComplete: boolean;
}

export interface UseStreamingResponseOptions {
  sessionId: string;
  conversationHistory: ConversationMessage[];
  onMessageComplete?: (content: string) => void;
  onError?: (error: Error) => void;
}

export function useStreamingResponse({
  sessionId,
  conversationHistory,
  onMessageComplete,
  onError,
}: UseStreamingResponseOptions) {
  const [streamingMessage, setStreamingMessage] = useState<StreamingMessage | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);

  const startStreaming = useCallback(
    async (userMessage: string) => {
      setIsStreaming(true);
      setStreamingMessage({
        id: `streaming-${Date.now()}`,
        role: 'assistant',
        content: '',
        isComplete: false,
      });

      await streamConversationalResponse(
        sessionId,
        userMessage,
        conversationHistory,
        // onChunk
        (chunk: string) => {
          setStreamingMessage((prev) => {
            if (!prev) return null;
            return {
              ...prev,
              content: prev.content + chunk,
            };
          });
        },
        // onComplete
        (fullText: string) => {
          setStreamingMessage((prev) => {
            if (!prev) return null;
            return {
              ...prev,
              content: fullText,
              isComplete: true,
            };
          });
          setIsStreaming(false);
          if (onMessageComplete) {
            onMessageComplete(fullText);
          }
        },
        // onError
        (error: Error) => {
          setIsStreaming(false);
          setStreamingMessage(null);
          if (onError) {
            onError(error);
          }
        }
      );
    },
    [sessionId, conversationHistory, onMessageComplete, onError]
  );

  const stopStreaming = useCallback(() => {
    setIsStreaming(false);
    setStreamingMessage(null);
  }, []);

  return {
    streamingMessage,
    isStreaming,
    startStreaming,
    stopStreaming,
  };
}
