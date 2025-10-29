// ABOUTME: Dialog for selecting AI model, provider, and template for PRD regeneration
// ABOUTME: Allows users to choose provider and model, then regenerate with selected template

import { useEffect, useState } from 'react';
import React from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { AlertCircle, Loader2, RefreshCw, Sparkles, DollarSign, Zap, AlertTriangle, FileText } from 'lucide-react';
import { ideateService } from '@/services/ideate';
import { useCurrentUser } from '@/hooks/useUsers';
import { useModels } from '@/hooks/useModels';
import type { PRDTemplate } from '@/services/ideate';

interface RegenerateTemplateDialogProps {
  sessionId: string;
  onSuccess?: () => void;
  onClose: () => void;
}

interface Model {
  id: string;
  provider: string;
  model: string;
  display_name: string;
  description: string;
  cost_per_1k_input_tokens: number;
  cost_per_1k_output_tokens: number;
  max_context_tokens: number;
  is_available: boolean;
}

export function RegenerateTemplateDialog({
  sessionId,
  onSuccess,
  onClose,
}: RegenerateTemplateDialogProps) {
  const [templates, setTemplates] = useState<PRDTemplate[]>([]);
  const [selectedTemplateId, setSelectedTemplateId] = useState<string>('');
  const [selectedProvider, setSelectedProvider] = useState<string>('anthropic');
  const [selectedModel, setSelectedModel] = useState<string>('claude-3-5-sonnet-20241022');
  const [isLoading, setIsLoading] = useState(true);
  const [isRegenerating, setIsRegenerating] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const { data: currentUser, isLoading: userLoading } = useCurrentUser();
  const { data: models, isLoading: modelsLoading } = useModels();

  // Get available providers (those with API keys configured)
  const availableProviders = React.useMemo(() => {
    if (!currentUser) return [];

    const providers: Array<{ value: string; label: string; hasKey: boolean }> = [
      {
        value: 'anthropic',
        label: 'Anthropic (Claude)',
        hasKey: !!currentUser.has_anthropic_api_key,
      },
      {
        value: 'openai',
        label: 'OpenAI (GPT)',
        hasKey: !!currentUser.has_openai_api_key,
      },
      {
        value: 'google',
        label: 'Google (Gemini)',
        hasKey: !!currentUser.has_google_api_key,
      },
      {
        value: 'xai',
        label: 'xAI (Grok)',
        hasKey: !!currentUser.has_xai_api_key,
      },
    ];

    return providers.filter((p) => p.hasKey);
  }, [currentUser]);

  // Get models for selected provider
  const availableModels = React.useMemo(() => {
    if (!models || !selectedProvider) return [];

    return models
      .filter((model: Model) => model.provider === selectedProvider)
      .map((model: Model) => ({
        value: model.model,
        label: model.display_name,
        inputCost: model.cost_per_1k_input_tokens,
        outputCost: model.cost_per_1k_output_tokens,
        contextWindow: model.max_context_tokens,
      }));
  }, [models, selectedProvider]);

  // Get selected model details for display
  const selectedModelDetails = React.useMemo(() => {
    return availableModels.find((m) => m.value === selectedModel);
  }, [availableModels, selectedModel]);

  // Fetch available output templates on mount
  useEffect(() => {
    const loadTemplates = async () => {
      try {
        setIsLoading(true);
        const data = await ideateService.getTemplates('output');
        setTemplates(data);
        if (data.length > 0) {
          setSelectedTemplateId(data[0].id);
        }
      } catch (err) {
        setError(
          err instanceof Error ? err.message : 'Failed to load templates'
        );
      } finally {
        setIsLoading(false);
      }
    };

    loadTemplates();
  }, []);

  // Reset to first available provider/model when component mounts or providers change
  useEffect(() => {
    if (!userLoading && !modelsLoading && availableProviders.length > 0) {
      const provider = availableProviders[0].value;
      setSelectedProvider(provider);

      // Set first model for this provider
      const providerModels = models?.filter((m: Model) => m.provider === provider) || [];
      if (providerModels.length > 0) {
        setSelectedModel(providerModels[0].model);
      }
    }
  }, [userLoading, modelsLoading, availableProviders, models]);

  // Update model when provider changes
  useEffect(() => {
    if (availableModels.length > 0) {
      const currentModelAvailable = availableModels.some((m) => m.value === selectedModel);
      if (!currentModelAvailable) {
        setSelectedModel(availableModels[0].value);
      }
    }
  }, [selectedProvider, availableModels, selectedModel]);

  const handleRegenerate = async () => {
    if (!selectedTemplateId) {
      setError('Please select a template');
      return;
    }

    try {
      setIsRegenerating(true);
      setError(null);
      await ideateService.regenerateWithTemplate(
        sessionId,
        selectedTemplateId,
        selectedProvider,
        selectedModel
      );
      // Give user feedback that regeneration succeeded
      onSuccess?.();
      onClose();
    } catch (err) {
      setError(
        err instanceof Error
          ? err.message
          : 'Failed to regenerate with template'
      );
    } finally {
      setIsRegenerating(false);
    }
  };

  return (
    <Dialog open onOpenChange={onClose}>
      <DialogContent className="sm:max-w-2xl max-h-[90vh] overflow-y-auto">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <RefreshCw className="h-5 w-5" />
            Regenerate with Template
          </DialogTitle>
          <DialogDescription>
            Select a template, AI provider, and model to regenerate your PRD with a
            different structure and style. AI will intelligently reformat your existing content.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4">
          {/* Template Selection */}
          {isLoading ? (
            <div className="flex items-center justify-center py-8">
              <div className="flex flex-col items-center gap-2 text-muted-foreground">
                <Loader2 className="h-6 w-6 animate-spin" />
                <p className="text-sm">Loading templates...</p>
              </div>
            </div>
          ) : error && templates.length === 0 ? (
            <Alert variant="destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          ) : (
            <>
              {/* Template Selector */}
              <div className="space-y-2">
                <Label htmlFor="template-select" className="text-base font-semibold">
                  <FileText className="h-4 w-4 inline mr-2" />
                  Output Template
                </Label>
                <Select value={selectedTemplateId} onValueChange={setSelectedTemplateId}>
                  <SelectTrigger id="template-select">
                    <SelectValue placeholder="Choose a template..." />
                  </SelectTrigger>
                  <SelectContent>
                    {templates.map((template) => (
                      <SelectItem key={template.id} value={template.id}>
                        {template.name}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>

              {selectedTemplateId && templates.length > 0 && (
                <div className="rounded-lg bg-muted p-3 text-sm">
                  <p className="text-muted-foreground">
                    {
                      templates.find((t) => t.id === selectedTemplateId)
                        ?.description
                    }
                  </p>
                </div>
              )}

              {/* Provider Selector */}
              <div className="space-y-2 pt-4">
                <Label htmlFor="provider-select" className="text-base font-semibold">
                  <Sparkles className="h-4 w-4 inline mr-2" />
                  AI Provider
                </Label>
                <Select value={selectedProvider} onValueChange={setSelectedProvider}>
                  <SelectTrigger id="provider-select">
                    <SelectValue placeholder="Select provider..." />
                  </SelectTrigger>
                  <SelectContent>
                    {availableProviders.map((provider) => (
                      <SelectItem key={provider.value} value={provider.value}>
                        {provider.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                {availableProviders.length === 0 && (
                  <p className="text-xs text-muted-foreground">
                    No providers configured. Add API keys in Settings.
                  </p>
                )}
              </div>

              {/* Model Selector */}
              {availableProviders.length > 0 && (
                <div className="space-y-2">
                  <Label htmlFor="model-select" className="text-base font-semibold">
                    <Zap className="h-4 w-4 inline mr-2" />
                    Model
                  </Label>
                  <Select value={selectedModel} onValueChange={setSelectedModel}>
                    <SelectTrigger id="model-select">
                      <SelectValue placeholder="Select model..." />
                    </SelectTrigger>
                    <SelectContent>
                      {availableModels.map((model) => (
                        <SelectItem key={model.value} value={model.value}>
                          {model.label}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                </div>
              )}

              {/* Model Details - Pricing and Info */}
              {selectedModelDetails && (
                <div className="space-y-3 rounded-lg bg-accent/50 p-3">
                  <p className="text-sm font-semibold text-foreground flex items-center gap-2">
                    <Sparkles className="h-4 w-4" />
                    Model Details
                  </p>
                  <div className="flex items-center justify-between">
                    <span className="text-xs text-muted-foreground">
                      <DollarSign className="h-3 w-3 inline mr-1" />
                      Input: ${(selectedModelDetails.inputCost * 1000).toFixed(2)}/1M
                    </span>
                    <span className="text-xs text-muted-foreground">
                      <DollarSign className="h-3 w-3 inline mr-1" />
                      Output: ${(selectedModelDetails.outputCost * 1000).toFixed(2)}/1M
                    </span>
                  </div>
                  {selectedModelDetails.contextWindow && (
                    <p className="text-xs text-muted-foreground flex items-center gap-1">
                      <Zap className="h-3 w-3" />
                      Context: {(selectedModelDetails.contextWindow / 1000).toFixed(0)}K tokens
                    </p>
                  )}
                </div>
              )}

              {/* Error Message */}
              {error && (
                <Alert variant="destructive">
                  <AlertCircle className="h-4 w-4" />
                  <AlertDescription>{error}</AlertDescription>
                </Alert>
              )}
            </>
          )}
        </div>

        <DialogFooter className="gap-2 sm:gap-0">
          <Button
            variant="outline"
            onClick={onClose}
            disabled={isRegenerating}
          >
            Cancel
          </Button>
          <Button
            onClick={handleRegenerate}
            disabled={
              isRegenerating ||
              isLoading ||
              !selectedTemplateId ||
              templates.length === 0 ||
              availableProviders.length === 0
            }
            className="gap-2"
          >
            {isRegenerating ? (
              <>
                <Loader2 className="h-4 w-4 animate-spin" />
                Regenerating...
              </>
            ) : (
              <>
                <RefreshCw className="h-4 w-4" />
                Regenerate
              </>
            )}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
