import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from '@/components/ui/card';
import { Separator } from '@/components/ui/separator';
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs';
import { Alert, AlertDescription } from '@/components/ui/alert';
import {
  FileText,
  GitBranch,
  CheckCircle2,
  Calendar,
  AlertCircle,
} from 'lucide-react';
import { useSpec, useSpecRequirements } from '@/hooks/useSpecs';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import rehypeSanitize from 'rehype-sanitize';

interface SpecDetailsViewProps {
  projectId: string;
  specId: string;
  onEdit?: () => void;
}

export function SpecDetailsView({ projectId, specId, onEdit }: SpecDetailsViewProps) {
  const { data: spec, isLoading: specLoading } = useSpec(projectId, specId);
  const { data: requirements = [], isLoading: requirementsLoading } = useSpecRequirements(
    projectId,
    specId
  );

  const isLoading = specLoading || requirementsLoading;

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="text-center space-y-2">
          <div className="text-muted-foreground">Loading spec details...</div>
        </div>
      </div>
    );
  }

  if (!spec) {
    return (
      <Alert variant="destructive">
        <AlertCircle className="h-4 w-4" />
        <AlertDescription>Spec not found or failed to load.</AlertDescription>
      </Alert>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-start justify-between">
        <div className="space-y-1">
          <div className="flex items-center gap-2">
            <h2 className="text-2xl font-bold">{spec.name}</h2>
            <Badge variant="outline">v{spec.version}</Badge>
            <Badge
              variant={
                spec.status === 'active'
                  ? 'default'
                  : spec.status === 'deprecated'
                  ? 'secondary'
                  : 'outline'
              }
            >
              {spec.status}
            </Badge>
          </div>
          {spec.prdId && (
            <div className="flex items-center gap-1 text-sm text-muted-foreground">
              <GitBranch className="h-3 w-3" />
              Linked to PRD
            </div>
          )}
        </div>

        {onEdit && (
          <Button onClick={onEdit} variant="outline">
            Edit Spec
          </Button>
        )}
      </div>

      {/* Metadata Cards */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Requirements</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{spec.requirementCount}</div>
            <p className="text-xs text-muted-foreground">Total requirements defined</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Scenarios</CardTitle>
            <CheckCircle2 className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {requirements.reduce((sum, req) => sum + req.scenarios.length, 0)}
            </div>
            <p className="text-xs text-muted-foreground">Test scenarios across all requirements</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">Last Updated</CardTitle>
            <Calendar className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {new Date(spec.updatedAt).toLocaleDateString('en-US', { month: 'short', day: 'numeric' })}
            </div>
            <p className="text-xs text-muted-foreground">
              {new Date(spec.updatedAt).toLocaleDateString()}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* Main Content Tabs */}
      <Tabs defaultValue="overview">
        <TabsList>
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="requirements">
            Requirements ({requirements.length})
          </TabsTrigger>
          {spec.designMarkdown && <TabsTrigger value="design">Design</TabsTrigger>}
        </TabsList>

        <TabsContent value="overview" className="space-y-4">
          <Card>
            <CardHeader>
              <CardTitle>Purpose</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="prose prose-sm max-w-none">
                <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                  {spec.purpose || 'No purpose specified'}
                </ReactMarkdown>
              </div>
            </CardContent>
          </Card>

          {spec.specMarkdown && (
            <Card>
              <CardHeader>
                <CardTitle>Specification</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="prose prose-sm max-w-none">
                  <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                    {spec.specMarkdown}
                  </ReactMarkdown>
                </div>
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="requirements" className="space-y-4">
          {requirements.length === 0 ? (
            <Alert>
              <AlertDescription>No requirements defined for this spec.</AlertDescription>
            </Alert>
          ) : (
            requirements.map((req, index) => (
              <Card key={req.id || index}>
                <CardHeader>
                  <div className="flex items-start justify-between">
                    <div className="space-y-1">
                      <CardTitle className="text-lg">{req.name}</CardTitle>
                      <CardDescription>
                        {req.scenarios.length} scenario{req.scenarios.length !== 1 ? 's' : ''}
                      </CardDescription>
                    </div>
                    <Badge variant="secondary">#{index + 1}</Badge>
                  </div>
                </CardHeader>
                <CardContent className="space-y-4">
                  <div className="prose prose-sm max-w-none">
                    <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                      {req.content}
                    </ReactMarkdown>
                  </div>

                  {req.scenarios.length > 0 && (
                    <>
                      <Separator />
                      <div className="space-y-3">
                        <h4 className="text-sm font-medium">Test Scenarios</h4>
                        {req.scenarios.map((scenario, scenarioIndex) => (
                          <div
                            key={scenario.id || scenarioIndex}
                            className="rounded-lg border p-4 space-y-2 bg-muted/30"
                          >
                            <div className="font-medium text-sm flex items-center gap-2">
                              <CheckCircle2 className="h-4 w-4 text-green-600" />
                              {scenario.name}
                            </div>
                            <div className="space-y-1 text-sm pl-6">
                              <div>
                                <strong className="text-muted-foreground">WHEN:</strong>{' '}
                                {scenario.whenClause}
                              </div>
                              <div>
                                <strong className="text-muted-foreground">THEN:</strong>{' '}
                                {scenario.thenClause}
                              </div>
                              {scenario.andClauses && scenario.andClauses.length > 0 && (
                                <div>
                                  <strong className="text-muted-foreground">AND:</strong>
                                  <ul className="list-disc list-inside ml-4">
                                    {scenario.andClauses.map((clause, clauseIndex) => (
                                      <li key={clauseIndex}>{clause}</li>
                                    ))}
                                  </ul>
                                </div>
                              )}
                            </div>
                          </div>
                        ))}
                      </div>
                    </>
                  )}
                </CardContent>
              </Card>
            ))
          )}
        </TabsContent>

        {spec.designMarkdown && (
          <TabsContent value="design">
            <Card>
              <CardHeader>
                <CardTitle>Design Documentation</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="prose prose-sm max-w-none">
                  <ReactMarkdown remarkPlugins={[remarkGfm]} rehypePlugins={[rehypeSanitize]}>
                    {spec.designMarkdown}
                  </ReactMarkdown>
                </div>
              </CardContent>
            </Card>
          </TabsContent>
        )}
      </Tabs>
    </div>
  );
}
