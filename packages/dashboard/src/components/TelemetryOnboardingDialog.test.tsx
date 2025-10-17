// ABOUTME: Tests for TelemetryOnboardingDialog component
// ABOUTME: Validates user interactions, form state, and API integration

import React from 'react';
import { describe, it, expect, vi, beforeEach } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';

// Mock telemetry service with simple stubs
vi.mock('@/services/telemetry', () => ({
  telemetryService: {
    completeOnboarding: vi.fn(),
    trackEvent: vi.fn(),
    getStatus: vi.fn(),
    getSettings: vi.fn(),
    updateSettings: vi.fn(),
    deleteAllData: vi.fn(),
    trackError: vi.fn(),
    trackPageView: vi.fn(),
    trackAction: vi.fn(),
    destroy: vi.fn(),
  },
}));

// Mock Dialog components to bypass Radix UI React duplicate instance issue
vi.mock('@/components/ui/dialog', () => ({
  Dialog: ({ children, open }: { children: React.ReactNode; open: boolean }) =>
    open ? <div role="dialog">{children}</div> : null,
  DialogContent: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  DialogHeader: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
  DialogTitle: ({ children }: { children: React.ReactNode }) => (
    <h2>{children}</h2>
  ),
  DialogDescription: ({ children }: { children: React.ReactNode }) => (
    <p>{children}</p>
  ),
  DialogFooter: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

// Mock Button component
vi.mock('@/components/ui/button', () => ({
  Button: ({
    children,
    onClick,
    disabled,
    className,
  }: {
    children: React.ReactNode;
    onClick?: () => void;
    disabled?: boolean;
    className?: string;
  }) => (
    <button onClick={onClick} disabled={disabled} className={className}>
      {children}
    </button>
  ),
}));

// Mock Switch component
vi.mock('@/components/ui/switch', () => ({
  Switch: ({
    id,
    checked,
    onCheckedChange,
    disabled,
  }: {
    id?: string;
    checked: boolean;
    onCheckedChange: (checked: boolean) => void;
    disabled?: boolean;
  }) => (
    <button
      id={id}
      role="switch"
      aria-checked={checked}
      data-state={checked ? 'checked' : 'unchecked'}
      disabled={disabled}
      onClick={() => onCheckedChange(!checked)}
    />
  ),
}));

// Mock Label component
vi.mock('@/components/ui/label', () => ({
  Label: ({
    children,
    htmlFor,
    className,
  }: {
    children: React.ReactNode;
    htmlFor?: string;
    className?: string;
  }) => (
    <label htmlFor={htmlFor} className={className}>
      {children}
    </label>
  ),
}));

// Mock Alert components
vi.mock('@/components/ui/alert', () => ({
  Alert: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  AlertDescription: ({ children }: { children: React.ReactNode }) => (
    <div>{children}</div>
  ),
}));

// Mock lucide-react icons
vi.mock('lucide-react', () => ({
  Shield: () => null,
  Info: () => null,
  CheckCircle: () => null,
  XCircle: () => null,
}));

import { TelemetryOnboardingDialog } from './TelemetryOnboardingDialog';
import { telemetryService } from '@/services/telemetry';

describe('TelemetryOnboardingDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Rendering', () => {
    it('should render with default values', () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      // Check dialog title
      expect(screen.getByText(/Feedback & Privacy/i)).toBeInTheDocument();

      // Check that switches exist
      expect(screen.getByLabelText(/Error reporting/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/^Usage metrics$/i)).toBeInTheDocument();
      expect(
        screen.getByLabelText(/Non-anonymous usage metrics/i)
      ).toBeInTheDocument();

      // Check buttons
      expect(screen.getByRole('button', { name: /No thanks/i })).toBeInTheDocument();
      expect(
        screen.getByRole('button', { name: /Yes, help improve Orkee/i })
      ).toBeInTheDocument();
    });

    it('should show error reporting enabled by default', () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const errorReportingSwitch = screen.getByLabelText(
        /Error reporting/i
      ) as HTMLButtonElement;
      expect(errorReportingSwitch).toHaveAttribute('data-state', 'checked');
    });

    it('should show usage metrics enabled by default', () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const usageMetricsSwitch = screen.getByLabelText(
        /^Usage metrics$/i
      ) as HTMLButtonElement;
      expect(usageMetricsSwitch).toHaveAttribute('data-state', 'checked');
    });

    it('should show non-anonymous metrics disabled by default', () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const nonAnonymousSwitch = screen.getByLabelText(
        /Non-anonymous usage metrics/i
      ) as HTMLButtonElement;
      expect(nonAnonymousSwitch).toHaveAttribute('data-state', 'unchecked');
    });

    it('should show privacy information', () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      expect(
        screen.getByText(/What data do we collect?/i)
      ).toBeInTheDocument();
      expect(
        screen.getByText(/High-level usage metrics/i)
      ).toBeInTheDocument();
      expect(
        screen.getByText(/Performance and error data/i)
      ).toBeInTheDocument();
      expect(screen.getByText(/We do NOT collect/i)).toBeInTheDocument();
    });
  });

  describe('User interactions', () => {
    it('should toggle error reporting switch', async () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const errorReportingSwitch = screen.getByLabelText(/Error reporting/i);

      // Initially checked
      expect(errorReportingSwitch).toHaveAttribute('data-state', 'checked');

      // Toggle off
      fireEvent.click(errorReportingSwitch);
      await waitFor(() => {
        expect(errorReportingSwitch).toHaveAttribute('data-state', 'unchecked');
      });

      // Toggle back on
      fireEvent.click(errorReportingSwitch);
      await waitFor(() => {
        expect(errorReportingSwitch).toHaveAttribute('data-state', 'checked');
      });
    });

    it('should toggle usage metrics switch', async () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const usageMetricsSwitch = screen.getByLabelText(/^Usage metrics$/i);

      // Initially checked
      expect(usageMetricsSwitch).toHaveAttribute('data-state', 'checked');

      // Toggle off
      fireEvent.click(usageMetricsSwitch);
      await waitFor(() => {
        expect(usageMetricsSwitch).toHaveAttribute('data-state', 'unchecked');
      });
    });

    it('should toggle non-anonymous metrics switch', async () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const nonAnonymousSwitch = screen.getByLabelText(
        /Non-anonymous usage metrics/i
      );

      // Initially unchecked
      expect(nonAnonymousSwitch).toHaveAttribute('data-state', 'unchecked');

      // Toggle on
      fireEvent.click(nonAnonymousSwitch);
      await waitFor(() => {
        expect(nonAnonymousSwitch).toHaveAttribute('data-state', 'checked');
      });
    });

    it('should disable non-anonymous switch when usage metrics is off', async () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const usageMetricsSwitch = screen.getByLabelText(/^Usage metrics$/i);
      const nonAnonymousSwitch = screen.getByLabelText(
        /Non-anonymous usage metrics/i
      );

      // Turn off usage metrics
      fireEvent.click(usageMetricsSwitch);

      await waitFor(() => {
        expect(nonAnonymousSwitch).toBeDisabled();
      });
    });
  });

  describe('Form submission', () => {
    it('should call completeOnboarding with correct values when clicking "Yes, help improve Orkee"', async () => {
      const onOpenChange = vi.fn();
      const onComplete = vi.fn();
      vi.mocked(telemetryService.completeOnboarding).mockResolvedValue(undefined);
      vi.mocked(telemetryService.trackEvent).mockResolvedValue(undefined);

      render(
        <TelemetryOnboardingDialog
          open={true}
          onOpenChange={onOpenChange}
          onComplete={onComplete}
        />
      );

      const continueButton = screen.getByRole('button', {
        name: /Yes, help improve Orkee/i,
      });
      fireEvent.click(continueButton);

      await waitFor(() => {
        expect(telemetryService.completeOnboarding).toHaveBeenCalledWith({
          error_reporting: true,
          usage_metrics: true,
          non_anonymous_metrics: false,
        });
        expect(onComplete).toHaveBeenCalled();
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });

    it('should track onboarding_completed event when usage metrics is enabled', async () => {
      const onOpenChange = vi.fn();
      const onComplete = vi.fn();
      vi.mocked(telemetryService.completeOnboarding).mockResolvedValue(undefined);
      vi.mocked(telemetryService.trackEvent).mockResolvedValue(undefined);

      render(
        <TelemetryOnboardingDialog
          open={true}
          onOpenChange={onOpenChange}
          onComplete={onComplete}
        />
      );

      const continueButton = screen.getByRole('button', {
        name: /Yes, help improve Orkee/i,
      });
      fireEvent.click(continueButton);

      await waitFor(() => {
        expect(telemetryService.trackEvent).toHaveBeenCalledWith(
          'onboarding_completed',
          {
            error_reporting: true,
            usage_metrics: true,
            non_anonymous_metrics: false,
          }
        );
      });
    });

    it('should not track onboarding_completed event when usage metrics is disabled', async () => {
      const onOpenChange = vi.fn();
      const onComplete = vi.fn();
      vi.mocked(telemetryService.completeOnboarding).mockResolvedValue(undefined);

      render(
        <TelemetryOnboardingDialog
          open={true}
          onOpenChange={onOpenChange}
          onComplete={onComplete}
        />
      );

      // Disable usage metrics
      const usageMetricsSwitch = screen.getByLabelText(/^Usage metrics$/i);
      fireEvent.click(usageMetricsSwitch);

      // Button text is still "Yes, help improve Orkee" because error reporting is enabled
      const continueButton = screen.getByRole('button', {
        name: /Yes, help improve Orkee/i,
      });
      fireEvent.click(continueButton);

      await waitFor(() => {
        expect(telemetryService.completeOnboarding).toHaveBeenCalledWith({
          error_reporting: true,
          usage_metrics: false,
          non_anonymous_metrics: false,
        });
        expect(telemetryService.trackEvent).not.toHaveBeenCalled();
      });
    });

    it('should call completeOnboarding with all false when clicking "No thanks"', async () => {
      const onOpenChange = vi.fn();
      const onComplete = vi.fn();
      vi.mocked(telemetryService.completeOnboarding).mockResolvedValue(undefined);

      render(
        <TelemetryOnboardingDialog
          open={true}
          onOpenChange={onOpenChange}
          onComplete={onComplete}
        />
      );

      const optOutButton = screen.getByRole('button', { name: /No thanks/i });
      fireEvent.click(optOutButton);

      await waitFor(() => {
        expect(telemetryService.completeOnboarding).toHaveBeenCalledWith({
          error_reporting: false,
          usage_metrics: false,
          non_anonymous_metrics: false,
        });
        expect(onComplete).toHaveBeenCalled();
        expect(onOpenChange).toHaveBeenCalledWith(false);
      });
    });

    it('should handle API errors gracefully', async () => {
      const onOpenChange = vi.fn();
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
      vi.mocked(telemetryService.completeOnboarding).mockRejectedValue(
        new Error('API Error')
      );

      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const continueButton = screen.getByRole('button', {
        name: /Yes, help improve Orkee/i,
      });
      fireEvent.click(continueButton);

      await waitFor(() => {
        expect(consoleErrorSpy).toHaveBeenCalledWith(
          'Failed to save telemetry preferences:',
          expect.any(Error)
        );
      });

      consoleErrorSpy.mockRestore();
    });

    it('should disable buttons during submission', async () => {
      const onOpenChange = vi.fn();
      // Make the API call take time
      vi.mocked(telemetryService.completeOnboarding).mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const continueButton = screen.getByRole('button', {
        name: /Yes, help improve Orkee/i,
      });
      const optOutButton = screen.getByRole('button', { name: /No thanks/i });

      fireEvent.click(continueButton);

      // Buttons should be disabled immediately
      await waitFor(() => {
        expect(continueButton).toBeDisabled();
        expect(optOutButton).toBeDisabled();
      });
    });

    it('should disable switches during submission', async () => {
      const onOpenChange = vi.fn();
      // Make the API call take time
      vi.mocked(telemetryService.completeOnboarding).mockImplementation(
        () => new Promise((resolve) => setTimeout(resolve, 100))
      );

      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      const continueButton = screen.getByRole('button', {
        name: /Yes, help improve Orkee/i,
      });
      const errorReportingSwitch = screen.getByLabelText(/Error reporting/i);
      const usageMetricsSwitch = screen.getByLabelText(/^Usage metrics$/i);

      fireEvent.click(continueButton);

      // Switches should be disabled immediately
      await waitFor(() => {
        expect(errorReportingSwitch).toBeDisabled();
        expect(usageMetricsSwitch).toBeDisabled();
      });
    });

    it('should change button text when all telemetry is disabled', async () => {
      const onOpenChange = vi.fn();
      render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      // Disable all telemetry
      const errorReportingSwitch = screen.getByLabelText(/Error reporting/i);
      const usageMetricsSwitch = screen.getByLabelText(/^Usage metrics$/i);

      fireEvent.click(errorReportingSwitch);
      fireEvent.click(usageMetricsSwitch);

      await waitFor(() => {
        expect(
          screen.getByRole('button', { name: /^Continue$/i })
        ).toBeInTheDocument();
      });
    });
  });

  describe('Dialog visibility', () => {
    it('should not render when open is false', () => {
      const onOpenChange = vi.fn();
      const { container } = render(
        <TelemetryOnboardingDialog open={false} onOpenChange={onOpenChange} />
      );

      // Dialog should not be visible
      expect(container.querySelector('[role="dialog"]')).not.toBeInTheDocument();
    });

    it('should render when open is true', () => {
      const onOpenChange = vi.fn();
      const { container } = render(
        <TelemetryOnboardingDialog open={true} onOpenChange={onOpenChange} />
      );

      // Dialog should be visible
      expect(container.querySelector('[role="dialog"]')).toBeInTheDocument();
    });
  });
});
