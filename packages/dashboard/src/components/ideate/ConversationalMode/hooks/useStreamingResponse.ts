// ABOUTME: React hook for handling AI streaming responses
// ABOUTME: Manages real-time AI assistant responses in conversational mode using AI SDK

import { useState, useCallback, useRef, useEffect } from 'react';
import { streamConversationalResponse } from '@/services/conversational-ai';
import type { ConversationMessage } from '@/services/conversational';
import { STREAMING_CONFIG, ERROR_MESSAGES } from '../constants';

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
  const abortControllerRef = useRef<AbortController | null>(null);
  const timeoutIdRef = useRef<NodeJS.Timeout | null>(null);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }
      if (timeoutIdRef.current) {
        clearTimeout(timeoutIdRef.current);
      }
    };
  }, []);

  const cleanup = useCallback(() => {
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }
    if (timeoutIdRef.current) {
      clearTimeout(timeoutIdRef.current);
      timeoutIdRef.current = null;
    }
  }, []);

  const handleError = useCallback(
    (error: Error) => {
      cleanup();
      setIsStreaming(false);
      setStreamingMessage(null);

      // Categorize error for better user feedback
      let userError: Error;
      if (error.name === 'AbortError') {
        userError = new Error(ERROR_MESSAGES.STREAMING_ABORTED);
      } else if (error.message?.includes('timeout')) {
        userError = new Error(ERROR_MESSAGES.STREAMING_TIMEOUT);
      } else if (error.message?.includes('network') || error.message?.includes('fetch')) {
        userError = new Error(ERROR_MESSAGES.NETWORK_ERROR);
      } else {
        userError = new Error(ERROR_MESSAGES.GENERIC_ERROR);
      }

      if (onError) {
        onError(userError);
      }
    },
    [cleanup, onError]
  );

  const startStreaming = useCallback(
    async (userMessage: string) => {
      // Clean up any existing stream
      cleanup();

      // Create new AbortController for this request
      abortControllerRef.current = new AbortController();

      setIsStreaming(true);
      setStreamingMessage({
        id: `streaming-${Date.now()}`,
        role: 'assistant',
        content: '',
        isComplete: false,
      });

      // Set timeout
      timeoutIdRef.current = setTimeout(() => {
        if (abortControllerRef.current) {
          abortControllerRef.current.abort();
          handleError(new Error('timeout'));
        }
      }, STREAMING_CONFIG.TIMEOUT_MS);

      try {
        await streamConversationalResponse(
          sessionId,
          userMessage,
          conversationHistory,
          // onChunk
          (chunk: string) => {
            // Check if aborted
            if (abortControllerRef.current?.signal.aborted) {
              throw new Error('AbortError');
            }

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
            cleanup();

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
            handleError(error);
          },
          // abortSignal
          abortControllerRef.current?.signal
        );
      } catch (error) {
        const err = error instanceof Error ? error : new Error(ERROR_MESSAGES.GENERIC_ERROR);
        handleError(err);
      }
    },
    [sessionId, conversationHistory, onMessageComplete, cleanup, handleError]
  );

  const stopStreaming = useCallback(() => {
    cleanup();
    setIsStreaming(false);
    setStreamingMessage(null);
  }, [cleanup]);

  return {
    streamingMessage,
    isStreaming,
    startStreaming,
    stopStreaming,
  };
}
