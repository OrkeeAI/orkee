// ABOUTME: Telemetry context provider for managing telemetry state
// ABOUTME: Provides hooks for tracking events and managing telemetry settings

import React, { createContext, useContext, useEffect, useState, useCallback } from 'react';
import { telemetryService, TelemetrySettings, TelemetryStatus } from '@/services/telemetry';

interface TelemetryContextType {
  status: TelemetryStatus | null;
  settings: TelemetrySettings | null;
  loading: boolean;
  error: string | null;
  shouldShowOnboarding: boolean;
  updateSettings: (settings: TelemetrySettings) => Promise<void>;
  completeOnboarding: (settings: TelemetrySettings) => Promise<void>;
  deleteAllData: () => Promise<void>;
  trackEvent: (eventName: string, eventData?: Record<string, unknown>) => void;
  trackError: (errorName: string, errorMessage: string, stackTrace?: string) => void;
  trackPageView: (pageName: string, properties?: Record<string, unknown>) => void;
  trackAction: (action: string, properties?: Record<string, unknown>) => void;
  refreshStatus: () => Promise<void>;
}

const TelemetryContext = createContext<TelemetryContextType | undefined>(undefined);

export function TelemetryProvider({ children }: { children: React.ReactNode }) {
  const [status, setStatus] = useState<TelemetryStatus | null>(null);
  const [settings, setSettings] = useState<TelemetrySettings | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const fetchStatus = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      const telemetryStatus = await telemetryService.getStatus();
      setStatus(telemetryStatus);
      setSettings(telemetryStatus.settings);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to load telemetry status');
      console.error('Failed to fetch telemetry status:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  const updateSettings = useCallback(async (newSettings: TelemetrySettings) => {
    try {
      await telemetryService.updateSettings(newSettings);
      setSettings(newSettings);
      if (status) {
        setStatus({
          ...status,
          settings: newSettings,
        });
      }
    } catch (err) {
      console.error('Failed to update telemetry settings:', err);
      throw err;
    }
  }, [status]);

  const completeOnboarding = useCallback(async (newSettings: TelemetrySettings) => {
    try {
      await telemetryService.completeOnboarding(newSettings);
      setSettings(newSettings);
      if (status) {
        setStatus({
          ...status,
          first_run: false,
          onboarding_completed: true,
          settings: newSettings,
        });
      }
    } catch (err) {
      console.error('Failed to complete onboarding:', err);
      throw err;
    }
  }, [status]);

  const deleteAllData = useCallback(async () => {
    try {
      await telemetryService.deleteAllData();
    } catch (err) {
      console.error('Failed to delete telemetry data:', err);
      throw err;
    }
  }, []);

  const trackEvent = useCallback((eventName: string, eventData?: Record<string, unknown>) => {
    telemetryService.trackEvent(eventName, eventData);
  }, []);

  const trackError = useCallback((errorName: string, errorMessage: string, stackTrace?: string) => {
    telemetryService.trackError(errorName, errorMessage, stackTrace);
  }, []);

  const trackPageView = useCallback((pageName: string, properties?: Record<string, unknown>) => {
    telemetryService.trackPageView(pageName, properties);
  }, []);

  const trackAction = useCallback((action: string, properties?: Record<string, unknown>) => {
    telemetryService.trackAction(action, properties);
  }, []);

  useEffect(() => {
    fetchStatus();
  }, [fetchStatus]);

  const shouldShowOnboarding = !loading && status?.first_run && !status?.onboarding_completed;

  const value: TelemetryContextType = {
    status,
    settings,
    loading,
    error,
    shouldShowOnboarding,
    updateSettings,
    completeOnboarding,
    deleteAllData,
    trackEvent,
    trackError,
    trackPageView,
    trackAction,
    refreshStatus: fetchStatus,
  };

  return (
    <TelemetryContext.Provider value={value}>
      {children}
    </TelemetryContext.Provider>
  );
}

// eslint-disable-next-line react-refresh/only-export-components
export function useTelemetry() {
  const context = useContext(TelemetryContext);
  if (context === undefined) {
    throw new Error('useTelemetry must be used within a TelemetryProvider');
  }
  return context;
}

// Higher-order component for tracking page views
// eslint-disable-next-line react-refresh/only-export-components
export function withPageTracking<P extends object>(
  Component: React.ComponentType<P>,
  pageName: string
) {
  return function TrackedComponent(props: P) {
    const { trackPageView } = useTelemetry();

    useEffect(() => {
      trackPageView(pageName);
    }, [trackPageView]);

    return <Component {...props} />;
  };
}