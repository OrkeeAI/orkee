// ABOUTME: Dialog for selecting AI provider and model before analysis
// ABOUTME: Shows only providers with configured API keys and their available models
import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Label } from '@/components/ui/label';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Sparkles, DollarSign, Zap, AlertTriangle } from 'lucide-react';
import { useCurrentUser } from '@/hooks/useUsers';
import { useModels } from '@/hooks/useModels';

interface ModelSelectionDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onConfirm: (provider: string, model: string) => void;
  defaultProvider?: string;
  defaultModel?: string;
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

export function ModelSelectionDialog({
  open,
  onOpenChange,
  onConfirm,
  defaultProvider = 'anthropic',
  defaultModel = 'claude-3-5-sonnet-20241022',
}: ModelSelectionDialogProps) {
  const [selectedProvider, setSelectedProvider] = useState(defaultProvider);
  const [selectedModel, setSelectedModel] = useState(defaultModel);
  
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

  // Reset to first available provider/model when opening
  useEffect(() => {
    if (open && availableProviders.length > 0) {
      const provider = availableProviders[0].value;
      setSelectedProvider(provider);

      // Set first model for this provider
      const providerModels = models?.filter((m: Model) => m.provider === provider) || [];
      if (providerModels.length > 0) {
        setSelectedModel(providerModels[0].model);
      }
    }
  }, [open, availableProviders, models]);

  // Update model when provider changes
  useEffect(() => {
    if (availableModels.length > 0) {
      const currentModelAvailable = availableModels.some((m) => m.value === selectedModel);
      if (!currentModelAvailable) {
        setSelectedModel(availableModels[0].value);
      }
    }
  }, [selectedProvider, availableModels, selectedModel]);

  const handleConfirm = () => {
    onConfirm(selectedProvider, selectedModel);
    onOpenChange(false);
  };

  if (userLoading || modelsLoading) {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Sparkles className="h-5 w-5" />
              Select AI Model
            </DialogTitle>
          </DialogHeader>
          <div className="py-8 text-center text-muted-foreground">
            Loading available models...
          </div>
        </DialogContent>
      </Dialog>
    );
  }

  if (availableProviders.length === 0) {
    return (
      <Dialog open={open} onOpenChange={onOpenChange}>
        <DialogContent className="sm:max-w-[500px]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <AlertTriangle className="h-5 w-5 text-orange-500" />
              No API Keys Configured
            </DialogTitle>
            <DialogDescription>
              You need to configure at least one AI provider API key to use analysis features.
            </DialogDescription>
          </DialogHeader>
          
          <Alert>
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              Please go to <strong>Settings → Security</strong> to add your API keys for:
              <ul className="list-disc list-inside mt-2 space-y-1">
                <li>Anthropic (Claude models)</li>
                <li>OpenAI (GPT models)</li>
                <li>Google (Gemini models)</li>
                <li>xAI (Grok models)</li>
              </ul>
            </AlertDescription>
          </Alert>

          <DialogFooter>
            <Button onClick={() => onOpenChange(false)} variant="outline">
              Cancel
            </Button>
            <Button onClick={() => window.location.href = '/settings'}>
              Go to Settings
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    );
  }

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-[500px]">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Sparkles className="h-5 w-5" />
            Select AI Model for Analysis
          </DialogTitle>
          <DialogDescription>
            Choose the AI provider and model to analyze your PRD
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {/* Provider Selection */}
          <div className="space-y-2">
            <Label htmlFor="provider">AI Provider</Label>
            <Select value={selectedProvider} onValueChange={setSelectedProvider}>
              <SelectTrigger id="provider">
                <SelectValue placeholder="Select provider" />
              </SelectTrigger>
              <SelectContent>
                {availableProviders.map((provider) => (
                  <SelectItem key={provider.value} value={provider.value}>
                    {provider.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Model Selection */}
          <div className="space-y-2">
            <Label htmlFor="model">Model</Label>
            <Select value={selectedModel} onValueChange={setSelectedModel}>
              <SelectTrigger id="model">
                <SelectValue placeholder="Select model" />
              </SelectTrigger>
              <SelectContent>
                {availableModels.map((model) => (
                  <SelectItem key={model.value} value={model.value}>
                    <div className="flex items-center justify-between w-full">
                      <span>{model.label}</span>
                      <span className="text-xs text-muted-foreground ml-2">
                        ${(model.inputCost * 1000).toFixed(2)}/${(model.outputCost * 1000).toFixed(2)} per 1M
                      </span>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Model Details */}
          {selectedModelDetails && (
            <div className="rounded-lg border p-4 space-y-2 bg-muted/50">
              <h4 className="font-semibold text-sm flex items-center gap-2">
                <Sparkles className="h-4 w-4" />
                Model Details
              </h4>
              <div className="grid grid-cols-2 gap-2 text-sm">
                <div className="flex items-center gap-2">
                  <DollarSign className="h-3 w-3 text-muted-foreground" />
                  <span className="text-muted-foreground">Input:</span>
                  <span className="font-mono">
                    ${(selectedModelDetails.inputCost * 1000).toFixed(2)}/1M
                  </span>
                </div>
                <div className="flex items-center gap-2">
                  <DollarSign className="h-3 w-3 text-muted-foreground" />
                  <span className="text-muted-foreground">Output:</span>
                  <span className="font-mono">
                    ${(selectedModelDetails.outputCost * 1000).toFixed(2)}/1M
                  </span>
                </div>
                <div className="flex items-center gap-2 col-span-2">
                  <Zap className="h-3 w-3 text-muted-foreground" />
                  <span className="text-muted-foreground">Context:</span>
                  <span className="font-mono">
                    {(selectedModelDetails.contextWindow / 1000).toFixed(0)}K tokens
                  </span>
                </div>
              </div>
              
              {/* Cost Estimate */}
              <div className="pt-2 border-t">
                <p className="text-xs text-muted-foreground">
                  <strong>Estimated cost for typical PRD analysis:</strong>
                  <br />
                  ~5K input + ~15K output = $
                  {(
                    (5 * selectedModelDetails.inputCost) +
                    (15 * selectedModelDetails.outputCost)
                  ).toFixed(3)}
                </p>
              </div>
            </div>
          )}

          {/* Recommendations */}
          <Alert>
            <Sparkles className="h-4 w-4" />
            <AlertDescription className="text-xs">
              <strong>Recommendations:</strong>
              <ul className="list-disc list-inside mt-1 space-y-0.5">
                <li><strong>Development:</strong> Use cheaper models like Claude Haiku or Gemini Flash</li>
                <li><strong>Production:</strong> Use Claude 3.5 Sonnet or GPT-4o for best results</li>
                <li><strong>Complex analysis:</strong> Use Claude Opus or GPT-4 Turbo</li>
              </ul>
            </AlertDescription>
          </Alert>
        </div>

        <DialogFooter>
          <Button onClick={() => onOpenChange(false)} variant="outline">
            Cancel
          </Button>
          <Button onClick={handleConfirm} className="gap-2">
            <Sparkles className="h-4 w-4" />
            Analyze PRD
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
