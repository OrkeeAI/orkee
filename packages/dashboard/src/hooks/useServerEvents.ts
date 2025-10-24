// ABOUTME: React hook for SSE-based real-time server event updates
// ABOUTME: Implements retry logic with automatic fallback to polling after max retries

import { useState, useEffect } from 'react';
import { getApiBaseUrl } from '@/services/api';
import { getApiToken } from '@/lib/platform';

// Enable debug logging only in development
const DEBUG = import.meta.env.DEV;

interface ServerEvent {
  type: 'server_started' | 'server_stopped' | 'server_error' | 'initial_state';
  project_id?: string;
  active_servers?: string[];
  pid?: number;
  port?: number;
  framework?: string;
  error?: string;
}

/**
 * Sanitize server error messages to prevent information leakage
 * Removes file paths, stack traces, and other sensitive details
 */
function sanitizeErrorMessage(rawError: string): string {
  // Keep the raw error in console for debugging
  console.error('[SSE] Raw server error:', rawError);

  // Remove file paths (absolute and relative)
  // Patterns: /path/to/file, C:\path\to\file, ./relative/path, ~/home/path
  let sanitized = rawError.replace(/(?:[A-Za-z]:)?(?:\/|\\)[\w/\\\-._]+/g, '[path]');

  // Remove stack traces (lines starting with "at " or containing file:line:col)
  sanitized = sanitized.split('\n')[0]; // Take only first line

  // Remove common internal error prefixes
  sanitized = sanitized.replace(/^Error:\s*/i, '');
  sanitized = sanitized.replace(/^Failed to\s+/i, '');

  // Map common technical errors to user-friendly messages
  const errorMappings: Record<string, string> = {
    'ENOENT': 'File or directory not found',
    'EACCES': 'Permission denied',
    'EADDRINUSE': 'Port already in use',
    'ECONNREFUSED': 'Connection refused',
    'ENOTFOUND': 'Resource not found',
    'ETIMEDOUT': 'Connection timed out',
    'EPERM': 'Operation not permitted',
  };

  // Check for known error codes
  for (const [code, message] of Object.entries(errorMappings)) {
    if (sanitized.includes(code)) {
      return message;
    }
  }

  // Limit length to prevent verbose error messages
  if (sanitized.length > 100) {
    sanitized = sanitized.substring(0, 100) + '...';
  }

  // If error is now empty or too generic, provide a default message
  if (!sanitized.trim() || sanitized.length < 5) {
    return 'An error occurred while starting the server';
  }

  return sanitized.trim();
}

// SSE configuration constants with environment variable overrides
const MAX_RETRIES = (() => {
  const val = parseInt(import.meta.env.VITE_SSE_MAX_RETRIES, 10);
  // Allow 0 for no retries, fall back to 3 for invalid values
  return !isNaN(val) && val >= 0 ? val : 3;
})();
const RETRY_DELAY = (() => {
  const val = parseInt(import.meta.env.VITE_SSE_RETRY_DELAY, 10);
  // Must be positive, fall back to 2000ms for invalid values
  return !isNaN(val) && val > 0 ? val : 2000;
})();
const POLLING_INTERVAL = (() => {
  const val = parseInt(import.meta.env.VITE_SSE_POLLING_INTERVAL, 10);
  // Must be positive, fall back to 5000ms for invalid values
  return !isNaN(val) && val > 0 ? val : 5000;
})();

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
  const [serverErrors, setServerErrors] = useState<Map<string, string>>(new Map());

  // Initialize connection on mount
  useEffect(() => {
    let eventSource: EventSource | null = null;
    let pollingInterval: NodeJS.Timeout | null = null;
    let retryTimeout: NodeJS.Timeout | null = null;
    let retryCount = 0;
    let isCleanedUp = false;

    // Polling fallback function
    const startPolling = async () => {
      if (isCleanedUp) return;
      setConnectionMode('polling');

      const poll = async () => {
        if (isCleanedUp) return;
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
      if (!isCleanedUp) {
        pollingInterval = setInterval(poll, POLLING_INTERVAL);
      }
    };

    // SSE connection with retry logic
    const connectSSE = async (): Promise<void> => {
      if (isCleanedUp) return;
      try {
        const baseUrl = await getApiBaseUrl();
        const apiToken = await getApiToken();

        // Include API token as query parameter for authentication
        // EventSource API doesn't support custom headers, so we use query params
        const url = apiToken
          ? `${baseUrl}/api/preview/events?token=${encodeURIComponent(apiToken)}`
          : `${baseUrl}/api/preview/events`;

        eventSource = new EventSource(url);

        eventSource.onopen = () => {
          if (DEBUG) console.log('[SSE] Connection established');
          setConnectionMode('sse');
          retryCount = 0;
        };

        eventSource.onmessage = (event) => {
          try {
            const serverEvent: ServerEvent = JSON.parse(event.data);
            if (DEBUG) console.log('[SSE] Received event:', serverEvent);

            switch (serverEvent.type) {
              case 'initial_state':
                if (serverEvent.active_servers) {
                  setActiveServers(new Set(serverEvent.active_servers));
                }
                break;
              case 'server_started':
                if (serverEvent.project_id) {
                  setActiveServers((prev) => {
                    const next = new Set(prev);
                    next.add(serverEvent.project_id!);
                    return next;
                  });
                  // Clear any previous errors for this server
                  setServerErrors((prev) => {
                    const next = new Map(prev);
                    next.delete(serverEvent.project_id!);
                    return next;
                  });
                }
                break;
              case 'server_stopped':
                if (serverEvent.project_id) {
                  setActiveServers((prev) => {
                    const next = new Set(prev);
                    next.delete(serverEvent.project_id!);
                    return next;
                  });
                  // Clear any errors for stopped server
                  setServerErrors((prev) => {
                    const next = new Map(prev);
                    next.delete(serverEvent.project_id!);
                    return next;
                  });
                }
                break;
              case 'server_error':
                if (serverEvent.project_id && serverEvent.error) {
                  // Sanitize error message before displaying to users
                  const sanitizedError = sanitizeErrorMessage(serverEvent.error);
                  setServerErrors((prev) => {
                    const next = new Map(prev);
                    next.set(serverEvent.project_id!, sanitizedError);
                    return next;
                  });
                }
                break;
            }
          } catch (error) {
            console.error('[SSE] Failed to parse event:', error);
          }
        };

        eventSource.onerror = (error) => {
          if (isCleanedUp) return;

          console.error('[SSE] Connection error:', error);

          // Always close the EventSource to prevent browser auto-reconnect
          // This ensures we have full control over reconnection logic
          eventSource?.close();
          eventSource = null;

          retryCount += 1;

          if (retryCount < MAX_RETRIES) {
            if (DEBUG) console.log(`[SSE] Retrying connection (${retryCount}/${MAX_RETRIES})...`);
            setConnectionMode('connecting');

            // Clear any existing retry timeout to prevent duplicates
            if (retryTimeout) clearTimeout(retryTimeout);

            retryTimeout = setTimeout(() => {
              retryTimeout = null;
              connectSSE();
            }, RETRY_DELAY);
          } else {
            if (DEBUG) console.log('[SSE] Max retries reached, falling back to polling');
            startPolling();
          }
        };
      } catch (error) {
        if (isCleanedUp) return;

        console.error('[SSE] Failed to create EventSource:', error);
        retryCount += 1;

        if (retryCount < MAX_RETRIES) {
          if (DEBUG) console.log(`[SSE] Retrying after creation failure (${retryCount}/${MAX_RETRIES})...`);
          setConnectionMode('connecting');

          // Clear any existing retry timeout to prevent duplicates
          if (retryTimeout) clearTimeout(retryTimeout);

          retryTimeout = setTimeout(() => {
            retryTimeout = null;
            connectSSE();
          }, RETRY_DELAY);
        } else {
          if (DEBUG) console.log('[SSE] Max retries reached after creation failures, falling back to polling');
          startPolling();
        }
      }
    };

    // Start SSE connection
    connectSSE();

    // Cleanup on unmount
    return () => {
      isCleanedUp = true;
      eventSource?.close();
      if (pollingInterval) {
        clearInterval(pollingInterval);
      }
      if (retryTimeout) {
        clearTimeout(retryTimeout);
      }
    };
  }, []); // Empty deps - only run once on mount

  return {
    activeServers,
    connectionMode,
    isConnected: connectionMode !== 'connecting',
    serverErrors,
  };
}
