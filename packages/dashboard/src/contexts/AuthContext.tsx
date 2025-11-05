/* eslint-disable react-refresh/only-export-components */
import React, { useEffect, useState, useCallback } from 'react';
import { AuthContext, ProviderAuthStatus } from '@/lib/auth-context';
import { api } from '@/services/api';

interface AuthProviderProps {
  children: React.ReactNode;
}

interface AuthStatusResponse {
  providers: ProviderAuthStatus[];
}

export function AuthProvider({ children }: AuthProviderProps) {
  const [authStatus, setAuthStatus] = useState<Record<string, ProviderAuthStatus>>({});
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchAuthStatus = useCallback(async () => {
    try {
      const response = await api<AuthStatusResponse>('/api/auth/status');

      if (response.success && response.data) {
        const statusMap: Record<string, ProviderAuthStatus> = {};
        response.data.providers.forEach((provider) => {
          statusMap[provider.provider] = provider;
        });
        setAuthStatus(statusMap);
        setError(null);
      } else {
        setError(response.error || 'Failed to fetch auth status');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to fetch auth status');
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    // Initial fetch
    fetchAuthStatus();

    // Poll every 5 minutes to check for token expiration
    const interval = setInterval(fetchAuthStatus, 5 * 60 * 1000);

    return () => clearInterval(interval);
  }, [fetchAuthStatus]);

  const refreshAuth = useCallback(async () => {
    setIsLoading(true);
    await fetchAuthStatus();
  }, [fetchAuthStatus]);

  const getToken = useCallback(async (provider: string): Promise<string | null> => {
    try {
      const response = await api<{ token: string; expiresAt: number }>(
        `/api/auth/${provider}/token`,
        {
          method: 'POST',
        }
      );

      if (response.success && response.data) {
        return response.data.token;
      }

      return null;
    } catch (err) {
      console.error(`Failed to get token for ${provider}:`, err);
      return null;
    }
  }, []);

  const logout = useCallback(async (provider: string) => {
    try {
      const response = await api(`/api/auth/${provider}`, {
        method: 'DELETE',
      });

      if (response.success) {
        // Refresh status after logout
        await fetchAuthStatus();
      } else {
        setError(response.error || `Failed to logout from ${provider}`);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : `Failed to logout from ${provider}`);
    }
  }, [fetchAuthStatus]);

  const value = {
    authStatus,
    isLoading,
    error,
    refreshAuth,
    getToken,
    logout,
  };

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

// Hook to use the auth context
export function useAuth() {
  const context = React.useContext(AuthContext);
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider');
  }
  return context;
}
