// ABOUTME: AI Models Settings component for configuring per-task model preferences
// ABOUTME: Displays a compact table with rows for each task type and inline model selection

import { useState, useEffect } from 'react';
import { Brain, FileText, Search, Lightbulb, Code, CheckSquare, Settings, BookOpen, FileCode, AlertTriangle } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Badge } from '@/components/ui/badge';
import type { ModelInfo } from './ModelInfoBadge';
import type { TaskType, ModelConfig, Provider } from '@/types/models';
import { useModelPreferencesContext } from '@/contexts/ModelPreferencesContext';
import { useUpdateTaskModelPreference } from '@/services/model-preferences';
import { usersService } from '@/services/users';
import { apiClient } from '@/services/api';

/**
 * Task configuration metadata
 */
interface TaskConfig {
  type: TaskType;
  label: string;
  description: string;
  icon: React.ReactNode;
  category: 'ideate' | 'prd' | 'spec' | 'research';
}

const TASK_CONFIGS: TaskConfig[] = [
  {
    type: 'chat',
    label: 'Chat (Ideate Mode)',
    description: 'AI responses in Ideate mode conversations',
    icon: <Brain className="h-5 w-5" />,
    category: 'ideate',
  },
  {
    type: 'prd_generation',
    label: 'PRD Generation',
    description: 'Generating product requirement documents and sections',
    icon: <FileText className="h-5 w-5" />,
    category: 'prd',
  },
  {
    type: 'prd_analysis',
    label: 'PRD Analysis',
    description: 'Analyzing PRDs for clarity, completeness, and consistency',
    icon: <Search className="h-5 w-5" />,
    category: 'prd',
  },
  {
    type: 'insight_extraction',
    label: 'Insight Extraction',
    description: 'Extracting key insights from conversations',
    icon: <Lightbulb className="h-5 w-5" />,
    category: 'ideate',
  },
  {
    type: 'spec_generation',
    label: 'Spec Generation',
    description: 'Creating technical specifications from requirements',
    icon: <Code className="h-5 w-5" />,
    category: 'spec',
  },
  {
    type: 'task_suggestions',
    label: 'Task Suggestions',
    description: 'Suggesting tasks from specifications and requirements',
    icon: <CheckSquare className="h-5 w-5" />,
    category: 'spec',
  },
  {
    type: 'task_analysis',
    label: 'Task Analysis',
    description: 'Analyzing and validating task completeness',
    icon: <CheckSquare className="h-5 w-5" />,
    category: 'spec',
  },
  {
    type: 'spec_refinement',
    label: 'Spec Refinement',
    description: 'Refining and improving specifications',
    icon: <Settings className="h-5 w-5" />,
    category: 'spec',
  },
  {
    type: 'research_generation',
    label: 'Research Generation',
    description: 'Generating research content and documentation',
    icon: <BookOpen className="h-5 w-5" />,
    category: 'research',
  },
  {
    type: 'markdown_generation',
    label: 'Markdown Generation',
    description: 'Converting content to markdown format',
    icon: <FileCode className="h-5 w-5" />,
    category: 'research',
  },
];

export function AIModelsSettings() {
  const [availableModels, setAvailableModels] = useState<ModelInfo[]>([]);
  const [isLoadingModels, setIsLoadingModels] = useState(true);
  const [modelsError, setModelsError] = useState<string | null>(null);

  const { isLoading: isLoadingPreferences, preferences, getModelForTask } = useModelPreferencesContext();

  // Fetch available models from registry
  useEffect(() => {
    fetchModels();
  }, []);

  const fetchModels = async () => {
    setIsLoadingModels(true);
    setModelsError(null);
    try {
      const response = await apiClient.get<{ data: ModelInfo[]; pagination: unknown }>('/api/models');

      if (response.error) {
        throw new Error(response.error);
      }

      // Extract the models array from the paginated response
      const models = response.data?.data || [];
      setAvailableModels(models);
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Failed to fetch models';
      setModelsError(errorMessage);
      console.error('Failed to fetch models:', error);
    } finally {
      setIsLoadingModels(false);
    }
  };

  // Wait for both models and preferences to load
  if (isLoadingModels || isLoadingPreferences || !preferences) {
    return (
      <div className="rounded-lg border p-6">
        <h2 className="text-xl font-semibold mb-4">AI Model Preferences</h2>
        <p className="text-muted-foreground">Loading model preferences...</p>
      </div>
    );
  }

  if (modelsError) {
    return (
      <div className="rounded-lg border p-6">
        <h2 className="text-xl font-semibold mb-4">AI Model Preferences</h2>
        <Alert variant="destructive">
          <AlertDescription>{modelsError}</AlertDescription>
        </Alert>
      </div>
    );
  }

  const getCategoryBadge = (category: string) => {
    const colors = {
      ideate: 'bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300',
      prd: 'bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300',
      spec: 'bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300',
      research: 'bg-orange-100 text-orange-800 dark:bg-orange-900/30 dark:text-orange-300',
    };
    return colors[category as keyof typeof colors] || 'bg-gray-100 text-gray-800';
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="rounded-lg border p-6">
        <h2 className="text-xl font-semibold mb-2">AI Model Preferences</h2>
        <p className="text-muted-foreground text-sm mb-4">
          Configure which AI models to use for different operations. Each task type can use a different provider and model.
        </p>
        <Alert>
          <AlertDescription className="text-xs">
            <strong>Important:</strong> Model preferences require API keys to be configured in the Security tab.
          </AlertDescription>
        </Alert>
      </div>

      {/* Compact Table */}
      <div className="rounded-lg border">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-muted/50">
              <tr className="border-b">
                <th className="text-left p-3 font-medium text-sm">Task</th>
                <th className="text-left p-3 font-medium text-sm">Category</th>
                <th className="text-left p-3 font-medium text-sm w-48">Provider</th>
                <th className="text-left p-3 font-medium text-sm w-64">Model</th>
                <th className="text-center p-3 font-medium text-sm w-24">Status</th>
              </tr>
            </thead>
            <tbody>
              {TASK_CONFIGS.map((task) => (
                <TaskRow
                  key={task.type}
                  task={task}
                  currentConfig={getModelForTask(task.type)}
                  availableModels={availableModels}
                  getCategoryBadge={getCategoryBadge}
                />
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}

/**
 * Individual task row component
 */
interface TaskRowProps {
  task: TaskConfig;
  currentConfig: ModelConfig;
  availableModels: ModelInfo[];
  getCategoryBadge: (category: string) => string;
}

function TaskRow({ task, currentConfig, availableModels, getCategoryBadge }: TaskRowProps) {
  const [selectedProvider, setSelectedProvider] = useState<Provider>(currentConfig.provider);
  const [selectedModel, setSelectedModel] = useState<string>(currentConfig.model);
  const [hasApiKey, setHasApiKey] = useState(false);
  const [isCheckingKey, setIsCheckingKey] = useState(true);

  const updateMutation = useUpdateTaskModelPreference('default-user');

  // Sync local state with currentConfig when it changes
  useEffect(() => {
    setSelectedProvider(currentConfig.provider);
    setSelectedModel(currentConfig.model);
  }, [currentConfig.provider, currentConfig.model]);

  // Check if user has API key for the selected provider
  useEffect(() => {
    checkApiKey(selectedProvider);
  }, [selectedProvider]);

  const checkApiKey = async (provider: Provider) => {
    setIsCheckingKey(true);
    try {
      const user = await usersService.getCurrentUser();
      switch (provider) {
        case 'anthropic':
          setHasApiKey(user.has_anthropic_api_key);
          break;
        case 'openai':
          setHasApiKey(user.has_openai_api_key);
          break;
        case 'google':
          setHasApiKey(user.has_google_api_key);
          break;
        case 'xai':
          setHasApiKey(user.has_xai_api_key);
          break;
      }
    } catch (error) {
      console.error('Failed to check API key:', error);
      setHasApiKey(false);
    } finally {
      setIsCheckingKey(false);
    }
  };

  // Filter models by selected provider
  const modelsForProvider = availableModels.filter(m => m.provider === selectedProvider);

  // Get unique providers from available models
  const providers = Array.from(new Set(availableModels.map(m => m.provider as Provider)));

  const handleProviderChange = (provider: Provider) => {
    setSelectedProvider(provider);
    // Auto-select first model for new provider
    const firstModel = availableModels.find(m => m.provider === provider);
    if (firstModel) {
      setSelectedModel(firstModel.id);
      handleModelUpdate(provider, firstModel.id);
    }
  };

  const handleModelChange = (modelId: string) => {
    setSelectedModel(modelId);
    handleModelUpdate(selectedProvider, modelId);
  };

  const handleModelUpdate = async (provider: Provider, model: string) => {
    const newConfig: ModelConfig = { provider, model };
    try {
      await updateMutation.mutateAsync({ taskType: task.type, config: newConfig });
    } catch (error) {
      console.error('Failed to update model preference:', error);
    }
  };

  return (
    <tr className="border-b hover:bg-muted/30 transition-colors">
      {/* Task Name */}
      <td className="p-3">
        <div className="flex items-center gap-2">
          <div className="text-muted-foreground">{task.icon}</div>
          <div>
            <div className="font-medium text-sm">{task.label}</div>
            <div className="text-xs text-muted-foreground">{task.description}</div>
          </div>
        </div>
      </td>

      {/* Category Badge */}
      <td className="p-3">
        <Badge variant="secondary" className={`${getCategoryBadge(task.category)} text-xs capitalize`}>
          {task.category}
        </Badge>
      </td>

      {/* Provider Select */}
      <td className="p-3">
        <Select value={selectedProvider} onValueChange={handleProviderChange} disabled={updateMutation.isPending}>
          <SelectTrigger className="h-8">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {providers.map((provider) => (
              <SelectItem key={provider} value={provider}>
                <span className="capitalize">{provider}</span>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </td>

      {/* Model Select */}
      <td className="p-3">
        <Select
          value={selectedModel}
          onValueChange={handleModelChange}
          disabled={updateMutation.isPending || !hasApiKey}
        >
          <SelectTrigger className="h-8">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {modelsForProvider.map((model) => (
              <SelectItem key={model.id} value={model.id}>
                {model.name}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </td>

      {/* Status */}
      <td className="p-3">
        <div className="flex items-center justify-center">
          {isCheckingKey ? (
            <Badge variant="secondary" className="text-xs">
              Checking...
            </Badge>
          ) : hasApiKey ? (
            <Badge variant="secondary" className="text-xs bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300">
              Ready
            </Badge>
          ) : (
            <Badge variant="secondary" className="text-xs bg-red-100 text-red-800 dark:bg-red-900/30 dark:text-red-300">
              <AlertTriangle className="h-3 w-3 mr-1" />
              No Key
            </Badge>
          )}
        </div>
      </td>
    </tr>
  );
}
