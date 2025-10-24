// ABOUTME: React Query hooks for agent operations
// ABOUTME: Provides hooks for fetching available AI agents (models)
import { useQuery } from '@tanstack/react-query';
import { agentsService } from '@/services/agents';
import { queryKeys } from '@/lib/queryClient';

/**
 * Get all available AI agents
 */
export function useAgents() {
  return useQuery({
    queryKey: queryKeys.agents,
    queryFn: async () => {
      const response = await agentsService.listAgents();
      return response.data;
    },
    staleTime: 5 * 60 * 1000, // 5 minutes - agents don't change often
  });
}

/**
 * Get a specific agent by ID
 */
export function useAgent(agentId: string) {
  return useQuery({
    queryKey: [...queryKeys.agents, agentId],
    queryFn: () => agentsService.getAgent(agentId),
    enabled: !!agentId,
    staleTime: 5 * 60 * 1000,
  });
}
