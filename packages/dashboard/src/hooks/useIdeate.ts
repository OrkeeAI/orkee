// ABOUTME: React Query hooks for ideate session operations (fetch, create, update, delete, skip)
// ABOUTME: Includes cache invalidation and optimistic updates for responsive UI

import { useState, useEffect } from 'react';
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
  CreateFeatureDependencyInput,
  OptimizationStrategy,
  SimilarProject,
  CreateExpertPersonaInput,
  SuggestExpertsRequest,
  CreateRoundtableRequest,
  AddParticipantsRequest,
  StartRoundtableRequest,
  UserInterjectionInput,
  RoundtableMessage,
  ExtractInsightsRequest,
  RoundtableEvent,
  // Phase 7: PRD Generation & Export
  ExportOptions,
  // Phase 8: Templates
  ProjectType,
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
    mutationFn: (data: { title: string; contentMarkdown: string }) => ideateService.saveAsPRD(sessionId, data),
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

// Phase 4: Dependency Intelligence hooks

/**
 * Get all feature dependencies for a session
 */
export function useFeatureDependencies(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateFeatureDependencies(sessionId),
    queryFn: () => ideateService.getFeatureDependencies(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}

/**
 * Create a manual feature dependency
 */
export function useCreateFeatureDependency(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateFeatureDependencyInput) =>
      ideateService.createFeatureDependency(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateFeatureDependencies(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateBuildOrder(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateCircularDeps(sessionId) });
    },
  });
}

/**
 * Delete a feature dependency
 */
export function useDeleteFeatureDependency(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (dependencyId: string) =>
      ideateService.deleteFeatureDependency(sessionId, dependencyId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateFeatureDependencies(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateBuildOrder(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateCircularDeps(sessionId) });
    },
  });
}

/**
 * Analyze dependencies using AI
 */
export function useAnalyzeDependencies(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => ideateService.analyzeDependencies(sessionId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateFeatureDependencies(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateBuildOrder(sessionId) });
    },
  });
}

/**
 * Optimize build order
 */
export function useOptimizeBuildOrder(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (strategy: OptimizationStrategy) =>
      ideateService.optimizeBuildOrder(sessionId, strategy),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateBuildOrder(sessionId) });
    },
  });
}

/**
 * Get current build order
 */
export function useBuildOrder(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateBuildOrder(sessionId),
    queryFn: () => ideateService.getBuildOrder(sessionId),
    enabled: !!sessionId,
    staleTime: 2 * 60 * 1000, // 2 minutes
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.status === 404) return false;
      return failureCount < 2;
    },
  });
}

/**
 * Get circular dependencies
 */
export function useCircularDependencies(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateCircularDeps(sessionId),
    queryFn: () => ideateService.getCircularDependencies(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}

/**
 * Suggest quick-win features
 */
export function useQuickWins(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateQuickWins(sessionId),
    queryFn: () => ideateService.suggestQuickWins(sessionId),
    enabled: !!sessionId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
}

// Phase 5: Research Analysis hooks

/**
 * Analyze a competitor URL
 */
export function useAnalyzeCompetitor(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ url, projectDescription }: { url: string; projectDescription?: string }) =>
      ideateService.analyzeCompetitor(sessionId, url, projectDescription),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateCompetitors(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateResearch(sessionId) });
    },
  });
}

/**
 * Get all analyzed competitors
 */
export function useCompetitors(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateCompetitors(sessionId),
    queryFn: () => ideateService.getCompetitors(sessionId),
    enabled: !!sessionId,
    staleTime: 5 * 60 * 1000, // 5 minutes - competitor data changes infrequently
  });
}

/**
 * Perform gap analysis against competitors
 */
export function useAnalyzeGaps(sessionId: string) {
  return useMutation({
    mutationFn: (yourFeatures: string[]) =>
      ideateService.analyzeGaps(sessionId, yourFeatures),
  });
}

/**
 * Extract UI/UX patterns from a URL
 */
export function useExtractPatterns(sessionId: string) {
  return useMutation({
    mutationFn: ({ url, projectDescription }: { url: string; projectDescription?: string }) =>
      ideateService.extractPatterns(sessionId, url, projectDescription),
  });
}

/**
 * Add a similar project reference
 */
export function useAddSimilarProject(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (project: SimilarProject) =>
      ideateService.addSimilarProject(sessionId, project),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateSimilarProjects(sessionId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateResearch(sessionId) });
    },
  });
}

/**
 * Get all similar projects
 */
export function useSimilarProjects(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateSimilarProjects(sessionId),
    queryFn: () => ideateService.getSimilarProjects(sessionId),
    enabled: !!sessionId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

/**
 * Extract lessons from a similar project
 */
export function useExtractLessons(sessionId: string) {
  return useMutation({
    mutationFn: ({ projectName, projectDescription }: { projectName: string; projectDescription?: string }) =>
      ideateService.extractLessons(sessionId, projectName, projectDescription),
  });
}

/**
 * Synthesize all research findings
 */
export function useSynthesizeResearch(sessionId: string) {
  return useMutation({
    mutationFn: () => ideateService.synthesizeResearch(sessionId),
  });
}

// Phase 6: Expert Roundtable hooks

/**
 * List all expert personas
 */
export function useListExperts(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateExperts(sessionId),
    queryFn: () => ideateService.listExperts(sessionId),
    enabled: !!sessionId,
    staleTime: 5 * 60 * 1000, // 5 minutes - experts change infrequently
  });
}

/**
 * Create a custom expert persona
 */
export function useCreateExpert(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateExpertPersonaInput) =>
      ideateService.createExpert(sessionId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateExperts(sessionId) });
    },
  });
}

/**
 * Get AI-suggested experts for a topic
 */
export function useSuggestExperts(sessionId: string) {
  return useMutation({
    mutationFn: (request: SuggestExpertsRequest) =>
      ideateService.suggestExperts(sessionId, request),
  });
}

/**
 * Create a roundtable session
 */
export function useCreateRoundtable(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CreateRoundtableRequest) =>
      ideateService.createRoundtable(sessionId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoundtables(sessionId) });
    },
  });
}

/**
 * List all roundtables for a session
 */
export function useListRoundtables(sessionId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoundtables(sessionId),
    queryFn: () => ideateService.listRoundtables(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}

/**
 * Get roundtable details with participants
 */
export function useGetRoundtable(roundtableId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoundtableDetail(roundtableId),
    queryFn: () => ideateService.getRoundtable(roundtableId),
    enabled: !!roundtableId,
    staleTime: 30 * 1000, // 30 seconds
    retry: (failureCount, error) => {
      const apiError = error as ApiError;
      if (apiError?.status === 404) return false;
      return failureCount < 2;
    },
  });
}

/**
 * Add participants to a roundtable
 */
export function useAddParticipants(roundtableId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: AddParticipantsRequest) =>
      ideateService.addParticipants(roundtableId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoundtableDetail(roundtableId) });
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoundtableParticipants(roundtableId) });
    },
  });
}

/**
 * Get roundtable participants
 */
export function useGetParticipants(roundtableId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoundtableParticipants(roundtableId),
    queryFn: () => ideateService.getParticipants(roundtableId),
    enabled: !!roundtableId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}

/**
 * Start a roundtable discussion
 */
export function useStartDiscussion(roundtableId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: StartRoundtableRequest) =>
      ideateService.startDiscussion(roundtableId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoundtableDetail(roundtableId) });
    },
  });
}

/**
 * Custom hook for SSE stream of roundtable messages
 */
export function useRoundtableStream(roundtableId: string, enabled = true) {
  const [messages, setMessages] = useState<RoundtableMessage[]>([]);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!roundtableId || !enabled) return;

    const streamUrl = ideateService.getRoundtableStreamUrl(roundtableId);
    const eventSource = new EventSource(streamUrl);

    eventSource.onopen = () => {
      setIsConnected(true);
      setError(null);
    };

    eventSource.onmessage = (event) => {
      try {
        const roundtableEvent: RoundtableEvent = JSON.parse(event.data);

        if (roundtableEvent.type === 'message' && 'message' in roundtableEvent) {
          setMessages((prev) => [...prev, roundtableEvent.message]);
        } else if (roundtableEvent.type === 'error' && 'error' in roundtableEvent) {
          setError(roundtableEvent.error);
        }
      } catch (err) {
        console.error('Failed to parse SSE message:', err);
      }
    };

    eventSource.onerror = () => {
      setIsConnected(false);
      setError('Connection lost');
      eventSource.close();
    };

    return () => {
      eventSource.close();
      setIsConnected(false);
    };
  }, [roundtableId, enabled]);

  return { messages, isConnected, error };
}

/**
 * Send a user interjection to the discussion
 */
export function useSendInterjection(roundtableId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: UserInterjectionInput) =>
      ideateService.sendInterjection(roundtableId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoundtableMessages(roundtableId) });
    },
  });
}

/**
 * Get all roundtable messages
 */
export function useGetRoundtableMessages(roundtableId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoundtableMessages(roundtableId),
    queryFn: () => ideateService.getRoundtableMessages(roundtableId),
    enabled: !!roundtableId,
    staleTime: 10 * 1000, // 10 seconds - messages update frequently
  });
}

/**
 * Extract insights from discussion
 */
export function useExtractInsights(roundtableId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: ExtractInsightsRequest) =>
      ideateService.extractInsights(roundtableId, request),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.ideateRoundtableInsights(roundtableId) });
    },
  });
}

/**
 * Get roundtable insights grouped by category
 */
export function useGetInsights(roundtableId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoundtableInsights(roundtableId),
    queryFn: () => ideateService.getInsights(roundtableId),
    enabled: !!roundtableId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
}

/**
 * Get roundtable statistics
 */
export function useRoundtableStatistics(roundtableId: string) {
  return useQuery({
    queryKey: queryKeys.ideateRoundtableStatistics(roundtableId),
    queryFn: () => ideateService.getRoundtableStatistics(roundtableId),
    enabled: !!roundtableId,
    staleTime: 30 * 1000, // 30 seconds
  });
}

// =============================================================================
// Phase 7: PRD Generation & Export Hooks
// =============================================================================

/**
 * Generate PRD from collected session data
 */
export function useGeneratePRD(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ includeSkipped = false }: { includeSkipped?: boolean }) =>
      ideateService.generatePRD(sessionId, includeSkipped),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ideate', sessionId, 'prd'] });
      queryClient.invalidateQueries({ queryKey: ['ideate', sessionId, 'history'] });
    },
  });
}

/**
 * AI-fill skipped sections
 */
export function useFillSkippedSections(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (sections: string[]) =>
      ideateService.fillSkippedSections(sessionId, sections),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ideate', sessionId, 'preview'] });
      queryClient.invalidateQueries({ queryKey: ['ideate', sessionId, 'completeness'] });
    },
  });
}

/**
 * Regenerate specific section
 */
export function useRegenerateSection(sessionId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (section: string) =>
      ideateService.regenerateSection(sessionId, section),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['ideate', sessionId, 'preview'] });
    },
  });
}

/**
 * Get PRD preview (aggregated data)
 */
export function usePRDPreview(sessionId: string) {
  return useQuery({
    queryKey: ['ideate', sessionId, 'preview'],
    queryFn: () => ideateService.getPRDPreview(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}

/**
 * Export PRD in specified format
 */
export function useExportPRD(sessionId: string) {
  return useMutation({
    mutationFn: (options: ExportOptions) =>
      ideateService.exportPRD(sessionId, options),
  });
}

/**
 * Get completeness metrics
 */
export function useCompleteness(sessionId: string) {
  return useQuery({
    queryKey: ['ideate', sessionId, 'completeness'],
    queryFn: () => ideateService.getCompleteness(sessionId),
    enabled: !!sessionId,
    staleTime: 30 * 1000, // 30 seconds
  });
}

/**
 * Get generation history
 */
export function useGenerationHistory(sessionId: string) {
  return useQuery({
    queryKey: ['ideate', sessionId, 'history'],
    queryFn: () => ideateService.getGenerationHistory(sessionId),
    enabled: !!sessionId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
}

/**
 * Validate PRD
 */
export function useValidatePRD(sessionId: string) {
  return useQuery({
    queryKey: ['ideate', sessionId, 'validation'],
    queryFn: () => ideateService.validatePRD(sessionId),
    enabled: !!sessionId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
}

// =============================================================================
// Phase 8: Template Hooks
// =============================================================================

/**
 * Fetch all available templates
 */
export function useTemplates() {
  return useQuery({
    queryKey: ['ideate', 'templates'],
    queryFn: () => ideateService.getTemplates(),
    staleTime: 10 * 60 * 1000, // 10 minutes (templates rarely change)
  });
}

/**
 * Fetch a specific template by ID
 */
export function useTemplate(templateId: string | null) {
  return useQuery({
    queryKey: ['ideate', 'templates', templateId],
    queryFn: () => ideateService.getTemplate(templateId!),
    enabled: !!templateId,
    staleTime: 10 * 60 * 1000, // 10 minutes
  });
}

/**
 * Fetch templates filtered by project type
 */
export function useTemplatesByType(projectType: ProjectType | null) {
  return useQuery({
    queryKey: ['ideate', 'templates', 'type', projectType],
    queryFn: () => ideateService.getTemplatesByType(projectType!),
    enabled: !!projectType,
    staleTime: 10 * 60 * 1000, // 10 minutes
  });
}

/**
 * Suggest best matching template based on description
 */
export function useSuggestTemplate(description: string) {
  return useQuery({
    queryKey: ['ideate', 'templates', 'suggest', description],
    queryFn: () => ideateService.suggestTemplate(description),
    enabled: description.length > 10, // Only suggest if description is meaningful
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}
