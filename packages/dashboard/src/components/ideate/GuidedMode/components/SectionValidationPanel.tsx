// ABOUTME: Validation panel for Guided Mode sections showing quality score and issues
// ABOUTME: Displays after each section is completed, before proceeding to next

import { useState, useEffect } from 'react';
import { CheckCircle, XCircle, AlertTriangle, RefreshCw } from 'lucide-react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Progress } from '@/components/ui/progress';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { ideateService } from '@/services/ideate';
import type { SectionValidationResult } from '@/services/ideate';
import { toast } from 'sonner';

interface SectionValidationPanelProps {
  sessionId: string;
  sectionName: string;
  sectionContent: string;
  onContinue: () => void;
  onRegenerate: () => void;
}

export function SectionValidationPanel({
  sessionId,
  sectionName,
  sectionContent,
  onContinue,
  onRegenerate,
}: SectionValidationPanelProps) {
  const [validation, setValidation] = useState<SectionValidationResult | null>(null);
  const [isValidating, setIsValidating] = useState(false);

  useEffect(() => {
    validateSection();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [sectionContent]);

  const validateSection = async () => {
    if (!sectionContent.trim()) {
      setValidation(null);
      return;
    }

    try {
      setIsValidating(true);
      const result = await ideateService.validateSection(sessionId, sectionName, sectionContent);
      setValidation(result);

      // Store validation feedback
      await ideateService.storeValidationFeedback(sessionId, {
        section_name: sectionName,
        validation_status: result.is_valid ? 'approved' : 'rejected',
        quality_score: result.quality_score,
      });
    } catch (error) {
      console.error('Failed to validate section:', error);
      toast.error('Failed to validate section');
    } finally {
      setIsValidating(false);
    }
  };

  const getQualityColor = (score: number) => {
    if (score >= 80) return 'text-green-600';
    if (score >= 60) return 'text-yellow-600';
    return 'text-red-600';
  };

  const getQualityLabel = (score: number) => {
    if (score >= 80) return 'Excellent';
    if (score >= 60) return 'Good';
    return 'Needs Improvement';
  };

  if (isValidating) {
    return (
      <Card>
        <CardContent className="py-6">
          <div className="flex items-center justify-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm text-muted-foreground">Validating section quality...</span>
          </div>
        </CardContent>
      </Card>
    );
  }

  if (!validation) {
    return null;
  }

  return (
    <Card className={validation.is_valid ? 'border-green-200' : 'border-yellow-200'}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">Section Quality</CardTitle>
          <Badge variant={validation.is_valid ? 'default' : 'secondary'}>
            {validation.is_valid ? 'Valid' : 'Needs Attention'}
          </Badge>
        </div>
      </CardHeader>
      <CardContent className="space-y-4">
        {/* Quality Score */}
        <div className="space-y-2">
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">Quality Score</span>
            <span className={`text-sm font-bold ${getQualityColor(validation.quality_score)}`}>
              {validation.quality_score}/100 - {getQualityLabel(validation.quality_score)}
            </span>
          </div>
          <Progress value={validation.quality_score} className="h-2" />
        </div>

        {/* Issues */}
        {validation.issues.length > 0 && (
          <Alert variant="destructive">
            <XCircle className="h-4 w-4" />
            <AlertDescription>
              <div className="font-semibold mb-2">Issues Found:</div>
              <ul className="list-disc pl-5 space-y-1">
                {validation.issues.map((issue, index) => (
                  <li key={index} className="text-sm">{issue}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}

        {/* Suggestions */}
        {validation.suggestions.length > 0 && (
          <Alert>
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <div className="font-semibold mb-2">Suggestions:</div>
              <ul className="list-disc pl-5 space-y-1">
                {validation.suggestions.map((suggestion, index) => (
                  <li key={index} className="text-sm">{suggestion}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}

        {/* Action Buttons */}
        <div className="flex items-center justify-between pt-4 border-t">
          {validation.quality_score < 60 ? (
            <>
              <Button variant="outline" onClick={onRegenerate} className="gap-2">
                <RefreshCw className="h-4 w-4" />
                Regenerate
              </Button>
              <Button variant="outline" onClick={onContinue}>
                Continue Anyway
              </Button>
            </>
          ) : (
            <>
              <Button variant="outline" onClick={onRegenerate} className="gap-2">
                <RefreshCw className="h-4 w-4" />
                Regenerate
              </Button>
              <Button onClick={onContinue} className="gap-2">
                <CheckCircle className="h-4 w-4" />
                Continue
              </Button>
            </>
          )}
        </div>
      </CardContent>
    </Card>
  );
}
