// ABOUTME: AI telemetry tracking infrastructure for usage monitoring
// ABOUTME: Wraps AI SDK calls to capture tokens, costs, duration, and tool usage

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
    const apiBaseUrl = window.location.origin.includes('localhost')
      ? 'http://localhost:4001'
      : window.location.origin;

    const response = await fetch(`${apiBaseUrl}/api/ai-usage`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        project_id: data.projectId,
        request_id: data.requestId,
        operation: data.operation,
        model: data.model,
        provider: data.provider,
        input_tokens: data.inputTokens,
        output_tokens: data.outputTokens,
        total_tokens: data.totalTokens,
        estimated_cost: data.estimatedCost,
        duration_ms: data.durationMs,
        tool_calls_count: data.toolCallsCount,
        tool_calls_json: data.toolCallsJson,
        response_metadata: data.responseMetadata,
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

  try {
    const result = await aiFunction();

    // Handle streaming responses differently
    if (result.textStream) {
      // For streaming, set up onFinish callback
      const originalOnFinish = result.onFinish;

      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      result.onFinish = async (finalResult: any) => {
        const durationMs = Math.round(performance.now() - startTime);
        const usage = finalResult.usage || {};
        const toolCalls = extractToolCalls(finalResult);

        // Calculate cost from usage
        const estimatedCost = usage.totalTokens ? usage.totalTokens * 0.00001 : 0; // Rough estimate

        const telemetryData: AITelemetryData = {
          operation,
          projectId,
          requestId,
          model,
          provider,
          inputTokens: usage.promptTokens || 0,
          outputTokens: usage.completionTokens || 0,
          totalTokens: usage.totalTokens || 0,
          estimatedCost,
          durationMs,
          toolCallsCount: toolCalls.length,
          toolCallsJson: toolCalls.length > 0 ? JSON.stringify(toolCalls) : undefined,
          responseMetadata: JSON.stringify({
            finishReason: finalResult.finishReason,
            id: finalResult.id,
            ...finalResult.experimental_providerMetadata,
          }),
        };

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
      const usage = result.usage || {};
      const toolCalls = extractToolCalls(result);

      // Calculate cost from usage
      const estimatedCost = usage.totalTokens ? usage.totalTokens * 0.00001 : 0; // Rough estimate

      const telemetryData: AITelemetryData = {
        operation,
        projectId,
        requestId,
        model,
        provider,
        inputTokens: usage.promptTokens || 0,
        outputTokens: usage.completionTokens || 0,
        totalTokens: usage.totalTokens || 0,
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

      await sendTelemetry(telemetryData);

      return result;
    }
  } catch (error) {
    // Log failed attempt
    const durationMs = Math.round(performance.now() - startTime);

    const telemetryData: AITelemetryData = {
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
        const usage = finalResult.usage || {};
        const toolCalls = extractToolCalls(finalResult);

        const inputTokens = usage.promptTokens || 0;
        const outputTokens = usage.completionTokens || 0;
        const estimatedCost = calculateCostFn(inputTokens, outputTokens);

        const telemetryData: AITelemetryData = {
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
            finishReason: finalResult.finishReason,
            id: finalResult.id,
            ...finalResult.experimental_providerMetadata,
          }),
        };

        await sendTelemetry(telemetryData);

        if (originalOnFinish) {
          await originalOnFinish(finalResult);
        }
      };

      return result;
    } else {
      // Non-streaming response
      const durationMs = Math.round(performance.now() - startTime);
      const usage = result.usage || {};
      const toolCalls = extractToolCalls(result);

      const inputTokens = usage.promptTokens || 0;
      const outputTokens = usage.completionTokens || 0;
      const estimatedCost = calculateCostFn(inputTokens, outputTokens);

      const telemetryData: AITelemetryData = {
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

      await sendTelemetry(telemetryData);

      return result;
    }
  } catch (error) {
    const durationMs = Math.round(performance.now() - startTime);

    const telemetryData: AITelemetryData = {
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
