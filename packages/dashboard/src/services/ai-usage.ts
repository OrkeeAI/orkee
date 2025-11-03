// ABOUTME: AI usage logging service for tracking token usage and costs
// ABOUTME: Non-blocking logging to backend for analytics and monitoring

/**
 * AI usage log entry for tracking token consumption and costs
 */
export interface AiUsageLog {
  projectId: string;
  operation: string;
  provider: string;
  model: string;
  inputTokens: number;
  outputTokens: number;
  totalTokens: number;
  estimatedCost: number;
  durationMs: number;
}

/**
 * Log AI usage to console for monitoring
 *
 * Non-blocking - failures do not throw errors
 * Future: Can be extended to send to backend endpoint or PostHog
 *
 * @param log - AI usage log entry
 */
export async function logAiUsage(log: AiUsageLog): Promise<void> {
  try {
    console.log('[ai-usage] AI operation logged:', {
      operation: log.operation,
      model: `${log.provider}/${log.model}`,
      tokens: `${log.inputTokens} in + ${log.outputTokens} out = ${log.totalTokens} total`,
      cost: `$${log.estimatedCost.toFixed(4)}`,
      duration: `${log.durationMs}ms`,
      projectId: log.projectId,
    });

    // Future: Send to backend endpoint
    // const { apiClient } = await import('./api');
    // await apiClient.post('/api/ai-usage-logs', log);
  } catch (error) {
    // Non-blocking - don't throw errors for logging failures
    console.warn('[ai-usage] Failed to log AI usage:', error);
  }
}
