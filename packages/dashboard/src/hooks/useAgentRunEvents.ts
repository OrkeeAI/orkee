// ABOUTME: React hook for SSE-based real-time agent run event streaming
// ABOUTME: Subscribes to a single run's event stream with automatic reconnection and polling fallback

import { useState, useEffect, useCallback, useRef } from 'react';
import { getApiBaseUrl } from '@/services/api';
import { getApiToken } from '@/lib/platform';
import type { RunEvent, AgentRun } from '@/services/agent-runs';
import { getRun } from '@/services/agent-runs';

const DEBUG = import.meta.env.DEV;
const MAX_RETRIES = 3;
const RETRY_DELAY = 2000;
const POLLING_INTERVAL = 5000;

export type ConnectionMode = 'sse' | 'polling' | 'connecting' | 'disconnected';

export interface AgentRunEventState {
  events: RunEvent[];
  connectionMode: ConnectionMode;
  isConnected: boolean;
  run: AgentRun | null;
}

/**
 * Subscribe to real-time events for a specific agent run.
 * Falls back to polling the run status if SSE is unavailable.
 */
export function useAgentRunEvents(runId: string | null) {
  const [events, setEvents] = useState<RunEvent[]>([]);
  const [connectionMode, setConnectionMode] = useState<ConnectionMode>('disconnected');
  const [run, setRun] = useState<AgentRun | null>(null);
  const eventsRef = useRef<RunEvent[]>([]);

  // Append events without losing reference stability
  const appendEvent = useCallback((event: RunEvent) => {
    eventsRef.current = [...eventsRef.current, event];
    setEvents(eventsRef.current);
  }, []);

  // Reset when runId changes
  useEffect(() => {
    eventsRef.current = [];
    setEvents([]);
    setRun(null);
    if (!runId) {
      setConnectionMode('disconnected');
    }
  }, [runId]);

  useEffect(() => {
    if (!runId) return;

    let eventSource: EventSource | null = null;
    let pollingInterval: ReturnType<typeof setInterval> | null = null;
    let retryTimeout: ReturnType<typeof setTimeout> | null = null;
    let retryCount = 0;
    let isCleanedUp = false;

    // Fetch current run state (used for both SSE initial load and polling)
    const fetchRunState = async () => {
      if (isCleanedUp) return;
      try {
        const currentRun = await getRun(runId);
        setRun(currentRun);
        return currentRun;
      } catch (error) {
        if (DEBUG) console.error('[AgentRunSSE] Failed to fetch run:', error);
        return null;
      }
    };

    // Polling fallback
    const startPolling = async () => {
      if (isCleanedUp) return;
      setConnectionMode('polling');

      await fetchRunState();

      if (!isCleanedUp) {
        pollingInterval = setInterval(async () => {
          const currentRun = await fetchRunState();
          // Stop polling when run completes
          if (currentRun && ['completed', 'failed', 'cancelled'].includes(currentRun.status)) {
            if (pollingInterval) clearInterval(pollingInterval);
            setConnectionMode('disconnected');
          }
        }, POLLING_INTERVAL);
      }
    };

    // SSE connection
    const connectSSE = async (): Promise<void> => {
      if (isCleanedUp) return;

      // Fetch initial run state
      await fetchRunState();

      try {
        const baseUrl = await getApiBaseUrl();
        const apiToken = await getApiToken();

        const url = apiToken
          ? `${baseUrl}/api/agent-runs/${runId}/events?token=${encodeURIComponent(apiToken)}`
          : `${baseUrl}/api/agent-runs/${runId}/events`;

        eventSource = new EventSource(url);

        eventSource.onopen = () => {
          if (DEBUG) console.log('[AgentRunSSE] Connected');
          setConnectionMode('sse');
          retryCount = 0;
        };

        eventSource.onmessage = (sseEvent) => {
          try {
            const event: RunEvent = JSON.parse(sseEvent.data);
            if (DEBUG) console.log('[AgentRunSSE] Event:', event.type);
            appendEvent(event);

            // Update run state from lifecycle events
            if (event.type === 'run_completed' || event.type === 'run_failed') {
              fetchRunState();
            }
          } catch (error) {
            console.error('[AgentRunSSE] Failed to parse event:', error);
          }
        };

        eventSource.onerror = () => {
          if (isCleanedUp) return;

          eventSource?.close();
          eventSource = null;
          retryCount += 1;

          if (retryCount < MAX_RETRIES) {
            if (DEBUG) console.log(`[AgentRunSSE] Retrying (${retryCount}/${MAX_RETRIES})...`);
            setConnectionMode('connecting');

            if (retryTimeout) clearTimeout(retryTimeout);
            retryTimeout = setTimeout(() => {
              retryTimeout = null;
              connectSSE();
            }, RETRY_DELAY);
          } else {
            if (DEBUG) console.log('[AgentRunSSE] Falling back to polling');
            startPolling();
          }
        };
      } catch (error) {
        if (isCleanedUp) return;
        console.error('[AgentRunSSE] Failed to create EventSource:', error);
        retryCount += 1;

        if (retryCount < MAX_RETRIES) {
          setConnectionMode('connecting');
          if (retryTimeout) clearTimeout(retryTimeout);
          retryTimeout = setTimeout(() => {
            retryTimeout = null;
            connectSSE();
          }, RETRY_DELAY);
        } else {
          startPolling();
        }
      }
    };

    setConnectionMode('connecting');
    connectSSE();

    return () => {
      isCleanedUp = true;
      eventSource?.close();
      if (pollingInterval) clearInterval(pollingInterval);
      if (retryTimeout) clearTimeout(retryTimeout);
    };
  }, [runId, appendEvent]);

  const refetchRun = useCallback(async () => {
    if (!runId) return;
    try {
      const currentRun = await getRun(runId);
      setRun(currentRun);
    } catch (error) {
      if (DEBUG) console.error('[AgentRunSSE] Failed to refetch run:', error);
    }
  }, [runId]);

  return {
    events,
    connectionMode,
    isConnected: connectionMode === 'sse' || connectionMode === 'polling',
    run,
    refetchRun,
  };
}
