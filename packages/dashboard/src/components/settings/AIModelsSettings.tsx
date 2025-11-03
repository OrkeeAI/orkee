// ABOUTME: AI Models Settings component for configuring per-task model preferences
// ABOUTME: Displays 10 task cards for selecting models for Ideate, PRD, and Task features

import { useState, useEffect } from 'react';
import { Brain, FileText, Search, Lightbulb, Code, CheckSquare, Settings, BookOpen, FileCode } from 'lucide-react';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { ModelSelector } from './ModelSelector';
import type { ModelInfo } from './ModelInfoBadge';
import type { TaskType, ModelConfig } from '@/types/models';
import { useModelPreferencesContext } from '@/contexts/ModelPreferencesContext';
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

  const { preferences, isLoading: isLoadingPreferences, getModelForTask } = useModelPreferencesContext();

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

  if (isLoadingModels || isLoadingPreferences) {
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

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="rounded-lg border p-6">
        <h2 className="text-xl font-semibold mb-2">AI Model Preferences</h2>
        <p className="text-muted-foreground text-sm mb-4">
          Configure which AI models to use for different operations in Ideate, PRD, and Task features.
          These settings are separate from agent-specific models configured elsewhere.
        </p>
        <Alert>
          <AlertDescription className="text-xs">
            <strong>Important:</strong> Model preferences require API keys to be configured in the Security tab.
            Agent conversations use separate model settings from <code className="bg-muted px-1 py-0.5 rounded">user_agents.preferred_model_id</code>.
          </AlertDescription>
        </Alert>
      </div>

      {/* Task Cards Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {TASK_CONFIGS.map((task) => (
          <TaskCard
            key={task.type}
            task={task}
            currentConfig={getModelForTask(task.type)}
            availableModels={availableModels}
            userId="default-user"
          />
        ))}
      </div>
    </div>
  );
}

/**
 * Individual task card component
 */
interface TaskCardProps {
  task: TaskConfig;
  currentConfig: ModelConfig;
  availableModels: ModelInfo[];
  userId: string;
}

function TaskCard({ task, currentConfig, availableModels, userId }: TaskCardProps) {
  const [localConfig, setLocalConfig] = useState(currentConfig);

  // Sync with prop changes
  useEffect(() => {
    setLocalConfig(currentConfig);
  }, [currentConfig]);

  const getCategoryColor = (category: string) => {
    switch (category) {
      case 'ideate':
        return 'bg-purple-50 dark:bg-purple-950/30 border-purple-200 dark:border-purple-800';
      case 'prd':
        return 'bg-blue-50 dark:bg-blue-950/30 border-blue-200 dark:border-blue-800';
      case 'spec':
        return 'bg-green-50 dark:bg-green-950/30 border-green-200 dark:border-green-800';
      case 'research':
        return 'bg-orange-50 dark:bg-orange-950/30 border-orange-200 dark:border-orange-800';
      default:
        return 'bg-gray-50 dark:bg-gray-950/30 border-gray-200 dark:border-gray-800';
    }
  };

  return (
    <div className={`rounded-lg border p-4 ${getCategoryColor(task.category)}`}>
      {/* Task Header */}
      <div className="flex items-start gap-3 mb-4">
        <div className="mt-0.5 text-foreground">{task.icon}</div>
        <div className="flex-1">
          <h3 className="font-semibold text-sm text-foreground">{task.label}</h3>
          <p className="text-xs text-muted-foreground mt-1">{task.description}</p>
        </div>
      </div>

      {/* Model Selector */}
      <ModelSelector
        userId={userId}
        taskType={task.type}
        taskLabel={task.label}
        currentConfig={localConfig}
        availableModels={availableModels}
        onModelChange={setLocalConfig}
      />
    </div>
  );
}
