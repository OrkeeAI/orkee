// ABOUTME: React Query hooks for AI usage tracking operations (stats and logs)
// ABOUTME: Provides hooks for fetching AI usage statistics and detailed logs with filtering

import { useQuery } from '@tanstack/react-query';
import { getAiUsageStats, getAiUsageLogs } from '@/services/aiUsage';
import { queryKeys } from '@/lib/queryClient';
import type { AiUsageQueryParams } from '@/services/aiUsage';

export function useAiUsageStats(params?: {
  projectId?: string;
  startDate?: string;
  endDate?: string;
}) {
  return useQuery({
    queryKey: queryKeys.aiUsageStats(params),
    queryFn: () => getAiUsageStats(params),
    staleTime: 60 * 1000, // 1 minute
    refetchInterval: 5 * 60 * 1000, // Auto-refresh every 5 minutes
  });
}

export function useAiUsageLogs(params?: AiUsageQueryParams) {
  return useQuery({
    queryKey: queryKeys.aiUsageLogs(params),
    queryFn: () => getAiUsageLogs(params),
    enabled: !!params, // Only fetch when params are provided
    staleTime: 30 * 1000, // 30 seconds
  });
}
