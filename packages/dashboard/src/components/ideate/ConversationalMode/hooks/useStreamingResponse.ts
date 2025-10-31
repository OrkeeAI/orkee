// ABOUTME: React hook for handling Server-Sent Events (SSE) streaming responses
// ABOUTME: Manages real-time AI assistant responses in conversational mode

import { useState, useEffect, useCallback, useRef } from 'react';
import { conversationalService, ConversationMessage } from '@/services/conversational';

export interface StreamingMessage {
  id: string;
  role: 'assistant';
  content: string;
  isComplete: boolean;
}

export interface UseStreamingResponseOptions {
  sessionId: string;
  onMessageComplete?: (message: ConversationMessage) => void;
  onError?: (error: Error) => void;
}

export function useStreamingResponse({
  sessionId,
  onMessageComplete,
  onError,
}: UseStreamingResponseOptions) {
  const [streamingMessage, setStreamingMessage] = useState<StreamingMessage | null>(null);
  const [isStreaming, setIsStreaming] = useState(false);
  const eventSourceRef = useRef<EventSource | null>(null);

  const closeConnection = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }
    setIsStreaming(false);
  }, []);

  const startStreaming = useCallback(() => {
    closeConnection();

    try {
      setIsStreaming(true);
      setStreamingMessage({
        id: `streaming-${Date.now()}`,
        role: 'assistant',
        content: '',
        isComplete: false,
      });

      const streamUrl = conversationalService.getStreamUrl(sessionId);
      const eventSource = new EventSource(streamUrl);
      eventSourceRef.current = eventSource;

      eventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);

          if (data.type === 'chunk') {
            setStreamingMessage((prev) => {
              if (!prev) return null;
              return {
                ...prev,
                content: prev.content + data.content,
              };
            });
          } else if (data.type === 'complete') {
            setStreamingMessage((prev) => {
              if (!prev) return null;
              return {
                ...prev,
                id: data.message_id,
                content: data.content,
                isComplete: true,
              };
            });

            if (onMessageComplete && data.message) {
              onMessageComplete(data.message as ConversationMessage);
            }

            closeConnection();
          } else if (data.type === 'error') {
            const error = new Error(data.message || 'Streaming error');
            if (onError) {
              onError(error);
            }
            closeConnection();
          }
        } catch (err) {
          console.error('Failed to parse SSE data:', err);
        }
      };

      eventSource.onerror = (err) => {
        console.error('SSE connection error:', err);
        const error = new Error('Connection to server lost');
        if (onError) {
          onError(error);
        }
        closeConnection();
      };
    } catch (err) {
      const error = err instanceof Error ? err : new Error('Failed to start streaming');
      if (onError) {
        onError(error);
      }
      closeConnection();
    }
  }, [sessionId, onMessageComplete, onError, closeConnection]);

  useEffect(() => {
    return () => {
      closeConnection();
    };
  }, [closeConnection]);

  return {
    streamingMessage,
    isStreaming,
    startStreaming,
    stopStreaming: closeConnection,
  };
}
