// ABOUTME: Telemetry onboarding dialog shown on first run
// ABOUTME: Allows users to opt-in to different levels of telemetry with clear explanations

import React, { useState } from 'react';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Button } from '@/components/ui/button';
import { Switch } from '@/components/ui/switch';
import { Label } from '@/components/ui/label';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Shield, Info, CheckCircle, XCircle } from 'lucide-react';
import { telemetryService } from '@/services/telemetry';

interface TelemetryOnboardingDialogProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  onComplete?: () => void;
}

export function TelemetryOnboardingDialog({
  open,
  onOpenChange,
  onComplete,
}: TelemetryOnboardingDialogProps) {
  const [errorReporting, setErrorReporting] = useState(true);
  const [usageMetrics, setUsageMetrics] = useState(true);
  const [nonAnonymousMetrics, setNonAnonymousMetrics] = useState(false);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleContinue = async () => {
    setIsSubmitting(true);
    try {
      await telemetryService.completeOnboarding({
        error_reporting: errorReporting,
        usage_metrics: usageMetrics,
        non_anonymous_metrics: nonAnonymousMetrics,
      });

      // Track the onboarding completion event (if enabled)
      if (usageMetrics) {
        await telemetryService.trackEvent('onboarding_completed', {
          error_reporting: errorReporting,
          usage_metrics: usageMetrics,
          non_anonymous_metrics: nonAnonymousMetrics,
        });
      }

      onComplete?.();
      onOpenChange(false);
    } catch (error) {
      console.error('Failed to save telemetry preferences:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  const handleOptOut = async () => {
    setIsSubmitting(true);
    try {
      await telemetryService.completeOnboarding({
        error_reporting: false,
        usage_metrics: false,
        non_anonymous_metrics: false,
      });

      onComplete?.();
      onOpenChange(false);
    } catch (error) {
      console.error('Failed to opt out of telemetry:', error);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-2xl">
        <DialogHeader>
          <div className="flex items-center gap-2">
            <Shield className="h-6 w-6 text-primary" />
            <DialogTitle className="text-2xl">Feedback & Privacy</DialogTitle>
          </div>
          <DialogDescription className="text-base mt-3">
            Help us improve Orkee by sharing usage data and allowing us to contact you if needed.
          </DialogDescription>
        </DialogHeader>

        <div className="space-y-6 py-4">
          {/* Data Collection Info */}
          <Alert>
            <Info className="h-4 w-4" />
            <AlertDescription>
              <div className="space-y-3">
                <h4 className="font-semibold">What data do we collect?</h4>

                <div className="space-y-2">
                  <div className="flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-600 mt-0.5 flex-shrink-0" />
                    <div>
                      <span className="font-medium">High-level usage metrics</span>
                      <p className="text-sm text-muted-foreground">
                        Number of projects created, features used, session duration
                      </p>
                    </div>
                  </div>

                  <div className="flex items-start gap-2">
                    <CheckCircle className="h-4 w-4 text-green-600 mt-0.5 flex-shrink-0" />
                    <div>
                      <span className="font-medium">Performance and error data</span>
                      <p className="text-sm text-muted-foreground">
                        Application crashes, response times, technical issues
                      </p>
                    </div>
                  </div>

                  <div className="flex items-start gap-2">
                    <XCircle className="h-4 w-4 text-red-600 mt-0.5 flex-shrink-0" />
                    <div>
                      <span className="font-medium">We do NOT collect</span>
                      <p className="text-sm text-muted-foreground">
                        Project names, file paths, code snippets, personal data, or API keys
                      </p>
                    </div>
                  </div>
                </div>

                <p className="text-sm">
                  This helps us prioritize improvements. You can change these preferences anytime in Settings.
                </p>
              </div>
            </AlertDescription>
          </Alert>

          {/* Telemetry Options */}
          <div className="space-y-4">
            {/* Error Reporting */}
            <div className="flex items-center justify-between p-4 border rounded-lg">
              <div className="space-y-0.5">
                <Label htmlFor="error-reporting" className="text-base font-medium">
                  Error reporting
                </Label>
                <p className="text-sm text-muted-foreground">
                  Toggle reporting of application crashes and errors.
                </p>
              </div>
              <Switch
                id="error-reporting"
                checked={errorReporting}
                onCheckedChange={setErrorReporting}
                disabled={isSubmitting}
              />
            </div>

            {/* Usage Metrics */}
            <div className="flex items-center justify-between p-4 border rounded-lg">
              <div className="space-y-0.5">
                <Label htmlFor="usage-metrics" className="text-base font-medium">
                  Usage metrics
                </Label>
                <p className="text-sm text-muted-foreground">
                  Toggle sharing of usage statistics.
                </p>
              </div>
              <Switch
                id="usage-metrics"
                checked={usageMetrics}
                onCheckedChange={setUsageMetrics}
                disabled={isSubmitting}
              />
            </div>

            {/* Non-anonymous Usage Metrics */}
            <div className="flex items-center justify-between p-4 border rounded-lg">
              <div className="space-y-0.5">
                <Label htmlFor="non-anonymous" className="text-base font-medium">
                  Non-anonymous usage metrics
                </Label>
                <p className="text-sm text-muted-foreground">
                  Toggle sharing of identifiable usage statistics.
                </p>
              </div>
              <Switch
                id="non-anonymous"
                checked={nonAnonymousMetrics}
                onCheckedChange={setNonAnonymousMetrics}
                disabled={isSubmitting || !usageMetrics}
              />
            </div>
          </div>
        </div>

        <DialogFooter className="flex-col sm:flex-row gap-2">
          <Button
            variant="outline"
            onClick={handleOptOut}
            disabled={isSubmitting}
            className="w-full sm:w-auto"
          >
            <XCircle className="mr-2 h-4 w-4" />
            No thanks
          </Button>
          <Button
            onClick={handleContinue}
            disabled={isSubmitting}
            className="w-full sm:w-auto"
          >
            <CheckCircle className="mr-2 h-4 w-4" />
            {errorReporting || usageMetrics
              ? 'Yes, help improve Orkee'
              : 'Continue'}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}