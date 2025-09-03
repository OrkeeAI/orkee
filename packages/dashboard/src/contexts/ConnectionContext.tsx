import React, { createContext, useContext, useEffect, useState } from 'react';
import { healthService, HealthStatus } from '../services/health';

export type ConnectionState = 'connected' | 'connecting' | 'disconnected';

interface ConnectionContextType {
  connectionState: ConnectionState;
  lastHealthCheck: HealthStatus | null;
  error: string | null;
}

const ConnectionContext = createContext<ConnectionContextType | undefined>(undefined);

export function useConnection() {
  const context = useContext(ConnectionContext);
  if (context === undefined) {
    throw new Error('useConnection must be used within a ConnectionProvider');
  }
  return context;
}

interface ConnectionProviderProps {
  children: React.ReactNode;
}

export function ConnectionProvider({ children }: ConnectionProviderProps) {
  const [connectionState, setConnectionState] = useState<ConnectionState>('connecting');
  const [lastHealthCheck, setLastHealthCheck] = useState<HealthStatus | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let intervalId: number;

    const performHealthCheck = async () => {
      setConnectionState('connecting');
      setError(null);

      try {
        const health = await healthService.checkHealth();
        if (health) {
          setConnectionState('connected');
          setLastHealthCheck(health);
          setError(null);
        } else {
          setConnectionState('disconnected');
          setError('Health check failed');
        }
      } catch (err) {
        setConnectionState('disconnected');
        setError(err instanceof Error ? err.message : 'Unknown error');
        setLastHealthCheck(null);
      }
    };

    // Perform initial health check
    performHealthCheck();

    // Set up polling every 20 seconds
    intervalId = setInterval(performHealthCheck, 20000);

    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, []);

  const value = {
    connectionState,
    lastHealthCheck,
    error,
  };

  return (
    <ConnectionContext.Provider value={value}>
      {children}
    </ConnectionContext.Provider>
  );
}