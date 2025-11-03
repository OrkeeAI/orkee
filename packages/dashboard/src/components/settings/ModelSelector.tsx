// ABOUTME: Model selector component for configuring task-specific AI model preferences
// ABOUTME: Allows selection of provider and model with API key validation and optimistic updates

import { useState, useEffect } from 'react';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { AlertTriangle, Check } from 'lucide-react';
import { Badge } from '@/components/ui/badge';
import { ModelInfoBadge, type ModelInfo } from './ModelInfoBadge';
import type { TaskType, ModelConfig, Provider } from '@/types/models';
import { useUpdateTaskModelPreference } from '@/services/model-preferences';
import { usersService } from '@/services/users';

interface ModelSelectorProps {
  userId: string;
  taskType: TaskType;
  taskLabel: string;
  currentConfig: ModelConfig;
  availableModels: ModelInfo[];
  onModelChange?: (config: ModelConfig) => void;
}

/**
 * Model selector component for a specific task type
 * Handles provider selection, model selection, and API key validation
 */
export function ModelSelector({
  userId,
  taskType,
  taskLabel,
  currentConfig,
  availableModels,
  onModelChange,
}: ModelSelectorProps) {
  const [selectedProvider, setSelectedProvider] = useState<Provider>(currentConfig.provider);
  const [selectedModel, setSelectedModel] = useState<string>(currentConfig.model);
  const [hasApiKey, setHasApiKey] = useState(false);
  const [isCheckingKey, setIsCheckingKey] = useState(true);

  const updateMutation = useUpdateTaskModelPreference(userId);

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

  // Find current model info
  const currentModelInfo = availableModels.find(m => m.id === selectedModel);

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

    // Optimistic update
    onModelChange?.(newConfig);

    // Save to backend
    try {
      await updateMutation.mutateAsync({ taskType, config: newConfig });
    } catch (error) {
      console.error('Failed to update model preference:', error);
      // Note: React Query will handle rollback automatically on error
    }
  };

  return (
    <div className="space-y-4">
      {/* Provider Selection */}
      <div className="space-y-2">
        <Label htmlFor={`${taskType}-provider`}>Provider</Label>
        <Select
          value={selectedProvider}
          onValueChange={handleProviderChange}
          disabled={updateMutation.isPending}
        >
          <SelectTrigger id={`${taskType}-provider`}>
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {providers.map((provider) => (
              <SelectItem key={provider} value={provider}>
                <div className="flex items-center gap-2 capitalize">
                  {provider === 'anthropic' && '🟣'}
                  {provider === 'openai' && '🟢'}
                  {provider === 'google' && '🔵'}
                  {provider === 'xai' && '⚪'}
                  <span>{provider}</span>
                </div>
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* API Key Warning */}
      {!isCheckingKey && !hasApiKey && (
        <Alert variant="destructive" className="text-xs">
          <AlertTriangle className="h-3 w-3" />
          <AlertDescription>
            No {selectedProvider} API key configured. Add your API key in the Security tab to use this provider.
          </AlertDescription>
        </Alert>
      )}

      {/* Model Selection */}
      <div className="space-y-2">
        <div className="flex items-center justify-between">
          <Label htmlFor={`${taskType}-model`}>Model</Label>
          {hasApiKey && (
            <Badge variant="secondary" className="text-xs">
              <Check className="h-3 w-3 mr-1" />
              API Key Ready
            </Badge>
          )}
        </div>
        <Select
          value={selectedModel}
          onValueChange={handleModelChange}
          disabled={updateMutation.isPending || !hasApiKey}
        >
          <SelectTrigger id={`${taskType}-model`}>
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
      </div>

      {/* Model Information */}
      <ModelInfoBadge model={currentModelInfo} />
    </div>
  );
}
