import { apiClient } from './api';

// TypeScript interfaces matching Rust backend types
export interface AiUsageLog {
  id: string;
  projectId: string;
  requestId?: string;
  operation: string;
  model: string;
  provider: string;
  inputTokens?: number;
  outputTokens?: number;
  totalTokens?: number;
  estimatedCost?: number;
  durationMs?: number;
  error?: string;
  createdAt: string;
}

export interface OperationStats {
  operation: string;
  count: number;
  totalTokens: number;
  totalCost: number;
}

export interface ModelStats {
  model: string;
  count: number;
  totalTokens: number;
  totalCost: number;
}

export interface ProviderStats {
  provider: string;
  count: number;
  totalTokens: number;
  totalCost: number;
}

export interface AiUsageStats {
  totalRequests: number;
  successfulRequests: number;
  failedRequests: number;
  totalInputTokens: number;
  totalOutputTokens: number;
  totalTokens: number;
  totalCost: number;
  averageDurationMs: number;
  byOperation: OperationStats[];
  byModel: ModelStats[];
  byProvider: ProviderStats[];
}

export interface AiUsageQueryParams {
  projectId?: string;
  startDate?: string;
  endDate?: string;
  operation?: string;
  model?: string;
  provider?: string;
  limit?: number;
  offset?: number;
}

export interface ToolUsageStats {
  tool_name: string;
  call_count: number;
  success_count: number;
  failure_count: number;
  average_duration_ms: number;
  total_duration_ms: number;
}

export interface TimeSeriesDataPoint {
  timestamp: string;
  request_count: number;
  token_count: number;
  cost: number;
  tool_call_count: number;
}

/**
 * Fetches AI usage logs with optional filtering
 */
export async function getAiUsageLogs(params?: AiUsageQueryParams): Promise<AiUsageLog[]> {
  const queryParams = new URLSearchParams();

  if (params?.projectId) queryParams.append('projectId', params.projectId);
  if (params?.startDate) queryParams.append('startDate', params.startDate);
  if (params?.endDate) queryParams.append('endDate', params.endDate);
  if (params?.operation) queryParams.append('operation', params.operation);
  if (params?.model) queryParams.append('model', params.model);
  if (params?.provider) queryParams.append('provider', params.provider);
  if (params?.limit) queryParams.append('limit', params.limit.toString());
  if (params?.offset) queryParams.append('offset', params.offset.toString());

  const url = `/ai-usage/logs${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
  return apiClient.get<AiUsageLog[]>(url);
}

/**
 * Fetches aggregated AI usage statistics
 */
export async function getAiUsageStats(params?: {
  projectId?: string;
  startDate?: string;
  endDate?: string;
}): Promise<AiUsageStats> {
  const queryParams = new URLSearchParams();

  if (params?.projectId) queryParams.append('projectId', params.projectId);
  if (params?.startDate) queryParams.append('startDate', params.startDate);
  if (params?.endDate) queryParams.append('endDate', params.endDate);

  const url = `/ai-usage/stats${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
  return apiClient.get<AiUsageStats>(url);
}

/**
 * Format cost in a readable way
 */
export function formatCost(cost: number | undefined | null): string {
  if (cost === undefined || cost === null) return '$0.00';
  if (cost === 0) return '$0.00';
  if (cost < 0.01) return `${(cost * 100).toFixed(3)}¢`;
  return `$${cost.toFixed(2)}`;
}

/**
 * Format token count with commas
 */
export function formatTokens(tokens: number | undefined | null): string {
  if (tokens === undefined || tokens === null) return '0';
  return tokens.toLocaleString();
}

/**
 * Format duration in milliseconds
 */
export function formatDuration(ms: number | undefined | null): string {
  if (ms === undefined || ms === null) return '0ms';
  if (ms < 1000) return `${ms.toFixed(0)}ms`;
  return `${(ms / 1000).toFixed(2)}s`;
}

/**
 * Fetches tool usage statistics
 */
export async function getToolUsageStats(params?: {
  projectId?: string;
  startDate?: string;
  endDate?: string;
}): Promise<ToolUsageStats[]> {
  const queryParams = new URLSearchParams();

  if (params?.projectId) queryParams.append('projectId', params.projectId);
  if (params?.startDate) queryParams.append('startDate', params.startDate);
  if (params?.endDate) queryParams.append('endDate', params.endDate);

  const url = `/ai-usage/tools${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
  return apiClient.get<ToolUsageStats[]>(url);
}

/**
 * Fetches time-series data for usage charts
 */
export async function getTimeSeriesData(params?: {
  projectId?: string;
  startDate?: string;
  endDate?: string;
  interval?: 'hour' | 'day' | 'week' | 'month';
}): Promise<TimeSeriesDataPoint[]> {
  const queryParams = new URLSearchParams();

  if (params?.projectId) queryParams.append('projectId', params.projectId);
  if (params?.startDate) queryParams.append('startDate', params.startDate);
  if (params?.endDate) queryParams.append('endDate', params.endDate);
  if (params?.interval) queryParams.append('interval', params.interval);

  const url = `/ai-usage/time-series${queryParams.toString() ? `?${queryParams.toString()}` : ''}`;
  return apiClient.get<TimeSeriesDataPoint[]>(url);
}
