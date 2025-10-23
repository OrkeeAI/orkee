import { useState } from 'react';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Alert, AlertDescription } from '@/components/ui/alert';
import { useSpecs } from '@/hooks/useSpecs';
import { usePRDs } from '@/hooks/usePRDs';
import { Loader2, FileText, CheckSquare, Code, Shield } from 'lucide-react';

interface ContextTemplatesProps {
  projectId: string;
  onTemplateApplied?: (context: string) => void;
}

interface Template {
  id: string;
  name: string;
  description: string;
  type: 'prd' | 'capability' | 'task' | 'validation';
  includePatterns: string[];
  excludePatterns: string[];
  icon: React.ReactNode;
}

const DEFAULT_TEMPLATES: Template[] = [
  {
    id: 'prd-full',
    name: 'Full PRD Context',
    description: 'Include all code related to a PRD for comprehensive implementation',
    type: 'prd',
    includePatterns: ['src/**/*.ts', 'src/**/*.tsx', 'lib/**/*.ts', 'README.md'],
    excludePatterns: ['**/*.test.ts', '**/*.spec.ts', 'node_modules'],
    icon: <FileText className="h-4 w-4" />,
  },
  {
    id: 'capability-focused',
    name: 'Capability Implementation',
    description: 'Context for implementing a specific spec capability',
    type: 'capability',
    includePatterns: ['src/**/*.ts', 'src/**/*.tsx'],
    excludePatterns: ['**/*.test.ts', '**/*.spec.ts'],
    icon: <Code className="h-4 w-4" />,
  },
  {
    id: 'task-focused',
    name: 'Task Implementation',
    description: 'Minimal context for implementing a specific task',
    type: 'task',
    includePatterns: ['src/**/*.ts'],
    excludePatterns: ['**/*.test.ts', '**/*.spec.ts', '**/tests/**'],
    icon: <CheckSquare className="h-4 w-4" />,
  },
  {
    id: 'validation',
    name: 'Spec Validation',
    description: 'Include tests and implementation for validating spec requirements',
    type: 'validation',
    includePatterns: ['src/**/*.ts', 'tests/**/*.test.ts', 'tests/**/*.spec.ts'],
    excludePatterns: ['node_modules', 'dist'],
    icon: <Shield className="h-4 w-4" />,
  },
];

export function ContextTemplates({ projectId, onTemplateApplied }: ContextTemplatesProps) {
  const [selectedTemplate, setSelectedTemplate] = useState<Template>();
  const [linkedSpec, setLinkedSpec] = useState<string>();
  const [linkedPRD, setLinkedPRD] = useState<string>();
  const [isGenerating, setIsGenerating] = useState(false);
  const [error, setError] = useState<string>();

  const { data: specsData, isLoading: specsLoading } = useSpecs(projectId);
  const { data: prdsData, isLoading: prdsLoading } = usePRDs(projectId);

  const specs = Array.isArray(specsData) ? specsData : specsData?.items || [];
  const prds = Array.isArray(prdsData) ? prdsData : prdsData?.items || [];

  const applyTemplate = async () => {
    if (!selectedTemplate) return;

    setIsGenerating(true);
    setError(undefined);

    try {
      // Build the API endpoint
      const endpoint = `/api/projects/${projectId}/context/from-template`;
      
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          template_id: selectedTemplate.id,
          template_type: selectedTemplate.type,
          spec_id: linkedSpec,
          prd_id: linkedPRD,
          include_patterns: selectedTemplate.includePatterns,
          exclude_patterns: selectedTemplate.excludePatterns,
        }),
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || 'Failed to generate context');
      }

      const result = await response.json();
      
      // Notify parent component
      if (onTemplateApplied && result.content) {
        onTemplateApplied(result.content);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to apply template');
    } finally {
      setIsGenerating(false);
    }
  };

  const canApply = selectedTemplate && (
    selectedTemplate.type === 'task' ||
    (selectedTemplate.type === 'capability' && linkedSpec) ||
    (selectedTemplate.type === 'prd' && linkedPRD) ||
    selectedTemplate.type === 'validation'
  );

  return (
    <div className="space-y-4">
      {/* Template selector */}
      <Card>
        <CardHeader>
          <CardTitle>Context Templates</CardTitle>
          <CardDescription>
            Generate context using predefined templates for common scenarios
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {/* Template selection */}
          <div className="space-y-2">
            <label className="text-sm font-medium">Select Template</label>
            <Select
              value={selectedTemplate?.id}
              onValueChange={(id) => {
                const template = DEFAULT_TEMPLATES.find(t => t.id === id);
                setSelectedTemplate(template);
                // Reset linked items when template changes
                setLinkedSpec(undefined);
                setLinkedPRD(undefined);
              }}
            >
              <SelectTrigger>
                <SelectValue placeholder="Choose a template" />
              </SelectTrigger>
              <SelectContent>
                {DEFAULT_TEMPLATES.map(template => (
                  <SelectItem key={template.id} value={template.id}>
                    <div className="flex items-center gap-2">
                      {template.icon}
                      <div>
                        <div className="font-medium">{template.name}</div>
                        <div className="text-xs text-muted-foreground">
                          {template.description}
                        </div>
                      </div>
                    </div>
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {/* Link to spec capability */}
          {selectedTemplate?.type === 'capability' && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Link to Spec Capability</label>
              <Select value={linkedSpec} onValueChange={setLinkedSpec}>
                <SelectTrigger>
                  <SelectValue placeholder={specsLoading ? "Loading specs..." : "Select a spec capability"} />
                </SelectTrigger>
                <SelectContent>
                  {specs.map((spec: { id: string; name: string }) => (
                    <SelectItem key={spec.id} value={spec.id}>
                      {spec.name}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {/* Link to PRD */}
          {selectedTemplate?.type === 'prd' && (
            <div className="space-y-2">
              <label className="text-sm font-medium">Link to PRD</label>
              <Select value={linkedPRD} onValueChange={setLinkedPRD}>
                <SelectTrigger>
                  <SelectValue placeholder={prdsLoading ? "Loading PRDs..." : "Select a PRD"} />
                </SelectTrigger>
                <SelectContent>
                  {prds.map((prd: { id: string; title: string }) => (
                    <SelectItem key={prd.id} value={prd.id}>
                      {prd.title}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
          )}

          {/* Apply button */}
          <Button 
            onClick={applyTemplate} 
            disabled={!canApply || isGenerating}
            className="w-full"
          >
            {isGenerating && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            {isGenerating ? 'Generating Context...' : 'Apply Template'}
          </Button>

          {/* Error display */}
          {error && (
            <Alert variant="destructive">
              <AlertDescription>{error}</AlertDescription>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Template details */}
      {selectedTemplate && (
        <Card>
          <CardHeader>
            <CardTitle className="text-base">Template Configuration</CardTitle>
          </CardHeader>
          <CardContent className="space-y-3">
            <div>
              <span className="text-sm font-medium">Type:</span>
              <div className="mt-1">
                <Badge variant="secondary">{selectedTemplate.type}</Badge>
              </div>
            </div>
            <div>
              <span className="text-sm font-medium">Include Patterns:</span>
              <div className="flex flex-wrap gap-1 mt-1">
                {selectedTemplate.includePatterns.map(pattern => (
                  <Badge key={pattern} variant="secondary" className="font-mono text-xs">
                    {pattern}
                  </Badge>
                ))}
              </div>
            </div>
            <div>
              <span className="text-sm font-medium">Exclude Patterns:</span>
              <div className="flex flex-wrap gap-1 mt-1">
                {selectedTemplate.excludePatterns.map(pattern => (
                  <Badge key={pattern} variant="outline" className="font-mono text-xs">
                    {pattern}
                  </Badge>
                ))}
              </div>
            </div>
          </CardContent>
        </Card>
      )}
    </div>
  );
}
