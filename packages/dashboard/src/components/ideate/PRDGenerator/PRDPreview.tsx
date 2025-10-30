// ABOUTME: Preview component for aggregated PRD data with section regeneration
// ABOUTME: Displays all PRD sections with expand/collapse and per-section regeneration

import { useState } from 'react';
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from '@/components/ui/accordion';
import {
  FileText,
  RefreshCw,
  CheckCircle2,
  AlertCircle,
  Info,
  Loader2,
} from 'lucide-react';
import { useRegenerateSection } from '@/hooks/useIdeate';
import type { AggregatedPRDData } from '@/services/ideate';

interface PRDPreviewProps {
  data: AggregatedPRDData;
  sessionId: string;
  onRegenerateSection: (section: string) => void;
}

interface SectionData {
  title: string;
  key: keyof AggregatedPRDData;
  status: 'completed' | 'skipped' | 'ai-filled' | 'empty';
  content: unknown;
}

export function PRDPreview({ data, sessionId, onRegenerateSection }: PRDPreviewProps) {
  const [regeneratingSection, setRegeneratingSection] = useState<string | null>(null);
  const regenerateMutation = useRegenerateSection(sessionId);

  const handleRegenerate = async (sectionKey: string) => {
    setRegeneratingSection(sectionKey);
    onRegenerateSection(sectionKey);
    try {
      await regenerateMutation.mutateAsync(sectionKey);
    } finally {
      setRegeneratingSection(null);
    }
  };

  const sections: SectionData[] = [
    {
      title: 'Overview',
      key: 'overview',
      status: data.overview ? 'completed' : 'empty',
      content: data.overview,
    },
    {
      title: 'UX & Design',
      key: 'ux',
      status: data.ux ? 'completed' : 'empty',
      content: data.ux,
    },
    {
      title: 'Technical Requirements',
      key: 'technical',
      status: data.technical ? 'completed' : 'empty',
      content: data.technical,
    },
    {
      title: 'Roadmap',
      key: 'roadmap',
      status: data.roadmap ? 'completed' : 'empty',
      content: data.roadmap,
    },
    {
      title: 'Dependencies',
      key: 'dependencies',
      status: data.dependencies ? 'completed' : 'empty',
      content: data.dependencies,
    },
    {
      title: 'Risks & Mitigation',
      key: 'risks',
      status: data.risks ? 'completed' : 'empty',
      content: data.risks,
    },
    {
      title: 'Research & Insights',
      key: 'research',
      status: data.research ? 'completed' : 'empty',
      content: data.research,
    },
    {
      title: 'Expert Roundtable Insights',
      key: 'roundtableInsights',
      status: data.roundtableInsights && data.roundtableInsights.length > 0 ? 'completed' : 'empty',
      content: data.roundtableInsights,
    },
  ];

  const getStatusBadge = (status: SectionData['status']) => {
    switch (status) {
      case 'completed':
        return (
          <Badge variant="default" className="gap-1">
            <CheckCircle2 className="h-3 w-3" />
            Complete
          </Badge>
        );
      case 'skipped':
        return (
          <Badge variant="secondary" className="gap-1">
            <AlertCircle className="h-3 w-3" />
            Skipped
          </Badge>
        );
      case 'ai-filled':
        return (
          <Badge variant="outline" className="gap-1">
            <Info className="h-3 w-3" />
            AI-Filled
          </Badge>
        );
      case 'empty':
        return (
          <Badge variant="secondary" className="gap-1">
            <AlertCircle className="h-3 w-3" />
            Empty
          </Badge>
        );
    }
  };

  const renderSectionContent = (section: SectionData) => {
    if (!section.content) {
      return (
        <Alert>
          <Info className="h-4 w-4" />
          <AlertDescription>
            This section is empty. Generate the PRD or fill skipped sections to populate it.
          </AlertDescription>
        </Alert>
      );
    }

    // For roundtable insights (array)
    if (section.key === 'roundtableInsights' && Array.isArray(section.content)) {
      return (
        <div className="space-y-3">
          {section.content.map((insight: unknown, idx: number) => {
            const data = insight as { insight_text?: string; priority?: string; category?: string; suggested_by?: string };
            return (
              <div key={idx} className="rounded-lg border p-3 space-y-2">
              <div className="flex items-start justify-between">
                <p className="font-medium">{data.insight_text}</p>
                <Badge variant="outline">{data.priority}</Badge>
              </div>
              <p className="text-sm text-muted-foreground">{data.category}</p>
              {data.suggested_by && (
                <p className="text-xs text-muted-foreground">
                  Suggested by: {data.suggested_by}
                </p>
              )}
              </div>
            );
          })}
        </div>
      );
    }

    // For object sections, render as JSON preview
    return (
      <div className="rounded-lg bg-muted p-4">
        <pre className="text-xs overflow-auto max-h-96">
          {JSON.stringify(section.content, null, 2)}
        </pre>
      </div>
    );
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <FileText className="h-5 w-5" />
          PRD Preview
        </CardTitle>
      </CardHeader>
      <CardContent>
        {/* Session Info */}
        <div className="mb-4 rounded-lg border p-3 space-y-1">
          <h3 className="font-semibold">PRD Preview</h3>
          {data.session.initial_description && (
            <p className="text-sm text-muted-foreground">{data.session.initial_description}</p>
          )}
          <div className="flex gap-2 pt-2">
            <Badge variant="outline">{data.session.mode}</Badge>
            <Badge variant="outline">{data.session.status}</Badge>
          </div>
        </div>

        {/* Sections Accordion */}
        <Accordion type="multiple" className="space-y-2">
          {sections.map((section) => (
            <AccordionItem
              key={section.key}
              value={section.key}
              className="rounded-lg border px-4"
            >
              <AccordionTrigger className="hover:no-underline">
                <div className="flex items-center justify-between w-full pr-4">
                  <span className="font-medium">{section.title}</span>
                  {getStatusBadge(section.status)}
                </div>
              </AccordionTrigger>
              <AccordionContent className="pt-4 pb-4 space-y-3">
                {renderSectionContent(section)}

                {/* Regenerate Button */}
                {section.content && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => handleRegenerate(section.key)}
                    disabled={regeneratingSection === section.key}
                    className="gap-2"
                  >
                    {regeneratingSection === section.key ? (
                      <>
                        <Loader2 className="h-3 w-3 animate-spin" />
                        Regenerating...
                      </>
                    ) : (
                      <>
                        <RefreshCw className="h-3 w-3" />
                        Regenerate Section
                      </>
                    )}
                  </Button>
                )}
              </AccordionContent>
            </AccordionItem>
          ))}
        </Accordion>

        {/* Skipped Sections Notice */}
        {data.skippedSections.length > 0 && (
          <Alert className="mt-4">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription>
              {data.skippedSections.length} section{data.skippedSections.length !== 1 ? 's' : ''}{' '}
              skipped: {data.skippedSections.join(', ')}
            </AlertDescription>
          </Alert>
        )}
      </CardContent>
    </Card>
  );
}
