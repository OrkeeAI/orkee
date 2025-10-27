// ABOUTME: Validation panel displaying errors and warnings for PRD quality checks
// ABOUTME: Shows validation issues organized by severity with section references

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
import {
  Shield,
  AlertCircle,
  AlertTriangle,
  CheckCircle2,
} from 'lucide-react';
import type { ValidationResponse } from '@/services/ideate';

interface ValidationPanelProps {
  validation: ValidationResponse;
}

export function ValidationPanel({ validation }: ValidationPanelProps) {
  const { status, errors, warnings } = validation;

  const getStatusIcon = () => {
    if (status === 'invalid') {
      return <AlertCircle className="h-5 w-5 text-red-600" />;
    }
    if (status === 'warnings') {
      return <AlertTriangle className="h-5 w-5 text-yellow-600" />;
    }
    return <CheckCircle2 className="h-5 w-5 text-green-600" />;
  };

  const getStatusText = () => {
    if (status === 'invalid') {
      return 'Validation Failed';
    }
    if (status === 'warnings') {
      return 'Validation Warnings';
    }
    return 'Validation Passed';
  };

  const getStatusColor = () => {
    if (status === 'invalid') return 'text-red-600';
    if (status === 'warnings') return 'text-yellow-600';
    return 'text-green-600';
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            <Shield className="h-5 w-5" />
            PRD Validation
          </CardTitle>
          <div className="flex items-center gap-2">
            {getStatusIcon()}
            <span className={`font-semibold ${getStatusColor()}`}>
              {getStatusText()}
            </span>
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Summary Stats */}
        <div className="flex gap-4">
          <div className="flex items-center gap-2 rounded-lg border px-3 py-2">
            <AlertCircle className="h-4 w-4 text-red-600" />
            <div className="flex flex-col">
              <span className="text-xs text-muted-foreground">Errors</span>
              <span className="text-lg font-semibold">{errors.length}</span>
            </div>
          </div>
          <div className="flex items-center gap-2 rounded-lg border px-3 py-2">
            <AlertTriangle className="h-4 w-4 text-yellow-600" />
            <div className="flex flex-col">
              <span className="text-xs text-muted-foreground">Warnings</span>
              <span className="text-lg font-semibold">{warnings.length}</span>
            </div>
          </div>
        </div>

        {/* No Issues - Success State */}
        {errors.length === 0 && warnings.length === 0 && (
          <Alert className="border-green-600 bg-green-50">
            <CheckCircle2 className="h-4 w-4 text-green-600" />
            <AlertTitle className="text-green-900">All validation checks passed!</AlertTitle>
            <AlertDescription className="text-green-800">
              Your PRD meets all quality requirements and is ready for export.
            </AlertDescription>
          </Alert>
        )}

        {/* Errors Section */}
        {errors.length > 0 && (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <AlertCircle className="h-4 w-4 text-red-600" />
              <h3 className="font-semibold text-red-900">
                Errors ({errors.length})
              </h3>
            </div>
            <div className="space-y-2">
              {errors.map((error, idx) => (
                <Alert key={idx} variant="destructive">
                  <div className="flex items-start justify-between gap-2">
                    <div className="space-y-1 flex-1">
                      <AlertTitle className="text-sm font-semibold">
                        {error.rule}
                      </AlertTitle>
                      <AlertDescription className="text-sm">
                        {error.message}
                      </AlertDescription>
                      {error.section && (
                        <Badge variant="outline" className="mt-1">
                          Section: {error.section}
                        </Badge>
                      )}
                    </div>
                  </div>
                </Alert>
              ))}
            </div>
          </div>
        )}

        {/* Warnings Section */}
        {warnings.length > 0 && (
          <div className="space-y-3">
            <div className="flex items-center gap-2">
              <AlertTriangle className="h-4 w-4 text-yellow-600" />
              <h3 className="font-semibold text-yellow-900">
                Warnings ({warnings.length})
              </h3>
            </div>
            <div className="space-y-2">
              {warnings.map((warning, idx) => (
                <Alert key={idx} className="border-yellow-600 bg-yellow-50">
                  <div className="flex items-start justify-between gap-2">
                    <div className="space-y-1 flex-1">
                      <AlertTitle className="text-sm font-semibold text-yellow-900">
                        {warning.rule}
                      </AlertTitle>
                      <AlertDescription className="text-sm text-yellow-800">
                        {warning.message}
                      </AlertDescription>
                      {warning.section && (
                        <Badge variant="outline" className="mt-1">
                          Section: {warning.section}
                        </Badge>
                      )}
                    </div>
                  </div>
                </Alert>
              ))}
            </div>
          </div>
        )}

        {/* Guidance */}
        {errors.length > 0 && (
          <Alert>
            <Shield className="h-4 w-4" />
            <AlertTitle>Action Required</AlertTitle>
            <AlertDescription>
              Please address all validation errors before exporting your PRD. You can regenerate
              sections or fill skipped sections to resolve these issues.
            </AlertDescription>
          </Alert>
        )}
        {errors.length === 0 && warnings.length > 0 && (
          <Alert>
            <Shield className="h-4 w-4" />
            <AlertTitle>Review Recommended</AlertTitle>
            <AlertDescription>
              Your PRD can be exported, but reviewing these warnings is recommended for best quality.
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
}
