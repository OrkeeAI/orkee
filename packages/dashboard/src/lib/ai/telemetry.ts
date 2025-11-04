// ABOUTME: AI telemetry tracking infrastructure for usage monitoring
// ABOUTME: Wraps AI SDK calls to capture tokens, costs, duration, and tool usage

import { calculateCost } from './config';
import { getApiBaseUrl } from '@/services/api';

/**
 * Telemetry data structure matching backend API
 */
export interface AITelemetryData {
  operation: string;
  projectId?: string | null;
  requestId?: string;
  model: string;
  provider: string;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  durationMs: number;
  toolCallsCount: number;
  toolCallsJson?: string;
  responseMetadata?: string;
  error?: string;
}

/**
 * Tool call structure
 */
export interface ToolCall {
  name: string;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  arguments: Record<string, any>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  result?: any;
  durationMs?: number;
  error?: string;
}

/**
 * AI SDK response types
 */
export interface AIResponse {
  // Common fields
  usage?: {
    promptTokens?: number;
    completionTokens?: number;
    totalTokens?: number;
  };
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  experimental_providerMetadata?: Record<string, any>;
  finishReason?: string;
  id?: string;

  // generateText response
  text?: string;

  // generateObject response
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  object?: any;

  // streamText response
  textStream?: AsyncIterable<string>;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  onFinish?: (result: any) => void | Promise<void>;
}

/**
 * Extract tool calls from AI SDK response
 * Checks multiple possible locations for tool calls depending on SDK version
 */
// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function extractToolCalls(response: any): ToolCall[] {
  const toolCalls: ToolCall[] = [];

  // Check experimental_toolCalls (newer AI SDK versions)
  if (response.experimental_toolCalls && Array.isArray(response.experimental_toolCalls)) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    response.experimental_toolCalls.forEach((call: any) => {
      toolCalls.push({
        name: call.toolName || call.name,
        arguments: call.args || call.arguments || {},
        result: call.result,
      });
    });
  }

  // Check toolCalls (older AI SDK versions)
  if (response.toolCalls && Array.isArray(response.toolCalls)) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    response.toolCalls.forEach((call: any) => {
      toolCalls.push({
        name: call.name || call.tool,
        arguments: call.arguments || call.input || {},
        result: call.output || call.result,
      });
    });
  }

  return toolCalls;
}

/**
 * Detect provider from model string
 */
export function detectProvider(model: string): string {
  const modelLower = model.toLowerCase();

  if (modelLower.includes('gpt') || modelLower.includes('openai')) {
    return 'openai';
  }
  if (modelLower.includes('claude') || modelLower.includes('anthropic')) {
    return 'anthropic';
  }
  if (modelLower.includes('gemini') || modelLower.includes('google')) {
    return 'google';
  }
  if (modelLower.includes('llama') || modelLower.includes('meta')) {
    return 'meta';
  }
  if (modelLower.includes('grok') || modelLower.includes('xai')) {
    return 'xai';
  }

  return 'unknown';
}

/**
 * Extract telemetry data from AI SDK response
 */
export function extractTelemetryData(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  response: any,
  operation: string,
  model: string,
  provider: string,
  estimatedCost: number
): Omit<AITelemetryData, 'durationMs' | 'projectId'> {
  const usage = response.usage || {};
  const toolCalls = extractToolCalls(response);

  return {
    operation,
    model,
    provider,
    inputTokens: usage.promptTokens || 0,
    outputTokens: usage.completionTokens || 0,
    totalTokens: usage.totalTokens || 0,
    estimatedCost,
    toolCallsCount: toolCalls.length,
    toolCallsJson: toolCalls.length > 0 ? JSON.stringify(toolCalls) : undefined,
    responseMetadata: JSON.stringify({
      finishReason: response.finishReason,
      id: response.id,
      ...response.experimental_providerMetadata,
    }),
  };
}

/**
 * Send telemetry data to backend API
 */
async function sendTelemetry(data: AITelemetryData): Promise<void> {
  try {
    const apiBaseUrl = await getApiBaseUrl();

    const response = await fetch(`${apiBaseUrl}/api/ai-usage`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        projectId: data.projectId,
        requestId: data.requestId,
        operation: data.operation,
        model: data.model,
        provider: data.provider,
        inputTokens: data.inputTokens,
        outputTokens: data.outputTokens,
        totalTokens: data.totalTokens,
        estimatedCost: data.estimatedCost,
        durationMs: data.durationMs,
        toolCallsCount: data.toolCallsCount,
        toolCallsJson: data.toolCallsJson,
        responseMetadata: data.responseMetadata,
        error: data.error,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      console.warn('[Telemetry] Failed to send telemetry:', errorText);
    }
  } catch (error) {
    // Fail silently - telemetry should not break AI operations
    console.warn('[Telemetry] Error sending telemetry:', error);
  }
}

/**
 * Build telemetry data from AI SDK result
 */
function buildTelemetryData(
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  result: any,
  operation: string,
  projectId: string | null,
  requestId: string,
  model: string,
  provider: string,
  durationMs: number,
  calculateCostFn: (inputTokens: number, outputTokens: number) => number
): AITelemetryData {
  const usage = result.usage || {};
  const toolCalls = extractToolCalls(result);

  const inputTokens = usage.inputTokens || usage.promptTokens || 0;
  const outputTokens = usage.outputTokens || usage.completionTokens || 0;
  const estimatedCost = calculateCostFn(inputTokens, outputTokens);

  return {
    operation,
    projectId,
    requestId,
    model,
    provider,
    inputTokens,
    outputTokens,
    totalTokens: usage.totalTokens || inputTokens + outputTokens,
    estimatedCost,
    durationMs,
    toolCallsCount: toolCalls.length,
    toolCallsJson: toolCalls.length > 0 ? JSON.stringify(toolCalls) : undefined,
    responseMetadata: JSON.stringify({
      finishReason: result.finishReason,
      id: result.id,
      ...result.experimental_providerMetadata,
    }),
  };
}

/**
 * Build error telemetry data
 */
function buildErrorTelemetryData(
  error: unknown,
  operation: string,
  projectId: string | null,
  requestId: string,
  model: string,
  provider: string,
  durationMs: number
): AITelemetryData {
  return {
    operation,
    projectId,
    requestId,
    model,
    provider,
    inputTokens: 0,
    outputTokens: 0,
    totalTokens: 0,
    estimatedCost: 0,
    durationMs,
    toolCallsCount: 0,
    error: error instanceof Error ? error.message : String(error),
  };
}

/**
 * Track an AI operation with automatic telemetry
 *
 * @param operation - Operation name (e.g., 'generate_project_ideas', 'chat_message')
 * @param projectId - Optional project ID to associate with this operation
 * @param model - Model name being used
 * @param provider - Provider name (openai, anthropic, google, etc.)
 * @param estimatedCost - Pre-calculated cost based on token usage
 * @param aiFunction - The async function that performs the AI operation
 * @returns The result from the AI function
 *
 * @example
 * ```typescript
 * const result = await trackAIOperation(
 *   'generate_tasks',
 *   projectId,
 *   'claude-sonnet-4-5-20250929',
 *   'anthropic',
 *   0.05,
 *   () => generateObject({ model, schema, prompt })
 * );
 * ```
 */
export async function trackAIOperation<T extends AIResponse>(
  operation: string,
  projectId: string | null,
  model: string,
  provider: string,
  aiFunction: () => Promise<T>
): Promise<T> {
  const startTime = performance.now();
  const requestId = crypto.randomUUID();

  // Cost calculation function using legacy approach
  const calcCost = (inputTokens: number, outputTokens: number) => {
    const providerType = (provider === 'openai' || provider === 'anthropic') ? provider : 'anthropic';
    return calculateCost(providerType, model, inputTokens, outputTokens);
  };

  try {
    const result = await aiFunction();

    // Handle streaming responses differently
    if (result.textStream) {
      // For streaming, set up onFinish callback
      const originalOnFinish = result.onFinish;

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      result.onFinish = async (finalResult: any) => {
        const durationMs = Math.round(performance.now() - startTime);
        const telemetryData = buildTelemetryData(
          finalResult,
          operation,
          projectId,
          requestId,
          model,
          provider,
          durationMs,
          calcCost
        );

        await sendTelemetry(telemetryData);

        // Call original onFinish if it exists
        if (originalOnFinish) {
          await originalOnFinish(finalResult);
        }
      };

      return result;
    } else {
      // For non-streaming responses, send telemetry immediately
      const durationMs = Math.round(performance.now() - startTime);
      const telemetryData = buildTelemetryData(
        result,
        operation,
        projectId,
        requestId,
        model,
        provider,
        durationMs,
        calcCost
      );

      await sendTelemetry(telemetryData);

      return result;
    }
  } catch (error) {
    const durationMs = Math.round(performance.now() - startTime);
    const telemetryData = buildErrorTelemetryData(
      error,
      operation,
      projectId,
      requestId,
      model,
      provider,
      durationMs
    );

    await sendTelemetry(telemetryData);

    throw error;
  }
}

/**
 * Helper to track AI operations with cost calculation
 * This version calculates cost using the calculateCost function
 *
 * @param operation - Operation name
 * @param projectId - Optional project ID
 * @param model - Model name
 * @param provider - Provider name
 * @param calculateCostFn - Function to calculate cost (inputTokens, outputTokens) => cost
 * @param aiFunction - The async function that performs the AI operation
 * @returns The result from the AI function along with usage data
 */
export async function trackAIOperationWithCost<T extends AIResponse>(
  operation: string,
  projectId: string | null,
  model: string,
  provider: 'openai' | 'anthropic' | 'google' | 'xai',
  calculateCostFn: (inputTokens: number, outputTokens: number) => number,
  aiFunction: () => Promise<T>
): Promise<T> {
  const startTime = performance.now();
  const requestId = crypto.randomUUID();

  try {
    const result = await aiFunction();

    // Handle streaming responses
    if (result.textStream) {
      const originalOnFinish = result.onFinish;

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      result.onFinish = async (finalResult: any) => {
        const durationMs = Math.round(performance.now() - startTime);
        const telemetryData = buildTelemetryData(
          finalResult,
          operation,
          projectId,
          requestId,
          model,
          provider,
          durationMs,
          calculateCostFn
        );

        await sendTelemetry(telemetryData);

        if (originalOnFinish) {
          await originalOnFinish(finalResult);
        }
      };

      return result;
    } else {
      // Non-streaming response
      const durationMs = Math.round(performance.now() - startTime);
      const telemetryData = buildTelemetryData(
        result,
        operation,
        projectId,
        requestId,
        model,
        provider,
        durationMs,
        calculateCostFn
      );

      await sendTelemetry(telemetryData);

      return result;
    }
  } catch (error) {
    const durationMs = Math.round(performance.now() - startTime);
    const telemetryData = buildErrorTelemetryData(
      error,
      operation,
      projectId,
      requestId,
      model,
      provider,
      durationMs
    );

    await sendTelemetry(telemetryData);

    throw error;
  }
}

/**
 * Send telemetry data from an AIResult object
 * Helper for services that return AIResult<T> with usage data
 *
 * @param operation - Operation name
 * @param projectId - Optional project ID
 * @param model - Model name
 * @param provider - Provider name
 * @param usage - Usage data from AIResult
 * @param estimatedCost - Calculated cost
 * @param durationMs - Duration in milliseconds
 * @param error - Optional error message
 */
export async function sendAIResultTelemetry(
  operation: string,
  projectId: string | null,
  model: string,
  provider: string,
  usage: { inputTokens: number; outputTokens: number; totalTokens: number },
  estimatedCost: number,
  durationMs: number,
  error?: string
): Promise<void> {
  const telemetryData: AITelemetryData = {
    operation,
    projectId,
    requestId: crypto.randomUUID(),
    model,
    provider,
    inputTokens: usage.inputTokens,
    outputTokens: usage.outputTokens,
    totalTokens: usage.totalTokens,
    estimatedCost,
    durationMs,
    toolCallsCount: 0,
    error,
  };

  await sendTelemetry(telemetryData);
}
