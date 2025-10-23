// ABOUTME: React hook for SSE-based real-time server event updates
// ABOUTME: Implements retry logic with automatic fallback to polling after max retries

import { useState, useEffect, useRef, useCallback } from 'react';
import { getApiBaseUrl } from '@/services/api';

interface ServerEvent {
  type: 'server_started' | 'server_stopped' | 'server_error' | 'initial_state';
  project_id?: string;
  active_servers?: string[];
  pid?: number;
  port?: number;
  framework?: string;
  error?: string;
}

const MAX_RETRIES = 3;
const RETRY_DELAY = 2000; // 2 seconds
const POLLING_INTERVAL = 5000; // 5 seconds

export function useServerEvents() {
  const [activeServers, setActiveServers] = useState<Set<string>>(new Set());
  const [connectionMode, setConnectionMode] = useState<'sse' | 'polling' | 'connecting'>('connecting');
  const retryCountRef = useRef(0);
  const eventSourceRef = useRef<EventSource | null>(null);
  const pollingIntervalRef = useRef<NodeJS.Timeout | null>(null);

  // Polling fallback
  const startPolling = useCallback(async () => {
    setConnectionMode('polling');

    const poll = async () => {
      try {
        const baseUrl = await getApiBaseUrl();
        const response = await fetch(`${baseUrl}/api/preview/servers`);
        const data = await response.json();

        if (data.success && data.data?.servers) {
          const serverIds = data.data.servers.map((s: { project_id: string }) => s.project_id);
          setActiveServers(new Set(serverIds));
        }
      } catch (error) {
        console.error('[Polling] Failed to fetch servers:', error);
      }
    };

    // Initial poll
    await poll();

    // Set up polling interval
    pollingIntervalRef.current = setInterval(poll, POLLING_INTERVAL);
  }, []);

  // SSE connection with retry logic
  const connectSSE = useCallback(async () => {
    try {
      const baseUrl = await getApiBaseUrl();
      const eventSource = new EventSource(`${baseUrl}/api/preview/events`);

      eventSource.onopen = () => {
        console.log('[SSE] Connection established');
        setConnectionMode('sse');
        retryCountRef.current = 0;
      };

      eventSource.onmessage = (event) => {
        try {
          const serverEvent: ServerEvent = JSON.parse(event.data);
          console.log('[SSE] Received event:', serverEvent);

          switch (serverEvent.type) {
            case 'initial_state':
              if (serverEvent.active_servers) {
                setActiveServers(new Set(serverEvent.active_servers));
              }
              break;
            case 'server_started':
              if (serverEvent.project_id) {
                setActiveServers((prev) => new Set([...prev, serverEvent.project_id!]));
              }
              break;
            case 'server_stopped':
              if (serverEvent.project_id) {
                setActiveServers((prev) => {
                  const next = new Set(prev);
                  next.delete(serverEvent.project_id!);
                  return next;
                });
              }
              break;
            case 'server_error':
              console.error('[SSE] Server error:', serverEvent.error);
              break;
          }
        } catch (error) {
          console.error('[SSE] Failed to parse event:', error);
        }
      };

      eventSource.onerror = (error) => {
        console.error('[SSE] Connection error:', error);
        eventSource.close();
        eventSourceRef.current = null;

        retryCountRef.current += 1;

        if (retryCountRef.current < MAX_RETRIES) {
          console.log(`[SSE] Retrying connection (${retryCountRef.current}/${MAX_RETRIES})...`);
          setConnectionMode('connecting');
          setTimeout(connectSSE, RETRY_DELAY);
        } else {
          console.log('[SSE] Max retries reached, falling back to polling');
          startPolling();
        }
      };

      eventSourceRef.current = eventSource;
    } catch (error) {
      console.error('[SSE] Failed to create EventSource:', error);
      retryCountRef.current += 1;

      if (retryCountRef.current < MAX_RETRIES) {
        setTimeout(connectSSE, RETRY_DELAY);
      } else {
        startPolling();
      }
    }
  }, [startPolling]);

  // Initialize connection on mount
  useEffect(() => {
    connectSSE();

    // Cleanup on unmount
    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
        eventSourceRef.current = null;
      }
      if (pollingIntervalRef.current) {
        clearInterval(pollingIntervalRef.current);
        pollingIntervalRef.current = null;
      }
    };
  }, [connectSSE]);

  return {
    activeServers,
    connectionMode,
    isConnected: connectionMode !== 'connecting',
  };
}
