// ABOUTME: React Query hooks for security and encryption management
// ABOUTME: Provides hooks for security status, key status, and password operations

import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { securityService } from '@/services/security';
import type { SecurityStatus, KeysStatusResponse } from '@/services/security';

/**
 * Hook to get current security and encryption status
 */
export function useSecurityStatus() {
  return useQuery<SecurityStatus, Error>({
    queryKey: ['security', 'status'],
    queryFn: () => securityService.getSecurityStatus(),
    staleTime: 30000, // 30 seconds
  });
}

/**
 * Hook to get status of all API keys
 */
export function useKeysStatus() {
  return useQuery<KeysStatusResponse, Error>({
    queryKey: ['security', 'keys-status'],
    queryFn: () => securityService.getKeysStatus(),
    staleTime: 10000, // 10 seconds
  });
}

/**
 * Hook to set password for password-based encryption
 */
export function useSetPassword() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (password: string) => securityService.setPassword(password),
    onSuccess: () => {
      // Invalidate security status and keys status to refetch
      queryClient.invalidateQueries({ queryKey: ['security', 'status'] });
      queryClient.invalidateQueries({ queryKey: ['security', 'keys-status'] });
    },
  });
}

/**
 * Hook to change encryption password
 */
export function useChangePassword() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ currentPassword, newPassword }: { currentPassword: string; newPassword: string }) =>
      securityService.changePassword(currentPassword, newPassword),
    onSuccess: () => {
      // Invalidate security status and keys status to refetch
      queryClient.invalidateQueries({ queryKey: ['security', 'status'] });
      queryClient.invalidateQueries({ queryKey: ['security', 'keys-status'] });
    },
  });
}

/**
 * Hook to remove password-based encryption
 */
export function useRemovePassword() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (currentPassword: string) => securityService.removePassword(currentPassword),
    onSuccess: () => {
      // Invalidate security status and keys status to refetch
      queryClient.invalidateQueries({ queryKey: ['security', 'status'] });
      queryClient.invalidateQueries({ queryKey: ['security', 'keys-status'] });
    },
  });
}
