// Error boundary component that automatically tracks errors to telemetry
import React, { Component, ErrorInfo, ReactNode } from 'react';
import { telemetryService } from '@/services/telemetry';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import { AlertCircle } from 'lucide-react';
import { Button } from '@/components/ui/button';

interface Props {
  children: ReactNode;
  fallback?: (error: Error, resetError: () => void) => ReactNode;
}

interface State {
  hasError: boolean;
  error: Error | null;
}

/**
 * Sanitize a stack trace to remove sensitive information before sending to telemetry
 */
function sanitizeStackTrace(stackTrace: string | undefined): string {
  if (!stackTrace) {
    return 'No stack trace available';
  }

  return stackTrace
    // Strip absolute file paths (e.g., /Users/joe/Projects/orkee -> orkee)
    .replace(/\/[^/\s]+(?:\/[^/\s]+)*\/([\w-]+\/)/g, '$1')
    // Remove query parameters from URLs (e.g., ?token=abc123)
    .replace(/\?[^\s)]+/g, '')
    // Redact potential email addresses
    .replace(/\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b/g, '[EMAIL]')
    // Redact common API key prefixes (PostHog, Stripe, etc.)
    .replace(/\b(phc_|phx_|sk_|pk_|rk_)[a-zA-Z0-9_-]{20,}\b/g, '[API_KEY]')
    // Redact base64-encoded tokens (40+ chars)
    .replace(/\b[A-Za-z0-9+/]{40,}={0,2}\b/g, '[BASE64_TOKEN]')
    // Keep only relative file paths and line numbers
    .trim();
}

export class TelemetryErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = {
      hasError: false,
      error: null,
    };
  }

  static getDerivedStateFromError(error: Error): State {
    // Update state so the next render will show the fallback UI
    return {
      hasError: true,
      error,
    };
  }

  componentDidCatch(error: Error, errorInfo: ErrorInfo) {
    // Sanitize stack trace before sending to telemetry to protect sensitive data
    const rawStackTrace = errorInfo.componentStack || error.stack;
    const sanitizedStackTrace = sanitizeStackTrace(rawStackTrace);

    telemetryService.trackError(
      'react_error_boundary',
      error.message,
      sanitizedStackTrace
    );

    // Log to console for development (keep full trace for local debugging)
    console.error('Error caught by boundary:', error, errorInfo);
  }

  resetError = () => {
    this.setState({
      hasError: false,
      error: null,
    });
  };

  render() {
    if (this.state.hasError && this.state.error) {
      // Custom fallback if provided
      if (this.props.fallback) {
        return this.props.fallback(this.state.error, this.resetError);
      }

      // Default fallback UI
      return (
        <div className="flex items-center justify-center min-h-screen p-4">
          <Alert variant="destructive" className="max-w-2xl">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Something went wrong</AlertTitle>
            <AlertDescription className="mt-2 space-y-4">
              <p className="text-sm">
                An unexpected error occurred. The error has been logged and will help us improve the application.
              </p>
              <p className="text-sm font-mono bg-muted p-2 rounded">
                {this.state.error.message}
              </p>
              <div className="flex gap-2">
                <Button onClick={this.resetError} variant="outline">
                  Try Again
                </Button>
                <Button
                  onClick={() => window.location.reload()}
                  variant="default"
                >
                  Reload Page
                </Button>
              </div>
            </AlertDescription>
          </Alert>
        </div>
      );
    }

    return this.props.children;
  }
}
