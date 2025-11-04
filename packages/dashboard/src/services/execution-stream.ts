// ABOUTME: SSE client for streaming execution logs in real-time
// ABOUTME: Handles EventSource connection with auto-reconnect and error recovery

export interface LogEntry {
  id: string;
  execution_id: string;
  timestamp: string;
  log_level: string;
  message: string;
  source?: string;
  metadata?: Record<string, unknown>;
  stack_trace?: string;
  sequence_number: number;
}

export interface ExecutionEvent {
  type: 'log' | 'status' | 'container_status' | 'resource_usage' | 'complete' | 'heartbeat' | 'sync';
  log?: LogEntry;
  execution_id?: string;
  status?: string;
  container_id?: string;
  error_message?: string;
  memory_used_mb?: number;
  cpu_usage_percent?: number;
  success?: boolean;
  timestamp?: string;
  lagged?: number;
}

export interface ExecutionStreamOptions {
  /**
   * Last sequence number received (for resume functionality)
   */
  lastSequence?: number;

  /**
   * Callback for log events
   */
  onLog?: (log: LogEntry) => void;

  /**
   * Callback for status changes
   */
  onStatus?: (status: string, errorMessage?: string) => void;

  /**
   * Callback for completion
   */
  onComplete?: (success: boolean, errorMessage?: string) => void;

  /**
   * Callback for errors
   */
  onError?: (error: Error) => void;

  /**
   * Callback for connection state changes
   */
  onConnectionChange?: (state: 'connecting' | 'connected' | 'disconnected' | 'error') => void;
}

/**
 * Client for streaming execution logs via Server-Sent Events
 */
export class ExecutionStreamClient {
  private eventSource: EventSource | null = null;
  private executionId: string;
  private options: ExecutionStreamOptions;
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;
  private reconnectDelay = 2000; // Start with 2 seconds

  constructor(executionId: string, options: ExecutionStreamOptions = {}) {
    this.executionId = executionId;
    this.options = options;
  }

  /**
   * Connect to the SSE endpoint and start streaming
   */
  connect(): void {
    if (this.eventSource) {
      console.warn('EventSource already connected');
      return;
    }

    this.options.onConnectionChange?.('connecting');

    // Build URL with query parameters
    const url = new URL(
      `/api/sandbox/executions/${this.executionId}/logs/stream`,
      window.location.origin
    );
    if (this.options.lastSequence !== undefined) {
      url.searchParams.set('lastSequence', this.options.lastSequence.toString());
    }

    this.eventSource = new EventSource(url.toString());

    // Handle connection open
    this.eventSource.onopen = () => {
      console.log(`SSE connected for execution ${this.executionId}`);
      this.reconnectAttempts = 0;
      this.reconnectDelay = 2000;
      this.options.onConnectionChange?.('connected');
    };

    // Handle log events
    this.eventSource.addEventListener('log', (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as ExecutionEvent;
        if (data.log) {
          this.options.onLog?.(data.log);
        }
      } catch (error) {
        console.error('Failed to parse log event:', error);
      }
    });

    // Handle status events
    this.eventSource.addEventListener('status', (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as ExecutionEvent;
        if (data.status) {
          this.options.onStatus?.(data.status, data.error_message);
        }
      } catch (error) {
        console.error('Failed to parse status event:', error);
      }
    });

    // Handle complete events
    this.eventSource.addEventListener('complete', (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as ExecutionEvent;
        if (data.success !== undefined) {
          this.options.onComplete?.(data.success, data.error_message);
        }
      } catch (error) {
        console.error('Failed to parse complete event:', error);
      }
    });

    // Handle heartbeat events (just log for debugging)
    this.eventSource.addEventListener('heartbeat', () => {
      // Heartbeat received, connection is alive
    });

    // Handle sync events (client lagged behind)
    this.eventSource.addEventListener('sync', (event: MessageEvent) => {
      try {
        const data = JSON.parse(event.data) as ExecutionEvent;
        console.warn(`Client lagged behind, missed ${data.lagged} events`);
        // Could trigger a full log refetch here if needed
      } catch (error) {
        console.error('Failed to parse sync event:', error);
      }
    });

    // Handle errors
    this.eventSource.onerror = (event) => {
      console.error('SSE error for execution', this.executionId, event);

      this.options.onConnectionChange?.('error');

      // Attempt to reconnect with exponential backoff
      if (this.reconnectAttempts < this.maxReconnectAttempts) {
        this.reconnectAttempts++;
        const delay = this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1);

        console.log(
          `Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts})`
        );

        setTimeout(() => {
          this.disconnect();
          this.connect();
        }, delay);
      } else {
        console.error('Max reconnection attempts reached');
        this.options.onError?.(new Error('Failed to connect to event stream'));
        this.disconnect();
      }
    };
  }

  /**
   * Disconnect from the SSE endpoint
   */
  disconnect(): void {
    if (this.eventSource) {
      this.eventSource.close();
      this.eventSource = null;
      this.options.onConnectionChange?.('disconnected');
    }
  }

  /**
   * Check if currently connected
   */
  isConnected(): boolean {
    return this.eventSource?.readyState === EventSource.OPEN;
  }
}
