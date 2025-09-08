import { useQuery, useMutation, useQueryClient, keepPreviousData } from '@tanstack/react-query'
import { projectsService } from '@/services/projects'
import { queryKeys, invalidateProjectQueries, invalidateProject } from '@/lib/queryClient'
import type { 
  Project, 
  ProjectCreateInput, 
  ProjectUpdateInput 
} from '@/services/projects'

// Type for API errors
interface ApiError {
  message?: string;
  status?: number;
}

// Query hooks
export function useProjects() {
  return useQuery({
    queryKey: queryKeys.projectsList(),
    queryFn: () => projectsService.getAllProjects(),
    staleTime: 2 * 60 * 1000, // 2 minutes for list data
    refetchInterval: 30 * 1000, // Refetch every 30 seconds to stay current
  })
}

export function useProject(id: string) {
  return useQuery({
    queryKey: queryKeys.projectDetail(id),
    queryFn: () => projectsService.getProject(id),
    enabled: !!id,
    staleTime: 5 * 60 * 1000, // 5 minutes for detail data
    retry: (failureCount, error) => {
      // Don't retry if project not found
      const apiError = error as ApiError;
      if (apiError?.message?.includes('not found')) {
        return false
      }
      return failureCount < 2
    },
  })
}

export function useProjectByName(name: string) {
  return useQuery({
    queryKey: queryKeys.projectByName(name),
    queryFn: () => projectsService.getProjectByName(name),
    enabled: !!name,
    staleTime: 5 * 60 * 1000,
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.message?.includes('not found')) {
        return false
      }
      return failureCount < 2
    },
  })
}

export function useProjectByPath(path: string) {
  return useQuery({
    queryKey: queryKeys.projectByPath(path),
    queryFn: () => projectsService.getProjectByPath(path),
    enabled: !!path,
    staleTime: 5 * 60 * 1000,
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.message?.includes('not found')) {
        return false
      }
      return failureCount < 2
    },
  })
}

// Search hook with debouncing and previous data
export function useSearchProjects(query: string, enabled = true) {
  return useQuery({
    queryKey: queryKeys.projectsSearch(query),
    queryFn: async () => {
      // For now, we'll implement search as client-side filtering
      // until we have a backend search endpoint
      const allProjects = await projectsService.getAllProjects()
      if (!query.trim()) {
        return allProjects
      }
      
      const searchTerm = query.toLowerCase()
      return allProjects.filter(project => 
        project.name.toLowerCase().includes(searchTerm) ||
        project.description?.toLowerCase().includes(searchTerm) ||
        project.projectRoot.toLowerCase().includes(searchTerm) ||
        project.tags?.some(tag => tag.toLowerCase().includes(searchTerm))
      )
    },
    enabled: enabled && query.length >= 2, // Only search with 2+ characters
    staleTime: 1 * 60 * 1000, // 1 minute for search results
    placeholderData: keepPreviousData, // Keep previous results while new ones load
  })
}

// Mutation hooks with optimistic updates
export function useCreateProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (input: ProjectCreateInput) => projectsService.createProject(input),
    onMutate: async (newProject) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: queryKeys.projectsList() })

      // Snapshot previous value
      const previousProjects = queryClient.getQueryData<Project[]>(queryKeys.projectsList())

      // Optimistically update cache
      queryClient.setQueryData<Project[]>(queryKeys.projectsList(), (old = []) => {
        const optimisticProject: Project = {
          id: `temp-${Date.now()}`,
          name: newProject.name,
          projectRoot: newProject.projectRoot,
          description: newProject.description,
          status: newProject.status || 'active',
          priority: newProject.priority || 'medium',
          rank: newProject.rank,
          tags: newProject.tags,
          setupScript: newProject.setupScript,
          devScript: newProject.devScript,
          cleanupScript: newProject.cleanupScript,
          taskSource: newProject.taskSource,
          manualTasks: newProject.manualTasks,
          mcpServers: newProject.mcpServers,
          createdAt: new Date().toISOString(),
          updatedAt: new Date().toISOString(),
        }
        return [optimisticProject, ...old]
      })

      return { previousProjects }
    },
    onError: (_err, _newProject, context) => {
      // Rollback optimistic update
      if (context?.previousProjects) {
        queryClient.setQueryData(queryKeys.projectsList(), context.previousProjects)
      }
    },
    onSuccess: (createdProject) => {
      // Remove optimistic update and add real data
      queryClient.setQueryData<Project[]>(queryKeys.projectsList(), (old = []) => {
        // Remove the temporary project and add the real one
        const filtered = old.filter(p => !p.id.startsWith('temp-'))
        return [createdProject, ...filtered]
      })
      
      // Cache the individual project
      queryClient.setQueryData(queryKeys.projectDetail(createdProject.id), createdProject)
    },
    onSettled: () => {
      // Always refetch to ensure consistency
      queryClient.invalidateQueries({ queryKey: queryKeys.projectsList() })
    },
  })
}

export function useUpdateProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: ({ id, input }: { id: string; input: ProjectUpdateInput }) => 
      projectsService.updateProject(id, input),
    onMutate: async ({ id, input }) => {
      // Cancel queries for this project
      await queryClient.cancelQueries({ queryKey: queryKeys.projectDetail(id) })
      await queryClient.cancelQueries({ queryKey: queryKeys.projectsList() })

      // Snapshot current values
      const previousProject = queryClient.getQueryData<Project>(queryKeys.projectDetail(id))
      const previousProjects = queryClient.getQueryData<Project[]>(queryKeys.projectsList())

      // Optimistic update for individual project
      if (previousProject) {
        const updatedProject = {
          ...previousProject,
          ...input,
          updatedAt: new Date().toISOString(),
        }
        queryClient.setQueryData(queryKeys.projectDetail(id), updatedProject)
      }

      // Optimistic update for project list
      if (previousProjects) {
        queryClient.setQueryData<Project[]>(queryKeys.projectsList(), (old = []) =>
          old.map(project => 
            project.id === id 
              ? { ...project, ...input, updatedAt: new Date().toISOString() }
              : project
          )
        )
      }

      return { previousProject, previousProjects }
    },
    onError: (_err, { id }, context) => {
      // Rollback optimistic updates
      if (context?.previousProject) {
        queryClient.setQueryData(queryKeys.projectDetail(id), context.previousProject)
      }
      if (context?.previousProjects) {
        queryClient.setQueryData(queryKeys.projectsList(), context.previousProjects)
      }
    },
    onSuccess: (updatedProject, { id }) => {
      // Update with real data
      queryClient.setQueryData(queryKeys.projectDetail(id), updatedProject)
      
      // Update in the list
      queryClient.setQueryData<Project[]>(queryKeys.projectsList(), (old = []) =>
        old.map(project => 
          project.id === id ? updatedProject : project
        )
      )
    },
    onSettled: (_updatedProject, _error, { id }) => {
      // Invalidate to ensure consistency
      invalidateProject(id)
    },
  })
}

export function useDeleteProject() {
  const queryClient = useQueryClient()

  return useMutation({
    mutationFn: (id: string) => projectsService.deleteProject(id),
    onMutate: async (id) => {
      // Cancel outgoing refetches
      await queryClient.cancelQueries({ queryKey: queryKeys.projectsList() })

      // Snapshot previous value
      const previousProjects = queryClient.getQueryData<Project[]>(queryKeys.projectsList())

      // Optimistically remove from cache
      queryClient.setQueryData<Project[]>(queryKeys.projectsList(), (old = []) =>
        old.filter(project => project.id !== id)
      )

      return { previousProjects }
    },
    onError: (_err, _id, context) => {
      // Rollback optimistic update
      if (context?.previousProjects) {
        queryClient.setQueryData(queryKeys.projectsList(), context.previousProjects)
      }
    },
    onSuccess: (_, id) => {
      // Remove from individual cache
      queryClient.removeQueries({ queryKey: queryKeys.projectDetail(id) })
    },
    onSettled: () => {
      // Ensure list is up to date
      queryClient.invalidateQueries({ queryKey: queryKeys.projectsList() })
    },
  })
}

// Utility hooks
export function usePrefetchProject(id: string) {
  const queryClient = useQueryClient()
  
  return () => {
    queryClient.prefetchQuery({
      queryKey: queryKeys.projectDetail(id),
      queryFn: () => projectsService.getProject(id),
      staleTime: 5 * 60 * 1000,
    })
  }
}

export function useInvalidateProjects() {
  return () => invalidateProjectQueries()
}

export function useInvalidateProject(id: string) {
  return () => invalidateProject(id)
}