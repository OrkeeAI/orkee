// ABOUTME: React Context provider for model preferences with React Query integration
// ABOUTME: Provides access to user model preferences and convenient hooks for AI model selection

/* eslint-disable react-refresh/only-export-components */
import React, { createContext, useContext } from 'react';
import { useModelPreferences, getModelForTask } from '@/services/model-preferences';
import type { ModelPreferences, TaskType, ModelConfig } from '@/types/models';
import { DEFAULT_MODEL_CONFIG } from '@/types/models';

interface ModelPreferencesContextType {
  preferences: ModelPreferences | undefined;
  isLoading: boolean;
  isError: boolean;
  error: Error | null;
  getModelForTask: (taskType: TaskType) => ModelConfig;
}

const ModelPreferencesContext = createContext<ModelPreferencesContextType | undefined>(undefined);

interface ModelPreferencesProviderProps {
  children: React.ReactNode;
  userId?: string;
}

/**
 * Provider component that wraps React Query hooks for model preferences
 * Provides model preferences for a specific user (defaults to 'default-user')
 */
export function ModelPreferencesProvider({
  children,
  userId = 'default-user'
}: ModelPreferencesProviderProps) {
  const { data: preferences, isLoading, isError, error } = useModelPreferences(userId);

  const getModelForTaskWrapper = (taskType: TaskType): ModelConfig => {
    return getModelForTask(preferences, taskType);
  };

  const value: ModelPreferencesContextType = {
    preferences,
    isLoading,
    isError,
    error: error as Error | null,
    getModelForTask: getModelForTaskWrapper,
  };

  return (
    <ModelPreferencesContext.Provider value={value}>
      {children}
    </ModelPreferencesContext.Provider>
  );
}

/**
 * Hook to access model preferences context
 * Returns preferences, loading state, and utility to get model for a specific task
 */
export function useModelPreferencesContext() {
  const context = useContext(ModelPreferencesContext);
  if (context === undefined) {
    throw new Error('useModelPreferencesContext must be used within a ModelPreferencesProvider');
  }
  return context;
}

/**
 * Convenience hook to get model configuration for a specific task
 * Returns the configured model or default if preferences not loaded
 */
export function useTaskModel(taskType: TaskType): ModelConfig {
  const context = useContext(ModelPreferencesContext);

  // Gracefully handle usage outside provider - return default config
  if (context === undefined) {
    console.warn('useTaskModel called outside ModelPreferencesProvider, returning default config');
    return DEFAULT_MODEL_CONFIG;
  }

  return context.getModelForTask(taskType);
}
