import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { ideateService } from '@/services/ideate';
import { PRDTemplate, CreateTemplateInput } from '@/types/ideate';

const queryKeys = {
  all: ['quickstart-templates'] as const,
  list: () => [...queryKeys.all, 'list'] as const,
  detail: (id: string) => [...queryKeys.all, 'detail', id] as const,
  byType: (type: string) => [...queryKeys.all, 'byType', type] as const,
};

/**
 * Fetch all quickstart templates
 */
export function useQuickstartTemplates() {
  return useQuery({
    queryKey: queryKeys.list(),
    queryFn: () => ideateService.getTemplates(),
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
}

/**
 * Fetch templates by project type
 */
export function useTemplatesByType(projectType: string) {
  return useQuery({
    queryKey: queryKeys.byType(projectType),
    queryFn: () => ideateService.getTemplatesByType(projectType),
    enabled: !!projectType,
    staleTime: 5 * 60 * 1000,
  });
}

/**
 * Fetch single template
 */
export function useQuickstartTemplate(templateId: string) {
  return useQuery({
    queryKey: queryKeys.detail(templateId),
    queryFn: () => ideateService.getTemplate(templateId),
    enabled: !!templateId,
  });
}

/**
 * Create new template
 */
export function useCreateTemplate() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateTemplateInput) => ideateService.createTemplate(input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.list() });
    },
  });
}

/**
 * Update template
 */
export function useUpdateTemplate(templateId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (input: CreateTemplateInput) => ideateService.updateTemplate(templateId, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.list() });
      queryClient.invalidateQueries({ queryKey: queryKeys.detail(templateId) });
    },
  });
}

/**
 * Delete template
 */
export function useDeleteTemplate() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (templateId: string) => ideateService.deleteTemplate(templateId),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: queryKeys.list() });
    },
  });
}
