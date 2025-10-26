// ABOUTME: React Query hooks for user operations
// ABOUTME: Provides hooks for fetching current user info including API key status
import { useQuery } from '@tanstack/react-query';
import { usersService } from '@/services/users';
import { queryKeys } from '@/lib/queryClient';

/**
 * Get current user with masked credentials
 * Returns boolean flags for which API keys are configured
 */
export function useCurrentUser() {
  return useQuery({
    queryKey: queryKeys.currentUser,
    queryFn: () => usersService.getCurrentUser(),
    staleTime: 2 * 60 * 1000, // 2 minutes
    retry: (failureCount, error: unknown) => {
      // Don't retry if user doesn't exist
      if (error && typeof error === 'object' && 'message' in error && typeof error.message === 'string' && error.message.includes('not found')) {
        return false;
      }
      return failureCount < 2;
    },
  });
}
