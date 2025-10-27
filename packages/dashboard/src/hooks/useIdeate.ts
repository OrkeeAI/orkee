// ABOUTME: React Query hooks for ideate session operations (fetch, create, update, delete, skip)
// ABOUTME: Includes cache invalidation and optimistic updates for responsive UI

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ideateService } from '@/services/ideate';
import { queryKeys } from '@/lib/queryClient';
import type {
  CreateIdeateInput,
  UpdateIdeateInput,
  SkipSectionInput,
  QuickGenerateInput,
  QuickExpandInput,
  IdeateOverview,
  IdeateUX,
  IdeateTechnical,
  IdeateRoadmap,
  IdeateDependencies,
  IdeateRisks,
  IdeateResearch,
} from '@/services/ideate';

interface ApiError {
  message?: string;
  status?: number;
}

/**
 * Fetch all ideate sessions for a project
 */
export function useIdeateSessions(projectId: string) {
  return useQuery({
    queryKey: queryKeys.ideateList(projectId),
    queryFn: () => ideateService.listSessions(projectId),
    enabled: !!projectId,
    staleTime: 2 * 60 * 1000,
  });
}

/**
 * Fetch a single ideate session by ID
 */
export function useIdeateSession(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateDetail(sessionId),
    queryFn: () => ideateService.getSession(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.status === 404) return false;
      return failureCount < 2;
    },
  });
}

/**
 * Fetch session completion status
 */
export function useIdeateStatus(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateStatus(sessionId),
    queryFn: () => ideateService.getCompletionStatus(sessionId),
    enabled: !!sessionId,
    staleTime: 30 * 1000, // 30 seconds - status changes frequently
  });
}

/**
 * Create a new ideate session
 */
export function useCreateIdeateSession(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateIdeateInput) => ideateService.createSession(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
    },
  });
}

/**
 * Update a ideate session
 */
export function useUpdateIdeateSession(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: UpdateIdeateInput) =>
      ideateService.updateSession(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete a ideate session
 */
export function useDeleteIdeateSession(projectId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sessionId: string) => ideateService.deleteSession(sessionId),
    onSuccess: (_, sessionId) => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
      queryClient.removeQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.removeQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Skip a section with optional AI fill
 */
export function useSkipSection(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: SkipSectionInput) => ideateService.skipSection(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Generate PRD from session description (Quick Mode)
 */
export function useQuickGenerate(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input?: QuickGenerateInput) => ideateService.quickGenerate(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Expand specific PRD sections (Quick Mode)
 */
export function useQuickExpand(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: QuickExpandInput) => ideateService.quickExpand(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Preview PRD before saving
 */
export function usePreviewPRD(sessionId: string) {
  return useQuery({
    queryKey: ['ideate-preview', sessionId],
    queryFn: () => ideateService.previewPRD(sessionId),
    enabled: !!sessionId,
    staleTime: 30 * 1000,
  });
}

/**
 * Save generated PRD to OpenSpec system
 */
export function useSaveAsPRD(projectId: string, sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.saveAsPRD(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateList(projectId) });
      queryClient.invalidateQueries({ queryKey: ['prds', projectId] });
    },
  });
}

// Section hooks for Guided Mode

/**
 * Fetch overview section
 */
export function useIdeateOverview(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateOverview(sessionId),
    queryFn: () => ideateService.getOverview(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save overview section
 */
export function useSaveOverview(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (overview: Omit<IdeateOverview, 'id' | 'created_at'>) =>
      ideateService.saveOverview(sessionId, overview),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateOverview(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete overview section
 */
export function useDeleteOverview(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteOverview(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateOverview(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Fetch UX section
 */
export function useIdeateUX(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateUX(sessionId),
    queryFn: () => ideateService.getUX(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save UX section
 */
export function useSaveUX(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (ux: Omit<IdeateUX, 'id' | 'created_at'>) =>
      ideateService.saveUX(sessionId, ux),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateUX(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete UX section
 */
export function useDeleteUX(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteUX(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateUX(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Fetch technical section
 */
export function useIdeateTechnical(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateTechnical(sessionId),
    queryFn: () => ideateService.getTechnical(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save technical section
 */
export function useSaveTechnical(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (technical: Omit<IdeateTechnical, 'id' | 'created_at'>) =>
      ideateService.saveTechnical(sessionId, technical),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateTechnical(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete technical section
 */
export function useDeleteTechnical(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteTechnical(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateTechnical(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Fetch roadmap section
 */
export function useIdeateRoadmap(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoadmap(sessionId),
    queryFn: () => ideateService.getRoadmap(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save roadmap section
 */
export function useSaveRoadmap(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (roadmap: Omit<IdeateRoadmap, 'id' | 'created_at'>) =>
      ideateService.saveRoadmap(sessionId, roadmap),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoadmap(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete roadmap section
 */
export function useDeleteRoadmap(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteRoadmap(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoadmap(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Fetch dependencies section
 */
export function useIdeateDependencies(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateDependencies(sessionId),
    queryFn: () => ideateService.getDependencies(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save dependencies section
 */
export function useSaveDependencies(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (dependencies: Omit<IdeateDependencies, 'id' | 'created_at'>) =>
      ideateService.saveDependencies(sessionId, dependencies),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDependencies(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete dependencies section
 */
export function useDeleteDependencies(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteDependencies(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDependencies(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Fetch risks section
 */
export function useIdeateRisks(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRisks(sessionId),
    queryFn: () => ideateService.getRisks(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save risks section
 */
export function useSaveRisks(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (risks: Omit<IdeateRisks, 'id' | 'created_at'>) =>
      ideateService.saveRisks(sessionId, risks),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRisks(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete risks section
 */
export function useDeleteRisks(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteRisks(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRisks(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Fetch research section
 */
export function useIdeateResearch(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateResearch(sessionId),
    queryFn: () => ideateService.getResearch(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000,
  });
}

/**
 * Save research section
 */
export function useSaveResearch(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (research: Omit<IdeateResearch, 'id' | 'created_at'>) =>
      ideateService.saveResearch(sessionId, research),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateResearch(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

/**
 * Delete research section
 */
export function useDeleteResearch(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.deleteResearch(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateResearch(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateStatus(sessionId) });
    },
  });
}

// Navigation hooks for Guided Mode

/**
 * Get next incomplete section
 */
export function useNextSection(sessionId: string) {
  return useQuery({
    queryKey: ['ideate-next-section', sessionId],
    queryFn: () => ideateService.getNextSection(sessionId),
    enabled: !!sessionId,
    staleTime: 30 * 1000,
  });
}

/**
 * Navigate to a specific section
 */
export function useNavigateToSection(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (section: string) => ideateService.navigateTo(sessionId, section),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateDetail(sessionId) });
    },
  });
}
