import { QueryClient } from '@tanstack/react-query'

export const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes (formerly cacheTime)
      retry: (failureCount, error) => {
        // Don't retry on 4xx errors except 408, 429
        const apiError = error as { status?: number };
        if (apiError?.status && apiError.status >= 400 && apiError.status < 500) {
          if (apiError.status === 408 || apiError.status === 429) {
            return failureCount < 2
          }
          return false
        }
        return failureCount < 3
      },
      refetchOnWindowFocus: true,
      refetchOnReconnect: true,
      // Don't retry on mount if data is stale but still present
      refetchOnMount: (query) => {
        return query.state.data == null
      },
    },
    mutations: {
      retry: 1,
      // Global error handler for mutations
      onError: (error) => {
        console.error('Mutation error:', error)
        // Could add toast notification here
      },
    },
  },
})

// Query invalidation utilities
export const queryKeys = {
  // Base keys
  projects: ['projects'] as const,
  directories: ['directories'] as const,
  health: ['health'] as const,
  prds: ['prds'] as const,

  // Project-specific keys
  projectsList: () => [...queryKeys.projects, 'list'] as const,
  projectsSearch: (query: string) => [...queryKeys.projects, 'search', query] as const,
  projectDetail: (id: string) => [...queryKeys.projects, 'detail', id] as const,
  projectByName: (name: string) => [...queryKeys.projects, 'by-name', name] as const,
  projectByPath: (path: string) => [...queryKeys.projects, 'by-path', path] as const,

  // Directory keys
  directoryList: (path: string) => [...queryKeys.directories, 'list', path] as const,

  // PRD keys
  prdsList: (projectId: string) => [...queryKeys.prds, 'list', projectId] as const,
  prdDetail: (projectId: string, prdId: string) => [...queryKeys.prds, 'detail', projectId, prdId] as const,
  prdAnalysis: (projectId: string, prdId: string) => [...queryKeys.prds, 'analysis', projectId, prdId] as const,

  // Spec keys
  specs: ['specs'] as const,
  specsList: (projectId: string) => [...queryKeys.specs, 'list', projectId] as const,
  specDetail: (projectId: string, specId: string) => [...queryKeys.specs, 'detail', projectId, specId] as const,
  specRequirements: (projectId: string, specId: string) => [...queryKeys.specs, 'requirements', projectId, specId] as const,

  // AI Usage keys
  aiUsage: ['ai-usage'] as const,
  aiUsageStats: (params?: { projectId?: string; startDate?: string; endDate?: string }) =>
    [...queryKeys.aiUsage, 'stats', params] as const,
  aiUsageLogs: (params?: { projectId?: string; startDate?: string; endDate?: string; operation?: string; model?: string; provider?: string; limit?: number; offset?: number }) =>
    [...queryKeys.aiUsage, 'logs', params] as const,
}

// Helper function to invalidate all project-related queries
export const invalidateProjectQueries = () => {
  queryClient.invalidateQueries({ queryKey: queryKeys.projects })
}

// Helper function to invalidate specific project
export const invalidateProject = (id: string) => {
  queryClient.invalidateQueries({ queryKey: queryKeys.projectDetail(id) })
  queryClient.invalidateQueries({ queryKey: queryKeys.projectsList() })
}

// PRD invalidation helpers
export const invalidatePRDQueries = (projectId: string) => {
  queryClient.invalidateQueries({ queryKey: queryKeys.prdsList(projectId) })
}

export const invalidatePRD = (projectId: string, prdId: string) => {
  queryClient.invalidateQueries({ queryKey: queryKeys.prdDetail(projectId, prdId) })
  queryClient.invalidateQueries({ queryKey: queryKeys.prdsList(projectId) })
  queryClient.invalidateQueries({ queryKey: queryKeys.prdAnalysis(projectId, prdId) })
}

// Spec invalidation helpers
export const invalidateSpecQueries = (projectId: string) => {
  queryClient.invalidateQueries({ queryKey: queryKeys.specsList(projectId) })
}

export const invalidateSpec = (projectId: string, specId: string) => {
  queryClient.invalidateQueries({ queryKey: queryKeys.specDetail(projectId, specId) })
  queryClient.invalidateQueries({ queryKey: queryKeys.specsList(projectId) })
  queryClient.invalidateQueries({ queryKey: queryKeys.specRequirements(projectId, specId) })
}

// Prefetch utilities
export const prefetchProjects = () => {
  return queryClient.prefetchQuery({
    queryKey: queryKeys.projectsList(),
    queryFn: async () => {
      const { projectsService } = await import('@/services/projects')
      return projectsService.getAllProjects()
    },
    staleTime: 5 * 60 * 1000,
  })
}

export const prefetchProject = (id: string) => {
  return queryClient.prefetchQuery({
    queryKey: queryKeys.projectDetail(id),
    queryFn: async () => {
      const { projectsService } = await import('@/services/projects')
      return projectsService.getProject(id)
    },
    staleTime: 5 * 60 * 1000,
  })
}