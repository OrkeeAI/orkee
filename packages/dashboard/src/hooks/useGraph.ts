// ABOUTME: React Query hooks for graph data fetching
// ABOUTME: Provides caching, loading states, and error handling for graph visualization

import { useQuery } from '@tanstack/react-query';
import { graphService, GraphType, GraphQueryOptions, CodeGraph } from '@/services/graph';

/**
 * Hook to fetch project graph data with React Query
 */
export function useProjectGraph(
  projectId: string,
  graphType: GraphType,
  options?: GraphQueryOptions
) {
  return useQuery<CodeGraph, Error>({
    queryKey: ['project-graph', projectId, graphType, options],
    queryFn: () => graphService.getGraph(projectId, graphType, options),
    staleTime: 5 * 60 * 1000, // 5 minutes
    enabled: !!projectId,
    retry: 2,
  });
}

/**
 * Hook to fetch dependency graph
 */
export function useDependencyGraph(
  projectId: string,
  options?: GraphQueryOptions
) {
  return useQuery<CodeGraph, Error>({
    queryKey: ['dependency-graph', projectId, options],
    queryFn: () => graphService.getDependencyGraph(projectId, options),
    staleTime: 5 * 60 * 1000,
    enabled: !!projectId,
    retry: 2,
  });
}

/**
 * Hook to fetch symbol graph
 */
export function useSymbolGraph(
  projectId: string,
  options?: GraphQueryOptions
) {
  return useQuery<CodeGraph, Error>({
    queryKey: ['symbol-graph', projectId, options],
    queryFn: () => graphService.getSymbolGraph(projectId, options),
    staleTime: 5 * 60 * 1000,
    enabled: !!projectId,
    retry: 2,
  });
}

/**
 * Hook to fetch module graph
 */
export function useModuleGraph(
  projectId: string,
  options?: GraphQueryOptions
) {
  return useQuery<CodeGraph, Error>({
    queryKey: ['module-graph', projectId, options],
    queryFn: () => graphService.getModuleGraph(projectId, options),
    staleTime: 5 * 60 * 1000,
    enabled: !!projectId,
    retry: 2,
  });
}

/**
 * Hook to fetch spec-mapping graph
 */
export function useSpecMappingGraph(
  projectId: string,
  options?: GraphQueryOptions
) {
  return useQuery<CodeGraph, Error>({
    queryKey: ['spec-mapping-graph', projectId, options],
    queryFn: () => graphService.getSpecMappingGraph(projectId, options),
    staleTime: 5 * 60 * 1000,
    enabled: !!projectId,
    retry: 2,
  });
}
