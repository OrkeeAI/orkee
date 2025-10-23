// ABOUTME: React hook for SSE-based real-time server event updates
// ABOUTME: Implements retry logic with automatic fallback to polling after max retries

import { useState, useEffect } from 'react';
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

/**
 * React hook for real-time server event updates via SSE.
 * Automatically falls back to polling if SSE connection fails.
 *
 * @returns Server state and connection info
 * @returns activeServers - Set of active project IDs
 * @returns connectionMode - Current connection mode (sse/polling/connecting)
 * @returns isConnected - Whether any connection is established
 */
export function useServerEvents() {
  const [activeServers, setActiveServers] = useState<Set<string>>(new Set());
  const [connectionMode, setConnectionMode] = useState<'sse' | 'polling' | 'connecting'>('connecting');

  // Initialize connection on mount
  useEffect(() => {
    let eventSource: EventSource | null = null;
    let pollingInterval: NodeJS.Timeout | null = null;
    let retryCount = 0;

    // Polling fallback function
    const startPolling = async () => {
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
      pollingInterval = setInterval(poll, POLLING_INTERVAL);
    };

    // SSE connection with retry logic
    const connectSSE = async (): Promise<void> => {
      try {
        const baseUrl = await getApiBaseUrl();
        eventSource = new EventSource(`${baseUrl}/api/preview/events`);

        eventSource.onopen = () => {
          console.log('[SSE] Connection established');
          setConnectionMode('sse');
          retryCount = 0;
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
          eventSource?.close();
          eventSource = null;

          retryCount += 1;

          if (retryCount < MAX_RETRIES) {
            console.log(`[SSE] Retrying connection (${retryCount}/${MAX_RETRIES})...`);
            setConnectionMode('connecting');
            setTimeout(connectSSE, RETRY_DELAY);
          } else {
            console.log('[SSE] Max retries reached, falling back to polling');
            startPolling();
          }
        };
      } catch (error) {
        console.error('[SSE] Failed to create EventSource:', error);
        retryCount += 1;

        if (retryCount < MAX_RETRIES) {
          setTimeout(connectSSE, RETRY_DELAY);
        } else {
          startPolling();
        }
      }
    };

    // Start SSE connection
    connectSSE();

    // Cleanup on unmount
    return () => {
      eventSource?.close();
      if (pollingInterval) {
        clearInterval(pollingInterval);
      }
    };
  }, []); // Empty deps - only run once on mount

  return {
    activeServers,
    connectionMode,
    isConnected: connectionMode !== 'connecting',
  };
}
