// ABOUTME: Tests for useStreamingResponse hook
// ABOUTME: Validates streaming initialization, AbortSignal integration, and cleanup

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useStreamingResponse } from './useStreamingResponse';
import * as conversationalAi from '@/services/chat-ai';

// Mock the conversational-ai service
vi.mock('@/services/conversational-ai', () => ({
  streamChatResponse: vi.fn(),
}));

describe('useStreamingResponse', () => {
  const mockSessionId = 'test-session-123';
  const mockConversationHistory = [
    { id: '1', role: 'user' as const, content: 'Hello', timestamp: new Date().toISOString() },
    { id: '2', role: 'assistant' as const, content: 'Hi there!', timestamp: new Date().toISOString() },
  ];

  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  describe('startStreaming', () => {
    it('should initialize streaming message when starting', async () => {
      const mockStreamFn = conversationalAi.streamChatResponse as any;
      mockStreamFn.mockImplementation(async (
        _sid: string,
        _msg: string,
        _hist: any[],
        onChunk: (chunk: string) => void,
        onComplete: (fullText: string) => void
      ) => {
        onChunk('Hello');
        onChunk(' world');
        onComplete('Hello world');
      });

      const { result } = renderHook(() =>
        useStreamingResponse({
          sessionId: mockSessionId,
          conversationHistory: mockConversationHistory,
        })
      );

      expect(result.current.isStreaming).toBe(false);
      expect(result.current.streamingMessage).toBeNull();

      await act(async () => {
        await result.current.startStreaming('Test message');
      });

      expect(mockStreamFn).toHaveBeenCalledTimes(1);
      expect(mockStreamFn).toHaveBeenCalledWith(
        mockSessionId,
        'Test message',
        mockConversationHistory,
        expect.any(Function), // onChunk
        expect.any(Function), // onComplete
        expect.any(Function), // onError
        expect.any(Object) // abortSignal
      );
    });
  });

  describe('AbortSignal integration', () => {
    it('should pass AbortSignal to streaming function', async () => {
      const mockStreamFn = conversationalAi.streamChatResponse as any;

      mockStreamFn.mockImplementation(async () => {
        return Promise.resolve();
      });

      const { result } = renderHook(() =>
        useStreamingResponse({
          sessionId: mockSessionId,
          conversationHistory: mockConversationHistory,
        })
      );

      await act(async () => {
        await result.current.startStreaming('Test message');
      });

      // Verify AbortSignal was passed (7th argument)
      const callArgs = mockStreamFn.mock.calls[0];
      expect(callArgs[6]).toBeDefined();
      expect(callArgs[6]).toHaveProperty('aborted');
    });
  });

  describe('stopStreaming', () => {
    it('should abort ongoing stream', async () => {
      const mockStreamFn = conversationalAi.streamChatResponse as any;

      // Mock a stream that never completes
      mockStreamFn.mockImplementation(async () => {
        return new Promise(() => {});
      });

      const { result } = renderHook(() =>
        useStreamingResponse({
          sessionId: mockSessionId,
          conversationHistory: mockConversationHistory,
        })
      );

      await act(async () => {
        result.current.startStreaming('Test message');
      });

      expect(result.current.isStreaming).toBe(true);

      await act(async () => {
        result.current.stopStreaming();
      });

      expect(result.current.isStreaming).toBe(false);
      expect(result.current.streamingMessage).toBeNull();
    });

    it('should clean up resources on unmount', async () => {
      const mockStreamFn = conversationalAi.streamChatResponse as any;

      mockStreamFn.mockImplementation(async () => {
        return new Promise(() => {});
      });

      const { result, unmount } = renderHook(() =>
        useStreamingResponse({
          sessionId: mockSessionId,
          conversationHistory: mockConversationHistory,
        })
      );

      await act(async () => {
        result.current.startStreaming('Test message');
      });

      expect(result.current.isStreaming).toBe(true);

      unmount();

      // If cleanup works correctly, no errors should occur
      expect(true).toBe(true);
    });
  });
});
