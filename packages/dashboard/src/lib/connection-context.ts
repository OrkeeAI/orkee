import { createContext } from 'react';
import type { HealthStatus } from '@/services/health';

export type ConnectionState = 'connected' | 'connecting' | 'disconnected';

export interface ConnectionContextType {
  connectionState: ConnectionState;
  lastHealthCheck: HealthStatus | null;
  error: string | null;
}

export const ConnectionContext = createContext<ConnectionContextType | undefined>(undefined);