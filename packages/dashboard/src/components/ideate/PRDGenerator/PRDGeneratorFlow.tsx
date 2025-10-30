// ABOUTME: Main orchestrator component for PRD generation and export
// ABOUTME: Manages generation flow, preview, validation, and export for all ideation modes

import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { Badge } from '@/components/ui/badge';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import {
  FileText,
  Download,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Loader2,
  Eye,
  History,
  Shield
} from 'lucide-react';
import {
  useGeneratePRD,
  usePRDPreview,
  useCompleteness,
  useValidatePRD,
  useGenerationHistory,
} from '@/hooks/useIdeate';
import { CompletenessIndicator } from './CompletenessIndicator';
import { PRDPreview } from './PRDPreview';
import { ValidationPanel } from './ValidationPanel';
import { ExportDialog } from './ExportDialog';
import { GenerationHistory } from './GenerationHistory';
import { SectionFillDialog } from './SectionFillDialog';
import { RegenerateTemplateDialog } from './RegenerateTemplateDialog';
import type { IdeateSession } from '@/services/ideate';

interface PRDGeneratorFlowProps {
  session: IdeateSession;
}

export function PRDGeneratorFlow({ session }: PRDGeneratorFlowProps) {
  const [showExportDialog, setShowExportDialog] = useState(false);
  const [showFillDialog, setShowFillDialog] = useState(false);
  const [showRegenerateDialog, setShowRegenerateDialog] = useState(false);
  const [activeTab, setActiveTab] = useState('preview');

  // React Query hooks
  const generateMutation = useGeneratePRD(session.id);
  const { data: preview, isLoading: previewLoading } = usePRDPreview(session.id);
  const { data: completeness, isLoading: completenessLoading } = useCompleteness(session.id);
  const { data: validation } = useValidatePRD(session.id);
  const { data: history } = useGenerationHistory(session.id);

  const handleGenerate = async () => {
    try {
      await generateMutation.mutateAsync({ includeSkipped: false });
    } catch (error) {
      console.error('Failed to generate PRD:', error);
    }
  };

  const isReady = session.status === 'ready_for_prd' || session.status === 'completed';
  const hasSkippedSections = (completeness?.skipped_sections || 0) > 0;
  const hasErrors = (validation?.errors?.length || 0) > 0;
  const hasWarnings = (validation?.warnings?.length || 0) > 0;

  return (
    <div className="space-y-6">
      {/* Header with Status */}
      <Card>
        <CardHeader>
          <div className="flex items-start justify-between">
            <div className="space-y-1">
              <CardTitle className="flex items-center gap-2">
                <FileText className="h-5 w-5" />
                PRD Generation
              </CardTitle>
              <CardDescription>
                Generate a comprehensive Product Requirements Document from your ideation session
              </CardDescription>
            </div>
            <div className="flex items-center gap-2">
              {isReady ? (
                <Badge variant="default" className="gap-1">
                  <CheckCircle2 className="h-3 w-3" />
                  Ready
                </Badge>
              ) : (
                <Badge variant="secondary" className="gap-1">
                  <AlertCircle className="h-3 w-3" />
                  In Progress
                </Badge>
              )}
            </div>
          </div>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Completeness Indicator */}
          {!completenessLoading && completeness && (
            <CompletenessIndicator completeness={completeness} />
          )}

          {/* Warning: Not Ready */}
          {!isReady && (
            <Alert>
              <AlertCircle className="h-4 w-4" />
              <AlertDescription>
                Complete the required sections before generating your PRD.
                {hasSkippedSections && ' You have skipped sections that can be AI-filled.'}
              </AlertDescription>
            </Alert>
          )}

          {/* Validation Warnings/Errors */}
          {validation && (hasErrors || hasWarnings) && (
            <Alert variant={hasErrors ? 'destructive' : 'default'}>
              <Shield className="h-4 w-4" />
              <AlertDescription>
                {hasErrors
                  ? `${validation.errors.length} validation error(s) found. Please review before exporting.`
                  : `${validation.warnings.length} warning(s) found. Review recommended before exporting.`
                }
              </AlertDescription>
            </Alert>
          )}

          {/* Action Buttons */}
          <div className="flex flex-wrap gap-2">
            <Button
              onClick={handleGenerate}
              disabled={!isReady || generateMutation.isPending}
              className="gap-2"
            >
              {generateMutation.isPending ? (
                <>
                  <Loader2 className="h-4 w-4 animate-spin" />
                  Generating...
                </>
              ) : (
                <>
                  <RefreshCw className="h-4 w-4" />
                  Generate PRD
                </>
              )}
            </Button>

            {hasSkippedSections && (
              <Button
                variant="outline"
                onClick={() => setShowFillDialog(true)}
                className="gap-2"
              >
                <RefreshCw className="h-4 w-4" />
                Fill Skipped Sections
              </Button>
            )}

            <Button
              variant="outline"
              onClick={() => setShowRegenerateDialog(true)}
              disabled={!preview}
              className="gap-2"
            >
              <RefreshCw className="h-4 w-4" />
              Regenerate with Template
            </Button>

            <Button
              variant="outline"
              onClick={() => setShowExportDialog(true)}
              disabled={!preview}
              className="gap-2"
            >
              <Download className="h-4 w-4" />
              Export
            </Button>
          </div>
        </CardContent>
      </Card>

      {/* Tabs: Preview, Validation, History */}
      <Tabs value={activeTab} onValueChange={setActiveTab} className="space-y-4">
        <TabsList>
          <TabsTrigger value="preview" className="gap-2">
            <Eye className="h-4 w-4" />
            Preview
          </TabsTrigger>
          <TabsTrigger value="validation" className="gap-2">
            <Shield className="h-4 w-4" />
            Validation
            {hasErrors && (
              <Badge variant="destructive" className="ml-1">
                {validation?.errors.length}
              </Badge>
            )}
            {!hasErrors && hasWarnings && (
              <Badge variant="secondary" className="ml-1">
                {validation?.warnings.length}
              </Badge>
            )}
          </TabsTrigger>
          <TabsTrigger value="history" className="gap-2">
            <History className="h-4 w-4" />
            History
            {history && history.length > 0 && (
              <Badge variant="secondary" className="ml-1">
                {history.length}
              </Badge>
            )}
          </TabsTrigger>
        </TabsList>

        <TabsContent value="preview" className="space-y-4">
          {previewLoading ? (
            <Card>
              <CardContent className="flex items-center justify-center py-12">
                <div className="flex flex-col items-center gap-2 text-muted-foreground">
                  <Loader2 className="h-8 w-8 animate-spin" />
                  <p>Loading preview...</p>
                </div>
              </CardContent>
            </Card>
          ) : preview ? (
            <PRDPreview
              data={preview}
              sessionId={session.id}
              onRegenerateSection={(section) => {
                console.log('Regenerate section:', section);
                // Will be handled by PRDPreview component
              }}
            />
          ) : (
            <Card>
              <CardContent className="flex items-center justify-center py-12">
                <div className="flex flex-col items-center gap-2 text-muted-foreground">
                  <FileText className="h-8 w-8" />
                  <p>No preview available. Generate your PRD to see a preview.</p>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="validation" className="space-y-4">
          {validation ? (
            <ValidationPanel validation={validation} />
          ) : (
            <Card>
              <CardContent className="flex items-center justify-center py-12">
                <div className="flex flex-col items-center gap-2 text-muted-foreground">
                  <Shield className="h-8 w-8" />
                  <p>No validation results available yet.</p>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="history" className="space-y-4">
          {history && history.length > 0 ? (
            <GenerationHistory history={history} />
          ) : (
            <Card>
              <CardContent className="flex items-center justify-center py-12">
                <div className="flex flex-col items-center gap-2 text-muted-foreground">
                  <History className="h-8 w-8" />
                  <p>No generation history yet. Generate your first PRD to see history.</p>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>
      </Tabs>

      {/* Dialogs */}
      {showRegenerateDialog && preview && (
        <RegenerateTemplateDialog
          sessionId={session.id}
          onSuccess={() => {
            // Preview will auto-refresh via React Query
          }}
          onClose={() => setShowRegenerateDialog(false)}
        />
      )}

      {showExportDialog && preview && (
        <ExportDialog
          sessionId={session.id}
          onClose={() => setShowExportDialog(false)}
        />
      )}

      {showFillDialog && completeness && (
        <SectionFillDialog
          sessionId={session.id}
          skippedSections={preview?.skippedSections || []}
          onClose={() => setShowFillDialog(false)}
        />
      )}
    </div>
  );
}
