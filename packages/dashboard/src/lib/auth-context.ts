import { createContext } from 'react';

export interface ProviderAuthStatus {
  provider: string;
  authenticated: boolean;
  expiresAt: number | null;
  accountEmail: string | null;
  subscriptionType: string | null;
}

export interface AuthContextType {
  authStatus: Record<string, ProviderAuthStatus>;
  isLoading: boolean;
  error: string | null;
  refreshAuth: () => Promise<void>;
  getToken: (provider: string) => Promise<string | null>;
  logout: (provider: string) => Promise<void>;
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined);
