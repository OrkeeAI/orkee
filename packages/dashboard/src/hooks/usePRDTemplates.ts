// ABOUTME: React Query hook for fetching PRD output templates
// ABOUTME: Provides loading states and automatic refetching for template list

import { useQuery } from '@tanstack/react-query';
import { templatesService, type PRDTemplate } from '@/services/templates';

export function usePRDTemplates() {
  return useQuery<PRDTemplate[], Error>({
    queryKey: ['prd-templates'],
    queryFn: () => templatesService.getAll(),
    staleTime: 5 * 60 * 1000, // Templates don't change often - 5 minutes
  });
}
