// ABOUTME: Streaming utilities for AI operations with real-time updates
// ABOUTME: Provides hooks and utilities for streaming AI responses to the UI

import { useState, useCallback } from 'react';
import { streamObject, streamText } from 'ai';
import { getPreferredModel } from './providers';
import { AI_CONFIG, calculateCost } from './config';
import type { z } from 'zod';

/**
 * Stream status for tracking progress
 */
export interface StreamStatus {
  isStreaming: boolean;
  progress: number; // 0-100
  message: string;
  error?: string;
}

/**
 * Hook for streaming object generation with progress updates
 */
export function useStreamObject<T>() {
  const [status, setStatus] = useState<StreamStatus>({
    isStreaming: false,
    progress: 0,
    message: '',
  });
  const [result, setResult] = useState<T | null>(null);

  const stream = useCallback(async <Schema extends z.ZodType<T>>(
    schema: Schema,
    prompt: string,
    onProgress?: (partial: Partial<T>) => void
  ) => {
    setStatus({ isStreaming: true, progress: 10, message: 'Initializing...' });
    setResult(null);

    try {
      const { provider, model } = getPreferredModel();

      setStatus({ isStreaming: true, progress: 20, message: 'Connecting to AI...' });

      const { partialObjectStream, object } = await streamObject({
        model,
        schema,
        prompt,
        temperature: AI_CONFIG.defaults.temperature,
        maxTokens: AI_CONFIG.defaults.maxTokens,
      });

      setStatus({ isStreaming: true, progress: 30, message: 'Generating response...' });

      // Stream partial updates
      for await (const partialObject of partialObjectStream) {
        if (onProgress) {
          onProgress(partialObject as Partial<T>);
        }
        setStatus({
          isStreaming: true,
          progress: Math.min(30 + Math.random() * 60, 90),
          message: 'Processing...',
        });
      }

      setStatus({ isStreaming: true, progress: 95, message: 'Finalizing...' });

      const finalResult = await object;

      setResult(finalResult as T);
      setStatus({
        isStreaming: false,
        progress: 100,
        message: 'Complete!',
      });

      return finalResult as T;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Streaming failed';
      setStatus({
        isStreaming: false,
        progress: 0,
        message: '',
        error: errorMessage,
      });
      throw error;
    }
  }, []);

  const reset = useCallback(() => {
    setStatus({ isStreaming: false, progress: 0, message: '' });
    setResult(null);
  }, []);

  return {
    stream,
    status,
    result,
    reset,
  };
}

/**
 * Hook for streaming text generation with progress updates
 */
export function useStreamText() {
  const [status, setStatus] = useState<StreamStatus>({
    isStreaming: false,
    progress: 0,
    message: '',
  });
  const [text, setText] = useState('');

  const stream = useCallback(async (
    prompt: string,
    onChunk?: (chunk: string) => void
  ) => {
    setStatus({ isStreaming: true, progress: 10, message: 'Initializing...' });
    setText('');

    try {
      const { provider, model } = getPreferredModel();

      setStatus({ isStreaming: true, progress: 20, message: 'Connecting to AI...' });

      const { textStream } = await streamText({
        model,
        prompt,
        temperature: AI_CONFIG.defaults.temperature,
        maxTokens: AI_CONFIG.defaults.maxTokens,
      });

      setStatus({ isStreaming: true, progress: 30, message: 'Generating text...' });

      let fullText = '';
      for await (const chunk of textStream) {
        fullText += chunk;
        setText(fullText);

        if (onChunk) {
          onChunk(chunk);
        }

        setStatus({
          isStreaming: true,
          progress: Math.min(30 + (fullText.length / 10), 90),
          message: 'Streaming...',
        });
      }

      setStatus({
        isStreaming: false,
        progress: 100,
        message: 'Complete!',
      });

      return fullText;
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Streaming failed';
      setStatus({
        isStreaming: false,
        progress: 0,
        message: '',
        error: errorMessage,
      });
      throw error;
    }
  }, []);

  const reset = useCallback(() => {
    setStatus({ isStreaming: false, progress: 0, message: '' });
    setText('');
  }, []);

  return {
    stream,
    status,
    text,
    reset,
  };
}

/**
 * Progress indicator component props
 */
export interface StreamingProgressProps {
  status: StreamStatus;
  showMessage?: boolean;
}

/**
 * Simple streaming progress display
 */
export function StreamingProgress({ status, showMessage = true }: StreamingProgressProps) {
  if (!status.isStreaming && status.progress === 0) {
    return null;
  }

  return (
    <div className="space-y-2">
      <div className="flex items-center justify-between text-sm">
        {showMessage && <span className="text-muted-foreground">{status.message}</span>}
        <span className="font-medium">{Math.round(status.progress)}%</span>
      </div>
      <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
        <div
          className="bg-primary h-full transition-all duration-300 ease-out"
          style={{ width: `${status.progress}%` }}
        />
      </div>
      {status.error && (
        <p className="text-sm text-destructive">{status.error}</p>
      )}
    </div>
  );
}
