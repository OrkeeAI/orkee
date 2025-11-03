// ABOUTME: Tests for AIModelsSettings component
// ABOUTME: Validates model preference UI, provider selection, and API key validation

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, waitFor } from '@testing-library/react';
import type { ModelInfo, ModelConfig, Provider } from '@/types/models';

// Mock UI components to bypass Radix UI issues
vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children, variant }: { children: React.ReactNode; variant?: string }) => (
    <div data-testid="alert" data-variant={variant}>{children}</div>
  ),
  AlertDescription: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="alert-description">{children}</div>
  ),
}));

vi.mock('@/components/ui/select', () => ({
  Select: ({ children, value }: any) => (
    <div data-testid="select" data-value={value}>{children}</div>
  ),
  SelectTrigger: ({ children }: any) => <button data-testid="select-trigger">{children}</button>,
  SelectValue: ({ placeholder }: any) => <span>{placeholder}</span>,
  SelectContent: ({ children }: any) => <div>{children}</div>,
  SelectItem: ({ children, value }: any) => <option value={value}>{children}</option>,
}));

vi.mock('@/components/ui/badge', () => ({
  Badge: ({ children, className }: { children: React.ReactNode; className?: string }) => (
    <span data-testid="badge" className={className}>{children}</span>
  ),
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Brain: () => <span data-icon="brain" />,
  FileText: () => <span data-icon="file-text" />,
  Search: () => <span data-icon="search" />,
  Lightbulb: () => <span data-icon="lightbulb" />,
  Code: () => <span data-icon="code" />,
  CheckSquare: () => <span data-icon="check-square" />,
  Settings: () => <span data-icon="settings" />,
  BookOpen: () => <span data-icon="book-open" />,
  FileCode: () => <span data-icon="file-code" />,
  AlertTriangle: () => <span data-icon="alert-triangle" />,
  Check: () => <span data-icon="check" />,
  X: () => <span data-icon="x" />,
}));

// Mock ModelPreferencesContext
const mockGetModelForTask = vi.fn();
const mockPreferences = {
  userId: 'test-user',
  chat: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  prdGeneration: { provider: 'openai' as Provider, model: 'gpt-4-turbo' },
  prdAnalysis: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  insightExtraction: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  specGeneration: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  taskSuggestions: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  taskAnalysis: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  specRefinement: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  researchGeneration: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  markdownGeneration: { provider: 'anthropic' as Provider, model: 'claude-sonnet-4-5-20250929' },
  updatedAt: '2025-01-15T12:00:00Z',
};

vi.mock('@/contexts/ModelPreferencesContext', () => ({
  useModelPreferencesContext: () => ({
    preferences: mockPreferences,
    isLoading: false,
    getModelForTask: mockGetModelForTask,
  }),
}));

// Mock useUpdateTaskModelPreference hook
const mockMutate = vi.fn();
vi.mock('@/services/model-preferences', () => ({
  useUpdateTaskModelPreference: () => ({
    mutate: mockMutate,
    isPending: false,
    isSuccess: false,
    isError: false,
  }),
}));

// Mock API client first (before services that use it)
vi.mock('@/services/api', () => ({
  apiClient: {
    get: vi.fn(),
  },
}));

// Mock users service
vi.mock('@/services/users', () => ({
  usersService: {
    getCurrentUser: vi.fn().mockResolvedValue({
      has_anthropic_api_key: true,
      has_openai_api_key: true,
      has_google_api_key: false,
      has_xai_api_key: false,
    }),
  },
}));

import { AIModelsSettings } from './AIModelsSettings';
import { apiClient } from '@/services/api';

describe('AIModelsSettings', () => {
  const mockModels: ModelInfo[] = [
    {
      id: 'claude-sonnet-4-5-20250929',
      name: 'Claude Sonnet 4.5',
      provider: 'anthropic',
      contextWindow: 200000,
      inputCost: 0.003,
      outputCost: 0.015,
      capabilities: { streaming: true },
    },
    {
      id: 'gpt-4-turbo',
      name: 'GPT-4 Turbo',
      provider: 'openai',
      contextWindow: 128000,
      inputCost: 0.01,
      outputCost: 0.03,
      capabilities: { streaming: true },
    },
  ];

  beforeEach(() => {
    vi.clearAllMocks();

    // Default: successful model fetch
    (apiClient.get as any).mockResolvedValue({
      data: { data: mockModels },
      error: null,
    });

    // Default: return model configs per task
    mockGetModelForTask.mockImplementation((taskType: string) => {
      const taskMap: Record<string, ModelConfig> = {
        chat: { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' },
        prd_generation: { provider: 'openai', model: 'gpt-4-turbo' },
      };
      return taskMap[taskType] || { provider: 'anthropic', model: 'claude-sonnet-4-5-20250929' };
    });
  });

  describe('Loading state', () => {
    it('should show loading message while fetching models', () => {
      // Create a never-resolving promise to keep loading state
      apiClient.get.mockReturnValue(new Promise(() => {}));

      render(<AIModelsSettings />);

      expect(screen.getByText('AI Model Preferences')).toBeInTheDocument();
      expect(screen.getByText('Loading model preferences...')).toBeInTheDocument();
    });
  });

  describe('Error handling', () => {
    it('should display error message when models fail to load', async () => {
      apiClient.get.mockResolvedValue({
        data: null,
        error: 'Failed to fetch models',
      });

      render(<AIModelsSettings />);

      // Wait for the error to appear
      await vi.waitFor(() => {
        expect(screen.getByText('Failed to fetch models')).toBeInTheDocument();
      });
    });
  });

  describe('Successful render', () => {
    it('should render header with title and description', async () => {
      render(<AIModelsSettings />);

      await waitFor(() => {
        expect(screen.getByText(/Configure which AI models to use/)).toBeInTheDocument();
      });
    });

    it('should display API key warning alert', async () => {
      render(<AIModelsSettings />);

      await vi.waitFor(() => {
        const alerts = screen.getAllByTestId('alert-description');
        const hasApiKeyWarning = alerts.some(alert =>
          alert.textContent?.includes('Model preferences require API keys')
        );
        expect(hasApiKeyWarning).toBe(true);
      });
    });

    it('should render table with all 10 task types', async () => {
      render(<AIModelsSettings />);

      await vi.waitFor(() => {
        expect(screen.getByText('Chat (Ideate Mode)')).toBeInTheDocument();
      });

      // Check for all task labels
      expect(screen.getByText('Chat (Ideate Mode)')).toBeInTheDocument();
      expect(screen.getByText('PRD Generation')).toBeInTheDocument();
      expect(screen.getByText('PRD Analysis')).toBeInTheDocument();
      expect(screen.getByText('Insight Extraction')).toBeInTheDocument();
      expect(screen.getByText('Spec Generation')).toBeInTheDocument();
      expect(screen.getByText('Task Suggestions')).toBeInTheDocument();
      expect(screen.getByText('Task Analysis')).toBeInTheDocument();
      expect(screen.getByText('Spec Refinement')).toBeInTheDocument();
      expect(screen.getByText('Research Generation')).toBeInTheDocument();
      expect(screen.getByText('Markdown Generation')).toBeInTheDocument();
    });

    it('should display table headers', async () => {
      render(<AIModelsSettings />);

      await vi.waitFor(() => {
        expect(screen.getByText('Task')).toBeInTheDocument();
      });

      expect(screen.getByText('Category')).toBeInTheDocument();
      expect(screen.getByText('Provider')).toBeInTheDocument();
      expect(screen.getByText('Model')).toBeInTheDocument();
      expect(screen.getByText('Status')).toBeInTheDocument();
    });

    it('should display category badges for tasks', async () => {
      render(<AIModelsSettings />);

      await waitFor(() => {
        const badges = screen.getAllByTestId('badge');
        expect(badges.length).toBeGreaterThan(0);
      });

      // Check that categories are rendered (some appear multiple times)
      expect(screen.getAllByText('ideate').length).toBeGreaterThan(0);
      expect(screen.getAllByText('prd').length).toBeGreaterThan(0);
      expect(screen.getAllByText('spec').length).toBeGreaterThan(0);
      expect(screen.getAllByText('research').length).toBeGreaterThan(0);
    });
  });

  describe('Model configuration display', () => {
    it('should show current model configuration for each task', async () => {
      render(<AIModelsSettings />);

      await vi.waitFor(() => {
        expect(mockGetModelForTask).toHaveBeenCalled();
      });

      // Verify getModelForTask was called for all task types
      expect(mockGetModelForTask).toHaveBeenCalledWith('chat');
      expect(mockGetModelForTask).toHaveBeenCalledWith('prd_generation');
      expect(mockGetModelForTask).toHaveBeenCalledWith('prd_analysis');
    });
  });

  describe('API fetching', () => {
    it('should fetch available models on mount', async () => {
      render(<AIModelsSettings />);

      await vi.waitFor(() => {
        expect(apiClient.get).toHaveBeenCalledWith('/api/models');
      });
    });

    it('should handle empty models response', async () => {
      apiClient.get.mockResolvedValue({
        data: { data: [] },
        error: null,
      });

      render(<AIModelsSettings />);

      await vi.waitFor(() => {
        // Component should still render with empty models
        expect(screen.getByText('AI Model Preferences')).toBeInTheDocument();
      });
    });
  });
});
