// ABOUTME: React hook for fetching and managing discovery questions
// ABOUTME: Provides suggested questions based on chat context

import { useState, useEffect, useCallback } from 'react';
import { chatService, DiscoveryQuestion } from '@/services/chat';

export interface UseDiscoveryQuestionsOptions {
  sessionId: string;
  category?: string;
  autoLoad?: boolean;
}

export function useDiscoveryQuestions({
  sessionId,
  category,
  autoLoad = true,
}: UseDiscoveryQuestionsOptions) {
  const [questions, setQuestions] = useState<DiscoveryQuestion[]>([]);
  const [suggestedQuestions, setSuggestedQuestions] = useState<DiscoveryQuestion[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const loadQuestions = useCallback(async () => {
    try {
      setIsLoading(true);
      setError(null);
      const data = await chatService.getDiscoveryQuestions(category);
      setQuestions(data);
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Failed to load discovery questions'));
    } finally {
      setIsLoading(false);
    }
  }, [category]);

  const loadSuggestedQuestions = useCallback(async () => {
    try {
      const data = await chatService.getSuggestedQuestions(sessionId);
      setSuggestedQuestions(data);
    } catch (err) {
      console.error('Failed to load suggested questions:', err);
    }
  }, [sessionId]);

  const refresh = useCallback(async () => {
    await Promise.all([loadQuestions(), loadSuggestedQuestions()]);
  }, [loadQuestions, loadSuggestedQuestions]);

  useEffect(() => {
    if (autoLoad) {
      loadQuestions();
      loadSuggestedQuestions();
    }
  }, [autoLoad, loadQuestions, loadSuggestedQuestions]);

  return {
    questions,
    suggestedQuestions,
    isLoading,
    error,
    refresh,
    loadQuestions,
    loadSuggestedQuestions,
  };
}
