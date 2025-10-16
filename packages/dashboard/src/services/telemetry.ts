// ABOUTME: Telemetry service for frontend tracking and API communication
// ABOUTME: Handles telemetry settings, event tracking, and onboarding

import { API_BASE_URL } from './api';

export interface TelemetrySettings {
  error_reporting: boolean;
  usage_metrics: boolean;
  non_anonymous_metrics: boolean;
}

export interface TelemetryStatus {
  first_run: boolean;
  onboarding_completed: boolean;
  telemetry_enabled: boolean;
  settings: TelemetrySettings;
}

interface TelemetryEvent {
  event_name: string;
  event_data?: Record<string, unknown>;
  timestamp: string;
}

class TelemetryService {
  private sessionId: string;
  private eventQueue: TelemetryEvent[] = [];
  private flushInterval: number | null = null;
  private settings: TelemetrySettings | null = null;

  constructor() {
    // Generate a session ID for this session
    this.sessionId = this.generateSessionId();

    // Start periodic flush
    this.startFlushInterval();
  }

  private generateSessionId(): string {
    return `${Date.now()}-${Math.random().toString(36).substr(2, 9)}`;
  }

  private startFlushInterval() {
    // Flush events every 30 seconds
    this.flushInterval = window.setInterval(() => {
      this.flushEvents();
    }, 30000);
  }

  private async flushEvents() {
    if (this.eventQueue.length === 0) {
      return;
    }

    // Events are sent to backend which will handle batching and sending to telemetry endpoint
    // We don't send directly from frontend to avoid CORS issues and to maintain privacy
    const events = [...this.eventQueue];
    this.eventQueue = [];

    try {
      // Send events to backend for processing
      await Promise.all(
        events.map(event =>
          fetch(`${API_BASE_URL}/telemetry/track`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({
              ...event,
              session_id: this.sessionId,
            }),
          }).catch(err => {
            console.debug('Failed to send telemetry event:', err);
          })
        )
      );
    } catch (error) {
      console.debug('Failed to flush telemetry events:', error);
    }
  }

  async getStatus(): Promise<TelemetryStatus> {
    const response = await fetch(`${API_BASE_URL}/telemetry/status`);
    const result = await response.json();

    if (result.success) {
      this.settings = result.data.settings;
      return result.data;
    }

    throw new Error(result.error || 'Failed to get telemetry status');
  }

  async getSettings(): Promise<TelemetrySettings> {
    const response = await fetch(`${API_BASE_URL}/telemetry/settings`);
    const result = await response.json();

    if (result.success) {
      this.settings = result.data;
      return result.data;
    }

    throw new Error(result.error || 'Failed to get telemetry settings');
  }

  async updateSettings(settings: TelemetrySettings): Promise<void> {
    const response = await fetch(`${API_BASE_URL}/telemetry/settings`, {
      method: 'PUT',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    });

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.error || 'Failed to update telemetry settings');
    }

    this.settings = settings;
  }

  async completeOnboarding(settings: TelemetrySettings): Promise<void> {
    const response = await fetch(`${API_BASE_URL}/telemetry/onboarding/complete`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(settings),
    });

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.error || 'Failed to complete onboarding');
    }

    this.settings = settings;
  }

  async deleteAllData(): Promise<void> {
    const response = await fetch(`${API_BASE_URL}/telemetry/data`, {
      method: 'DELETE',
    });

    const result = await response.json();

    if (!result.success) {
      throw new Error(result.error || 'Failed to delete telemetry data');
    }
  }

  // Track a usage event
  async trackEvent(eventName: string, eventData?: Record<string, unknown>) {
    // Check if telemetry is enabled
    if (!this.settings?.usage_metrics) {
      return;
    }

    const event: TelemetryEvent = {
      event_name: eventName,
      event_data: eventData,
      timestamp: new Date().toISOString(),
    };

    // Add to queue for batching
    this.eventQueue.push(event);

    // If queue is getting large, flush immediately
    if (this.eventQueue.length >= 20) {
      this.flushEvents();
    }
  }

  // Track an error event
  async trackError(errorName: string, errorMessage: string, stackTrace?: string) {
    // Check if error reporting is enabled
    if (!this.settings?.error_reporting) {
      return;
    }

    const event: TelemetryEvent = {
      event_name: `error.${errorName}`,
      event_data: {
        message: errorMessage,
        stack_trace: stackTrace,
      },
      timestamp: new Date().toISOString(),
    };

    // Add to queue for batching
    this.eventQueue.push(event);

    // Errors should be sent quickly
    this.flushEvents();
  }

  // Track page views
  trackPageView(pageName: string, properties?: Record<string, unknown>) {
    this.trackEvent(`page_view.${pageName}`, properties);
  }

  // Track user actions
  trackAction(action: string, properties?: Record<string, unknown>) {
    this.trackEvent(`action.${action}`, properties);
  }

  // Cleanup on unmount
  destroy() {
    if (this.flushInterval !== null) {
      clearInterval(this.flushInterval);
      this.flushInterval = null;
    }

    // Final flush
    this.flushEvents();
  }
}

// Export a singleton instance
export const telemetryService = new TelemetryService();

// Cleanup on window unload
if (typeof window !== 'undefined') {
  window.addEventListener('beforeunload', () => {
    telemetryService.destroy();
  });
}