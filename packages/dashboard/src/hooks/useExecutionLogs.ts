// ABOUTME: React hook for streaming execution logs via SSE
// ABOUTME: Provides real-time log updates with automatic connection management

import { useEffect, useRef, useState } from 'react';
import {
  ExecutionStreamClient,
  type LogEntry,
  type ExecutionStreamOptions,
} from '../services/execution-stream';

export interface UseExecutionLogsOptions {
  /**
   * Execution ID to stream logs from
   */
  executionId: string;

  /**
   * Last sequence number (for resuming from a specific point)
   */
  lastSequence?: number;

  /**
   * Whether to automatically connect on mount
   * @default true
   */
  autoConnect?: boolean;

  /**
   * Callback when execution completes
   */
  onComplete?: (success: boolean, errorMessage?: string) => void;

  /**
   * Callback when status changes
   */
  onStatusChange?: (status: string, errorMessage?: string) => void;
}

export interface UseExecutionLogsResult {
  /**
   * Array of log entries received so far
   */
  logs: LogEntry[];

  /**
   * Current execution status
   */
  status: string | null;

  /**
   * Current connection state
   */
  connectionState: 'connecting' | 'connected' | 'disconnected' | 'error';

  /**
   * Whether the execution is complete
   */
  isComplete: boolean;

  /**
   * Whether the execution failed
   */
  isFailed: boolean;

  /**
   * Error message if execution failed
   */
  errorMessage: string | null;

  /**
   * Manually connect to the stream
   */
  connect: () => void;

  /**
   * Manually disconnect from the stream
   */
  disconnect: () => void;

  /**
   * Clear all logs
   */
  clearLogs: () => void;
}

/**
 * Hook for streaming execution logs in real-time via SSE
 *
 * @example
 * ```tsx
 * function ExecutionViewer({ executionId }: { executionId: string }) {
 *   const { logs, connectionState, isComplete } = useExecutionLogs({
 *     executionId,
 *     onComplete: (success) => {
 *       if (success) {
 *         toast.success('Execution completed successfully');
 *       } else {
 *         toast.error('Execution failed');
 *       }
 *     },
 *   });
 *
 *   return (
 *     <div>
 *       <ConnectionStatus state={connectionState} />
 *       {logs.map((log) => (
 *         <LogEntry key={log.id} log={log} />
 *       ))}
 *       {isComplete && <CompletionBanner />}
 *     </div>
 *   );
 * }
 * ```
 */
export function useExecutionLogs(
  options: UseExecutionLogsOptions
): UseExecutionLogsResult {
  const {
    executionId,
    lastSequence,
    autoConnect = true,
    onComplete,
    onStatusChange,
  } = options;

  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [status, setStatus] = useState<string | null>(null);
  const [connectionState, setConnectionState] = useState<
    'connecting' | 'connected' | 'disconnected' | 'error'
  >('disconnected');
  const [isComplete, setIsComplete] = useState(false);
  const [isFailed, setIsFailed] = useState(false);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);

  const clientRef = useRef<ExecutionStreamClient | null>(null);

  // Create client options
  const clientOptions: ExecutionStreamOptions = {
    lastSequence,
    onLog: (log: LogEntry) => {
      setLogs((prev) => [...prev, log]);
    },
    onStatus: (newStatus: string, error?: string) => {
      setStatus(newStatus);
      if (error) {
        setErrorMessage(error);
        setIsFailed(true);
      }
      onStatusChange?.(newStatus, error);
    },
    onComplete: (success: boolean, error?: string) => {
      setIsComplete(true);
      if (!success) {
        setIsFailed(true);
        setErrorMessage(error || null);
      }
      onComplete?.(success, error);
    },
    onError: (error: Error) => {
      console.error('Execution stream error:', error);
      setConnectionState('error');
    },
    onConnectionChange: (state) => {
      setConnectionState(state);
    },
  };

  // Connect function
  const connect = () => {
    if (!clientRef.current) {
      clientRef.current = new ExecutionStreamClient(executionId, clientOptions);
    }
    clientRef.current.connect();
  };

  // Disconnect function
  const disconnect = () => {
    if (clientRef.current) {
      clientRef.current.disconnect();
      clientRef.current = null;
    }
  };

  // Clear logs function
  const clearLogs = () => {
    setLogs([]);
  };

  // Auto-connect on mount if enabled
  useEffect(() => {
    if (autoConnect) {
      connect();
    }

    // Cleanup on unmount
    return () => {
      disconnect();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [executionId, autoConnect]);

  return {
    logs,
    status,
    connectionState,
    isComplete,
    isFailed,
    errorMessage,
    connect,
    disconnect,
    clearLogs,
  };
}
