import { useContext } from 'react';
import { ConnectionContext } from '@/lib/connection-context';

export function useConnection() {
  const context = useContext(ConnectionContext);
  if (context === undefined) {
    throw new Error('useConnection must be used within a ConnectionProvider');
  }
  return context;
}