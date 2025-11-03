// ABOUTME: Model information badge component displaying model capabilities and specifications
// ABOUTME: Shows context window, pricing, and capability indicators for selected AI models

import { Badge } from '@/components/ui/badge';
import { Eye, Zap, Code, Globe } from 'lucide-react';

export interface ModelInfo {
  id: string;
  name: string;
  provider: string;
  max_context_tokens?: number;
  max_output_tokens?: number;
  pricing?: {
    input_per_million_tokens?: number;
    output_per_million_tokens?: number;
  };
  capabilities?: {
    tools?: boolean;
    vision?: boolean;
    extended_thinking?: boolean;
    extended_reasoning?: boolean;
    code_execution?: boolean;
    code_optimized?: boolean;
    web_search?: boolean;
  };
}

interface ModelInfoBadgeProps {
  model: ModelInfo | undefined;
}

/**
 * Format context window size with appropriate units
 */
function formatContextWindow(tokens: number): string {
  if (tokens >= 1_000_000) {
    return `${(tokens / 1_000_000).toFixed(1)}M`;
  } else if (tokens >= 1_000) {
    return `${Math.round(tokens / 1_000)}K`;
  }
  return `${tokens}`;
}

/**
 * Format pricing in a human-readable format
 */
function formatPricing(inputCost: number | undefined, outputCost: number | undefined): string {
  if (inputCost === undefined || outputCost === undefined) {
    return 'N/A';
  }

  // Show input cost as the primary metric (per 1M tokens)
  const inputStr = inputCost < 1 ? `$${inputCost.toFixed(2)}` : `$${Math.round(inputCost)}`;
  return `${inputStr}/1M`;
}

export function ModelInfoBadge({ model }: ModelInfoBadgeProps) {
  if (!model) {
    return (
      <div className="text-xs text-muted-foreground">
        Select a model to view details
      </div>
    );
  }

  const capabilities = model.capabilities || {};
  const hasVision = capabilities.vision;
  const hasTools = capabilities.tools;
  const hasExtendedThinking = capabilities.extended_thinking || capabilities.extended_reasoning;
  const hasCodeExecution = capabilities.code_execution || capabilities.code_optimized;
  const hasWebSearch = capabilities.web_search;

  return (
    <div className="space-y-2">
      {/* Model Specifications */}
      <div className="flex flex-wrap gap-2">
        {model.max_context_tokens && (
          <Badge variant="secondary" className="text-xs">
            {formatContextWindow(model.max_context_tokens)} context
          </Badge>
        )}
        {model.pricing && (
          <Badge variant="secondary" className="text-xs">
            {formatPricing(
              model.pricing.input_per_million_tokens,
              model.pricing.output_per_million_tokens
            )}
          </Badge>
        )}
      </div>

      {/* Capabilities */}
      {(hasVision || hasTools || hasExtendedThinking || hasCodeExecution || hasWebSearch) && (
        <div className="flex flex-wrap gap-1.5">
          {hasVision && (
            <Badge variant="outline" className="text-xs flex items-center gap-1">
              <Eye className="h-3 w-3" />
              Vision
            </Badge>
          )}
          {hasExtendedThinking && (
            <Badge variant="outline" className="text-xs flex items-center gap-1">
              <Zap className="h-3 w-3" />
              Thinking
            </Badge>
          )}
          {hasCodeExecution && (
            <Badge variant="outline" className="text-xs flex items-center gap-1">
              <Code className="h-3 w-3" />
              Code
            </Badge>
          )}
          {hasWebSearch && (
            <Badge variant="outline" className="text-xs flex items-center gap-1">
              <Globe className="h-3 w-3" />
              Web
            </Badge>
          )}
        </div>
      )}
    </div>
  );
}
